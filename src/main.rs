use clap::Parser;
use code_grep::{
    batch_replace_files, Cli, Commands, Config, OutputFormatter, SearchEngine,
};
use std::process;

fn main() {
    let cli = Cli::parse();
    
    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> anyhow::Result<()> {
    // Handle subcommands
    if let Some(command) = &cli.command {
        return handle_subcommand(command, &cli);
    }
    
    // Handle regex checking
    if let Some(ref pattern) = cli.check_regex {
        return check_regex_pattern(pattern);
    }
    
    // Main search functionality
    if cli.pattern.is_none() && cli.and.is_empty() && cli.or.is_empty() {
        eprintln!("Error: No search pattern provided");
        process::exit(1);
    }
    
    // Load configuration
    let _config = Config::find_and_load().unwrap_or_default();
    
    // Create search engine
    let engine = SearchEngine::new(cli.clone())?;
    
    // Perform search
    let (file_matches, stats) = engine.search()?;
    
    // Handle replacement if requested
    if cli.has_replacement() {
        let replacement = cli.replace.as_ref().unwrap();
        let pattern_matcher = code_grep::PatternMatcher::new(&cli)?;
        
        let _replacement_results = batch_replace_files(
            &file_matches,
            &pattern_matcher,
            replacement,
            &cli,
        )?;
        
        if !cli.preview && !cli.interactive && !cli.write {
            println!("Note: Use --preview, --interactive, or --write to apply replacements");
        }
    } else {
        // Regular search output
        let mut formatter = OutputFormatter::new(cli);
        formatter.print_results(&file_matches, &stats)?;
    }
    
    Ok(())
}

fn handle_subcommand(command: &Commands, cli: &Cli) -> anyhow::Result<()> {
    match command {
        Commands::Config { path, default } => {
            if *default {
                let default_config = Config::default();
                let yaml = serde_yaml::to_string(&default_config)?;
                println!("{}", yaml);
            } else if let Some(config_path) = path {
                let config = Config::load_from_file(config_path)?;
                let yaml = serde_yaml::to_string(&config)?;
                println!("{}", yaml);
            } else {
                let config = Config::find_and_load().unwrap_or_default();
                let yaml = serde_yaml::to_string(&config)?;
                println!("{}", yaml);
            }
        }
        Commands::Benchmark { pattern, path, iterations } => {
            run_benchmark(pattern, path.as_ref(), *iterations, cli)?;
        }
        Commands::LangHelp { language } => {
            show_language_help(language.as_deref());
        }
    }
    Ok(())
}

fn check_regex_pattern(pattern: &str) -> anyhow::Result<()> {
    // Test with basic regex
    match regex::Regex::new(pattern) {
        Ok(_) => println!("✓ Valid basic regex pattern"),
        Err(e) => println!("✗ Invalid basic regex: {}", e),
    }
    
    // Test with fancy regex
    match fancy_regex::Regex::new(pattern) {
        Ok(_) => println!("✓ Valid fancy regex pattern"),
        Err(e) => println!("✗ Invalid fancy regex: {}", e),
    }
    
    Ok(())
}

fn run_benchmark(
    pattern: &str,
    path: Option<&std::path::PathBuf>,
    iterations: usize,
    cli: &Cli,
) -> anyhow::Result<()> {
    use std::time::Instant;
    
    let search_path = path.cloned().unwrap_or_else(|| ".".into());
    
    let benchmark_cli = Cli {
        pattern: Some(pattern.to_string()),
        paths: vec![search_path],
        ..cli.clone()
    };
    
    println!("Running benchmark: pattern='{}', iterations={}", pattern, iterations);
    
    let mut total_time = std::time::Duration::ZERO;
    let mut total_files = 0;
    let mut total_matches = 0;
    
    for i in 1..=iterations {
        let start = Instant::now();
        let engine = SearchEngine::new(benchmark_cli.clone())?;
        let (file_matches, stats) = engine.search()?;
        let elapsed = start.elapsed();
        
        total_time += elapsed;
        total_files += stats.files_searched;
        total_matches += stats.total_matches;
        
        println!("Iteration {}: {:.3}s, {} files, {} matches",
                 i, elapsed.as_secs_f64(), stats.files_searched, stats.total_matches);
    }
    
    let avg_time = total_time / iterations as u32;
    let avg_files = total_files / iterations;
    let avg_matches = total_matches / iterations;
    
    println!("\nBenchmark Results:");
    println!("Average time: {:.3}s", avg_time.as_secs_f64());
    println!("Average files: {}", avg_files);
    println!("Average matches: {}", avg_matches);
    println!("Files per second: {:.1}", avg_files as f64 / avg_time.as_secs_f64());
    
    Ok(())
}

fn show_language_help(language: Option<&str>) {
    match language {
        Some("rust") | Some("rs") => {
            println!("Rust Language Support:");
            println!("  Functions: fn name() {{}}");
            println!("  Structs: struct Name {{}}");
            println!("  Enums: enum Name {{}}");
            println!("  Impls: impl Name {{}}");
            println!("  Imports: use module::item;");
            println!("  Comments: // or /* */");
            println!("\nExamples:");
            println!("  cg --functions \"handle\" --type rust");
            println!("  cg --in-function \"main\" --type rust");
            println!("  cg --imports-only \"serde\" --type rust");
        }
        Some("go") => {
            println!("Go Language Support:");
            println!("  Functions: func name() {{}}");
            println!("  Structs: type Name struct {{}}");
            println!("  Interfaces: type Name interface {{}}");
            println!("  Imports: import \"module\"");
            println!("  Comments: // or /* */");
            println!("\nExamples:");
            println!("  cg --functions \"Handle\" --type go");
            println!("  cg --in-function \"main\" --type go");
            println!("  cg --imports-only \"json\" --type go");
        }
        Some("js") | Some("javascript") | Some("ts") | Some("typescript") => {
            println!("JavaScript/TypeScript Language Support:");
            println!("  Functions: function name() {{}} or () => {{}}");
            println!("  Classes: class Name {{}}");
            println!("  Imports: import {{ item }} from 'module'");
            println!("  Comments: // or /* */");
            println!("\nExamples:");
            println!("  cg --functions \"handle\" --type js");
            println!("  cg --in-class \"Component\" --type ts");
            println!("  cg --imports-only \"react\" --type jsx");
        }
        Some("py") | Some("python") => {
            println!("Python Language Support:");
            println!("  Functions: def name():");
            println!("  Classes: class Name:");
            println!("  Imports: import module or from module import item");
            println!("  Comments: #");
            println!("\nExamples:");
            println!("  cg --functions \"handle\" --type py");
            println!("  cg --in-class \"Handler\" --type py");
            println!("  cg --imports-only \"requests\" --type py");
        }
        None => {
            println!("Supported Languages:");
            println!("  rust (rs)          - Rust language");
            println!("  go                 - Go language");
            println!("  javascript (js)    - JavaScript");
            println!("  typescript (ts)    - TypeScript");
            println!("  python (py)        - Python");
            println!("  java              - Java");
            println!("  c                 - C language");
            println!("  cpp (cxx, cc)     - C++");
            println!("\nUse 'cg lang-help <language>' for specific language help");
        }
        Some(lang) => {
            println!("Language '{}' is not specifically supported yet.", lang);
            println!("Generic parsing will be used.");
            println!("Use 'cg lang-help' to see supported languages.");
        }
    }
}
