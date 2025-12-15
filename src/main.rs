use clap::Parser;
use anyhow::{Result, Context};
use crate::native::style::{red, green, yellow, blue, cyan, magenta, bold, dim};
use std::path::Path;

mod dependency;
mod resolver;
mod installer;
mod venv;
mod config;
mod cli;
mod daemon;
mod process_monitor;
mod visual_installer;
mod sandbox;
mod handler;
mod charmer;
mod resolver_ai;
mod system_libs;
mod recommender;
mod hallucinatory_fangs;
mod solid_snake;
mod snakeskin;
mod logger;
mod pep440;
mod solver;
mod markers;
mod lockfile;
mod snake_egg;
mod native;

use cli::Cli;
use config::{SnakepitConfig, ProjectConfig};
use dependency::{Dependency, ProjectDependencies};
use installer::{PackageInstaller, InstallerBackend};
use venv::{VirtualEnvironmentManager, VenvBackend};
use resolver::DependencyResolver;
use daemon::{DaemonManager, DaemonConfig};
use handler::SnakepitHandler;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load configuration
    let config = SnakepitConfig::load().unwrap_or_default();
    
    match cli.command {
        cli::Commands::Install { package, version, dev } => {
            install_package(&package, version.as_deref(), dev, &config).await?;
        }
        cli::Commands::Uninstall { package } => {
            uninstall_package(&package, &config).await?;
        }
        cli::Commands::List => {
            list_packages(&config).await?;
        }
        cli::Commands::Sync => {
            sync_dependencies(&config).await?;
        }
        cli::Commands::Init { name } => {
            init_project(name.as_deref(), &config).await?;
        }
        cli::Commands::Venv { command } => {
            handle_venv_command(command, &config).await?;
        }
        cli::Commands::Daemon { command } => {
            handle_daemon_command(command, &config).await?;
        }
        cli::Commands::Fix { command } => {
            if command.is_empty() {
                println!("{}", yellow("Please provide a command to fix, e.g., 'snakepit fix -- adk'"));
                return Ok(());
            }

            let cmd_str = command.join(" ");
            println!("{}", cyan(format!("🔧 Running command to diagnose: {}", cmd_str)));

            let max_retries = 5;
            let mut attempts = 0;

            loop {
                if attempts >= max_retries {
                    println!("{}", red("❌ Maximum fix attempts reached. Giving up."));
                    break;
                }

                attempts += 1;
                if attempts > 1 {
                    println!("{}", cyan(format!("🔄 Attempt {}/{}: Re-running command...", attempts, max_retries)));
                }

                let output = std::process::Command::new(&command[0])
                    .args(&command[1..])
                    .output();

                match output {
                    Ok(output) => {
                        if output.status.success() {
                            println!("{}", green("✅ Command ran successfully! Fix complete."));
                            return Ok(());
                        }

                        let stderr = String::from_utf8_lossy(&output.stderr);
                        
                        // First, check for system library errors
                        let sys_detector = system_libs::SystemLibDetector::new();
                        if let Some(lib_name) = sys_detector.extract_library_from_error(&stderr) {
                            println!("{}", yellow(format!("🔧 SYSTEM: Detected missing library: {}", lib_name)));
                            
                            if let Some(lib) = sys_detector.find_package(&lib_name) {
                                println!("{}", green(format!("💡 SUGGESTION: Install system package '{}'", lib.package_name)));
                                
                                if let Some(cmd) = sys_detector.get_install_command(&lib) {
                                    println!("{}", bold(format!("\nRun this command:")));
                                    println!("  {}", cyan(&cmd));
                                    println!("\n{}", dim("After installing, press Enter to retry..."));
                                    
                                    let mut input = String::new();
                                    std::io::stdin().read_line(&mut input)?;
                                    
                                    // Continue loop to retry
                                    continue;
                                } else {
                                    println!("{}", yellow("⚠️  Could not determine install command for your OS."));
                                    break;
                                }
                            } else {
                                println!("{}", yellow(format!("⚠️  Unknown system library: {}", lib_name)));
                                println!("{}", dim("This might require manual installation."));
                                break;
                            }
                        }
                        
                        // If not a system library error, try Python package diagnosis
                        println!("{}", magenta("❌ Command failed. Consulting Snake Charmer..."));

                        if let Ok(charmer) = charmer::SnakeCharmer::new() {
                            match charmer.diagnose_error(&cmd_str, &stderr).await {
                                Ok(Some(package)) => {
                                    println!("{}", magenta(format!("🐍 CHARMER: Diagnosis complete. Missing package: {}", package)));
                                    println!("{}", green(format!("💡 Suggestion: Install '{}' to fix the error.", package)));
                                    
                                    // Auto-install
                                    let mut handler = handler::SnakepitHandler::new();
                                    if handler.handle_package(&package, None, None).await? {
                                        println!("{}", green("✅ Fix applied! Verifying..."));
                                        // Loop continues to re-run command
                                    } else {
                                        println!("{}", red("❌ Failed to apply fix."));
                                        break;
                                    }
                                }
                                Ok(None) => {
                                    println!("{}", yellow("🐍 CHARMER: Could not identify a missing package."));
                                    println!("Error output:\n{}", stderr);
                                    break;
                                }
                                Err(e) => {
                                    println!("{}", red(format!("🐍 CHARMER: Diagnosis failed: {}", e)));
                                    break;
                                }
                            }
                        } else {
                            println!("{}", yellow("⚠️  Snake Charmer not available (check GEMINI_API_KEY)."));
                            println!("Error output:\n{}", stderr);
                            break;
                        }
                    }
                    Err(e) => {
                        println!("{}", red(format!("❌ Failed to execute command: {}", e)));
                        break;
                    }
                }
            }
        }
        cli::Commands::Recommend { query, context } => {
            println!("{}", cyan("🔮 ORACLE: Analyzing your request..."));
            
            let recommender = recommender::PackageRecommender::new()
                .context("Failed to initialize recommender (check GEMINI_API_KEY)")?;
            
            let recommendations = recommender.recommend(&query, context.as_deref()).await?;
            
            if recommendations.is_empty() {
                println!("{}", red("❌ No recommendations found. Try rephrasing your query."));
                return Ok(());
            }
            
            recommender.display_recommendations(&recommendations);
            
            match recommender.prompt_install(&recommendations)? {
                Some(package) => {
                    println!("\n{}", cyan(format!("📦 Installing {}...", package)));
                    let mut handler = handler::SnakepitHandler::new();
                    handler.handle_package(&package, None, None).await?;
                }
                None => {
                    println!("{}", dim("Skipped installation."));
                }
            }
        }
        cli::Commands::Fangs { action } => {
            use hallucinatory_fangs::*;
            
            let mut fangs = HallucinatoryFangs::new()?;
            
            match action {
                cli::FangsAction::Fork { module } => {
                    fangs.fork_module(&module)?;
                }
                cli::FangsAction::Log { module, function } => {
                    let fork_dir = fangs.fork_module(&module)?;
                    fangs.add_modification(ModificationRule {
                        target_module: module.clone(),
                        target_function: function.clone(),
                        modification_type: ModificationType::InjectLogging,
                    });
                    fangs.apply_modifications(&fork_dir)?;
                }
                cli::FangsAction::Retry { module, function, max_attempts, backoff_ms } => {
                    let fork_dir = fangs.fork_module(&module)?;
                    fangs.add_modification(ModificationRule {
                        target_module: module.clone(),
                        target_function: function.clone(),
                        modification_type: ModificationType::InjectRetry {
                            max_attempts,
                            backoff_ms,
                        },
                    });
                    fangs.apply_modifications(&fork_dir)?;
                }
                cli::FangsAction::Cache { module, function, ttl } => {
                    let fork_dir = fangs.fork_module(&module)?;
                    fangs.add_modification(ModificationRule {
                        target_module: module.clone(),
                        target_function: function.clone(),
                        modification_type: ModificationType::InjectCache {
                            ttl_seconds: ttl,
                        },
                    });
                    fangs.apply_modifications(&fork_dir)?;
                }
                cli::FangsAction::Mock { module, function, value } => {
                    let fork_dir = fangs.fork_module(&module)?;
                    fangs.add_modification(ModificationRule {
                        target_module: module.clone(),
                        target_function: function.clone(),
                        modification_type: ModificationType::MockReturn {
                            return_value: value.clone(),
                        },
                    });
                    fangs.apply_modifications(&fork_dir)?;
                }
                cli::FangsAction::Custom { module, function, code } => {
                    let fork_dir = fangs.fork_module(&module)?;
                    fangs.add_modification(ModificationRule {
                        target_module: module.clone(),
                        target_function: function.clone(),
                        modification_type: ModificationType::CustomCode {
                            code: code.clone(),
                        },
                    });
                    fangs.apply_modifications(&fork_dir)?;
                }
                cli::FangsAction::List => {
                    let forks = fangs.list_forks()?;
                    if forks.is_empty() {
                        println!("{}", dim("No forked modules found"));
                    } else {
                        println!("{}", cyan(format!("🧪 Forked modules ({}):", forks.len())));
                        for fork in forks {
                            println!("   • {}", fork);
                        }
                    }
                }
                cli::FangsAction::Rollback { module } => {
                    fangs.rollback(&module)?;
                }
            }
        }
        cli::Commands::Snake { action } => {
            use solid_snake::*;
            
            let mut snake = SolidSnakeEngine::new()?;
            
            match action {
                cli::SnakeAction::Discover => {
                    snake.discover_devices().await?;
                }
                cli::SnakeAction::Connect { ip, port } => {
                    snake.connect_wifi(&ip, port).await?;
                }
                cli::SnakeAction::Disconnect { ip, port } => {
                    snake.disconnect_wifi(&ip, port).await?;
                }
                cli::SnakeAction::Install { device, package } => {
                    // First discover devices to populate the list
                    snake.discover_devices().await?;
                    snake.install_package(&device, &package).await?;
                }
                cli::SnakeAction::Test { device, test_file } => {
                    snake.discover_devices().await?;
                    let results = snake.run_tests(&device, &test_file).await?;
                    
                    println!("\n📊 Test Results:");
                    println!("   Duration: {} ms", results.duration_ms);
                    println!("   Status: {}", if results.passed { "✅ PASSED" } else { "❌ FAILED" });
                    
                    if !results.stdout.is_empty() {
                        println!("\n📝 Output:");
                        println!("{}", results.stdout);
                    }
                    
                    if !results.stderr.is_empty() {
                        println!("\n⚠️  Errors:");
                        println!("{}", results.stderr);
                    }
                }
                cli::SnakeAction::Profile { device, script } => {
                    snake.discover_devices().await?;
                    let metrics = snake.profile_performance(&device, &script).await?;
                    
                    println!("\n📊 Performance Profile:");
                    println!("{}", metrics.profile_data);
                }
                cli::SnakeAction::Logs { device } => {
                    snake.discover_devices().await?;
                    snake.stream_logs(&device).await?;
                }
            }
        }
        cli::Commands::Snapshot { action } => {
            use crate::uninstaller::Uninstaller;
            let uninstaller = Uninstaller::new()?;
            
            match action {
                cli::SnapshotAction::List => {
                    let snapshots = uninstaller.list_snapshots().await?;
                    if snapshots.is_empty() {
                        println!("{}", yellow("No snapshots found."));
                    } else {
                        println!("{}", blue("Available snapshots:"));
                        for s in snapshots {
                            println!("  • {} (ID: {})", s.package, s.id);
                        }
                    }
                }
                cli::SnapshotAction::Restore { id } => {
                    uninstaller.restore_snapshot(&id).await?;
                }
            }
        }
    }
    
    Ok(())
}

