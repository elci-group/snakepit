#!/usr/bin/env python3
"""
Smart Snakepit Package Handler - Main Integration Script

Unified interface for the Smart Snakepit Package Handling Strategy implementing:
1. Ingest - Download package into temporary container
2. Test/Collaborate - Validate package functionality  
3. Kill/Destroy - Remove failed packages
4. Conscript/Install - Install successful packages locally

This script integrates:
- Core package handler (snakepit_handler.py)
- CLI interface (snakepit_cli.py)
- Validation framework (validation_framework.py)
- Configuration management (smart_config.py)
- Integration with existing Rust snakepit

Author: adminx
"""

import sys
import os
import subprocess
from pathlib import Path
from typing import Optional, List, Dict

# Add current directory to Python path for imports
sys.path.insert(0, str(Path(__file__).parent))

try:
    from snakepit_handler import SnakepitHandler, PackageStatus
    from snakepit_cli import SnakepitCLI
    from validation_framework import ValidationFramework, ValidationLevel
    from smart_config import SmartConfig, HandlerConfig, ProjectConfig
except ImportError as e:
    print(f"❌ Import error: {e}")
    print("Make sure all required modules are in the same directory")
    sys.exit(1)


class SmartSnakepitIntegration:
    """Main integration class for Smart Snakepit Package Handler"""
    
    def __init__(self, config_path: Optional[str] = None):
        self.config_manager = SmartConfig(config_path)
        self.handler: Optional[SnakepitHandler] = None
        self.validation_framework: Optional[ValidationFramework] = None
        self.cli: Optional[SnakepitCLI] = None
        
    def initialize(self):
        """Initialize all components"""
        print("🐍 Initializing Smart Snakepit Package Handler...")
        
        # Load configuration
        config = self.config_manager.get_merged_config()
        
        # Initialize handler
        self.handler = SnakepitHandler(config_path=self.config_manager.config_path)
        
        # Initialize validation framework
        self.validation_framework = ValidationFramework(self.handler.logger)
        
        # Initialize CLI
        self.cli = SnakepitCLI()
        
        # Detect container engine
        container_engine = self.handler.container_engine
        if container_engine:
            print(f"🐳 Detected container engine: {container_engine}")
        else:
            print("📦 Using virtual environment sandbox (no container engine)")
            
        # Check Rust integration
        rust_binary = self.config_manager.detect_rust_binary()
        if rust_binary:
            print(f"🦀 Rust snakepit binary found: {rust_binary}")
        else:
            print("⚠️  Rust snakepit binary not found (Python-only mode)")
            
        print("✅ Smart Snakepit initialized successfully!")
        
    def handle_package_smart(self, 
                           package_name: str,
                           version: str = "",
                           validation_level: str = "standard",
                           test_script: Optional[str] = None) -> bool:
        """
        Smart package handling with all safety checks
        
        Args:
            package_name: Name of package to handle
            version: Specific version (optional)
            validation_level: Validation strictness (basic/standard/comprehensive)
            test_script: Custom test script (optional)
            
        Returns:
            True if package successfully installed, False otherwise
        """
        if not self.handler:
            raise RuntimeError("Handler not initialized. Call initialize() first.")
            
        print(f"\n🚀 Starting Smart Snakepit handling for {package_name}")
        print("=" * 60)
        
        try:
            # Phase 1: Ingest
            print("📥 Phase 1: INGEST - Creating sandbox environment...")
            pkg_meta = self.handler.ingest(package_name, version, test_script)
            
            if pkg_meta.status == PackageStatus.FAILED:
                print(f"❌ Ingestion failed: {pkg_meta.error_log}")
                self.handler.kill_destroy(package_name)
                return False
                
            print(f"✅ Package {package_name} ingested successfully")
            print(f"   Sandbox ID: {pkg_meta.sandbox_id}")
            
            # Phase 2: Test/Collaborate
            print("\n🧪 Phase 2: TEST/COLLABORATE - Validating package...")
            test_success = self.handler.test_collaborate(package_name)
            
            if not test_success:
                print(f"❌ Validation failed:")
                for error in pkg_meta.error_log[-3:]:
                    print(f"   • {error}")
                self.handler.kill_destroy(package_name)
                return False
                
            print("✅ Package validation successful")
            
            # Display validation results
            if pkg_meta.validation_results.get('stdout'):
                print("📊 Validation Output:")
                for line in pkg_meta.validation_results['stdout'].split('\n')[:10]:
                    if line.strip():
                        print(f"   {line}")
                        
            # Phase 4: Conscript (Phase 3 cleanup happens automatically on success)
            print("\n⚔️  Phase 4: CONSCRIPT - Installing package locally...")
            install_success = self.handler.conscript_install(package_name)
            
            if install_success:
                print(f"✅ Package {package_name} successfully installed and registered")
                
                # Update project configuration
                self.config_manager.add_dependency(
                    name=package_name,
                    version=version,
                    validation_score=1.0  # Successful validation
                )
                
                return True
            else:
                print(f"❌ Installation failed: {pkg_meta.error_log}")
                return False
                
        except Exception as e:
            print(f"❌ Unexpected error during smart handling: {e}")
            # Cleanup on error
            try:
                self.handler.kill_destroy(package_name)
            except:
                pass
            return False
            
    def validate_only(self, 
                     package_name: str, 
                     version: str = "",
                     validation_level: str = "comprehensive") -> Dict:
        """
        Validate package without installing (test-only mode)
        
        Args:
            package_name: Name of package to validate
            version: Specific version (optional)
            validation_level: Validation strictness
            
        Returns:
            Validation results dictionary
        """
        if not self.handler:
            raise RuntimeError("Handler not initialized")
            
        print(f"🔍 Validating {package_name} (test-only mode)")
        
        # Enable dry-run mode
        original_dry_run = self.handler.config.get('dry_run', False)
        self.handler.config['dry_run'] = True
        
        try:
            # Ingest and test
            pkg_meta = self.handler.ingest(package_name, version)
            if pkg_meta.status == PackageStatus.FAILED:
                return {'success': False, 'errors': pkg_meta.error_log}
                
            test_success = self.handler.test_collaborate(package_name)
            
            # Cleanup
            self.handler.kill_destroy(package_name)
            
            return {
                'success': test_success,
                'package_name': package_name,
                'version': version,
                'validation_results': pkg_meta.validation_results,
                'errors': pkg_meta.error_log,
                'warnings': pkg_meta.success_log
            }
            
        finally:
            # Restore original dry-run setting
            self.handler.config['dry_run'] = original_dry_run
            
    def status_report(self) -> Dict:
        """Generate comprehensive status report"""
        if not self.handler:
            raise RuntimeError("Handler not initialized")
            
        active_packages = self.handler.list_packages()
        config = self.config_manager.get_merged_config()
        
        report = {
            'smart_snakepit_version': '1.0.0',
            'configuration': {
                'sandbox_dir': config.get('sandbox_dir'),
                'container_engine': self.handler.container_engine,
                'validation_timeout': config.get('validation_timeout'),
                'dry_run_mode': config.get('dry_run', False),
                'security_scan': config.get('security_scan', True),
            },
            'active_packages': len(active_packages),
            'package_details': {},
            'system_info': {
                'python_version': sys.version,
                'platform': sys.platform,
                'cwd': str(Path.cwd()),
            }
        }
        
        # Add package details
        for pkg_name, pkg_meta in active_packages.items():
            report['package_details'][pkg_name] = {
                'status': pkg_meta.status.value,
                'version': pkg_meta.version,
                'sandbox_id': pkg_meta.sandbox_id,
                'ingest_time': pkg_meta.ingest_time,
                'test_time': pkg_meta.test_time,
                'error_count': len(pkg_meta.error_log),
                'success_count': len(pkg_meta.success_log)
            }
            
        return report
        
    def cleanup_all(self):
        """Cleanup all active packages and sandboxes"""
        if self.handler:
            print("🧹 Cleaning up all active packages...")
            self.handler.cleanup_all()
            print("✅ Cleanup complete")
            
    def run_cli(self, args: Optional[List[str]] = None):
        """Run the CLI interface"""
        if not self.cli:
            self.cli = SnakepitCLI()
        return self.cli.run(args)


