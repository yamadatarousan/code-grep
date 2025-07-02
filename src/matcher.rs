use crate::{Cli, CodeGrepError, CodeGrepResult};
use fancy_regex::Regex as FancyRegex;
use regex::Regex;

#[derive(Clone)]
pub enum PatternMatcher {
    Literal(String),
    Basic(Regex),
    Fancy(FancyRegex),
    Multiple(Vec<PatternMatcher>),
}

impl PatternMatcher {
    pub fn new(cli: &Cli) -> CodeGrepResult<Self> {
        // Handle multiple patterns (AND/OR)
        if !cli.and.is_empty() {
            let mut matchers = Vec::new();
            if let Some(ref pattern) = cli.pattern {
                matchers.push(Self::create_single_matcher(pattern, cli)?);
            }
            for pattern in &cli.and {
                matchers.push(Self::create_single_matcher(pattern, cli)?);
            }
            return Ok(PatternMatcher::Multiple(matchers));
        }
        
        if !cli.or.is_empty() {
            let pattern = if let Some(ref p) = cli.pattern {
                format!("({}|{})", p, cli.or.join("|"))
            } else {
                format!("({})", cli.or.join("|"))
            };
            return Self::create_single_matcher(&pattern, cli);
        }
        
        // Single pattern
        if let Some(ref pattern) = cli.pattern {
            Self::create_single_matcher(pattern, cli)
        } else {
            Err(CodeGrepError::Config("No pattern provided".to_string()))
        }
    }
    
    fn create_single_matcher(pattern: &str, cli: &Cli) -> CodeGrepResult<Self> {
        if cli.literal {
            Ok(PatternMatcher::Literal(pattern.to_string()))
        } else if cli.fancy_regex {
            let mut regex_pattern = pattern.to_string();
            
            // Add word boundaries if requested
            if cli.word_boundary {
                regex_pattern = format!(r"\b{}\b", regex_pattern);
            }
            
            let regex = if cli.case_sensitive {
                FancyRegex::new(&regex_pattern)?
            } else {
                FancyRegex::new(&format!("(?i){}", regex_pattern))?
            };
            
            Ok(PatternMatcher::Fancy(regex))
        } else if cli.regex {
            let mut regex_pattern = pattern.to_string();
            
            // Add word boundaries if requested
            if cli.word_boundary {
                regex_pattern = format!(r"\b{}\b", regex_pattern);
            }
            
            let mut builder = regex::RegexBuilder::new(&regex_pattern);
            builder.case_insensitive(!cli.case_sensitive);
            
            let regex = builder.build()?;
            Ok(PatternMatcher::Basic(regex))
        } else {
            // Default: treat as literal unless it contains regex metacharacters
            if pattern.chars().any(|c| matches!(c, '.' | '*' | '+' | '?' | '^' | '$' | '|' | '[' | ']' | '(' | ')' | '{' | '}' | '\\')) {
                Self::create_single_matcher(pattern, &Cli {
                    regex: true,
                    ..cli.clone()
                })
            } else {
                Ok(PatternMatcher::Literal(pattern.to_string()))
            }
        }
    }
    
    pub fn find_matches(&self, text: &str) -> Vec<Match> {
        match self {
            PatternMatcher::Literal(pattern) => {
                let mut matches = Vec::new();
                let search_text = text.to_lowercase();
                let search_pattern = pattern.to_lowercase();
                
                let mut start = 0;
                while let Some(pos) = search_text[start..].find(&search_pattern) {
                    let absolute_pos = start + pos;
                    matches.push(Match {
                        start: absolute_pos,
                        end: absolute_pos + pattern.len(),
                        text: text[absolute_pos..absolute_pos + pattern.len()].to_string(),
                    });
                    start = absolute_pos + 1;
                }
                matches
            }
            PatternMatcher::Basic(regex) => {
                regex.find_iter(text)
                    .map(|m| Match {
                        start: m.start(),
                        end: m.end(),
                        text: m.as_str().to_string(),
                    })
                    .collect()
            }
            PatternMatcher::Fancy(regex) => {
                regex.find_iter(text)
                    .filter_map(|m| m.ok())
                    .map(|m| Match {
                        start: m.start(),
                        end: m.end(),
                        text: m.as_str().to_string(),
                    })
                    .collect()
            }
            PatternMatcher::Multiple(matchers) => {
                // For AND operation, all patterns must match on the same line
                if matchers.iter().all(|matcher| !matcher.find_matches(text).is_empty()) {
                    // Return matches from the first pattern
                    matchers[0].find_matches(text)
                } else {
                    Vec::new()
                }
            }
        }
    }
    
    pub fn is_match(&self, text: &str) -> bool {
        !self.find_matches(text).is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct Match {
    pub start: usize,
    pub end: usize,
    pub text: String,
}

impl Match {
    pub fn len(&self) -> usize {
        self.end - self.start
    }
}

#[derive(Debug, Clone)]
pub struct LineMatch {
    pub line_number: usize,
    pub line_text: String,
    pub matches: Vec<Match>,
}

impl LineMatch {
    pub fn new(line_number: usize, line_text: String, matches: Vec<Match>) -> Self {
        Self {
            line_number,
            line_text,
            matches,
        }
    }
    
    pub fn has_matches(&self) -> bool {
        !self.matches.is_empty()
    }
}

pub fn find_in_text(text: &str, matcher: &PatternMatcher) -> Vec<LineMatch> {
    text.lines()
        .enumerate()
        .filter_map(|(line_num, line)| {
            let matches = matcher.find_matches(line);
            if !matches.is_empty() {
                Some(LineMatch::new(line_num + 1, line.to_string(), matches))
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_cli() -> Cli {
        Cli {
            pattern: Some("test".to_string()),
            paths: vec![],
            regex: false,
            fancy_regex: false,
            case_sensitive: false,
            word_boundary: false,
            literal: false,
            r#type: vec![],
            ext: vec![],
            ignore: vec![],
            respect_gitignore: true,
            hidden: false,
            binary: false,
            max_filesize: None,
            max_depth: None,
            modified_within: None,
            replace: None,
            preview: false,
            interactive: false,
            write: false,
            line_numbers: false,
            context: 0,
            before_context: None,
            after_context: None,
            output: crate::OutputFormat::Text,
            color: crate::ColorChoice::Auto,
            files_only: false,
            count_only: false,
            stats_only: false,
            threads: None,
            max_memory: None,
            fast: false,
            thorough: false,
            functions: false,
            in_function: None,
            in_class: None,
            in_scope: vec![],
            imports_only: false,
            comments_only: false,
            and: vec![],
            or: vec![],
            pattern_file: None,
            lang_rules: None,
            output_filter: None,
            config: None,
            interactive_mode: false,
            fuzzy: false,
            live: false,
            check_regex: None,
            command: None,
        }
    }
    
    #[test]
    fn test_literal_matcher() {
        let cli = Cli { literal: true, ..test_cli() };
        let matcher = PatternMatcher::new(&cli).unwrap();
        
        let matches = matcher.find_matches("This is a test string");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "test");
    }
    
    #[test]
    fn test_regex_matcher() {
        let cli = Cli { 
            pattern: Some(r"\d+".to_string()),
            regex: true, 
            ..test_cli() 
        };
        let matcher = PatternMatcher::new(&cli).unwrap();
        
        let matches = matcher.find_matches("There are 123 numbers and 456 more");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].text, "123");
        assert_eq!(matches[1].text, "456");
    }
}