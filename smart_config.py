#!/usr/bin/env python3
"""
Smart Snakepit Configuration Management

Manages configuration for the Smart Snakepit Package Handler system with:
- TOML configuration loading/saving
- Integration with existing Rust snakepit
- Environment variable support
- Project-specific settings
- Dependency tracking

Author: adminx
"""

import os
import sys
import json
import toml
from pathlib import Path
from typing import Dict, List, Optional, Any, Union
from dataclasses import dataclass, field, asdict
from datetime import datetime
import logging


@dataclass
class HandlerConfig:
    """Smart handler configuration settings"""
    # Sandbox settings
    sandbox_dir: str = "/tmp/snakepit-sandbox"
    container_image: str = "python:3.11-slim"
    validation_timeout: int = 60
    max_retries: int = 3
    
    # Behavior settings
    auto_install: bool = True
    dry_run: bool = False
    verbose: bool = True
    log_level: str = "INFO"
    
    # Security settings
    security_scan: bool = True
    allow_network_packages: bool = True
    allow_file_system_packages: bool = True
    
    # Performance settings
    parallel_validation: bool = False
    max_concurrent: int = 4
    cache_validation_results: bool = True
    
    # Integration settings
    integrate_with_rust: bool = True
    rust_binary_path: Optional[str] = None
    use_existing_venvs: bool = True


@dataclass
class ProjectDependency:
    """Project dependency information"""
    name: str
    version: str = ""
    source: str = "pypi"  # pypi, git, local
    dev: bool = False
    optional: bool = False
    extras: List[str] = field(default_factory=list)
    validation_level: str = "standard"
    last_validated: Optional[str] = None
    validation_score: Optional[float] = None


@dataclass 
class ProjectConfig:
    """Project-specific configuration"""
    name: str = ""
    version: str = "0.1.0"
    description: str = ""
    python_version: str = "3.11"
    backend: str = "pip"
    venv_name: str = ""
    
    # Dependencies
    dependencies: List[ProjectDependency] = field(default_factory=list)
    dev_dependencies: List[ProjectDependency] = field(default_factory=list)
    
    # Scripts
    scripts: Dict[str, str] = field(default_factory=dict)
    
    # Handler settings
    handler: HandlerConfig = field(default_factory=HandlerConfig)