def create_example_config():
    """Create example configuration files"""
    config_manager = SmartConfig()
    
    print("Creating example Smart Snakepit configuration...")
    config_manager.create_sample_config()
    
    # Show configuration
    project_config = config_manager.load_project_config()
    print("\n📋 Example Project Configuration:")
    print(f"   Name: {project_config.name}")
    print(f"   Python: {project_config.python_version}")
    print(f"   Dependencies: {len(project_config.dependencies)}")
    print(f"   Dev Dependencies: {len(project_config.dev_dependencies)}")
    

def demo_smart_handling():
    """Demonstrate Smart Snakepit package handling"""
    print("🎭 Smart Snakepit Package Handler Demo")
    print("=" * 50)
    
    # Initialize
    integration = SmartSnakepitIntegration()
    integration.initialize()
    
    # Demo packages to try (safe, well-known packages)
    demo_packages = [
        {"name": "click", "version": "8.0.0", "description": "Command line interface library"},
        {"name": "colorama", "version": "", "description": "Cross-platform colored terminal text"},
        {"name": "requests", "version": "2.31.0", "description": "HTTP library"},
    ]
    
    print(f"\n📦 Demo: Testing {len(demo_packages)} packages")
    
    results = {}
    for i, pkg_info in enumerate(demo_packages, 1):
        print(f"\n--- Demo {i}/{len(demo_packages)}: {pkg_info['name']} ---")
        print(f"Description: {pkg_info['description']}")
        
        try:
            # Use validation-only mode for demo
            result = integration.validate_only(
                package_name=pkg_info['name'],
                version=pkg_info['version']
            )
            results[pkg_info['name']] = result
            
            if result['success']:
                print(f"✅ {pkg_info['name']} - Validation PASSED")
            else:
                print(f"❌ {pkg_info['name']} - Validation FAILED")
                if result.get('errors'):
                    for error in result['errors'][-2:]:  # Show last 2 errors
                        print(f"   Error: {error}")
                        
        except Exception as e:
            print(f"❌ Demo error for {pkg_info['name']}: {e}")
            results[pkg_info['name']] = {'success': False, 'errors': [str(e)]}
    
    # Summary
    successful = sum(1 for r in results.values() if r.get('success', False))
    print(f"\n📊 Demo Results: {successful}/{len(demo_packages)} packages passed validation")
    
    # Cleanup
    integration.cleanup_all()
    
    return results


