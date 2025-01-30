// src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "todo")]
#[command(about = "A Rust-powered todo app", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task
    Add {
        #[arg(short = 'D', long)]
        description: String,
        #[arg(short, long)]
        due_date: Option<String>,
        #[arg(short, long)]
        tags: Vec<String>,
        #[arg(short, long)]
        priority: Option<String>, // Add this line
    },
    /// List all tasks
    List{
        #[arg(short = 'd', long, help = "Sort by due date")]
        sort_by_due_date: bool,
        #[arg(short = 't', long, help = "Filter by tags", num_args = 1..)]
        tags: Option<Vec<String>>,
        #[arg(short = 'p', long, help = "Sort by priority")]
        sort_by_priority: bool,
    },
    /// undo
    Undo,
    /// Mark as done
    Done {
        #[arg(short, long)]
        id: u32,
    },
    /// Delete a task
    Delete {
        #[arg(short, long)]
        id: u32,
    },
    /// Acheive the things done
    Archive,
    /// List the Acheived things
    ListArchived,
    /// TUI obviously
    Tui,
}
