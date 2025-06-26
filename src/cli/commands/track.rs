use anyhow::{Context, Result};
use clap::Args;
use colored::control::set_override;
use glob::glob;
use std::env;
use crate::cli::output::OutputFormatter;
use crate::core::{Language, Severity, DetectionEngine};
use crate::git::GitIntegration;

#[derive(Args)]
pub struct TrackArgs {
    /// Analyze specific file patterns (e.g., "*.ex")
    #[arg(long, value_name = "PATTERN")]
    pub files: Option<String>,
    
    /// Focus on specific language (elixir, javascript, etc.)
    #[arg(long, value_name = "LANG")]
    pub language: Option<Language>,
    
    /// Offer Claude Code automatic fixes (requires Claude Code CLI)
    #[arg(long)]
    pub auto_fix: bool,
    
    /// Show only issues of specified severity (critical, major, warning)
    #[arg(long, value_name = "LEVEL")]
    pub severity: Option<Severity>,
    
    /// Output results in JSON format
    #[arg(long)]
    pub json: bool,
    
    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,
}

pub async fn run(args: TrackArgs) -> Result<()> {
    // Configure colored output
    set_override(!args.no_color);

    // Initialize output formatter
    let formatter = OutputFormatter::new(!args.no_color, args.json);

    // Get files to analyze
    let files = if let Some(pattern) = args.files {
        // Use provided pattern
        glob(&pattern)?
            .filter_map(|entry| entry.ok())
            .map(|path| path.to_string_lossy().to_string())
            .collect()
    } else {
        // Try to use git to find changed files
        match GitIntegration::new(env::current_dir()?) {
            Ok(git) => {
                let changed = git.get_changed_files()?;
                if changed.is_empty() {
                    println!("No changed files detected in current branch");
                    return Ok(());
                }
                changed
            }
            Err(_) => {
                // Fall back to analyzing all files in current directory
                vec![".".to_string()]
            }
        }
    };

    if files.is_empty() {
        println!("No files to analyze");
        return Ok(());
    }

    // Create detection engine with registry
    let mut registry = crate::core::registry::PatternRegistry::new();
    registry.load_built_in_patterns()?;
    let engine = DetectionEngine::new(registry);

    // Run analysis
    let result = engine.scan_files(files)
        .context("Failed to scan files")?;

    // Filter results by severity if requested
    let detections = if let Some(min_severity) = args.severity {
        result.filter_by_severity(min_severity)
    } else {
        result.detections.iter().collect()
    };

    // Display results
    if !args.json {
        println!("🔍 Patingin Analysis Results\n");
    }

    for detection in &detections {
        println!("{}", formatter.format_detection(detection));
    }

    // Display summary
    println!("{}", formatter.format_summary(&result));

    // Handle auto-fix if requested
    if args.auto_fix && !detections.is_empty() {
        let fixable_count = detections.iter()
            .filter(|d| d.can_auto_fix)
            .count();

        if fixable_count > 0 {
            println!("\n💡 {} issues can be auto-fixed with Claude Code", fixable_count);
            // TODO: Implement Claude Code integration
        }
    }

    Ok(())
}