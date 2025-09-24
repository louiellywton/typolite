use anyhow::Result;
use pulldown_cmark::{Parser, Options, html, Event, Tag, CodeBlockKind};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocItem {
    pub level: u8,
    pub title: String,
    pub anchor: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedDocument {
    pub html: String,
    pub line_map: Vec<usize>,
    pub toc: Vec<TocItem>,
    pub word_count: usize,
    pub reading_time: u32, // in minutes
}

pub struct MarkdownParser {
    options: Options,
}

impl Default for MarkdownParser {
    fn default() -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_SMART_PUNCTUATION);
        
        Self { options }
    }
}

impl MarkdownParser {
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse markdown text into a structured document
    pub fn parse(&self, markdown: &str) -> Result<ParsedDocument> {
        debug!("Starting markdown parsing, length: {} chars", markdown.len());
        
        let parser = Parser::new_ext(markdown, self.options);
        let mut html_output = String::new();
        let mut line_map = Vec::new();
        let mut toc = Vec::new();
        let mut current_line = 1usize;
        let mut current_pos = 0usize;
        let mut heading_count = HashMap::new();
        
        // Process events to build line map and TOC
        let events: Vec<_> = parser.collect();
        
        for event in &events {
            match event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    // Track heading for TOC
                    if let Some(Event::Text(text)) = events.get(events.iter().position(|e| e == event).unwrap() + 1) {
                        let title = text.to_string();
                        let anchor = self.create_anchor(&title, &mut heading_count);
                        
                        toc.push(TocItem {
                            level: *level as u8,
                            title,
                            anchor,
                            line: current_line,
                        });
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    current_line += 1;
                    line_map.push(current_pos);
                }
                Event::Text(text) => {
                    current_pos += text.len();
                }
                _ => {}
            }
        }

        // Convert to HTML with syntax highlighting and math support
        let processed_events = self.process_events(events);
        html::push_html(&mut html_output, processed_events.into_iter());

        // Calculate reading statistics
        let word_count = self.count_words(markdown);
        let reading_time = (word_count / 200).max(1) as u32; // Average reading speed: 200 WPM

        let toc_len = toc.len();
        let parsed_doc = ParsedDocument {
            html: html_output,
            line_map,
            toc,
            word_count,
            reading_time,
        };

        info!("Markdown parsing complete: {} words, {} headings, {} min read", 
              word_count, toc_len, reading_time);

        Ok(parsed_doc)
    }

    /// Create a unique anchor for headings
    fn create_anchor(&self, title: &str, heading_count: &mut HashMap<String, usize>) -> String {
        let base_anchor = title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string();

        let count = heading_count.entry(base_anchor.clone()).or_insert(0);
        *count += 1;

        if *count == 1 {
            base_anchor
        } else {
            format!("{}-{}", base_anchor, count)
        }
    }

    /// Process events to add syntax highlighting and math support
    fn process_events<'a>(&self, events: Vec<Event<'a>>) -> Vec<Event<'a>> {
        let mut processed = Vec::new();
        let mut i = 0;

        while i < events.len() {
            let event = &events[i];
            
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                    // Handle syntax highlighting
                    if let Some(Event::Text(code)) = events.get(i + 1) {
                        let highlighted = self.highlight_code(code, lang);
                        processed.push(Event::Html(highlighted.into()));
                        i += 2; // Skip the text event
                        
                        // Skip the End event too
                        if matches!(events.get(i), Some(Event::End(Tag::CodeBlock(_)))) {
                            i += 1;
                        }
                        continue;
                    }
                }
                Event::Text(text) => {
                    // Handle inline math
                    if text.contains('$') {
                        let processed_text = self.process_math(text);
                        processed.push(Event::Html(processed_text.into()));
                        i += 1;
                        continue;
                    }
                }
                Event::Start(Tag::Heading(level, _, _)) => {
                    // Add anchor IDs to headings
                    if let Some(Event::Text(title)) = events.get(i + 1) {
                        let mut heading_count = HashMap::new();
                        let anchor = self.create_anchor(title, &mut heading_count);
                        let html = format!("<h{} id=\"{}\">", level, anchor);
                        processed.push(Event::Html(html.into()));
                        i += 1;
                        continue;
                    }
                }
                _ => {}
            }

            processed.push(event.clone());
            i += 1;
        }

        processed
    }

    /// Apply syntax highlighting to code blocks
    fn highlight_code(&self, code: &str, lang: &str) -> String {
        // For now, return basic highlighted code
        // In a full implementation, you'd use syntect here
        format!(
            "<pre class=\"language-{}\"><code class=\"language-{}\">{}</code></pre>",
            lang,
            lang,
            html_escape::encode_text(code)
        )
    }

    /// Process inline math expressions
    fn process_math(&self, text: &str) -> String {
        // Simple math processing - replace $...$ with KaTeX markup
        if text.starts_with('$') && text.ends_with('$') && text.len() > 2 {
            let math_content = &text[1..text.len()-1];
            format!("<span class=\"katex-inline\" data-math=\"{}\">${}$</span>", 
                   html_escape::encode_text(math_content),
                   html_escape::encode_text(math_content))
        } else {
            html_escape::encode_text(text).to_string()
        }
    }

    /// Count words in markdown text
    fn count_words(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_parsing() {
        let parser = MarkdownParser::new();
        let markdown = "# Hello World\n\nThis is a **test** document.";
        
        let result = parser.parse(markdown).unwrap();
        
        assert!(result.html.contains("<h1"));
        assert!(result.html.contains("<strong>test</strong>"));
        assert_eq!(result.word_count, 6);
        assert_eq!(result.toc.len(), 1);
        assert_eq!(result.toc[0].title, "Hello World");
    }

    #[test]
    fn test_toc_generation() {
        let parser = MarkdownParser::new();
        let markdown = "# Title 1\n## Subtitle 1\n### Sub-subtitle\n## Subtitle 2";
        
        let result = parser.parse(markdown).unwrap();
        
        assert_eq!(result.toc.len(), 4);
        assert_eq!(result.toc[0].level, 1);
        assert_eq!(result.toc[1].level, 2);
        assert_eq!(result.toc[2].level, 3);
        assert_eq!(result.toc[3].level, 2);
    }

    #[test]
    fn test_math_processing() {
        let parser = MarkdownParser::new();
        let text = "$x^2 + y^2 = z^2$";
        
        let result = parser.process_math(text);
        
        assert!(result.contains("katex-inline"));
        assert!(result.contains("x^2 + y^2 = z^2"));
    }
}
