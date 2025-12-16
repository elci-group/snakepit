use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "snakepit")]
#[command(version)]
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
    /// Search for packages
    Search {
        /// Query string
        query: String,
    },
    /// Show package details
    Show {
        /// Package name
        package: String,
    },
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
    /// Fix a broken command by analyzing its error output
    Fix {
        /// The command to run and analyze (use -- to separate args)
        #[arg(last = true)]
        command: Vec<String>,
    },
    /// Get AI-powered package recommendations
    Recommend {
        /// What you want to do (e.g., "web scraping", "data visualization")
        query: String,
        /// Project context for better recommendations
        #[arg(short, long)]
        context: Option<String>,
    },
    /// Hallucinatory Fangs: Modify module behavior safely
    Fangs {
        #[command(subcommand)]
        action: FangsAction,
    },
    /// Solid Snake: Test on Android devices
    Snake {
        #[command(subcommand)]
        action: SnakeAction,
    },
    /// Manage package snapshots
    Snapshot {
        #[command(subcommand)]
        action: SnapshotAction,
    },
    /// Quantum Nest Management (SnakeEgg)
    Nest {
        #[command(subcommand)]
        command: NestCommands,
    },
    /// Organic Egg Management (SnakeEgg)
    Egg {
        #[command(subcommand)]
        command: EggCommands,
    },
    /// Clutch Management (SnakeEgg)
    Clutch {
        #[command(subcommand)]
        command: ClutchCommands,
    },
    /// Protein Management (SnakeEgg)
    Protein {
        #[command(subcommand)]
        command: ProteinCommands,
    },
}

#[derive(Subcommand)]
pub enum NestCommands {
    /// Initialize a new quantum nest
    Init,
    /// Show nest status
    Status,
    /// Vacuum idle eggs to git ether
    Vacuum {
        /// Maximum idle time (e.g., "24h", "7d")
        #[arg(short, long, default_value = "24h")]
        max_idle: String,
    },
    /// Checkpoint all eggs to git
    Checkpoint,
    /// Observe an egg (materialize from ether)
    Observe {
        /// Egg name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum EggCommands {
    /// Create a new egg
    Create {
        /// Egg name
        name: String,
        /// Species (Service, Worker, etc.)
        #[arg(short, long, default_value = "Service")]
        species: String,
        /// Type (organic, metallic, dual)
        #[arg(short, long, default_value = "dual")]
        r#type: String,
    },
    /// Evolve an egg
    Evolve {
        /// Egg name
        name: String,
        /// Watch mode (continuous evolution)
        #[arg(short, long)]
        watch: bool,
    },
    /// Show egg status
    Status {
        /// Egg name
        name: String,
    },
    /// List all eggs
    List,
}

#[derive(Subcommand)]
pub enum ClutchCommands {
    /// Create a new clutch
    Create {
        /// Clutch name
        name: String,
    },
    /// Add eggs to a clutch
    Add {
        /// Clutch name
        name: String,
        /// Egg names
        eggs: Vec<String>,
    },
    /// Trigger thermal cycle (heat sharing)
    ThermalCycle {
        /// Clutch name
        name: String,
    },
    /// Show clutch status
    Status {
        /// Clutch name
        name: String,
    },
}

#[derive(Subcommand)]
pub enum ProteinCommands {
    /// List available proteins
    List,
    /// Extract proteins from an egg
    Extract {
        /// Egg name
        egg: String,
    },
}

#[derive(Subcommand)]
pub enum SnapshotAction {
    /// List all snapshots
    List,
    /// Restore a snapshot
    Restore {
        /// Snapshot ID
        id: String,
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

#[derive(Subcommand)]
pub enum FangsAction {
    /// Fork a module for modification
    Fork {
        /// Module name
        module: String,
    },
    /// Add logging to a function
    Log {
        /// Module name
        module: String,
        /// Function name
        function: String,
    },
    /// Add retry logic to a function
    Retry {
        /// Module name
        module: String,
        /// Function name
        function: String,
        /// Maximum retry attempts
        #[arg(short, long, default_value = "3")]
        max_attempts: u32,
        /// Backoff time in milliseconds
        #[arg(short, long, default_value = "1000")]
        backoff_ms: u64,
    },
    /// Add caching to a function
    Cache {
        /// Module name
        module: String,
        /// Function name
        function: String,
        /// TTL in seconds
        #[arg(short, long, default_value = "3600")]
        ttl: u64,
    },
    /// Mock function return value
    Mock {
        /// Module name
        module: String,
        /// Function name
        function: String,
        /// Return value (Python expression)
        value: String,
    },
    /// Inject custom code into a function
    Custom {
        /// Module name
        module: String,
        /// Function name
        function: String,
        /// Python code to inject
        code: String,
    },
    /// List all forked modules
    List,
    /// Rollback modifications to a module
    Rollback {
        /// Module name
        module: String,
    },
}

#[derive(Subcommand)]
pub enum SnakeAction {
    /// Discover connected Android devices
    Discover,
    
    /// Connect to device via WiFi
    Connect {
        /// IP address of device
        ip: String,
        
        /// Port (default: 5555)
        #[arg(short, long, default_value = "5555")]
        port: u16,
    },
    
    /// Disconnect from WiFi device
    Disconnect {
        /// IP address of device
        ip: String,
        
        /// Port (default: 5555)
        #[arg(short, long, default_value = "5555")]
        port: u16,
    },
    
    /// Install package on device
    Install {
        /// Device ID
        #[arg(short, long)]
        device: String,
        
        /// Package name
        package: String,
    },
    
    /// Run tests on device
    Test {
        /// Device ID
        #[arg(short, long)]
        device: String,
        
        /// Test file
        test_file: String,
    },
    
    /// Profile performance on device
    Profile {
        /// Device ID
        #[arg(short, long)]
        device: String,
        
        /// Script to profile
        script: String,
    },
    
    /// Stream logs from device
    Logs {
        /// Device ID
        #[arg(short, long)]
        device: String,
    },
}
