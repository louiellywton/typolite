use anyhow::{Result, Context};
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub path: PathBuf,
    pub size: u64,
    pub modified: u64, // Unix timestamp
    pub is_markdown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChangeEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf, to: PathBuf },
}

pub struct FileService {
    watchers: Arc<Mutex<HashMap<PathBuf, RecommendedWatcher>>>,
    debounce_delay: Duration,
    pending_events: Arc<Mutex<HashMap<PathBuf, Instant>>>,
}

impl Default for FileService {
    fn default() -> Self {
        Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            debounce_delay: Duration::from_millis(300),
            pending_events: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl FileService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_debounce_delay(mut self, delay: Duration) -> Self {
        self.debounce_delay = delay;
        self
    }

    /// Read a markdown file and return its content
    pub async fn read_file(&self, path: &Path) -> Result<String> {
        debug!("Reading file: {:?}", path);
        
        if !path.exists() {
            return Err(anyhow::anyhow!("File does not exist: {:?}", path));
        }

        let content = tokio::fs::read_to_string(path).await
            .with_context(|| format!("Failed to read file: {:?}", path))?;

        info!("Successfully read file: {:?} ({} bytes)", path, content.len());
        Ok(content)
    }

    /// Write content to a file atomically
    pub async fn write_file(&self, path: &Path, content: &str) -> Result<()> {
        debug!("Writing file: {:?} ({} bytes)", path, content.len());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .with_context(|| format!("Failed to create parent directories for: {:?}", path))?;
        }

        // Write to a temporary file first, then rename (atomic operation)
        let temp_path = path.with_extension("tmp");
        tokio::fs::write(&temp_path, content).await
            .with_context(|| format!("Failed to write temporary file: {:?}", temp_path))?;

        tokio::fs::rename(&temp_path, path).await
            .with_context(|| format!("Failed to rename temp file to: {:?}", path))?;

        info!("Successfully wrote file: {:?}", path);
        Ok(())
    }

    /// Get file metadata
    pub async fn get_metadata(&self, path: &Path) -> Result<FileMetadata> {
        let metadata = tokio::fs::metadata(path).await
            .with_context(|| format!("Failed to get metadata for: {:?}", path))?;

        let is_markdown = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| matches!(ext.to_lowercase().as_str(), "md" | "markdown" | "mdown" | "mkd"))
            .unwrap_or(false);

        let modified = metadata.modified()
            .with_context(|| format!("Failed to get modified time for: {:?}", path))?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(FileMetadata {
            path: path.to_path_buf(),
            size: metadata.len(),
            modified,
            is_markdown,
        })
    }

    /// Start watching a file for changes
    pub async fn watch_file<F>(&self, path: PathBuf, callback: F) -> Result<()>
    where
        F: Fn(FileChangeEvent) + Send + Sync + 'static,
    {
        info!("Starting to watch file: {:?}", path);

        let (tx, mut rx) = mpsc::unbounded_channel::<notify::Result<Event>>();
        let callback = Arc::new(callback);
        
        // Create debounced event handler
        let debounce_delay = self.debounce_delay;
        let _pending_events = self.pending_events.clone();
        let callback_clone = callback.clone();
        
        tokio::spawn(async move {
            let mut debounce_map: HashMap<PathBuf, Instant> = HashMap::new();
            
            loop {
                // Check for debounced events that are ready to fire
                let now = Instant::now();
                let ready_events: Vec<PathBuf> = debounce_map
                    .iter()
                    .filter(|(_, &time)| now.duration_since(time) >= debounce_delay)
                    .map(|(path, _)| path.clone())
                    .collect();

                for event_path in ready_events {
                    debounce_map.remove(&event_path);
                    callback_clone(FileChangeEvent {
                        path: event_path,
                        event_type: FileEventType::Modified,
                    });
                }

                // Process new events or wait a bit
                match tokio::time::timeout(Duration::from_millis(50), rx.recv()).await {
                    Ok(Some(event)) => {
                        if let Ok(event) = event {
                            for event_path in event.paths {
                                debounce_map.insert(event_path, now);
                            }
                        }
                    }
                    Ok(None) => break, // Channel closed
                    Err(_) => continue, // Timeout, check debounced events
                }
            }
        });

        // Create and configure the watcher
        let watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Err(e) = tx.send(res) {
                error!("Failed to send file event: {}", e);
            }
        })?;

        let mut watchers = self.watchers.lock().unwrap();
        watchers.insert(path.clone(), watcher);

        // Start watching the file
        if let Some(watcher) = watchers.get_mut(&path) {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }

        Ok(())
    }

    /// Stop watching a file
    pub fn unwatch_file(&self, path: &PathBuf) -> Result<()> {
        debug!("Stopping watch for file: {:?}", path);

        let mut watchers = self.watchers.lock().unwrap();
        if let Some(mut watcher) = watchers.remove(path) {
            if let Err(e) = watcher.unwatch(path) {
                warn!("Failed to unwatch file {:?}: {}", path, e);
            }
        }

        info!("Stopped watching file: {:?}", path);
        Ok(())
    }

    /// List markdown files in a directory
    pub async fn list_markdown_files(&self, dir: &Path) -> Result<Vec<FileMetadata>> {
        debug!("Listing markdown files in: {:?}", dir);

        let mut files = Vec::new();
        let mut entries = tokio::fs::read_dir(dir).await
            .with_context(|| format!("Failed to read directory: {:?}", dir))?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = self.get_metadata(&path).await {
                    if metadata.is_markdown {
                        files.push(metadata);
                    }
                }
            }
        }

        files.sort_by(|a, b| b.modified.cmp(&a.modified)); // Sort by most recent first
        info!("Found {} markdown files in {:?}", files.len(), dir);

        Ok(files)
    }

    /// Check if a file exists and is readable
    pub async fn is_file_accessible(&self, path: &Path) -> bool {
        match tokio::fs::metadata(path).await {
            Ok(metadata) => metadata.is_file(),
            Err(_) => false,
        }
    }

    /// Get the size of a file in bytes
    pub async fn get_file_size(&self, path: &Path) -> Result<u64> {
        let metadata = tokio::fs::metadata(path).await
            .with_context(|| format!("Failed to get file size for: {:?}", path))?;
        Ok(metadata.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_read_write_file() {
        let service = FileService::new();
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "# Test Markdown\n\nThis is a test.";
        
        write!(temp_file, "{}", content).unwrap();
        let temp_path = temp_file.path();

        // Test reading
        let read_content = service.read_file(temp_path).await.unwrap();
        assert_eq!(read_content, content);

        // Test writing
        let new_content = "# Updated Markdown\n\nThis is updated.";
        service.write_file(temp_path, new_content).await.unwrap();
        
        let updated_content = service.read_file(temp_path).await.unwrap();
        assert_eq!(updated_content, new_content);
    }

    #[tokio::test]
    async fn test_get_metadata() {
        let service = FileService::new();
        let mut temp_file = NamedTempFile::with_suffix(".md").unwrap();
        write!(temp_file, "# Test").unwrap();

        let metadata = service.get_metadata(temp_file.path()).await.unwrap();
        
        assert!(metadata.is_markdown);
        assert_eq!(metadata.size, 6); // "# Test" is 6 bytes
    }

    #[tokio::test]
    async fn test_file_accessibility() {
        let service = FileService::new();
        let temp_file = NamedTempFile::new().unwrap();
        
        assert!(service.is_file_accessible(temp_file.path()).await);
        
        drop(temp_file); // File should no longer exist
        
        // Note: This might still pass due to timing, but it's a basic test
    }
}
