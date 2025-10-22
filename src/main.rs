use clap::Parser;
use anyhow::Result;
use console::style;
use std::path::Path;

mod dependency;
mod resolver;
mod installer;
mod venv;
mod config;
mod cli;
mod daemon;
mod process_monitor;
mod installsnake;
mod game_runner;
mod visual_installer;

use cli::Cli;
use config::{SnakepitConfig, ProjectConfig};
use dependency::{Dependency, ProjectDependencies};
use installer::{PackageInstaller, InstallerBackend};
use venv::{VirtualEnvironmentManager, VenvBackend};
use resolver::DependencyResolver;
use daemon::{DaemonManager, DaemonConfig};
use installsnake::{InstallSnake, SnakeConfig, Theme, InstallEvent};

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
        cli::Commands::Game { theme, fps, width } => {
            play_game(&theme, fps, width).await?;
        }
    }
    
    Ok(())
}

async fn install_package(package: &str, version: Option<&str>, dev: bool, config: &SnakepitConfig) -> Result<()> {
    let backend = match config.default_backend.as_deref() {
        Some("conda") => InstallerBackend::Conda,
        Some("poetry") => InstallerBackend::Poetry,
        _ => InstallerBackend::Pip,
    };

    let installer = PackageInstaller::new()
        .with_backend(backend);

    let dependency = Dependency {
        name: package.to_string(),
        version: version.map(|v| v.to_string()),
        version_constraint: None,
        is_dev: dev,
        source: None,
    };

    println!("{}", style("Installing package...").blue());
    installer.install_package(package, version).await?;
    
    // Update project dependencies if we're in a project directory
    if Path::new("pyproject.toml").exists() || Path::new("requirements.txt").exists() {
        update_project_dependencies(&dependency, config).await?;
    }

    println!("{}", style("✓ Package installed successfully!").green());
    Ok(())
}

async fn uninstall_package(package: &str, config: &SnakepitConfig) -> Result<()> {
    let backend = match config.default_backend.as_deref() {
        Some("conda") => InstallerBackend::Conda,
        Some("poetry") => InstallerBackend::Poetry,
        _ => InstallerBackend::Pip,
    };

    let installer = PackageInstaller::new()
        .with_backend(backend);

    println!("{}", style("Uninstalling package...").blue());
    installer.uninstall_package(package).await?;
    
    println!("{}", style("✓ Package uninstalled successfully!").green());
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
    
    println!("{}", style("Installed packages:").blue());
    for package in packages {
        println!("  • {}", package);
    }
    
    Ok(())
}

async fn sync_dependencies(config: &SnakepitConfig) -> Result<()> {
    println!("{}", style("Syncing dependencies...").blue());
    
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
    
    println!("{}", style("✓ Dependencies synced successfully!").green());
    Ok(())
}

async fn init_project(name: Option<&str>, config: &SnakepitConfig) -> Result<()> {
    let project_name = name.unwrap_or("my-project");
    
    println!("{}", style(format!("Initializing project '{}'...", project_name)).blue());
    
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
        println!("{}", style(format!("✓ Virtual environment created at: {}", venv_path.display())).green());
    }
    
    println!("{}", style("✓ Project initialized successfully!").green());
    println!("{}", style(format!("  Run 'cd {}' to enter the project directory", project_name)).dim());
    
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
            println!("{}", style(format!("✓ Virtual environment '{}' created at: {}", name, venv_path.display())).green());
        }
        cli::VenvCommands::Activate { name } => {
            let python_path = venv_manager.activate_venv(&name).await?;
            println!("{}", style(format!("✓ Virtual environment '{}' activated", name)).green());
            println!("{}", style(format!("Python path: {}", python_path.display())).dim());
        }
        cli::VenvCommands::Delete { name } => {
            venv_manager.delete_venv(&name).await?;
            println!("{}", style(format!("✓ Virtual environment '{}' deleted", name)).green());
        }
        cli::VenvCommands::List => {
            let venvs = venv_manager.list_venvs().await?;
            if venvs.is_empty() {
                println!("{}", style("No virtual environments found").yellow());
            } else {
                println!("{}", style("Available virtual environments:").blue());
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
    println!("{}", style("Updating project dependencies...").dim());
    Ok(())
}

async fn handle_daemon_command(command: cli::DaemonCommands, config: &SnakepitConfig) -> Result<()> {
    let daemon_manager = DaemonManager::new();

    match command {
        cli::DaemonCommands::Start { daemon, config: _config_path } => {
            if daemon {
                println!("{}", style("Starting snakepit daemon in background...").blue());
                // In a real implementation, you'd fork the process here
                daemon_manager.start_daemon(config).await?;
            } else {
                println!("{}", style("Starting snakepit daemon in foreground...").blue());
                daemon_manager.start_daemon(config).await?;
            }
        }
        cli::DaemonCommands::Stop => {
            daemon_manager.stop_daemon().await?;
            println!("{}", style("✓ Daemon stopped").green());
        }
        cli::DaemonCommands::Status => {
            let status = daemon_manager.daemon_status().await?;
            println!("{}", style("Snakepit Daemon Status").blue());
            println!("  Running: {}", if status.running { "✅ Yes" } else { "❌ No" });
            println!("  Daemon ID: {}", status.daemon_id);
            println!("  Error Count: {}", status.error_count);
            println!("  Auto-install: {}", if status.config.auto_install { "✅ Yes" } else { "❌ No" });
            println!("  Check Interval: {}s", status.config.check_interval.as_secs());
        }
        cli::DaemonCommands::Restart => {
            println!("{}", style("Restarting daemon...").yellow());
            daemon_manager.stop_daemon().await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            daemon_manager.start_daemon(config).await?;
            println!("{}", style("✓ Daemon restarted").green());
        }
        cli::DaemonCommands::Test { module } => {
            println!("{}", style(format!("Testing missing module: {}", module)).cyan());
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
                    println!("{}", style(format!("Unknown configuration key: {}", key)).red());
                    return Ok(());
                }
            }
            
            daemon_manager.save_daemon_config(&config).await?;
            println!("{}", style(format!("✓ Set {} = {}", key, value)).green());
        }
        cli::DaemonConfigCommands::Show => {
            let config = daemon_manager.load_daemon_config().await?;
            println!("{}", style("Daemon Configuration:").blue());
            println!("  Auto-install: {}", config.auto_install);
            println!("  Check interval: {}s", config.check_interval.as_secs());
            println!("  Max install attempts: {}", config.max_install_attempts);
            println!("  Whitelist modules: {:?}", config.whitelist_modules);
            println!("  Blacklist modules: {:?}", config.blacklist_modules);
        }
        cli::DaemonConfigCommands::Reset => {
            let default_config = DaemonConfig::default();
            daemon_manager.save_daemon_config(&default_config).await?;
            println!("{}", style("✓ Configuration reset to defaults").green());
        }
    }
    
    Ok(())
}

async fn play_game(theme_str: &str, fps: u32, width: usize) -> Result<()> {
    use game_runner::GameRunner;
    
    let theme = match theme_str {
        "amber" => Theme::AmberTerminal,
        "matrix" => Theme::Matrix,
        "minimal" => Theme::MonochromeMinimal,
        "error" => Theme::ErrorPunk,
        _ => Theme::RetroGreen,
    };

    let height = 15;
    let config = SnakeConfig {
        width,
        height,
        fps,
        theme,
        sound: false,
        show_debug: false,
    };

    let mut runner = GameRunner::new(config);
    runner.run_demo(15)?;

    Ok(())
}
