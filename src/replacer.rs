use crate::{Cli, CodeGrepResult, FileMatch, PatternMatcher};
use std::fs;
use std::io::{self, Write};

pub struct Replacer {
    pattern_matcher: PatternMatcher,
    replacement: String,
    cli: Cli,
}

#[derive(Debug, Clone)]
pub struct ReplacementResult {
    pub file_path: String,
    pub original_content: String,
    pub new_content: String,
    pub replacements_made: usize,
    pub lines_affected: Vec<usize>,
}

impl Replacer {
    pub fn new(pattern_matcher: PatternMatcher, replacement: String, cli: Cli) -> Self {
        Self {
            pattern_matcher,
            replacement,
            cli,
        }
    }
    
    pub fn replace_in_file(&self, file_match: &FileMatch) -> CodeGrepResult<Option<ReplacementResult>> {
        let original_content = fs::read_to_string(&file_match.path)?;
        let mut new_content;
        let mut replacements_made = 0;
        let mut lines_affected = Vec::new();
        
        // Process line by line to maintain line structure
        let lines: Vec<&str> = original_content.lines().collect();
        let mut new_lines = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            let line_index = line_num + 1;
            let matches = self.pattern_matcher.find_matches(line);
            
            if !matches.is_empty() {
                // Perform replacements in this line
                let mut new_line = line.to_string();
                let mut offset = 0i32;
                
                for match_info in &matches {
                    let start = (match_info.start as i32 + offset) as usize;
                    let end = (match_info.end as i32 + offset) as usize;
                    
                    // Handle regex capture groups if using regex
                    let replacement_text = self.process_replacement(&match_info.text, line);
                    
                    // Replace the match
                    new_line.replace_range(start..end, &replacement_text);
                    
                    // Update offset for subsequent replacements in the same line
                    offset += replacement_text.len() as i32 - match_info.text.len() as i32;
                    
                    replacements_made += 1;
                }
                
                lines_affected.push(line_index);
                new_lines.push(new_line);
            } else {
                new_lines.push(line.to_string());
            }
        }
        
        if replacements_made > 0 {
            new_content = new_lines.join("\n");
            
            Ok(Some(ReplacementResult {
                file_path: file_match.path.display().to_string(),
                original_content,
                new_content,
                replacements_made,
                lines_affected,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn preview_replacement(&self, result: &ReplacementResult) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("File: {}\n", result.file_path));
        output.push_str(&format!("Replacements: {}\n", result.replacements_made));
        output.push_str("---\n");
        
        let original_lines: Vec<&str> = result.original_content.lines().collect();
        let new_lines: Vec<&str> = result.new_content.lines().collect();
        
        for &line_num in &result.lines_affected {
            let idx = line_num - 1;
            if idx < original_lines.len() && idx < new_lines.len() {
                output.push_str(&format!("Line {}: {} -> {}\n", 
                    line_num, 
                    original_lines[idx],
                    new_lines[idx]
                ));
            }
        }
        
        output
    }
    
    pub fn write_replacement(&self, result: &ReplacementResult) -> CodeGrepResult<()> {
        fs::write(&result.file_path, &result.new_content)?;
        Ok(())
    }
    
    pub fn interactive_replacement(&self, results: &[ReplacementResult]) -> CodeGrepResult<Vec<ReplacementResult>> {
        let mut confirmed_results = Vec::new();
        
        for result in results {
            println!("{}", self.preview_replacement(result));
            
            loop {
                print!("Apply this replacement? [y/n/a/q]: ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                let input = input.trim().to_lowercase();
                
                match input.as_str() {
                    "y" | "yes" => {
                        confirmed_results.push(result.clone());
                        break;
                    }
                    "n" | "no" => {
                        println!("Skipped.");
                        break;
                    }
                    "a" | "all" => {
                        // Apply this and all remaining
                        confirmed_results.push(result.clone());
                        for remaining_result in &results[confirmed_results.len()..] {
                            confirmed_results.push(remaining_result.clone());
                        }
                        return Ok(confirmed_results);
                    }
                    "q" | "quit" => {
                        println!("Aborted.");
                        return Ok(confirmed_results);
                    }
                    _ => {
                        println!("Please enter y/n/a/q");
                        continue;
                    }
                }
            }
        }
        
        Ok(confirmed_results)
    }
    
    fn process_replacement(&self, matched_text: &str, _full_line: &str) -> String {
        // Handle basic replacement patterns
        let mut replacement = self.replacement.clone();
        
        // Handle $0 (full match)
        replacement = replacement.replace("$0", matched_text);
        
        // Handle ${0} (full match)
        replacement = replacement.replace("${0}", matched_text);
        
        // For regex replacements, we'd need to handle capture groups here
        // This is a simplified implementation
        
        replacement
    }
}

pub fn batch_replace_files(
    file_matches: &[FileMatch],
    pattern_matcher: &PatternMatcher,
    replacement: &str,
    cli: &Cli,
) -> CodeGrepResult<Vec<ReplacementResult>> {
    let replacer = Replacer::new(
        pattern_matcher.clone(),
        replacement.to_string(),
        cli.clone(),
    );
    
    let mut all_results = Vec::new();
    
    for file_match in file_matches {
        if let Some(result) = replacer.replace_in_file(file_match)? {
            all_results.push(result);
        }
    }
    
    if cli.interactive {
        let confirmed_results = replacer.interactive_replacement(&all_results)?;
        
        if cli.write {
            for result in &confirmed_results {
                replacer.write_replacement(result)?;
                println!("Updated: {}", result.file_path);
            }
        }
        
        Ok(confirmed_results)
    } else if cli.preview {
        // Just show previews, don't write
        for result in &all_results {
            println!("{}", replacer.preview_replacement(result));
        }
        Ok(all_results)
    } else if cli.write {
        // Write all replacements without confirmation
        for result in &all_results {
            replacer.write_replacement(result)?;
            println!("Updated: {}", result.file_path);
        }
        Ok(all_results)
    } else {
        // Default: show preview without writing
        for result in &all_results {
            println!("{}", replacer.preview_replacement(result));
        }
        Ok(all_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineMatch, Match, PatternMatcher};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[test]
    fn test_replacement_basic() {
        let temp_file = NamedTempFile::new().unwrap();
        let content = "Hello world\nThis is a test\nHello again";
        temp_file.as_file().write_all(content.as_bytes()).unwrap();
        
        let file_match = FileMatch {
            path: temp_file.path().to_path_buf(),
            line_matches: vec![
                LineMatch {
                    line_number: 1,
                    line_text: "Hello world".to_string(),
                    matches: vec![Match {
                        start: 0,
                        end: 5,
                        text: "Hello".to_string(),
                    }],
                },
            ],
            total_matches: 1,
        };
        
        let cli = Cli::default();
        let pattern_matcher = PatternMatcher::Literal("Hello".to_string());
        let replacer = Replacer::new(pattern_matcher, "Hi".to_string(), cli);
        
        let result = replacer.replace_in_file(&file_match).unwrap();
        assert!(result.is_some());
        
        let result = result.unwrap();
        assert_eq!(result.replacements_made, 1);
        assert!(result.new_content.contains("Hi world"));
    }
}