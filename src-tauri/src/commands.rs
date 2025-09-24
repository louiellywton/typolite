use anyhow::Result;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tauri::{command, Window, State};
use tracing::{debug, info, warn, error};

use crate::parser::{MarkdownParser, ParsedDocument};
use crate::export::{ExportService, ExportOptions, ExportResult};
use crate::file_service::{FileService, FileMetadata, FileChangeEvent};

// Application state
#[derive(Default)]
pub struct AppState {
    pub parser: MarkdownParser,
    pub export_service: ExportService,
    pub file_service: FileService,
    pub current_file: Arc<Mutex<Option<PathBuf>>>,
    pub watchers: Arc<Mutex<HashMap<PathBuf, bool>>>,
}

// Command result types
#[derive(Debug, Serialize)]
pub struct CommandResult<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> CommandResult<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

// Command implementations

#[command]
pub async fn open_file_dialog() -> CommandResult<Option<PathBuf>> {
    debug!("Opening file dialog");
    
    match tauri::api::dialog::blocking::FileDialogBuilder::new()
        .add_filter("Markdown", &["md", "markdown", "mdown", "mkd"])
        .add_filter("All files", &["*"])
        .pick_file()
    {
        Some(path) => {
            info!("File selected: {:?}", path);
            CommandResult::ok(Some(path))
        }
        None => {
            debug!("No file selected");
            CommandResult::ok(None)
        }
    }
}

