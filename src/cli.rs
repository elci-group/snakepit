use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "snakepit")]
#[command(about = "A dynamic Rust-based Python dependency installer")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a Python package
    Install {
        /// Package name to install
        package: String,
        /// Specific version to install
        #[arg(short, long)]
        version: Option<String>,
        /// Install as development dependency
        #[arg(short, long)]
        dev: bool,
    },
    /// Uninstall a Python package
    Uninstall {
        /// Package name to uninstall
        package: String,
    },
    /// List installed packages
    List,
    /// Sync dependencies from requirements file
    Sync,
    /// Initialize a new project
    Init {
        /// Project name
        name: Option<String>,
    },
    /// Virtual environment management
    Venv {
        #[command(subcommand)]
        command: VenvCommands,
    },
    /// Daemon management
    Daemon {
        #[command(subcommand)]
        command: DaemonCommands,
    },
    /// Play InstallSnake game (demo)
    Game {
        /// Theme: retro, amber, matrix, minimal, error
        #[arg(short, long, default_value = "retro")]
        theme: String,
        /// FPS (frames per second)
        #[arg(short, long, default_value = "12")]
        fps: u32,
        /// Board width
        #[arg(short, long, default_value = "60")]
        width: usize,
    },
}

#[derive(Subcommand)]
pub enum VenvCommands {
    /// Create a new virtual environment
    Create {
        /// Virtual environment name
        name: String,
        /// Python version to use
        #[arg(short, long)]
        python_version: Option<String>,
    },
    /// Activate a virtual environment
    Activate {
        /// Virtual environment name
        name: String,
    },
    /// Delete a virtual environment
    Delete {
        /// Virtual environment name
        name: String,
    },
    /// List all virtual environments
    List,
}

#[derive(Subcommand)]
pub enum DaemonCommands {
    /// Start the snakepit daemon
    Start {
        /// Run in background
        #[arg(short, long)]
        daemon: bool,
        /// Configuration file path
        #[arg(short, long)]
        config: Option<String>,
    },
    /// Stop the snakepit daemon
    Stop,
    /// Show daemon status
    Status,
    /// Restart the daemon
    Restart,
    /// Simulate a missing module for testing
    Test {
        /// Module name to simulate
        module: String,
    },
    /// Configure daemon settings
    Config {
        #[command(subcommand)]
        command: DaemonConfigCommands,
    },
}

#[derive(Subcommand)]
pub enum DaemonConfigCommands {
    /// Set daemon configuration
    Set {
        /// Configuration key
        key: String,
        /// Configuration value
        value: String,
    },
    /// Show current configuration
    Show,
    /// Reset to default configuration
    Reset,
}
