#!/usr/bin/env python3
"""
Smart Snakepit CLI Interface

Command-line interface for the Smart Snakepit Package Handler implementing:
- Package installation with validation
- Status monitoring
- History viewing  
- Configuration management
- Audit trail access

Author: adminx
"""

import sys
import argparse
import json
import time
from pathlib import Path
from typing import Optional, List, Dict, Any
from datetime import datetime

from snakepit_handler import SnakepitHandler, PackageStatus, PackageMetadata


class SnakepitCLI:
    """Command-line interface for Smart Snakepit Package Handler"""
    
    def __init__(self):
        self.handler: Optional[SnakepitHandler] = None
        
    def init_handler(self, config_path: Optional[str] = None, dry_run: bool = False) -> SnakepitHandler:
        """Initialize handler with configuration"""
        if self.handler is None:
            self.handler = SnakepitHandler(config_path)
            if dry_run:
                self.handler.config['dry_run'] = True
        return self.handler
        
    def format_timestamp(self, timestamp: Optional[float]) -> str:
        """Format timestamp for display"""
        if timestamp is None:
            return "N/A"
        return datetime.fromtimestamp(timestamp).strftime('%Y-%m-%d %H:%M:%S')
        
    def format_duration(self, start: Optional[float], end: Optional[float]) -> str:
        """Format duration between timestamps"""
        if start is None or end is None:
            return "N/A"
        duration = end - start
        return f"{duration:.2f}s"
        
    def print_package_status(self, pkg_meta: PackageMetadata, verbose: bool = False):
        """Print package status information"""
        status_icons = {
            PackageStatus.PENDING: "⏳",
            PackageStatus.INGESTING: "📥", 
            PackageStatus.TESTING: "🧪",
            PackageStatus.COLLABORATING: "🤝",
            PackageStatus.APPROVED: "✅",
            PackageStatus.FAILED: "❌",
            PackageStatus.CONSCRIPTED: "⚔️", 
            PackageStatus.DESTROYED: "💀"
        }
        
        icon = status_icons.get(pkg_meta.status, "❓")
        print(f"{icon} {pkg_meta.name} v{pkg_meta.version or 'latest'} - {pkg_meta.status.value}")
        
        if verbose:
            print(f"   Sandbox ID: {pkg_meta.sandbox_id}")
            print(f"   Ingest Time: {self.format_timestamp(pkg_meta.ingest_time)}")
            if pkg_meta.test_time:
                print(f"   Test Time: {self.format_timestamp(pkg_meta.test_time)}")
            if pkg_meta.install_time:
                print(f"   Install Time: {self.format_timestamp(pkg_meta.install_time)}")
                
            if pkg_meta.success_log:
                print("   ✅ Success Log:")
                for log in pkg_meta.success_log:
                    print(f"      • {log}")
                    
            if pkg_meta.error_log:
                print("   ❌ Error Log:")
                for log in pkg_meta.error_log:
                    print(f"      • {log}")
                    
            if pkg_meta.validation_results and verbose:
                print("   🔍 Validation Results:")
                for key, value in pkg_meta.validation_results.items():
                    if key in ['stdout', 'stderr']:
                        continue  # Skip output - too verbose
                    print(f"      {key}: {value}")

    def cmd_install(self, args) -> int:
        """Handle install command"""
        handler = self.init_handler(args.config, args.dry_run)
        
        print(f"🐍 Smart Snakepit: Installing {args.package}")
        if args.version:
            print(f"   Version: {args.version}")
        if args.test_script:
            print(f"   Custom test: {args.test_script}")
            
        success = handler.handle_package(
            package_name=args.package,
            version=args.version or "",
            test_script=args.test_script
        )
        
        if success:
            print(f"✅ Successfully installed {args.package}")
            return 0
        else:
            print(f"❌ Failed to install {args.package}")
            if args.package in handler.active_packages:
                pkg_meta = handler.active_packages[args.package]
                if pkg_meta.error_log:
                    print("Error details:")
                    for error in pkg_meta.error_log[-3:]:  # Show last 3 errors
                        print(f"  • {error}")
            return 1
            
    def cmd_status(self, args) -> int:
        """Handle status command"""
        handler = self.init_handler(args.config)
        
        if args.package:
            # Show specific package status
            if args.package in handler.active_packages:
                pkg_meta = handler.active_packages[args.package]
                print(f"Package: {args.package}")
                self.print_package_status(pkg_meta, verbose=True)
            else:
                print(f"Package {args.package} not found in active packages")
                return 1
        else:
            # Show all active packages
            packages = handler.list_packages()
            if not packages:
                print("No active packages being processed")
                return 0
                
            print("Active packages:")
            for pkg_name, pkg_meta in packages.items():
                self.print_package_status(pkg_meta, verbose=args.verbose)
                print()
                
        return 0
        
    def cmd_history(self, args) -> int:
        """Handle history command"""
        handler = self.init_handler(args.config)
        
        if not handler.history_file.exists():
            print("No package history found")
            return 0
            
        try:
            with open(handler.history_file) as f:
                history = json.load(f)
                
            # Filter history
            filtered_history = history
            if args.package:
                filtered_history = [h for h in history if h['name'] == args.package]
            if args.status:
                filtered_history = [h for h in filtered_history if h['status'] == args.status]
                
            # Limit results
            if args.limit:
                filtered_history = filtered_history[-args.limit:]
                
            if not filtered_history:
                print("No matching history entries found")
                return 0
                
            print(f"Package History ({len(filtered_history)} entries):")
            print("-" * 60)
            
            for entry in filtered_history:
                status_icon = {
                    'approved': '✅', 'failed': '❌', 'conscripted': '⚔️', 
                    'destroyed': '💀', 'pending': '⏳'
                }.get(entry['status'], '❓')
                
                print(f"{status_icon} {entry['name']} v{entry.get('version', 'latest')} - {entry['status']}")
                print(f"   Time: {self.format_timestamp(entry['ingest_time'])}")
                
                if args.verbose:
                    if entry.get('test_time'):
                        duration = self.format_duration(entry['ingest_time'], entry['test_time'])
                        print(f"   Test Duration: {duration}")
                    if entry.get('install_time'):
                        duration = self.format_duration(entry['test_time'], entry['install_time']) 
                        print(f"   Install Duration: {duration}")
                        
                    if entry.get('error_log'):
                        print("   Errors:")
                        for error in entry['error_log'][-2:]:  # Show last 2 errors
                            print(f"      • {error}")
                            
                print()
                
        except Exception as e:
            print(f"Error reading history: {e}")
            return 1
            
        return 0
        
    def cmd_cleanup(self, args) -> int:
        """Handle cleanup command"""
        handler = self.init_handler(args.config)
        
        if args.package:
            # Cleanup specific package
            if args.package in handler.active_packages:
                success = handler.kill_destroy(args.package)
                if success:
                    print(f"✅ Cleaned up {args.package}")
                    return 0
                else:
                    print(f"❌ Failed to cleanup {args.package}")
                    return 1
            else:
                print(f"Package {args.package} not found in active packages")
                return 1
        else:
            # Cleanup all active packages
            packages = list(handler.active_packages.keys())
            if not packages:
                print("No active packages to cleanup")
                return 0
                
            print(f"Cleaning up {len(packages)} active packages...")
            handler.cleanup_all()
            print("✅ All packages cleaned up")
            return 0
            
    def cmd_test(self, args) -> int:
        """Handle test command for existing sandbox"""
        handler = self.init_handler(args.config)
        
        if args.package not in handler.active_packages:
            print(f"Package {args.package} not found in active packages")
            return 1
            
        print(f"🧪 Testing {args.package} in existing sandbox...")
        success = handler.test_collaborate(args.package)
        
        pkg_meta = handler.active_packages[args.package]
        if success:
            print(f"✅ {args.package} passed validation")
            if args.verbose and pkg_meta.validation_results.get('stdout'):
                print("Test Output:")
                print(pkg_meta.validation_results['stdout'])
        else:
            print(f"❌ {args.package} failed validation")
            if pkg_meta.validation_results.get('stderr'):
                print("Error Output:")
                print(pkg_meta.validation_results['stderr'])
                
        return 0 if success else 1
        
    def cmd_config(self, args) -> int:
        """Handle config command"""
        handler = self.init_handler(args.config)
        
        if args.show:
            print("Current Configuration:")
            print("-" * 30)
            for key, value in handler.config.items():
                print(f"{key}: {value}")
            return 0
            
        if args.set:
            key, value = args.set.split('=', 1)
            # Basic type conversion
            if value.lower() in ['true', 'false']:
                value = value.lower() == 'true'
            elif value.isdigit():
                value = int(value)
                
            handler.config[key] = value
            print(f"Set {key} = {value}")
            return 0
            
        return 0
        
    def cmd_validate(self, args) -> int:
        """Handle validate command - test package without installing"""
        handler = self.init_handler(args.config, dry_run=True)
        
        print(f"🧪 Validating {args.package} (test-only mode)")
        
        # Ingest package
        pkg_meta = handler.ingest(args.package, args.version or "", args.test_script)
        if pkg_meta.status == PackageStatus.FAILED:
            print(f"❌ Failed to ingest {args.package}")
            handler.kill_destroy(args.package)
            return 1
            
        # Test package
        success = handler.test_collaborate(args.package)
        
        # Show results
        if success:
            print(f"✅ {args.package} validation successful")
            if args.verbose and pkg_meta.validation_results.get('stdout'):
                print("\nValidation Output:")
                print(pkg_meta.validation_results['stdout'])
        else:
            print(f"❌ {args.package} validation failed")
            if pkg_meta.validation_results.get('stderr'):
                print("\nError Output:")
                print(pkg_meta.validation_results['stderr'])
                
        # Always cleanup in validate mode
        handler.kill_destroy(args.package)
        return 0 if success else 1

    def create_parser(self) -> argparse.ArgumentParser:
        """Create argument parser"""
        parser = argparse.ArgumentParser(
            prog='snakepit-smart',
            description='Smart Snakepit Package Handler - Safe Python package management',
            formatter_class=argparse.RawDescriptionHelpFormatter,
            epilog="""
Examples:
  snakepit-smart install requests --version 2.31.0
  snakepit-smart install numpy --test-script custom_test.py
  snakepit-smart validate pandas --verbose
  snakepit-smart status --package requests
  snakepit-smart history --limit 10
  snakepit-smart cleanup --package requests
  snakepit-smart config --show
            """
        )
        
        # Global arguments
        parser.add_argument('--config', '-c', help='Configuration file path')
        parser.add_argument('--verbose', '-v', action='store_true', help='Verbose output')
        parser.add_argument('--dry-run', action='store_true', help='Dry run mode (no actual installation)')
        
        subparsers = parser.add_subparsers(dest='command', help='Available commands')
        
        # Install command
        install_parser = subparsers.add_parser('install', help='Install a package with validation')
        install_parser.add_argument('package', help='Package name to install')
        install_parser.add_argument('--version', help='Specific version to install')
        install_parser.add_argument('--test-script', help='Custom test script path')
        install_parser.set_defaults(func=self.cmd_install)
        
        # Status command  
        status_parser = subparsers.add_parser('status', help='Show package status')
        status_parser.add_argument('--package', help='Show specific package status')
        status_parser.set_defaults(func=self.cmd_status)
        
        # History command
        history_parser = subparsers.add_parser('history', help='Show package history')
        history_parser.add_argument('--package', help='Filter by package name')
        history_parser.add_argument('--status', help='Filter by status')
        history_parser.add_argument('--limit', type=int, default=20, help='Limit number of results')
        history_parser.set_defaults(func=self.cmd_history)
        
        # Cleanup command
        cleanup_parser = subparsers.add_parser('cleanup', help='Cleanup sandboxes')
        cleanup_parser.add_argument('--package', help='Cleanup specific package')
        cleanup_parser.set_defaults(func=self.cmd_cleanup)
        
        # Test command
        test_parser = subparsers.add_parser('test', help='Test existing package sandbox')
        test_parser.add_argument('package', help='Package to test')
        test_parser.set_defaults(func=self.cmd_test)
        
        # Config command
        config_parser = subparsers.add_parser('config', help='Manage configuration')
        config_group = config_parser.add_mutually_exclusive_group()
        config_group.add_argument('--show', action='store_true', help='Show current configuration')
        config_group.add_argument('--set', help='Set configuration value (key=value)')
        config_parser.set_defaults(func=self.cmd_config)
        
        # Validate command
        validate_parser = subparsers.add_parser('validate', help='Validate package without installing')
        validate_parser.add_argument('package', help='Package name to validate')
        validate_parser.add_argument('--version', help='Specific version to validate')
        validate_parser.add_argument('--test-script', help='Custom test script path')
        validate_parser.set_defaults(func=self.cmd_validate)
        
        return parser

    def run(self, args: Optional[List[str]] = None) -> int:
        """Run CLI with provided arguments"""
        parser = self.create_parser()
        parsed_args = parser.parse_args(args)
        
        if not hasattr(parsed_args, 'func'):
            parser.print_help()
            return 1
            
        try:
            return parsed_args.func(parsed_args)
        except KeyboardInterrupt:
            print("\n🛑 Operation cancelled by user")
            if self.handler:
                print("Cleaning up...")
                self.handler.cleanup_all()
            return 130
        except Exception as e:
            print(f"❌ Unexpected error: {e}")
            if parsed_args.verbose:
                import traceback
                traceback.print_exc()
            return 1


def main():
    """Main entry point"""
    cli = SnakepitCLI()
    sys.exit(cli.run())


if __name__ == "__main__":
    main()