class SmartConfig:
    """Smart configuration manager for Snakepit"""
    
    def __init__(self, config_path: Optional[str] = None):
        self.config_path = Path(config_path) if config_path else Path("snakepit.toml")
        self.global_config_path = Path.home() / ".config" / "snakepit" / "config.toml"
        self.history_dir = Path.home() / ".snakepit"
        
        # Ensure directories exist
        self.global_config_path.parent.mkdir(parents=True, exist_ok=True)
        self.history_dir.mkdir(parents=True, exist_ok=True)
        
        self.project_config: Optional[ProjectConfig] = None
        self.global_config: Optional[HandlerConfig] = None
        self.logger = logging.getLogger(__name__)
        
    def load_global_config(self) -> HandlerConfig:
        """Load global configuration"""
        if self.global_config is not None:
            return self.global_config
            
        default_config = HandlerConfig()
        
        if self.global_config_path.exists():
            try:
                with open(self.global_config_path) as f:
                    config_data = toml.load(f)
                    
                # Update default config with loaded values
                handler_data = config_data.get('handler', {})
                for key, value in handler_data.items():
                    if hasattr(default_config, key):
                        setattr(default_config, key, value)
                        
            except Exception as e:
                self.logger.warning(f"Could not load global config: {e}")
                
        # Override with environment variables
        env_mappings = {
            'SNAKEPIT_SANDBOX_DIR': 'sandbox_dir',
            'SNAKEPIT_CONTAINER_IMAGE': 'container_image', 
            'SNAKEPIT_TIMEOUT': 'validation_timeout',
            'SNAKEPIT_DRY_RUN': 'dry_run',
            'SNAKEPIT_VERBOSE': 'verbose',
            'SNAKEPIT_LOG_LEVEL': 'log_level',
        }
        
        for env_var, config_key in env_mappings.items():
            if env_var in os.environ:
                value = os.environ[env_var]
                # Type conversion
                if config_key in ['validation_timeout', 'max_retries', 'max_concurrent']:
                    value = int(value)
                elif config_key in ['dry_run', 'verbose', 'auto_install', 'security_scan']:
                    value = value.lower() in ['true', '1', 'yes', 'on']
                    
                setattr(default_config, config_key, value)
                
        self.global_config = default_config
        return default_config
        
    def load_project_config(self) -> ProjectConfig:
        """Load project configuration"""
        if self.project_config is not None:
            return self.project_config
            
        default_config = ProjectConfig()
        
        if self.config_path.exists():
            try:
                with open(self.config_path) as f:
                    config_data = toml.load(f)
                    
                # Load basic project info
                for key in ['name', 'version', 'description', 'python_version', 'backend', 'venv_name']:
                    if key in config_data:
                        setattr(default_config, key, config_data[key])
                        
                # Load dependencies
                if 'dependencies' in config_data:
                    deps = []
                    for dep_spec in config_data['dependencies']:
                        dep = self._parse_dependency_spec(dep_spec)
                        deps.append(dep)
                    default_config.dependencies = deps
                    
                if 'dev_dependencies' in config_data:
                    dev_deps = []
                    for dep_spec in config_data['dev_dependencies']:
                        dep = self._parse_dependency_spec(dep_spec, dev=True)
                        dev_deps.append(dep)
                    default_config.dev_dependencies = dev_deps
                    
                # Load scripts
                if 'scripts' in config_data:
                    default_config.scripts = config_data['scripts']
                    
                # Load handler config
                if 'handler' in config_data:
                    handler_data = config_data['handler']
                    handler_config = HandlerConfig()
                    
                    for key, value in handler_data.items():
                        if hasattr(handler_config, key):
                            setattr(handler_config, key, value)
                            
                    default_config.handler = handler_config
                    
            except Exception as e:
                self.logger.warning(f"Could not load project config: {e}")
                
        self.project_config = default_config
        return default_config
        
    def _parse_dependency_spec(self, spec: Union[str, dict], dev: bool = False) -> ProjectDependency:
        """Parse dependency specification"""
        if isinstance(spec, str):
            # Simple string format: "package>=1.0.0"
            if '>=' in spec:
                name, version = spec.split('>=', 1)
                return ProjectDependency(name=name.strip(), version=version.strip(), dev=dev)
            elif '==' in spec:
                name, version = spec.split('==', 1)
                return ProjectDependency(name=name.strip(), version=version.strip(), dev=dev)
            else:
                return ProjectDependency(name=spec.strip(), dev=dev)
        elif isinstance(spec, dict):
            # Dictionary format with extended options
            return ProjectDependency(
                name=spec['name'],
                version=spec.get('version', ''),
                source=spec.get('source', 'pypi'),
                dev=spec.get('dev', dev),
                optional=spec.get('optional', False),
                extras=spec.get('extras', []),
                validation_level=spec.get('validation_level', 'standard')
            )
        else:
            raise ValueError(f"Invalid dependency spec: {spec}")
            
    def save_project_config(self, config: Optional[ProjectConfig] = None):
        """Save project configuration to file"""
        if config is None:
            config = self.project_config
        if config is None:
            return
            
        config_data = {
            'name': config.name,
            'version': config.version,
            'description': config.description,
            'python_version': config.python_version,
            'backend': config.backend,
        }
        
        if config.venv_name:
            config_data['venv_name'] = config.venv_name
            
        # Save dependencies
        if config.dependencies:
            deps = []
            for dep in config.dependencies:
                if dep.version:
                    deps.append(f"{dep.name}>={dep.version}")
                else:
                    deps.append(dep.name)
            config_data['dependencies'] = deps
            
        if config.dev_dependencies:
            dev_deps = []
            for dep in config.dev_dependencies:
                if dep.version:
                    dev_deps.append(f"{dep.name}>={dep.version}")
                else:
                    dev_deps.append(dep.name)
            config_data['dev_dependencies'] = dev_deps
            
        # Save scripts
        if config.scripts:
            config_data['scripts'] = config.scripts
            
        # Save handler config
        handler_dict = asdict(config.handler)
        config_data['handler'] = handler_dict
        
        try:
            with open(self.config_path, 'w') as f:
                toml.dump(config_data, f)
            self.logger.info(f"Saved project config to {self.config_path}")
        except Exception as e:
            self.logger.error(f"Could not save project config: {e}")
            
    def save_global_config(self, config: Optional[HandlerConfig] = None):
        """Save global configuration"""
        if config is None:
            config = self.global_config
        if config is None:
            return
            
        config_data = {'handler': asdict(config)}
        
        try:
            with open(self.global_config_path, 'w') as f:
                toml.dump(config_data, f)
            self.logger.info(f"Saved global config to {self.global_config_path}")
        except Exception as e:
            self.logger.error(f"Could not save global config: {e}")
            
    def add_dependency(self, 
                      name: str, 
                      version: str = "", 
                      dev: bool = False,
                      validation_score: Optional[float] = None):
        """Add dependency to project config"""
        config = self.load_project_config()
        
        # Create new dependency
        dep = ProjectDependency(
            name=name,
            version=version,
            dev=dev,
            last_validated=datetime.now().isoformat(),
            validation_score=validation_score
        )
        
        # Add to appropriate list
        if dev:
            # Remove existing dev dependency with same name
            config.dev_dependencies = [d for d in config.dev_dependencies if d.name != name]
            config.dev_dependencies.append(dep)
        else:
            # Remove existing dependency with same name
            config.dependencies = [d for d in config.dependencies if d.name != name]
            config.dependencies.append(dep)
            
        self.save_project_config(config)
        
    def remove_dependency(self, name: str, dev: bool = False):
        """Remove dependency from project config"""
        config = self.load_project_config()
        
        if dev:
            config.dev_dependencies = [d for d in config.dev_dependencies if d.name != name]
        else:
            config.dependencies = [d for d in config.dependencies if d.name != name]
            
        self.save_project_config(config)
        
    def get_merged_config(self) -> Dict[str, Any]:
        """Get merged configuration (global + project)"""
        global_config = self.load_global_config()
        project_config = self.load_project_config()
        
        # Start with global handler config
        merged = asdict(global_config)
        
        # Override with project handler config
        project_handler = asdict(project_config.handler)
        merged.update(project_handler)
        
        # Add project-specific settings
        merged.update({
            'project_name': project_config.name,
            'project_version': project_config.version,
            'python_version': project_config.python_version,
            'backend': project_config.backend,
            'venv_name': project_config.venv_name,
        })
        
        return merged
        
    def detect_rust_binary(self) -> Optional[str]:
        """Detect Rust snakepit binary location"""
        possible_paths = [
            './target/release/snakepit',
            './target/debug/snakepit',
            'snakepit',  # In PATH
            str(Path.cwd() / 'target' / 'release' / 'snakepit'),
            str(Path.cwd() / 'target' / 'debug' / 'snakepit'),
        ]
        
        for path in possible_paths:
            try:
                # Check if file exists and is executable
                path_obj = Path(path)
                if path_obj.exists() and os.access(path_obj, os.X_OK):
                    return str(path_obj.absolute())
            except:
                continue
                
        # Try to find in PATH
        import shutil
        rust_binary = shutil.which('snakepit')
        if rust_binary:
            return rust_binary
            
        return None
        
    def get_rust_integration_command(self, action: str, package_name: str, **kwargs) -> List[str]:
        """Get command to integrate with Rust snakepit"""
        config = self.get_merged_config()
        
        # Detect Rust binary
        rust_binary = config.get('rust_binary_path') or self.detect_rust_binary()
        if not rust_binary:
            raise RuntimeError("Rust snakepit binary not found")
            
        cmd = [rust_binary, action, package_name]
        
        # Add common flags
        if kwargs.get('version'):
            cmd.extend(['--version', kwargs['version']])
        if kwargs.get('venv'):
            cmd.extend(['--venv', kwargs['venv']])
        if config.get('verbose'):
            cmd.append('--verbose')
            
        return cmd
        
    def create_sample_config(self):
        """Create sample configuration files"""
        # Create sample project config
        sample_project = ProjectConfig(
            name="my-project",
            version="0.1.0", 
            description="Sample project configuration",
            python_version="3.11",
            dependencies=[
                ProjectDependency(name="requests", version="2.31.0"),
                ProjectDependency(name="click", version="8.0.0"),
            ],
            dev_dependencies=[
                ProjectDependency(name="pytest", version="7.0.0", dev=True),
                ProjectDependency(name="black", version="23.0.0", dev=True),
            ],
            scripts={
                "test": "pytest",
                "format": "black .",
                "lint": "flake8 ."
            }
        )
        
        # Save sample configs
        self.project_config = sample_project
        self.save_project_config()
        
        # Create sample global config
        sample_global = HandlerConfig(
            validation_timeout=120,
            security_scan=True,
            auto_install=True,
            verbose=True
        )
        
        self.global_config = sample_global
        self.save_global_config()
        
        print(f"✅ Created sample configuration files:")
        print(f"   Project: {self.config_path}")
        print(f"   Global: {self.global_config_path}")