async fn install_package(package: &str, version: Option<&str>, dev: bool, config: &SnakepitConfig) -> Result<()> {
    // Use Smart Snakepit Handler
    let mut handler = SnakepitHandler::new();
    let success = handler.handle_package(package, version, None).await?;
    
    if success {
        // Update project dependencies if we're in a project directory
        if Path::new("pyproject.toml").exists() || Path::new("requirements.txt").exists() {
            let dependency = Dependency {
                name: package.to_string(),
                version: version.map(|v| v.to_string()),
                version_constraint: None,
                is_dev: dev,
                source: None,
            };
            update_project_dependencies(&dependency, config).await?;
        }
    } else {
        return Err(anyhow::anyhow!("Failed to install package {}", package));
    }

    Ok(())
}

mod uninstaller;

// ... (imports)

async fn uninstall_package(package: &str, config: &SnakepitConfig) -> Result<()> {
    use crate::uninstaller::Uninstaller;
    
    let uninstaller = Uninstaller::new()?;
    
    // 1. Analyze Impact
    let report = uninstaller.analyze_impact(package).await?;
    
    if report.risk_score > 50 {
        println!("{}", yellow(format!("⚠️  High risk detected! Risk Score: {}", report.risk_score)));
        if !report.dependents.is_empty() {
            println!("The following packages depend on '{}':", package);
            for dep in &report.dependents {
                println!("  - {}", dep);
            }
        }
        
        if let Some(analysis) = &report.ai_analysis {
            println!("\n🧠 AI Analysis:\n{}", analysis);
        }
        
        println!("\n{}", dim("Proceeding will break these packages."));
        // In a real CLI, we'd ask for confirmation here.
        // For now, we'll just wait a bit to let the user read.
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    
    // 2. Create Snapshot
    match uninstaller.create_snapshot(package).await {
        Ok(snapshot) => println!("{}", green(format!("✓ Snapshot created: {}", snapshot.id))),
        Err(e) => println!("{}", yellow(format!("⚠️  Failed to create snapshot: {}", e))),
    }
    
    // 3. Uninstall
    println!("{}", blue("Uninstalling package..."));
    uninstaller.uninstall(package).await?;
    
    println!("{}", green("✓ Package uninstalled successfully!"));
    Ok(())
}

async fn list_packages(config: &SnakepitConfig) -> Result<()> {
    let backend = match config.default_backend.as_deref() {
        Some("conda") => InstallerBackend::Conda,
        Some("poetry") => InstallerBackend::Poetry,
        _ => InstallerBackend::Pip,
    };

    let installer = PackageInstaller::new()
        .with_backend(backend);

    let packages = installer.list_installed_packages().await?;
    
    println!("{}", blue("Installed packages:"));
    for package in packages {
        println!("  • {}", package);
    }
    
    Ok(())
}

async fn sync_dependencies(config: &SnakepitConfig) -> Result<()> {
    println!("{}", blue("Syncing dependencies..."));
    
    // Try to load dependencies from various sources
    let project_deps = if Path::new("pyproject.toml").exists() {
        ProjectDependencies::from_pyproject_toml("pyproject.toml")?
    } else if Path::new("requirements.txt").exists() {
        ProjectDependencies::from_requirements_txt("requirements.txt")?
    } else {
        return Err(anyhow::anyhow!("No dependency file found (pyproject.toml or requirements.txt)"));
    };

    let mut resolver = DependencyResolver::new();
    let resolved_deps = resolver.resolve_dependencies(&project_deps).await?;

    let backend = match config.default_backend.as_deref() {
        Some("conda") => InstallerBackend::Conda,
        Some("poetry") => InstallerBackend::Poetry,
        _ => InstallerBackend::Pip,
    };

    let installer = PackageInstaller::new()
        .with_backend(backend);

    // Install all dependencies
    let mut all_deps = resolved_deps.dependencies.clone();
    all_deps.extend(resolved_deps.dev_dependencies.clone());

    installer.install_dependencies(&all_deps).await?;
    
    println!("{}", green("✓ Dependencies synced successfully!"));
    Ok(())
}

async fn init_project(name: Option<&str>, config: &SnakepitConfig) -> Result<()> {
    let project_name = name.unwrap_or("my-project");
    
    println!("{}", blue(format!("Initializing project '{}'...", project_name)));
    
    // Create project directory
    std::fs::create_dir_all(project_name)?;
    
    // Create project configuration
    let project_config = ProjectConfig::new(project_name.to_string())
        .with_python_version(config.python_version.as_deref().unwrap_or("3.9"))
        .with_backend(config.default_backend.as_deref().unwrap_or("pip"));
    
    let config_path = format!("{}/snakepit.toml", project_name);
    project_config.save_to_file(&config_path)?;
    
    // Create basic requirements.txt
    let requirements_path = format!("{}/requirements.txt", project_name);
    std::fs::write(&requirements_path, "# Project dependencies\n")?;
    
    // Create virtual environment if configured
    if let Some(venv_backend) = &config.default_venv_backend {
        let venv_manager = VirtualEnvironmentManager::new()
            .with_backend(match venv_backend.as_str() {
                "conda" => VenvBackend::Conda,
                "poetry" => VenvBackend::Poetry,
                "virtualenv" => VenvBackend::Virtualenv,
                _ => VenvBackend::Venv,
            });
        
        let venv_path = venv_manager.create_venv(project_name, config.python_version.as_deref()).await?;
        println!("{}", green(format!("✓ Virtual environment created at: {}", venv_path.display())));
    }
    
    println!("{}", green("✓ Project initialized successfully!"));
    println!("{}", dim(format!("  Run 'cd {}' to enter the project directory", project_name)));
    
    Ok(())
}

async fn handle_venv_command(command: cli::VenvCommands, config: &SnakepitConfig) -> Result<()> {
    let venv_backend = match config.default_venv_backend.as_deref() {
        Some("conda") => VenvBackend::Conda,
        Some("poetry") => VenvBackend::Poetry,
        Some("virtualenv") => VenvBackend::Virtualenv,
        _ => VenvBackend::Venv,
    };

    let venv_manager = VirtualEnvironmentManager::new()
        .with_backend(venv_backend);

    match command {
        cli::VenvCommands::Create { name, python_version } => {
            let venv_path = venv_manager.create_venv(&name, python_version.as_deref()).await?;
            println!("{}", green(format!("✓ Virtual environment \'{}\' created at: {}", name, venv_path.display())));
        }
        cli::VenvCommands::Activate { name } => {
            let python_path = venv_manager.activate_venv(&name).await?;
            println!("{}", green(format!("✓ Virtual environment '{}' activated", name)));
            println!("{}", dim(format!("Python path: {}", python_path.display())));
        }
        cli::VenvCommands::Delete { name } => {
            venv_manager.delete_venv(&name).await?;
            println!("{}", green(format!("✓ Virtual environment '{}' deleted", name)));
        }
        cli::VenvCommands::List => {
            let venvs = venv_manager.list_venvs().await?;
            if venvs.is_empty() {
                println!("{}", yellow("No virtual environments found"));
            } else {
                println!("{}", blue("Available virtual environments:"));
                for venv in venvs {
                    println!("  • {}", venv);
                }
            }
        }
    }
    
    Ok(())
}

async fn update_project_dependencies(_dependency: &Dependency, _config: &SnakepitConfig) -> Result<()> {
    // This would update the project's dependency files
    // For now, just a placeholder
    println!("{}", dim("Updating project dependencies..."));
    Ok(())
}

async fn handle_daemon_command(command: cli::DaemonCommands, config: &SnakepitConfig) -> Result<()> {
    let daemon_manager = DaemonManager::new();

    match command {
        cli::DaemonCommands::Start { daemon, config: _config_path } => {
            if daemon {
                println!("{}", blue("Starting snakepit daemon in background..."));
                // In a real implementation, you'd fork the process here
                daemon_manager.start_daemon(config).await?;
            } else {
                println!("{}", blue("Starting snakepit daemon in foreground..."));
                daemon_manager.start_daemon(config).await?;
            }
        }
        cli::DaemonCommands::Stop => {
            daemon_manager.stop_daemon().await?;
            println!("{}", green("✓ Daemon stopped"));
        }
        cli::DaemonCommands::Status => {
            let status = daemon_manager.daemon_status().await?;
            println!("{}", blue("Snakepit Daemon Status"));
            println!("  Running: {}", if status.running { "✅ Yes" } else { "❌ No" });
            println!("  Daemon ID: {}", status.daemon_id);
            println!("  Error Count: {}", status.error_count);
            println!("  Auto-install: {}", if status.config.auto_install { "✅ Yes" } else { "❌ No" });
            println!("  Check Interval: {}s", status.config.check_interval.as_secs());
        }
        cli::DaemonCommands::Restart => {
            println!("{}", yellow("Restarting daemon..."));
            daemon_manager.stop_daemon().await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            daemon_manager.start_daemon(config).await?;
            println!("{}", green("✓ Daemon restarted"));
        }
        cli::DaemonCommands::Test { module } => {
            println!("{}", cyan(format!("Testing missing module: {}", module)));
            let daemon_config = daemon_manager.load_daemon_config().await?;
            let daemon = daemon::SnakepitDaemon::new(daemon_config, config);
            daemon.simulate_missing_module(&module).await?;
        }
        cli::DaemonCommands::Config { command } => {
            handle_daemon_config_command(command, &daemon_manager).await?;
        }
    }
    
    Ok(())
}

async fn handle_daemon_config_command(command: cli::DaemonConfigCommands, daemon_manager: &DaemonManager) -> Result<()> {
    match command {
        cli::DaemonConfigCommands::Set { key, value } => {
            let mut config = daemon_manager.load_daemon_config().await?;
            
            match key.as_str() {
                "auto_install" => {
                    config.auto_install = value.parse().unwrap_or(true);
                }
                "check_interval" => {
                    if let Ok(seconds) = value.parse::<u64>() {
                        config.check_interval = std::time::Duration::from_secs(seconds);
                    }
                }
                "max_install_attempts" => {
                    if let Ok(attempts) = value.parse::<u32>() {
                        config.max_install_attempts = attempts;
                    }
                }
                _ => {
                    println!("{}", red(format!("Unknown configuration key: {}", key)));
                    return Ok(());
                }
            }
            
            daemon_manager.save_daemon_config(&config).await?;
            println!("{}", green(format!("✓ Set {} = {}", key, value)));
        }
        cli::DaemonConfigCommands::Show => {
            let config = daemon_manager.load_daemon_config().await?;
            println!("{}", blue("Daemon Configuration:"));
            println!("  Auto-install: {}", config.auto_install);
            println!("  Check interval: {}s", config.check_interval.as_secs());
            println!("  Max install attempts: {}", config.max_install_attempts);
            println!("  Whitelist modules: {:?}", config.whitelist_modules);
            println!("  Blacklist modules: {:?}", config.blacklist_modules);
        }
        cli::DaemonConfigCommands::Reset => {
            let default_config = DaemonConfig::default();
            daemon_manager.save_daemon_config(&default_config).await?;
            println!("{}", green("✓ Configuration reset to defaults"));
        }
    }
    
    Ok(())
}
