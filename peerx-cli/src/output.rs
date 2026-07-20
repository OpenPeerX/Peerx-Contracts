use crate::error::{CliError, Result};
use colored::*;
use serde::Serialize;

/// Output formatter for different formats
pub struct OutputFormatter;

impl OutputFormatter {
    /// Format output based on the specified format
    pub fn format<T: Serialize>(data: &T, format: &str) -> Result<String> {
        match format.to_lowercase().as_str() {
            "json" => Self::format_json(data),
            "yaml" => Self::format_yaml(data),
            "human" => Self::format_human(data),
            _ => Err(CliError::Format(format!(
                "Unknown format: {}. Supported formats: json, yaml, human",
                format
            ))),
        }
    }
    
    /// Format as JSON
    fn format_json<T: Serialize>(data: &T) -> Result<String> {
        Ok(serde_json::to_string_pretty(data)?)
    }
    
    /// Format as YAML (simplified JSON for now)
    fn format_yaml<T: Serialize>(data: &T) -> Result<String> {
        // For now, use JSON. In a full implementation, add serde_yaml dependency
        Self::format_json(data)
    }
    
    /// Format as human-readable text
    fn format_human<T: Serialize>(data: &T) -> Result<String> {
        // Convert to JSON value for easier traversal
        let json_value = serde_json::to_value(data)?;
        Ok(Self::format_json_value(&json_value, 0))
    }
    
    /// Recursively format JSON value with indentation
    fn format_json_value(value: &serde_json::Value, indent: usize) -> String {
        let prefix = "  ".repeat(indent);
        
        match value {
            serde_json::Value::Object(map) => {
                let mut result = String::new();
                for (key, val) in map {
                    result.push_str(&format!("{}{}: ", prefix, key.cyan()));
                    
                    if val.is_object() || val.is_array() {
                        result.push('\n');
                        result.push_str(&Self::format_json_value(val, indent + 1));
                    } else {
                        result.push_str(&Self::format_json_value(val, 0));
                        result.push('\n');
                    }
                }
                result
            }
            serde_json::Value::Array(arr) => {
                let mut result = String::new();
                for (i, val) in arr.iter().enumerate() {
                    result.push_str(&format!("{}[{}] ", prefix, i));
                    result.push_str(&Self::format_json_value(val, indent + 1));
                }
                result
            }
            serde_json::Value::String(s) => s.to_string(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => {
                if *b {
                    b.to_string().green().to_string()
                } else {
                    b.to_string().red().to_string()
                }
            }
            serde_json::Value::Null => "null".dimmed().to_string(),
        }
    }
    
    /// Print success message
    pub fn success(message: &str) {
        println!("{} {}", "✓".green().bold(), message);
    }
    
    /// Print warning message
    pub fn warning(message: &str) {
        println!("{} {}", "⚠".yellow().bold(), message);
    }
    
    /// Print error message
    pub fn error(message: &str) {
        eprintln!("{} {}", "✗".red().bold(), message);
    }
    
    /// Print info message
    pub fn info(message: &str) {
        println!("{} {}", "ℹ".blue().bold(), message);
    }
}
