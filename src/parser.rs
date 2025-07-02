use crate::CodeGrepResult;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ParsedCode {
    pub functions: Vec<FunctionInfo>,
    pub classes: Vec<ClassInfo>,
    pub imports: Vec<ImportInfo>,
    pub comments: Vec<CommentInfo>,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub signature: String,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub methods: Vec<FunctionInfo>,
}

#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub line: usize,
    pub module: String,
    pub items: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommentInfo {
    pub line: usize,
    pub text: String,
    pub comment_type: CommentType,
}

#[derive(Debug, Clone)]
pub enum CommentType {
    SingleLine,
    MultiLine,
    Documentation,
}

pub struct CodeParser;

impl CodeParser {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse(&self, content: &str, path: &Path) -> CodeGrepResult<ParsedCode> {
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        match extension {
            "rs" => self.parse_rust(content),
            "go" => self.parse_go(content),
            "js" | "ts" | "jsx" | "tsx" => self.parse_javascript(content),
            "py" => self.parse_python(content),
            "java" => self.parse_java(content),
            "c" | "cpp" | "cxx" | "cc" | "h" | "hpp" => self.parse_c_cpp(content),
            _ => self.parse_generic(content),
        }
    }
    
    fn parse_rust(&self, content: &str) -> CodeGrepResult<ParsedCode> {
        let lines: Vec<&str> = content.lines().collect();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();
        let mut comments = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_index = line_num + 1;
            
            // Parse functions
            if trimmed.starts_with("fn ") || trimmed.contains(" fn ") {
                if let Some(func_info) = self.extract_rust_function(trimmed, line_index) {
                    functions.push(func_info);
                }
            }
            
            // Parse structs/enums (treat as classes)
            if trimmed.starts_with("struct ") || trimmed.starts_with("enum ") || trimmed.starts_with("impl ") {
                if let Some(class_info) = self.extract_rust_struct(trimmed, line_index) {
                    classes.push(class_info);
                }
            }
            
            // Parse imports (use statements)
            if trimmed.starts_with("use ") || trimmed.starts_with("extern crate ") {
                if let Some(import_info) = self.extract_rust_import(trimmed, line_index) {
                    imports.push(import_info);
                }
            }
            
            // Parse comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("///") {
                let comment_type = if trimmed.starts_with("///") {
                    CommentType::Documentation
                } else if trimmed.starts_with("/*") {
                    CommentType::MultiLine
                } else {
                    CommentType::SingleLine
                };
                
                comments.push(CommentInfo {
                    line: line_index,
                    text: trimmed.to_string(),
                    comment_type,
                });
            }
        }
        
        Ok(ParsedCode {
            functions,
            classes,
            imports,
            comments,
        })
    }
    
    fn parse_go(&self, content: &str) -> CodeGrepResult<ParsedCode> {
        let lines: Vec<&str> = content.lines().collect();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();
        let mut comments = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_index = line_num + 1;
            
            // Parse functions
            if trimmed.starts_with("func ") {
                if let Some(func_info) = self.extract_go_function(trimmed, line_index) {
                    functions.push(func_info);
                }
            }
            
            // Parse structs/interfaces
            if trimmed.starts_with("type ") && (trimmed.contains(" struct") || trimmed.contains(" interface")) {
                if let Some(class_info) = self.extract_go_type(trimmed, line_index) {
                    classes.push(class_info);
                }
            }
            
            // Parse imports
            if trimmed.starts_with("import ") {
                if let Some(import_info) = self.extract_go_import(trimmed, line_index) {
                    imports.push(import_info);
                }
            }
            
            // Parse comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                let comment_type = if trimmed.starts_with("/*") {
                    CommentType::MultiLine
                } else {
                    CommentType::SingleLine
                };
                
                comments.push(CommentInfo {
                    line: line_index,
                    text: trimmed.to_string(),
                    comment_type,
                });
            }
        }
        
        Ok(ParsedCode {
            functions,
            classes,
            imports,
            comments,
        })
    }
    
    fn parse_javascript(&self, content: &str) -> CodeGrepResult<ParsedCode> {
        let lines: Vec<&str> = content.lines().collect();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();
        let mut comments = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_index = line_num + 1;
            
            // Parse functions
            if trimmed.starts_with("function ") || 
               trimmed.contains("function(") ||
               trimmed.contains("=>") ||
               (trimmed.contains("(") && trimmed.contains("{") && !trimmed.starts_with("if") && !trimmed.starts_with("for")) {
                if let Some(func_info) = self.extract_js_function(trimmed, line_index) {
                    functions.push(func_info);
                }
            }
            
            // Parse classes
            if trimmed.starts_with("class ") {
                if let Some(class_info) = self.extract_js_class(trimmed, line_index) {
                    classes.push(class_info);
                }
            }
            
            // Parse imports
            if trimmed.starts_with("import ") || (trimmed.starts_with("const ") && trimmed.contains("require(")) {
                if let Some(import_info) = self.extract_js_import(trimmed, line_index) {
                    imports.push(import_info);
                }
            }
            
            // Parse comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                let comment_type = if trimmed.starts_with("/*") {
                    CommentType::MultiLine
                } else {
                    CommentType::SingleLine
                };
                
                comments.push(CommentInfo {
                    line: line_index,
                    text: trimmed.to_string(),
                    comment_type,
                });
            }
        }
        
        Ok(ParsedCode {
            functions,
            classes,
            imports,
            comments,
        })
    }
    
    fn parse_python(&self, content: &str) -> CodeGrepResult<ParsedCode> {
        let lines: Vec<&str> = content.lines().collect();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();
        let mut comments = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_index = line_num + 1;
            
            // Parse functions
            if trimmed.starts_with("def ") {
                if let Some(func_info) = self.extract_python_function(trimmed, line_index) {
                    functions.push(func_info);
                }
            }
            
            // Parse classes
            if trimmed.starts_with("class ") {
                if let Some(class_info) = self.extract_python_class(trimmed, line_index) {
                    classes.push(class_info);
                }
            }
            
            // Parse imports
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                if let Some(import_info) = self.extract_python_import(trimmed, line_index) {
                    imports.push(import_info);
                }
            }
            
            // Parse comments
            if trimmed.starts_with("#") {
                comments.push(CommentInfo {
                    line: line_index,
                    text: trimmed.to_string(),
                    comment_type: CommentType::SingleLine,
                });
            }
        }
        
        Ok(ParsedCode {
            functions,
            classes,
            imports,
            comments,
        })
    }
    
    fn parse_java(&self, content: &str) -> CodeGrepResult<ParsedCode> {
        // Simplified Java parsing - similar to other languages
        self.parse_generic(content)
    }
    
    fn parse_c_cpp(&self, content: &str) -> CodeGrepResult<ParsedCode> {
        // Simplified C/C++ parsing - similar to other languages
        self.parse_generic(content)
    }
    
    fn parse_generic(&self, content: &str) -> CodeGrepResult<ParsedCode> {
        // Generic parser that looks for common patterns
        let lines: Vec<&str> = content.lines().collect();
        let mut functions = Vec::new();
        let mut classes = Vec::new();
        let mut imports = Vec::new();
        let mut comments = Vec::new();
        
        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let line_index = line_num + 1;
            
            // Generic function detection
            if trimmed.contains("function") || trimmed.starts_with("def ") || trimmed.starts_with("fn ") {
                functions.push(FunctionInfo {
                    name: "unknown".to_string(),
                    start_line: line_index,
                    end_line: line_index,
                    signature: trimmed.to_string(),
                });
            }
            
            // Generic comment detection
            if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*") {
                comments.push(CommentInfo {
                    line: line_index,
                    text: trimmed.to_string(),
                    comment_type: CommentType::SingleLine,
                });
            }
        }
        
        Ok(ParsedCode {
            functions,
            classes,
            imports,
            comments,
        })
    }
    
    // Helper methods for extracting specific language constructs
    fn extract_rust_function(&self, line: &str, line_num: usize) -> Option<FunctionInfo> {
        if let Some(start) = line.find("fn ") {
            let after_fn = &line[start + 3..];
            if let Some(paren) = after_fn.find('(') {
                let name = after_fn[..paren].trim().to_string();
                return Some(FunctionInfo {
                    name,
                    start_line: line_num,
                    end_line: line_num, // Would need proper parsing to find end
                    signature: line.trim().to_string(),
                });
            }
        }
        None
    }
    
    fn extract_rust_struct(&self, line: &str, line_num: usize) -> Option<ClassInfo> {
        let keywords = ["struct ", "enum ", "impl "];
        for keyword in &keywords {
            if let Some(start) = line.find(keyword) {
                let after_keyword = &line[start + keyword.len()..];
                let name = after_keyword.split_whitespace().next()?.to_string();
                return Some(ClassInfo {
                    name,
                    start_line: line_num,
                    end_line: line_num,
                    methods: Vec::new(),
                });
            }
        }
        None
    }
    
    fn extract_rust_import(&self, line: &str, line_num: usize) -> Option<ImportInfo> {
        if line.starts_with("use ") {
            let module = line[4..].trim_end_matches(';').trim().to_string();
            Some(ImportInfo {
                line: line_num,
                module,
                items: Vec::new(),
            })
        } else {
            None
        }
    }
    
    // Similar extraction methods for other languages...
    fn extract_go_function(&self, line: &str, line_num: usize) -> Option<FunctionInfo> {
        // Simplified Go function extraction
        if let Some(start) = line.find("func ") {
            let after_func = &line[start + 5..];
            if let Some(paren) = after_func.find('(') {
                let name_part = &after_func[..paren];
                let name = if let Some(space) = name_part.find(' ') {
                    name_part[..space].trim().to_string()
                } else {
                    name_part.trim().to_string()
                };
                return Some(FunctionInfo {
                    name,
                    start_line: line_num,
                    end_line: line_num,
                    signature: line.trim().to_string(),
                });
            }
        }
        None
    }
    
    fn extract_go_type(&self, line: &str, line_num: usize) -> Option<ClassInfo> {
        if let Some(start) = line.find("type ") {
            let after_type = &line[start + 5..];
            let name = after_type.split_whitespace().next()?.to_string();
            Some(ClassInfo {
                name,
                start_line: line_num,
                end_line: line_num,
                methods: Vec::new(),
            })
        } else {
            None
        }
    }
    
    fn extract_go_import(&self, line: &str, line_num: usize) -> Option<ImportInfo> {
        if line.starts_with("import ") {
            let module = line[7..].trim().trim_matches('"').to_string();
            Some(ImportInfo {
                line: line_num,
                module,
                items: Vec::new(),
            })
        } else {
            None
        }
    }
    
    fn extract_js_function(&self, line: &str, line_num: usize) -> Option<FunctionInfo> {
        // Very simplified JS function extraction
        let name = if line.contains("function ") {
            line.split("function ").nth(1)?
                .split('(').next()?.trim().to_string()
        } else {
            "anonymous".to_string()
        };
        
        Some(FunctionInfo {
            name,
            start_line: line_num,
            end_line: line_num,
            signature: line.trim().to_string(),
        })
    }
    
    fn extract_js_class(&self, line: &str, line_num: usize) -> Option<ClassInfo> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            let name = after_class.split_whitespace().next()?.to_string();
            Some(ClassInfo {
                name,
                start_line: line_num,
                end_line: line_num,
                methods: Vec::new(),
            })
        } else {
            None
        }
    }
    
    fn extract_js_import(&self, line: &str, line_num: usize) -> Option<ImportInfo> {
        if line.starts_with("import ") {
            let module = line[7..].trim().to_string();
            Some(ImportInfo {
                line: line_num,
                module,
                items: Vec::new(),
            })
        } else {
            None
        }
    }
    
    fn extract_python_function(&self, line: &str, line_num: usize) -> Option<FunctionInfo> {
        if let Some(start) = line.find("def ") {
            let after_def = &line[start + 4..];
            if let Some(paren) = after_def.find('(') {
                let name = after_def[..paren].trim().to_string();
                return Some(FunctionInfo {
                    name,
                    start_line: line_num,
                    end_line: line_num,
                    signature: line.trim().to_string(),
                });
            }
        }
        None
    }
    
    fn extract_python_class(&self, line: &str, line_num: usize) -> Option<ClassInfo> {
        if let Some(start) = line.find("class ") {
            let after_class = &line[start + 6..];
            let name = after_class.split('(').next()?.split(':').next()?.trim().to_string();
            Some(ClassInfo {
                name,
                start_line: line_num,
                end_line: line_num,
                methods: Vec::new(),
            })
        } else {
            None
        }
    }
    
    fn extract_python_import(&self, line: &str, line_num: usize) -> Option<ImportInfo> {
        if line.starts_with("import ") {
            let module = line[7..].trim().to_string();
            Some(ImportInfo {
                line: line_num,
                module,
                items: Vec::new(),
            })
        } else if line.starts_with("from ") {
            if let Some(import_pos) = line.find(" import ") {
                let module = line[5..import_pos].trim().to_string();
                let items_str = &line[import_pos + 8..];
                let items: Vec<String> = items_str.split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                Some(ImportInfo {
                    line: line_num,
                    module,
                    items,
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    
    #[test]
    fn test_rust_function_parsing() {
        let parser = CodeParser::new();
        let content = "fn main() {\n    println!(\"Hello\");\n}";
        let path = PathBuf::from("test.rs");
        
        let parsed = parser.parse(content, &path).unwrap();
        assert_eq!(parsed.functions.len(), 1);
        assert_eq!(parsed.functions[0].name, "main");
    }
    
    #[test]
    fn test_python_function_parsing() {
        let parser = CodeParser::new();
        let content = "def hello_world():\n    print('Hello')";
        let path = PathBuf::from("test.py");
        
        let parsed = parser.parse(content, &path).unwrap();
        assert_eq!(parsed.functions.len(), 1);
        assert_eq!(parsed.functions[0].name, "hello_world");
    }
}