pub mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "patingin",
    version,
    about = "Git-aware code review assistant for anti-pattern detection",
    long_about = "patingin (Tagalog: 'can I look?') - Simple, focused, branch-based code quality checking. \
                  Analyze only what changed, show exactly where problems are."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Browse, search, and manage anti-pattern rules for your projects
    Rules(commands::rules::RulesArgs),
    
    /// Analyze git diff changes for anti-pattern violations  
    Review(commands::review::ReviewArgs),
    
    /// Comprehensive environment and configuration status check
    Setup,
}