def main():
    """CLI for configuration management"""
    import argparse
    
    parser = argparse.ArgumentParser(description="Smart Snakepit Configuration Manager")
    parser.add_argument('--config', '-c', help='Config file path')
    
    subparsers = parser.add_subparsers(dest='command', help='Commands')
    
    # Show command
    show_parser = subparsers.add_parser('show', help='Show current configuration')
    show_parser.add_argument('--global-config', action='store_true', help='Show global config')
    show_parser.add_argument('--project', action='store_true', help='Show project config')
    
    # Init command
    init_parser = subparsers.add_parser('init', help='Initialize configuration files')
    
    # Set command
    set_parser = subparsers.add_parser('set', help='Set configuration value')
    set_parser.add_argument('key', help='Configuration key')
    set_parser.add_argument('value', help='Configuration value')
    set_parser.add_argument('--global-config', action='store_true', help='Set global config')
    
    args = parser.parse_args()
    
    config_manager = SmartConfig(args.config)
    
    if args.command == 'show':
        if args.global_config or (not args.project and not args.global_config):
            print("Global Configuration:")
            print("-" * 30)
            global_config = config_manager.load_global_config()
            for key, value in asdict(global_config).items():
                print(f"{key}: {value}")
            print()
            
        if args.project or (not args.project and not args.global_config):
            print("Project Configuration:")
            print("-" * 30)
            project_config = config_manager.load_project_config()
            print(f"name: {project_config.name}")
            print(f"version: {project_config.version}")
            print(f"python_version: {project_config.python_version}")
            print(f"dependencies: {len(project_config.dependencies)}")
            print(f"dev_dependencies: {len(project_config.dev_dependencies)}")
            
    elif args.command == 'init':
        config_manager.create_sample_config()
        
    elif args.command == 'set':
        if args.global_config:
            global_config = config_manager.load_global_config()
            if hasattr(global_config, args.key):
                # Type conversion
                value = args.value
                if args.key in ['validation_timeout', 'max_retries', 'max_concurrent']:
                    value = int(value)
                elif args.key in ['dry_run', 'verbose', 'auto_install', 'security_scan']:
                    value = value.lower() in ['true', '1', 'yes', 'on']
                    
                setattr(global_config, args.key, value)
                config_manager.save_global_config(global_config)
                print(f"✅ Set global {args.key} = {value}")
            else:
                print(f"❌ Unknown global config key: {args.key}")
        else:
            project_config = config_manager.load_project_config()
            if hasattr(project_config, args.key):
                setattr(project_config, args.key, args.value)
                config_manager.save_project_config(project_config)
                print(f"✅ Set project {args.key} = {args.value}")
            else:
                print(f"❌ Unknown project config key: {args.key}")
    else:
        parser.print_help()


if __name__ == "__main__":
    main()