#[command]
pub async fn read_markdown_file(
    path: PathBuf,
    state: State<'_, AppState>,
) -> Result<CommandResult<String>, String> {
    debug!("Reading markdown file: {:?}", path);

    match state.file_service.read_file(&path).await {
        Ok(content) => {
            // Update current file in state
            *state.current_file.lock().unwrap() = Some(path);
            Ok(CommandResult::ok(content))
        }
        Err(e) => {
            error!("Failed to read file {:?}: {}", path, e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn parse_markdown(
    content: String,
    state: State<'_, AppState>,
) -> Result<CommandResult<ParsedDocument>, String> {
    debug!("Parsing markdown content ({} chars)", content.len());

    match state.parser.parse(&content) {
        Ok(parsed) => {
            info!("Markdown parsed successfully: {} words, {} headings", 
                  parsed.word_count, parsed.toc.len());
            Ok(CommandResult::ok(parsed))
        }
        Err(e) => {
            error!("Failed to parse markdown: {}", e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn export_to_pdf(
    html_content: String,
    output_path: PathBuf,
    options: Option<ExportOptions>,
    state: State<'_, AppState>,
) -> Result<CommandResult<ExportResult>, String> {
    debug!("Exporting to PDF: {:?}", output_path);

    let export_options = options.unwrap_or_default();
    
    match state.export_service.export(&html_content, &output_path, export_options).await {
        Ok(result) => {
            info!("PDF export completed: {:?} ({} bytes)", 
                  result.output_path, result.file_size);
            Ok(CommandResult::ok(result))
        }
        Err(e) => {
            error!("Failed to export PDF: {}", e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn save_file(
    path: PathBuf,
    content: String,
    state: State<'_, AppState>,
) -> Result<CommandResult<()>, String> {
    debug!("Saving file: {:?}", path);

    match state.file_service.write_file(&path, &content).await {
        Ok(()) => {
            info!("File saved successfully: {:?}", path);
            Ok(CommandResult::ok(()))
        }
        Err(e) => {
            error!("Failed to save file {:?}: {}", path, e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn get_app_config_dir() -> CommandResult<PathBuf> {
    debug!("Getting app config directory");

    match directories::ProjectDirs::from("com", "typolite", "Typora-Lite") {
        Some(proj_dirs) => {
            let config_dir = proj_dirs.config_dir().to_path_buf();
            info!("Config directory: {:?}", config_dir);
            CommandResult::ok(config_dir)
        }
        None => {
            let fallback = PathBuf::from("./config");
            warn!("Could not determine config directory, using fallback: {:?}", fallback);
            CommandResult::ok(fallback)
        }
    }
}

#[command]
pub async fn watch_file(
    path: PathBuf,
    window: Window,
    state: State<'_, AppState>,
) -> Result<CommandResult<()>, String> {
    info!("Starting file watch: {:?}", path);

    let window_clone = window.clone();
    let _path_clone = path.clone();
    
    let callback = move |event: FileChangeEvent| {
        debug!("File change detected: {:?}", event);
        
        if let Err(e) = window_clone.emit("file-changed", &event) {
            error!("Failed to emit file-changed event: {}", e);
        }
    };

    match state.file_service.watch_file(path.clone(), callback).await {
        Ok(()) => {
            // Track the watcher
            state.watchers.lock().unwrap().insert(path, true);
            Ok(CommandResult::ok(()))
        }
        Err(e) => {
            error!("Failed to start watching file {:?}: {}", path, e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn unwatch_file(
    path: PathBuf,
    state: State<'_, AppState>,
) -> Result<CommandResult<()>, String> {
    info!("Stopping file watch: {:?}", path);

    match state.file_service.unwatch_file(&path) {
        Ok(()) => {
            // Remove from tracking
            state.watchers.lock().unwrap().remove(&path);
            Ok(CommandResult::ok(()))
        }
        Err(e) => {
            error!("Failed to stop watching file {:?}: {}", path, e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn get_file_metadata(
    path: PathBuf,
    state: State<'_, AppState>,
) -> Result<CommandResult<FileMetadata>, String> {
    debug!("Getting file metadata: {:?}", path);

    match state.file_service.get_metadata(&path).await {
        Ok(metadata) => {
            Ok(CommandResult::ok(metadata))
        }
        Err(e) => {
            error!("Failed to get file metadata {:?}: {}", path, e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn list_recent_files(
    dir: Option<PathBuf>,
    state: State<'_, AppState>,
) -> Result<CommandResult<Vec<FileMetadata>>, String> {
    let search_dir = dir.unwrap_or_else(|| {
        directories::UserDirs::new()
            .and_then(|dirs| Some(dirs.document_dir()?.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
    });

    debug!("Listing recent files in: {:?}", search_dir);

    match state.file_service.list_markdown_files(&search_dir).await {
        Ok(files) => {
            info!("Found {} markdown files", files.len());
            Ok(CommandResult::ok(files))
        }
        Err(e) => {
            error!("Failed to list files in {:?}: {}", search_dir, e);
            Ok(CommandResult::err(e.to_string()))
        }
    }
}

#[command]
pub async fn get_app_version() -> CommandResult<String> {
    let version = env!("CARGO_PKG_VERSION").to_string();
    debug!("App version: {}", version);
    CommandResult::ok(version)
}

#[command]
pub async fn get_system_info() -> CommandResult<SystemInfo> {
    debug!("Getting system info");
    
    let info = SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    CommandResult::ok(info)
}

#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub version: String,
}

// Utility functions for commands

pub fn handle_command_error<T>(result: Result<T>) -> CommandResult<T> {
    match result {
        Ok(data) => CommandResult::ok(data),
        Err(e) => {
            error!("Command error: {}", e);
            CommandResult::err(e.to_string())
        }
    }
}

pub fn validate_markdown_file(path: &Path) -> Result<()> {
    if !path.exists() {
        return Err(anyhow::anyhow!("File does not exist: {:?}", path));
    }

    if !path.is_file() {
        return Err(anyhow::anyhow!("Path is not a file: {:?}", path));
    }

    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    if !matches!(extension.to_lowercase().as_str(), "md" | "markdown" | "mdown" | "mkd") {
        return Err(anyhow::anyhow!("Not a markdown file: {:?}", path));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_validate_markdown_file() {
        // Test valid markdown file
        let mut temp_file = NamedTempFile::with_suffix(".md").unwrap();
        write!(temp_file, "# Test").unwrap();
        
        assert!(validate_markdown_file(temp_file.path()).is_ok());

        // Test invalid extension
        let mut temp_file = NamedTempFile::with_suffix(".txt").unwrap();
        write!(temp_file, "# Test").unwrap();
        
        assert!(validate_markdown_file(temp_file.path()).is_err());
    }

    #[test]
    fn test_command_result() {
        let success = CommandResult::ok("test data");
        assert!(success.success);
        assert_eq!(success.data, Some("test data"));
        assert!(success.error.is_none());

        let error = CommandResult::<String>::err("test error".to_string());
        assert!(!error.success);
        assert!(error.data.is_none());
        assert_eq!(error.error, Some("test error".to_string()));
    }
}
