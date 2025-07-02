use crate::{
    Cli, CodeGrepResult, FileWalker, LineMatch, PatternMatcher,
    find_in_text,
};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct FileMatch {
    pub path: PathBuf,
    pub line_matches: Vec<LineMatch>,
    pub total_matches: usize,
}

impl FileMatch {
    pub fn new(path: PathBuf, line_matches: Vec<LineMatch>) -> Self {
        let total_matches = line_matches.iter().map(|lm| lm.matches.len()).sum();
        Self {
            path,
            line_matches,
            total_matches,
        }
    }
    
    pub fn has_matches(&self) -> bool {
        self.total_matches > 0
    }
}

#[derive(Debug)]
pub struct SearchStats {
    pub files_searched: usize,
    pub files_with_matches: usize,
    pub total_matches: usize,
    pub total_lines: usize,
    pub elapsed_time: std::time::Duration,
    pub search_rate: f64, // files per second
}

impl SearchStats {
    pub fn new(
        files_searched: usize,
        files_with_matches: usize,
        total_matches: usize,
        total_lines: usize,
        elapsed_time: std::time::Duration,
    ) -> Self {
        let search_rate = if elapsed_time.as_secs_f64() > 0.0 {
            files_searched as f64 / elapsed_time.as_secs_f64()
        } else {
            0.0
        };
        
        Self {
            files_searched,
            files_with_matches,
            total_matches,
            total_lines,
            elapsed_time,
            search_rate,
        }
    }
}

pub struct SearchEngine {
    matcher: PatternMatcher,
    walker: FileWalker,
    cli: Cli,
}

impl SearchEngine {
    pub fn new(cli: Cli) -> CodeGrepResult<Self> {
        let matcher = PatternMatcher::new(&cli)?;
        let walker = FileWalker::new(&cli);
        
        Ok(Self {
            matcher,
            walker,
            cli,
        })
    }
    
    pub fn search(&self) -> CodeGrepResult<(Vec<FileMatch>, SearchStats)> {
        let start_time = Instant::now();
        let files_searched = Arc::new(AtomicUsize::new(0));
        let total_matches = Arc::new(AtomicUsize::new(0));
        let total_lines = Arc::new(AtomicUsize::new(0));
        
        let results: Arc<Mutex<Vec<FileMatch>>> = Arc::new(Mutex::new(Vec::new()));
        
        // Collect all file paths first
        let file_paths: Vec<_> = self.walker
            .walk()
            .into_iter()
            .filter_map(|path_result| path_result.ok())
            .filter(|path| self.walker.should_include_file(path, &self.cli))
            .collect();
        
        // Search files in parallel
        file_paths.par_iter().for_each(|path| {
            if let Ok(file_match) = self.search_file(path) {
                files_searched.fetch_add(1, Ordering::Relaxed);
                
                if file_match.has_matches() {
                    total_matches.fetch_add(file_match.total_matches, Ordering::Relaxed);
                    total_lines.fetch_add(file_match.line_matches.len(), Ordering::Relaxed);
                    
                    let mut results = results.lock().unwrap();
                    results.push(file_match);
                }
            }
        });
        
        let elapsed_time = start_time.elapsed();
        let files_searched_count = files_searched.load(Ordering::Relaxed);
        let total_matches_count = total_matches.load(Ordering::Relaxed);
        let total_lines_count = total_lines.load(Ordering::Relaxed);
        
        let results = Arc::try_unwrap(results).unwrap().into_inner().unwrap();
        let mut file_matches = results;
        
        // Sort results by file path for consistent output
        file_matches.sort_by(|a, b| a.path.cmp(&b.path));
        
        let stats = SearchStats::new(
            files_searched_count,
            file_matches.len(),
            total_matches_count,
            total_lines_count,
            elapsed_time,
        );
        
        Ok((file_matches, stats))
    }
    
    fn search_file(&self, path: &Path) -> CodeGrepResult<FileMatch> {
        let content = fs::read_to_string(path)?;
        
        // Apply structured search filters if needed
        let filtered_content = if self.cli.is_structured_search() {
            self.apply_structured_filters(&content, path)?
        } else {
            content
        };
        
        let line_matches = find_in_text(&filtered_content, &self.matcher);
        Ok(FileMatch::new(path.to_path_buf(), line_matches))
    }
    
