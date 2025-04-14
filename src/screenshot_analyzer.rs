use std::path::Path;
use tesseract::{Tesseract, TesseractError};

pub struct ScreenshotAnalyzer;

impl ScreenshotAnalyzer {
    pub fn new() -> Result<Self, String> {
        // Just verify that Tesseract can be initialized
        let _tess = Tesseract::new(None, Some("eng"))
            .map_err(|e| format!("Failed to initialize Tesseract: {}", e))?;
        
        Ok(ScreenshotAnalyzer)
    }

    pub fn analyze_screenshot(&self, path: &Path) -> String {
        println!("Analyzing screenshot: {:?}", path);
        
        // Initialize Tesseract and perform OCR using method chaining
        let text = match self.perform_ocr(path) {
            Ok(text) => text,
            Err(e) => {
                println!("Failed to perform OCR: {}", e);
                return "Unknown".to_string();
            }
        };

        if text.is_empty() {
            println!("OCR analysis produced no text");
            return "Unknown".to_string();
        }

        // Process the OCR results
        // Split into lines and look for likely window titles
        let possible_titles: Vec<&str> = text
            .lines()
            .filter(|line| {
                let line = line.trim();
                // Filter criteria for window titles:
                // - Not empty
                // - Not too short (at least 3 chars)
                // - Not too long (less than 100 chars)
                // - Contains letters
                !line.is_empty() 
                    && line.len() >= 3 
                    && line.len() < 100
                    && line.chars().any(|c| c.is_alphabetic())
            })
            .collect();

        // Try to find the most likely window title
        // For now, we'll take the first line that matches our criteria
        if let Some(title) = possible_titles.first() {
            let app_name = title.trim().to_string();
            println!("OCR analysis complete. Detected title: {}", app_name);
            app_name
        } else {
            println!("OCR analysis complete. No valid window title detected.");
            "Unknown".to_string()
        }
    }

    fn perform_ocr(&self, path: &Path) -> Result<String, TesseractError> {
        Ok(Tesseract::new(None, Some("eng"))?
            .set_variable("tessedit_pageseg_mode", "7")?
            .set_variable("tessedit_char_whitelist", 
                "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_ ")?
            .set_image(path.to_str().unwrap_or(""))?
            .recognize()?
            .get_text()?)
    }
} 