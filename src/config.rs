use crate::{CodeGrepError, CodeGrepResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub default: DefaultConfig,
    
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultConfig {
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
    
    #[serde(default)]
    pub file_types: HashMap<String, Vec<String>>,
    
    #[serde(default)]
    pub output: OutputConfig,
    
    #[serde(default)]
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
    
    #[serde(default)]
    pub file_types: HashMap<String, Vec<String>>,
    
    #[serde(default)]
    pub custom_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_true")]
    pub colors: bool,
    
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    
    #[serde(default = "default_context")]
    pub context: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    #[serde(default)]
    pub max_threads: Option<usize>,
    
    #[serde(default)]
    pub max_memory_mb: Option<usize>,
    
    #[serde(default = "default_true")]
    pub fast_mode: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default: DefaultConfig::default(),
            projects: HashMap::new(),
        }
    }
}

impl Default for DefaultConfig {
    fn default() -> Self {
        let mut file_types = HashMap::new();
        file_types.insert("rust".to_string(), vec!["rs".to_string()]);
        file_types.insert("go".to_string(), vec!["go".to_string()]);
        file_types.insert("javascript".to_string(), vec!["js".to_string(), "jsx".to_string()]);
        file_types.insert("typescript".to_string(), vec!["ts".to_string(), "tsx".to_string()]);
        file_types.insert("python".to_string(), vec!["py".to_string(), "pyw".to_string()]);
        file_types.insert("java".to_string(), vec!["java".to_string()]);
        file_types.insert("c".to_string(), vec!["c".to_string(), "h".to_string()]);
        file_types.insert("cpp".to_string(), vec!["cpp".to_string(), "cxx".to_string(), "cc".to_string(), "hpp".to_string()]);
        
        Self {
            ignore_patterns: vec![
                "*.log".to_string(),
                "*.tmp".to_string(),
                "node_modules".to_string(),
                "target".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
            ],
            file_types,
            output: OutputConfig::default(),
            performance: PerformanceConfig::default(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            colors: true,
            line_numbers: true,
            context: 2,
        }
    }
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_threads: None,
            max_memory_mb: None,
            fast_mode: true,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_context() -> usize {
    2
}

impl Config {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> CodeGrepResult<Self> {
        let content = fs::read_to_string(path)?;
        
        // Try YAML first, then TOML, then JSON
        if let Ok(config) = serde_yaml::from_str::<Config>(&content) {
            return Ok(config);
        }
        
        if let Ok(config) = toml::from_str::<Config>(&content) {
            return Ok(config);
        }
        
        if let Ok(config) = serde_json::from_str::<Config>(&content) {
            return Ok(config);
        }
        
        Err(CodeGrepError::Config("Failed to parse config file".to_string()))
    }
    
    pub fn find_and_load() -> CodeGrepResult<Self> {
        // Look for config files in order of preference
        let config_paths = [
            "./.codegreeprc",
            "./.codegreeprc.yaml",
            "./.codegreeprc.yml",
            "./.codegreeprc.toml",
            "./.codegreeprc.json",
        ];
        
        // Check current directory first
        for path in &config_paths {
            if Path::new(path).exists() {
                return Self::load_from_file(path);
            }
        }
        
        // Check home directory
        if let Some(home_dir) = dirs::home_dir() {
            let home_config_paths = [
                ".codegreeprc",
                ".codegreeprc.yaml", 
                ".codegreeprc.yml",
                ".codegreeprc.toml",
                ".codegreeprc.json",
            ];
            
            for path in &home_config_paths {
                let full_path = home_dir.join(path);
                if full_path.exists() {
                    return Self::load_from_file(&full_path);
                }
            }
        }
        
        // Return default config if no file found
        Ok(Self::default())
    }
    
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> CodeGrepResult<()> {
        let content = serde_yaml::to_string(self)
            .map_err(|e| CodeGrepError::Config(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(path, content)?;
        Ok(())
    }
    
    pub fn get_file_extensions(&self, file_type: &str) -> Option<&Vec<String>> {
        self.default.file_types.get(file_type)
    }
    
    pub fn should_ignore(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        
        for pattern in &self.default.ignore_patterns {
            if glob_match(pattern, &path_str) {
                return true;
            }
        }
        
        false
    }
}

// Simple glob matching function
fn glob_match(pattern: &str, text: &str) -> bool {
    // Very basic implementation - in a real implementation you'd use a proper glob library
    if pattern.starts_with('*') && pattern.len() > 1 {
        let suffix = &pattern[1..];
        text.ends_with(suffix)
    } else if pattern.ends_with('*') && pattern.len() > 1 {
        let prefix = &pattern[..pattern.len() - 1];
        text.starts_with(prefix)
    } else {
        pattern == text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(!config.default.ignore_patterns.is_empty());
        assert!(!config.default.file_types.is_empty());
        assert!(config.default.output.colors);
    }
    
    #[test]
    fn test_config_yaml_serialization() {
        let config = Config::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
        
        assert_eq!(config.default.output.colors, parsed.default.output.colors);
    }
    
    #[test]
    fn test_glob_match() {
        assert!(glob_match("*.log", "test.log"));
        assert!(glob_match("node_modules*", "node_modules/something"));
        assert!(!glob_match("*.js", "test.rs"));
    }
}