use crate::{Cli, ColorChoice, FileMatch, LineMatch, OutputFormat, SearchStats};
use serde_json::json;
use std::io::{self, Write};
use termcolor::{Color, ColorChoice as TermColorChoice, ColorSpec, StandardStream, WriteColor};

pub struct OutputFormatter {
    cli: Cli,
    stdout: StandardStream,
}

impl OutputFormatter {
    pub fn new(cli: Cli) -> Self {
        let color_choice = match cli.color {
            ColorChoice::Always => TermColorChoice::Always,
            ColorChoice::Never => TermColorChoice::Never,
            ColorChoice::Auto => TermColorChoice::Auto,
        };
        
        Self {
            cli,
            stdout: StandardStream::stdout(color_choice),
        }
    }
    
    pub fn print_results(&mut self, file_matches: &[FileMatch], stats: &SearchStats) -> io::Result<()> {
        match self.cli.output {
            OutputFormat::Text => self.print_text_results(file_matches, stats),
            OutputFormat::Json => self.print_json_results(file_matches, stats),
            OutputFormat::Csv => self.print_csv_results(file_matches),
            OutputFormat::Xml => self.print_xml_results(file_matches),
        }
    }
    
    fn print_text_results(&mut self, file_matches: &[FileMatch], stats: &SearchStats) -> io::Result<()> {
        // Handle special output modes
        if self.cli.stats_only {
            return self.print_stats_only(stats);
        }
        
        if self.cli.count_only {
            let total_matches: usize = file_matches.iter().map(|fm| fm.total_matches).sum();
            println!("{}", total_matches);
            return Ok(());
        }
        
        if self.cli.files_only {
            for file_match in file_matches {
                if file_match.has_matches() {
                    println!("{}", file_match.path.display());
                }
            }
            return Ok(());
        }
        
        // Regular output
        for file_match in file_matches {
            if !file_match.has_matches() {
                continue;
            }
            
            self.print_file_header(&file_match.path.display().to_string())?;
            
            let (before_context, after_context) = self.cli.effective_context();
            
            for line_match in &file_match.line_matches {
                self.print_line_match(line_match, before_context, after_context)?;
            }
            
            // Add separator between files
            if file_matches.len() > 1 {
                println!();
            }
        }
        
        // Print stats if not in quiet mode
        if !self.cli.files_only && !self.cli.count_only {
            self.print_summary_stats(stats)?;
        }
        
        Ok(())
    }
    