    fn apply_structured_filters(&self, content: &str, path: &Path) -> CodeGrepResult<String> {
        // Basic structured search implementation
        let mut filtered_lines = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        
        for (line_num, line) in lines.iter().enumerate() {
            let mut include_line = true;
            
            // Comments only filter
            if self.cli.comments_only {
                include_line = self.is_comment_line(line, path);
            }
            
            // Imports only filter
            if self.cli.imports_only {
                include_line = self.is_import_line(line, path);
            }
            
            // Function filter (basic implementation)
            if self.cli.functions {
                include_line = self.is_function_line(line, path);
            }
            
            // Specific function filter
            if let Some(ref func_name) = self.cli.in_function {
                include_line = self.is_in_function(line_num, &lines, func_name, path);
            }
            
            // Specific class filter
            if let Some(ref class_name) = self.cli.in_class {
                include_line = self.is_in_class(line_num, &lines, class_name, path);
            }
            
            if include_line {
                filtered_lines.push(*line);
            }
        }
        
        Ok(filtered_lines.join("\n"))
    }
    
    fn is_comment_line(&self, line: &str, path: &Path) -> bool {
        let trimmed = line.trim();
        
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") | Some("js") | Some("ts") | Some("go") | Some("java") | Some("c") | Some("cpp") => {
                trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.ends_with("*/")
            }
            Some("py") => trimmed.starts_with("#"),
            Some("rb") => trimmed.starts_with("#"),
            Some("sh") => trimmed.starts_with("#"),
            _ => trimmed.starts_with("#") || trimmed.starts_with("//"),
        }
    }
    
    fn is_import_line(&self, line: &str, path: &Path) -> bool {
        let trimmed = line.trim();
        
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => trimmed.starts_with("use ") || trimmed.starts_with("extern crate"),
            Some("go") => trimmed.starts_with("import "),
            Some("js") | Some("ts") => {
                trimmed.starts_with("import ") || trimmed.starts_with("const ") && trimmed.contains("require(")
            }
            Some("py") => trimmed.starts_with("import ") || trimmed.starts_with("from "),
            Some("java") => trimmed.starts_with("import "),
            _ => trimmed.contains("import") || trimmed.contains("require"),
        }
    }
    
    fn is_function_line(&self, line: &str, path: &Path) -> bool {
        let trimmed = line.trim();
        
        match path.extension().and_then(|e| e.to_str()) {
            Some("rs") => trimmed.starts_with("fn ") || trimmed.contains(" fn "),
            Some("go") => trimmed.starts_with("func "),
            Some("js") | Some("ts") => {
                trimmed.starts_with("function ") || 
                trimmed.contains("function(") ||
                trimmed.contains("=> ") ||
                (trimmed.contains("(") && trimmed.contains("{"))
            }
            Some("py") => trimmed.starts_with("def "),
            Some("java") => {
                (trimmed.contains("public ") || trimmed.contains("private ") || trimmed.contains("protected ")) &&
                trimmed.contains("(") && trimmed.contains(")")
            }
            _ => trimmed.contains("function") || trimmed.starts_with("def "),
        }
    }
    
    fn is_in_function(&self, line_num: usize, lines: &[&str], func_name: &str, path: &Path) -> bool {
        // Basic implementation: check if we're inside a function with the given name
        // This is a simplified version - a full implementation would need proper parsing
        
        // Look backwards for function definition
        for i in (0..=line_num).rev() {
            let line = lines[i].trim();
            if self.is_function_line(line, path) && line.contains(func_name) {
                // Found the function, now check if we're still inside it
                let mut brace_count = 0;
                for j in i..=line_num {
                    let check_line = lines[j];
                    brace_count += check_line.matches('{').count() as i32;
                    brace_count -= check_line.matches('}').count() as i32;
                }
                return brace_count > 0;
            }
        }
        false
    }
    
    fn is_in_class(&self, line_num: usize, lines: &[&str], class_name: &str, _path: &Path) -> bool {
        // Similar to is_in_function but for classes
        for i in (0..=line_num).rev() {
            let line = lines[i].trim();
            if line.starts_with("class ") && line.contains(class_name) {
                let mut brace_count = 0;
                for j in i..=line_num {
                    let check_line = lines[j];
                    brace_count += check_line.matches('{').count() as i32;
                    brace_count -= check_line.matches('}').count() as i32;
                }
                return brace_count > 0;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;
    
    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let file_path = dir.join(name);
        let mut file = File::create(&file_path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file_path
    }
    
    #[test]
    fn test_search_basic() {
        let temp_dir = TempDir::new().unwrap();
        create_test_file(temp_dir.path(), "test.txt", "Hello world\nThis is a test\nGoodbye");
        
        let cli = Cli {
            pattern: Some("test".to_string()),
            paths: vec![temp_dir.path().to_path_buf()],
            ..Default::default()
        };
        
        let engine = SearchEngine::new(cli).unwrap();
        let (results, stats) = engine.search().unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].total_matches, 1);
        assert!(stats.files_searched > 0);
    }
}