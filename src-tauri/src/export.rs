use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_toc: bool,
    pub page_size: PageSize,
    pub margins: Margins,
    pub header: Option<String>,
    pub footer: Option<String>,
    pub css_theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Pdf,
    Html,
    Docx, // Future implementation
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PageSize {
    A4,
    Letter,
    Legal,
    A3,
    A5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Margins {
    pub top: f32,    // in inches
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Pdf,
            include_toc: true,
            page_size: PageSize::A4,
            margins: Margins {
                top: 1.0,
                right: 1.0,
                bottom: 1.0,
                left: 1.0,
            },
            header: None,
            footer: Some("Page {page} of {pages}".to_string()),
            css_theme: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub output_path: PathBuf,
    pub file_size: u64,
    pub pages: u32,
    pub export_time_ms: u64,
}

pub struct ExportService {
    temp_dir: PathBuf,
}

impl Default for ExportService {
    fn default() -> Self {
        let temp_dir = std::env::temp_dir().join("typolite-exports");
        if !temp_dir.exists() {
            if let Err(e) = std::fs::create_dir_all(&temp_dir) {
                error!("Failed to create temp export directory: {}", e);
            }
        }
        
        Self { temp_dir }
    }
}

impl ExportService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_temp_dir(mut self, temp_dir: PathBuf) -> Self {
        self.temp_dir = temp_dir;
        self
    }

    /// Export markdown content to the specified format
    pub async fn export(
        &self,
        html_content: &str,
        output_path: &Path,
        options: ExportOptions,
    ) -> Result<ExportResult> {
        let start_time = std::time::Instant::now();
        
        debug!("Starting export to {:?} with format {:?}", output_path, options.format);

        let result = match options.format {
            ExportFormat::Pdf => self.export_to_pdf(html_content, output_path, &options).await,
            ExportFormat::Html => self.export_to_html(html_content, output_path, &options).await,
            ExportFormat::Docx => {
                return Err(anyhow::anyhow!("DOCX export not yet implemented"));
            }
        }?;

        let export_time_ms = start_time.elapsed().as_millis() as u64;
        
        info!("Export completed in {}ms: {:?}", export_time_ms, output_path);

        Ok(ExportResult {
            output_path: result.output_path,
            file_size: result.file_size,
            pages: result.pages,
            export_time_ms,
        })
    }

    /// Export to PDF format
    async fn export_to_pdf(
        &self,
        html_content: &str,
        output_path: &Path,
        options: &ExportOptions,
    ) -> Result<ExportResult> {
        // Create a complete HTML document with CSS
        let full_html = self.create_complete_html(html_content, options)?;
        
        // Write HTML to temporary file
        let temp_html_path = self.temp_dir.join(format!("export-{}.html", uuid::Uuid::new_v4()));
        tokio::fs::write(&temp_html_path, full_html).await
            .with_context(|| "Failed to write temporary HTML file")?;

        // For now, we'll simulate PDF generation
        // In a real implementation, you would use a library like wkhtmltopdf, Chromium Headless, or similar
        let result = self.generate_pdf_mock(&temp_html_path, output_path).await?;

        // Clean up temporary file
        if let Err(e) = tokio::fs::remove_file(&temp_html_path).await {
            warn!("Failed to clean up temporary HTML file: {}", e);
        }

        Ok(result)
    }

    /// Export to HTML format
    async fn export_to_html(
        &self,
        html_content: &str,
        output_path: &Path,
        options: &ExportOptions,
    ) -> Result<ExportResult> {
        let full_html = self.create_complete_html(html_content, options)?;
        
        tokio::fs::write(output_path, full_html).await
            .with_context(|| format!("Failed to write HTML file: {:?}", output_path))?;

        let file_size = tokio::fs::metadata(output_path).await?.len();

        Ok(ExportResult {
            output_path: output_path.to_path_buf(),
            file_size,
            pages: 1, // HTML is single "page"
            export_time_ms: 0, // Will be calculated by caller
        })
    }

    /// Create a complete HTML document with styling
    fn create_complete_html(&self, content: &str, options: &ExportOptions) -> Result<String> {
        let css = self.get_export_css(options)?;
        let toc = if options.include_toc {
            self.generate_toc_from_html(content)?
        } else {
            String::new()
        };

        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Exported Document</title>
    <style>
        {}
    </style>
</head>
<body>
    <div class="document">
        {}
        <div class="content">
            {}
        </div>
    </div>
</body>
</html>"#,
            css,
            toc,
            content
        );

        Ok(html)
    }

    /// Get CSS styles for export
    fn get_export_css(&self, options: &ExportOptions) -> Result<String> {
        let base_css = r#"
        @page {
            margin: 1in;
            size: A4;
        }
        
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            font-size: 12pt;
            line-height: 1.6;
            color: #333;
            max-width: none;
            margin: 0;
            padding: 0;
        }
        
        .document {
            max-width: none;
            margin: 0;
            padding: 20px;
        }
        
        h1, h2, h3, h4, h5, h6 {
            page-break-after: avoid;
            margin-top: 1.5em;
            margin-bottom: 0.5em;
            font-weight: 600;
        }
        
        h1 { font-size: 24pt; }
        h2 { font-size: 20pt; }
        h3 { font-size: 16pt; }
        h4 { font-size: 14pt; }
        h5 { font-size: 12pt; }
        h6 { font-size: 11pt; }
        
        p {
            margin: 0 0 1em 0;
            orphans: 3;
            widows: 3;
        }
        
        pre, code {
            font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
            font-size: 85%;
        }
        
        pre {
            background: #f6f8fa;
            border-radius: 6px;
            padding: 16px;
            overflow-x: auto;
            page-break-inside: avoid;
        }
        
        code {
            background: #f6f8fa;
            padding: 2px 4px;
            border-radius: 3px;
        }
        
        blockquote {
            border-left: 4px solid #dfe2e5;
            padding: 0 16px;
            margin: 0 0 16px 0;
            color: #6a737d;
        }
        
        table {
            border-collapse: collapse;
            width: 100%;
            margin: 1em 0;
            page-break-inside: avoid;
        }
        
        th, td {
            border: 1px solid #dfe2e5;
            padding: 8px 12px;
            text-align: left;
        }
        
        th {
            background: #f6f8fa;
            font-weight: 600;
        }
        
        img {
            max-width: 100%;
            height: auto;
            page-break-inside: avoid;
        }
        
        .toc {
            page-break-after: always;
            margin-bottom: 2em;
        }
        
        .toc h2 {
            margin-top: 0;
        }
        
        .toc ul {
            list-style: none;
            padding-left: 0;
        }
        
        .toc li {
            margin: 0.5em 0;
        }
        
        .toc a {
            text-decoration: none;
            color: #0366d6;
        }
        
        .toc a:hover {
            text-decoration: underline;
        }
        
        @media print {
            .no-print {
                display: none;
            }
        }
        "#;

        // Apply custom theme CSS if provided
        let css = if let Some(theme_css) = &options.css_theme {
            format!("{}\n\n/* Custom Theme */\n{}", base_css, theme_css)
        } else {
            base_css.to_string()
        };

        Ok(css)
    }

    /// Generate table of contents from HTML content
    fn generate_toc_from_html(&self, html: &str) -> Result<String> {
        // Simple TOC generation - in a real implementation, you'd use an HTML parser
        let mut toc_items = Vec::new();
        
        for line in html.lines() {
            if let Some(heading) = self.extract_heading_from_line(line) {
                toc_items.push(heading);
            }
        }

        if toc_items.is_empty() {
            return Ok(String::new());
        }

        let toc_html = format!(
            r#"<div class="toc">
                <h2>Table of Contents</h2>
                <ul>
                    {}
                </ul>
            </div>"#,
            toc_items.join("\n")
        );

        Ok(toc_html)
    }

    /// Extract heading information from HTML line
    fn extract_heading_from_line(&self, line: &str) -> Option<String> {
        // This is a simplified implementation
        // In practice, you'd use a proper HTML parser like scraper or html5ever
        
        if line.trim_start().starts_with("<h") && line.contains('>') {
            // Extract heading level and content
            if let Some(start) = line.find('>') {
                if let Some(end) = line.find("</h") {
                    let content = &line[start + 1..end];
                    let level: usize = if line.contains("<h1") { 1 }
                    else if line.contains("<h2") { 2 }
                    else if line.contains("<h3") { 3 }
                    else if line.contains("<h4") { 4 }
                    else if line.contains("<h5") { 5 }
                    else if line.contains("<h6") { 6 }
                    else { return None; };
                    
                    let indent = "  ".repeat(level.saturating_sub(1));
                    return Some(format!("{}<li><a href=\"#{}\">{}</a></li>", 
                                       indent, 
                                       content.to_lowercase().replace(' ', "-"),
                                       content));
                }
            }
        }
        None
    }

    /// Mock PDF generation (placeholder implementation)
    async fn generate_pdf_mock(
        &self,
        _html_path: &Path,
        output_path: &Path,
    ) -> Result<ExportResult> {
        // In a real implementation, this would call a PDF generation library
        // For now, we'll just create a placeholder file
        
        let placeholder_pdf = b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n2 0 obj\n<<\n/Type /Pages\n/Kids [3 0 R]\n/Count 1\n>>\nendobj\n3 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n/Contents 4 0 R\n>>\nendobj\n4 0 obj\n<<\n/Length 44\n>>\nstream\nBT\n/F1 12 Tf\n72 720 Td\n(Typora-Lite Export) Tj\nET\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000206 00000 n \ntrailer\n<<\n/Size 5\n/Root 1 0 R\n>>\nstartxref\n299\n%%EOF";
        
        tokio::fs::write(output_path, placeholder_pdf).await
            .with_context(|| format!("Failed to write PDF file: {:?}", output_path))?;

        let file_size = tokio::fs::metadata(output_path).await?.len();

        info!("Generated mock PDF: {:?} ({} bytes)", output_path, file_size);

        Ok(ExportResult {
            output_path: output_path.to_path_buf(),
            file_size,
            pages: 1, // Mock single page
            export_time_ms: 0, // Will be calculated by caller
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_html_export() {
        let temp_dir = TempDir::new().unwrap();
        let service = ExportService::new().with_temp_dir(temp_dir.path().to_path_buf());
        
        let html_content = "<h1>Test Document</h1><p>This is a test.</p>";
        let output_path = temp_dir.path().join("test.html");
        let options = ExportOptions::default();

        let result = service.export(html_content, &output_path, options).await.unwrap();

        assert_eq!(result.output_path, output_path);
        assert!(result.file_size > 0);
        assert!(output_path.exists());
    }

    #[tokio::test]
    async fn test_pdf_export_mock() {
        let temp_dir = TempDir::new().unwrap();
        let service = ExportService::new().with_temp_dir(temp_dir.path().to_path_buf());
        
        let html_content = "<h1>Test Document</h1><p>This is a test.</p>";
        let output_path = temp_dir.path().join("test.pdf");
        let options = ExportOptions {
            format: ExportFormat::Pdf,
            ..Default::default()
        };

        let result = service.export(html_content, &output_path, options).await.unwrap();

        assert_eq!(result.output_path, output_path);
        assert!(result.file_size > 0);
        assert!(output_path.exists());
    }

    #[test]
    fn test_toc_generation() {
        let service = ExportService::new();
        let html = "<h1>Chapter 1</h1><h2>Section 1.1</h2><h2>Section 1.2</h2>";
        
        let toc = service.generate_toc_from_html(html).unwrap();
        
        assert!(toc.contains("Table of Contents"));
        assert!(toc.contains("Chapter 1"));
        assert!(toc.contains("Section 1.1"));
    }
}