def main():
    """Main entry point with command handling"""
    import argparse
    
    parser = argparse.ArgumentParser(
        prog='smart-snakepit',
        description='Smart Snakepit Package Handler - Safe Python package management',
        epilog="""
Examples:
  smart-snakepit install requests --version 2.31.0
  smart-snakepit validate numpy --comprehensive
  smart-snakepit demo
  smart-snakepit status
  smart-snakepit init-config
        """
    )
    
    # Global arguments
    parser.add_argument('--config', '-c', help='Configuration file path')
    parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
    
    subparsers = parser.add_subparsers(dest='command', help='Available commands')
    
    # Install command
    install_parser = subparsers.add_parser('install', help='Install package with smart validation')
    install_parser.add_argument('package', help='Package name')
    install_parser.add_argument('--version', help='Package version')
    install_parser.add_argument('--validation-level', default='standard', 
                               choices=['basic', 'standard', 'comprehensive'],
                               help='Validation strictness level')
    
    # Validate command
    validate_parser = subparsers.add_parser('validate', help='Validate package without installing')
    validate_parser.add_argument('package', help='Package name')
    validate_parser.add_argument('--version', help='Package version')
    validate_parser.add_argument('--comprehensive', action='store_true', help='Comprehensive validation')
    
    # Status command
    status_parser = subparsers.add_parser('status', help='Show system status')
    
    # Demo command
    demo_parser = subparsers.add_parser('demo', help='Run demonstration')
    
    # Config commands
    config_parser = subparsers.add_parser('init-config', help='Initialize configuration')
    
    # CLI passthrough
    cli_parser = subparsers.add_parser('cli', help='Use full CLI interface')
    cli_parser.add_argument('cli_args', nargs='*', help='CLI arguments')
    
    args = parser.parse_args()
    
    if not args.command:
        parser.print_help()
        return 1
    
    try:
        # Handle commands
        if args.command == 'install':
            integration = SmartSnakepitIntegration(args.config)
            integration.initialize()
            
            success = integration.handle_package_smart(
                package_name=args.package,
                version=args.version or "",
                validation_level=args.validation_level
            )
            return 0 if success else 1
            
        elif args.command == 'validate':
            integration = SmartSnakepitIntegration(args.config)
            integration.initialize()
            
            level = 'comprehensive' if args.comprehensive else 'standard'
            result = integration.validate_only(
                package_name=args.package,
                version=args.version or "",
                validation_level=level
            )
            
            if result['success']:
                print(f"✅ {args.package} validation PASSED")
                return 0
            else:
                print(f"❌ {args.package} validation FAILED")
                for error in result.get('errors', []):
                    print(f"   Error: {error}")
                return 1
                
        elif args.command == 'status':
            integration = SmartSnakepitIntegration(args.config)
            integration.initialize()
            
            report = integration.status_report()
            print("📊 Smart Snakepit Status Report")
            print("=" * 40)
            print(f"Version: {report['smart_snakepit_version']}")
            print(f"Active packages: {report['active_packages']}")
            print(f"Container engine: {report['configuration']['container_engine'] or 'None'}")
            print(f"Sandbox directory: {report['configuration']['sandbox_dir']}")
            print(f"Python: {report['system_info']['python_version'].split()[0]}")
            
            if report['package_details']:
                print("\nActive Packages:")
                for pkg_name, details in report['package_details'].items():
                    print(f"   • {pkg_name} v{details['version']} - {details['status']}")
            
            return 0
            
        elif args.command == 'demo':
            demo_smart_handling()
            return 0
            
        elif args.command == 'init-config':
            create_example_config()
            return 0
            
        elif args.command == 'cli':
            integration = SmartSnakepitIntegration(args.config)
            return integration.run_cli(args.cli_args)
            
        else:
            parser.print_help()
            return 1
            
    except KeyboardInterrupt:
        print("\n🛑 Operation cancelled by user")
        return 130
    except Exception as e:
        print(f"❌ Error: {e}")
        if args.verbose:
            import traceback
            traceback.print_exc()
        return 1


if __name__ == "__main__":
    sys.exit(main())