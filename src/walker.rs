use crate::{Cli, CodeGrepError, CodeGrepResult};
use ignore::{Walk, WalkBuilder};
use std::path::{Path, PathBuf};

pub struct FileWalker {
    builder: WalkBuilder,
}

impl FileWalker {
    pub fn new(cli: &Cli) -> Self {
        let mut builder = WalkBuilder::new(".");
        
        // Add additional paths
        for path in &cli.paths {
            builder.add(path);
        }
        
        // Configure walker based on CLI options
        builder
            .hidden(!cli.hidden)
            .git_ignore(cli.respect_gitignore)
            .git_exclude(cli.respect_gitignore)
            .threads(cli.effective_threads())
            .follow_links(false);
        
        // Set max depth if specified
        if let Some(depth) = cli.max_depth {
            builder.max_depth(Some(depth));
        }
        
        // Note: Custom ignore patterns would need a different approach
        // This is a simplified implementation
        for _pattern in &cli.ignore {
            // TODO: Implement custom ignore patterns
        }
        
        Self { builder }
    }
    
    pub fn walk(&self) -> Vec<CodeGrepResult<PathBuf>> {
        let mut results = Vec::new();
        for entry in self.builder.build() {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        results.push(Ok(path.to_path_buf()));
                    }
                    // Skip directories
                }
                Err(e) => {
                    // Convert ignore::Error to std::io::Error
                    let io_error = std::io::Error::new(std::io::ErrorKind::Other, e);
                    results.push(Err(CodeGrepError::Io(io_error)));
                }
            }
        }
        results
    }
    
    pub fn should_include_file(&self, path: &Path, cli: &Cli) -> bool {
        // Check file extensions
        if !cli.ext.is_empty() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if !cli.ext.iter().any(|e| e == ext) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check file types (basic mapping)
        if !cli.r#type.is_empty() {
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let matches_type = cli.r#type.iter().any(|t| {
                match t.as_str() {
                    "rust" | "rs" => ext == "rs",
                    "go" => ext == "go",
                    "js" | "javascript" => ext == "js",
                    "ts" | "typescript" => ext == "ts",
                    "jsx" => ext == "jsx",
                    "tsx" => ext == "tsx",
                    "py" | "python" => ext == "py",
                    "java" => ext == "java",
                    "c" => ext == "c",
                    "cpp" | "cxx" | "cc" => matches!(ext, "cpp" | "cxx" | "cc" | "hpp"),
                    "h" => ext == "h",
                    "json" => ext == "json",
                    "yaml" | "yml" => matches!(ext, "yaml" | "yml"),
                    "toml" => ext == "toml",
                    "md" | "markdown" => matches!(ext, "md" | "markdown"),
                    "txt" | "text" => ext == "txt",
                    _ => false,
                }
            });
            if !matches_type {
                return false;
            }
        }
        
        // Check file size
        if let Some(max_size_str) = &cli.max_filesize {
            if let Ok(metadata) = std::fs::metadata(path) {
                let max_size = parse_size(max_size_str).unwrap_or(u64::MAX);
                if metadata.len() > max_size {
                    return false;
                }
            }
        }
        
        // Check modification time
        if let Some(within_str) = &cli.modified_within {
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let duration = parse_duration(within_str).unwrap_or(std::time::Duration::MAX);
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > duration {
                            return false;
                        }
                    }
                }
            }
        }
        
        // Check if binary (basic heuristic)
        if !cli.binary {
            if is_binary_file(path) {
                return false;
            }
        }
        
        true
    }
}

fn parse_size(size_str: &str) -> Option<u64> {
    let size_str = size_str.trim().to_lowercase();
    
    if let Some(num_str) = size_str.strip_suffix("k") {
        num_str.parse::<u64>().ok().map(|n| n * 1024)
    } else if let Some(num_str) = size_str.strip_suffix("m") {
        num_str.parse::<u64>().ok().map(|n| n * 1024 * 1024)
    } else if let Some(num_str) = size_str.strip_suffix("g") {
        num_str.parse::<u64>().ok().map(|n| n * 1024 * 1024 * 1024)
    } else {
        size_str.parse::<u64>().ok()
    }
}

fn parse_duration(duration_str: &str) -> Option<std::time::Duration> {
    let duration_str = duration_str.trim().to_lowercase();
    
    if let Some(num_str) = duration_str.strip_suffix("s") {
        num_str.parse::<u64>().ok().map(std::time::Duration::from_secs)
    } else if let Some(num_str) = duration_str.strip_suffix("m") {
        num_str.parse::<u64>().ok().map(|n| std::time::Duration::from_secs(n * 60))
    } else if let Some(num_str) = duration_str.strip_suffix("h") {
        num_str.parse::<u64>().ok().map(|n| std::time::Duration::from_secs(n * 3600))
    } else if let Some(num_str) = duration_str.strip_suffix("d") {
        num_str.parse::<u64>().ok().map(|n| std::time::Duration::from_secs(n * 86400))
    } else {
        duration_str.parse::<u64>().ok().map(std::time::Duration::from_secs)
    }
}

fn is_binary_file(path: &Path) -> bool {
    // Basic binary file detection
    if let Ok(mut file) = std::fs::File::open(path) {
        use std::io::Read;
        let mut buffer = [0; 512];
        if let Ok(bytes_read) = file.read(&mut buffer) {
            // Check for null bytes (common in binary files)
            return buffer[..bytes_read].contains(&0);
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024"), Some(1024));
        assert_eq!(parse_size("1K"), Some(1024));
        assert_eq!(parse_size("1M"), Some(1024 * 1024));
        assert_eq!(parse_size("1G"), Some(1024 * 1024 * 1024));
    }
    
    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("60s"), Some(std::time::Duration::from_secs(60)));
        assert_eq!(parse_duration("1m"), Some(std::time::Duration::from_secs(60)));
        assert_eq!(parse_duration("1h"), Some(std::time::Duration::from_secs(3600)));
        assert_eq!(parse_duration("1d"), Some(std::time::Duration::from_secs(86400)));
    }
}