    fn print_file_header(&mut self, filename: &str) -> io::Result<()> {
        if self.cli.should_use_color() {
            self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Magenta)).set_bold(true))?;
            write!(self.stdout, "{}", filename)?;
            self.stdout.reset()?;
            writeln!(self.stdout)?;
        } else {
            println!("{}", filename);
        }
        Ok(())
    }
    
    fn print_line_match(&mut self, line_match: &LineMatch, _before: usize, _after: usize) -> io::Result<()> {
        // Line number
        if self.cli.line_numbers {
            if self.cli.should_use_color() {
                self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
                write!(self.stdout, "{}:", line_match.line_number)?;
                self.stdout.reset()?;
            } else {
                print!("{}:", line_match.line_number);
            }
        }
        
        // Print line with highlighted matches
        let mut last_end = 0;
        let line_text = &line_match.line_text;
        
        for match_info in &line_match.matches {
            // Print text before match
            print!("{}", &line_text[last_end..match_info.start]);
            
            // Print highlighted match
            if self.cli.should_use_color() {
                self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Red)).set_bold(true))?;
                write!(self.stdout, "{}", &match_info.text)?;
                self.stdout.reset()?;
            } else {
                print!("{}", &match_info.text);
            }
            
            last_end = match_info.end;
        }
        
        // Print remaining text
        println!("{}", &line_text[last_end..]);
        
        Ok(())
    }
    
    fn print_stats_only(&mut self, stats: &SearchStats) -> io::Result<()> {
        println!("Files searched: {}", stats.files_searched);
        println!("Files with matches: {}", stats.files_with_matches);
        println!("Total matches: {}", stats.total_matches);
        println!("Total lines: {}", stats.total_lines);
        println!("Elapsed time: {:.3}s", stats.elapsed_time.as_secs_f64());
        println!("Search rate: {:.1} files/s", stats.search_rate);
        Ok(())
    }
    
    fn print_summary_stats(&mut self, stats: &SearchStats) -> io::Result<()> {
        if self.cli.should_use_color() {
            self.stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            writeln!(
                self.stdout,
                "Searched {} files in {:.3}s ({} matches found)",
                stats.files_searched,
                stats.elapsed_time.as_secs_f64(),
                stats.total_matches
            )?;
            self.stdout.reset()?;
        } else {
            println!(
                "Searched {} files in {:.3}s ({} matches found)",
                stats.files_searched,
                stats.elapsed_time.as_secs_f64(),
                stats.total_matches
            );
        }
        Ok(())
    }
    
    fn print_json_results(&mut self, file_matches: &[FileMatch], stats: &SearchStats) -> io::Result<()> {
        let mut json_files = Vec::new();
        
        for file_match in file_matches {
            if !file_match.has_matches() {
                continue;
            }
            
            let mut json_lines = Vec::new();
            for line_match in &file_match.line_matches {
                let mut json_matches = Vec::new();
                for match_info in &line_match.matches {
                    json_matches.push(json!({
                        "start": match_info.start,
                        "end": match_info.end,
                        "text": match_info.text
                    }));
                }
                
                json_lines.push(json!({
                    "line_number": line_match.line_number,
                    "line_text": line_match.line_text,
                    "matches": json_matches
                }));
            }
            
            json_files.push(json!({
                "path": file_match.path.display().to_string(),
                "total_matches": file_match.total_matches,
                "lines": json_lines
            }));
        }
        
        let result = json!({
            "files": json_files,
            "stats": {
                "files_searched": stats.files_searched,
                "files_with_matches": stats.files_with_matches,
                "total_matches": stats.total_matches,
                "total_lines": stats.total_lines,
                "elapsed_time_seconds": stats.elapsed_time.as_secs_f64(),
                "search_rate_files_per_second": stats.search_rate
            }
        });
        
        println!("{}", serde_json::to_string_pretty(&result).unwrap());
        Ok(())
    }
    
    fn print_csv_results(&mut self, file_matches: &[FileMatch]) -> io::Result<()> {
        println!("file,line_number,line_text,match_start,match_end,match_text");
        
        for file_match in file_matches {
            if !file_match.has_matches() {
                continue;
            }
            
            for line_match in &file_match.line_matches {
                for match_info in &line_match.matches {
                    println!(
                        "\"{}\",{},\"{}\",{},{},\"{}\"",
                        file_match.path.display(),
                        line_match.line_number,
                        line_match.line_text.replace('"', "\"\""),
                        match_info.start,
                        match_info.end,
                        match_info.text.replace('"', "\"\"")
                    );
                }
            }
        }
        
        Ok(())
    }
    
    fn print_xml_results(&mut self, file_matches: &[FileMatch]) -> io::Result<()> {
        println!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        println!("<search_results>");
        
        for file_match in file_matches {
            if !file_match.has_matches() {
                continue;
            }
            
            println!("  <file path=\"{}\" total_matches=\"{}\">", 
                     html_escape(&file_match.path.display().to_string()),
                     file_match.total_matches);
            
            for line_match in &file_match.line_matches {
                println!("    <line number=\"{}\">", line_match.line_number);
                println!("      <text>{}</text>", html_escape(&line_match.line_text));
                
                for match_info in &line_match.matches {
                    println!("      <match start=\"{}\" end=\"{}\">{}</match>",
                             match_info.start,
                             match_info.end,
                             html_escape(&match_info.text));
                }
                
                println!("    </line>");
            }
            
            println!("  </file>");
        }
        
        println!("</search_results>");
        Ok(())
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineMatch, Match};
    use std::path::PathBuf;
    
    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("Hello & <world>"), "Hello &amp; &lt;world&gt;");
        assert_eq!(html_escape("\"test\""), "&quot;test&quot;");
    }
    
    #[test]
    fn test_output_formatter_creation() {
        let cli = Cli {
            color: ColorChoice::Never,
            output: OutputFormat::Text,
            ..Default::default()
        };
        
        let formatter = OutputFormatter::new(cli);
        // Just test that it creates without panicking
        assert!(true);
    }
}