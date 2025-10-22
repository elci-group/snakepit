#!/usr/bin/env python3
"""
Smart Snakepit Package Handler

Implements the four-phase package management strategy:
1. Ingest - Download package into temporary container
2. Test/Collaborate - Validate package functionality 
3. Kill/Destroy - Remove failed packages
4. Conscript/Install - Install successful packages locally

Author: adminx
"""

import os
import sys
import json
import subprocess
import tempfile
import shutil
import logging
import time
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, field
from enum import Enum
import uuid
import toml


class PackageStatus(Enum):
    """Package validation status"""
    PENDING = "pending"
    INGESTING = "ingesting"  
    TESTING = "testing"
    COLLABORATING = "collaborating"
    FAILED = "failed"
    APPROVED = "approved"
    CONSCRIPTED = "conscripted"
    DESTROYED = "destroyed"


@dataclass
class PackageMetadata:
    """Metadata for package validation process"""
    name: str
    version: str = ""
    status: PackageStatus = PackageStatus.PENDING
    ingest_time: float = field(default_factory=time.time)
    test_time: Optional[float] = None
    install_time: Optional[float] = None
    sandbox_id: str = field(default_factory=lambda: str(uuid.uuid4()))
    test_script: Optional[str] = None
    error_log: List[str] = field(default_factory=list)
    success_log: List[str] = field(default_factory=list)
    dependencies: List[str] = field(default_factory=list)
    validation_results: Dict[str, Any] = field(default_factory=dict)


class SnakepitHandler:
    """Smart package handler implementing the four-phase strategy"""

    def __init__(self, config_path: Optional[str] = None):
        self.config = self._load_config(config_path)
        self.logger = self._setup_logging()
        self.sandbox_dir = Path(self.config.get('sandbox_dir', '/tmp/snakepit-sandbox'))
        self.sandbox_dir.mkdir(exist_ok=True)
        
        # Package tracking
        self.active_packages: Dict[str, PackageMetadata] = {}
        self.history_file = Path(self.config.get('history_file', '~/.snakepit/package_history.json')).expanduser()
        self.history_file.parent.mkdir(exist_ok=True, parents=True)
        
        # Container engine detection
        self.container_engine = self._detect_container_engine()
        
    def _load_config(self, config_path: Optional[str] = None) -> Dict[str, Any]:
        """Load configuration from snakepit.toml or default config"""
        default_config = {
            'sandbox_dir': '/tmp/snakepit-sandbox',
            'container_image': 'python:3.11-slim',
            'validation_timeout': 60,
            'max_retries': 3,
            'auto_install': True,
            'dry_run': False,
            'verbose': True,
            'log_level': 'INFO'
        }
        
        # Try to load from snakepit.toml
        if config_path is None:
            config_path = 'snakepit.toml'
            
        if Path(config_path).exists():
            try:
                with open(config_path) as f:
                    file_config = toml.load(f)
                default_config.update(file_config.get('handler', {}))
            except Exception as e:
                print(f"Warning: Could not load config from {config_path}: {e}")
                
        return default_config
        
    def _setup_logging(self) -> logging.Logger:
        """Setup structured logging"""
        logger = logging.getLogger('snakepit-handler')
        logger.setLevel(getattr(logging, self.config['log_level']))
        
        if not logger.handlers:
            handler = logging.StreamHandler()
            formatter = logging.Formatter(
                '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
            )
            handler.setFormatter(formatter)
            logger.addHandler(handler)
            
        return logger
        
    def _detect_container_engine(self) -> Optional[str]:
        """Detect available container engine (Docker/Podman)"""
        for engine in ['podman', 'docker']:
            try:
                result = subprocess.run([engine, '--version'], 
                                      capture_output=True, text=True, timeout=5)
                if result.returncode == 0:
                    self.logger.info(f"Detected container engine: {engine}")
                    return engine
            except (subprocess.TimeoutExpired, FileNotFoundError):
                continue
                
        self.logger.warning("No container engine detected. Falling back to venv sandbox.")
        return None

    def ingest(self, package_name: str, version: str = "", test_script: Optional[str] = None) -> PackageMetadata:
        """
        Phase 1: Ingest package into temporary container sandbox
        
        Args:
            package_name: Name of package to install
            version: Specific version (optional)
            test_script: Custom test script path (optional)
            
        Returns:
            PackageMetadata for tracking
        """
        self.logger.info(f"🐍 INGEST: Starting ingestion of {package_name}")
        
        # Create package metadata
        pkg_meta = PackageMetadata(
            name=package_name,
            version=version,
            status=PackageStatus.INGESTING,
            test_script=test_script
        )
        
        self.active_packages[package_name] = pkg_meta
        
        try:
            # Create sandbox environment
            sandbox_path = self.sandbox_dir / pkg_meta.sandbox_id
            sandbox_path.mkdir(exist_ok=True)
            
            if self.container_engine:
                success = self._create_container_sandbox(pkg_meta, sandbox_path)
            else:
                success = self._create_venv_sandbox(pkg_meta, sandbox_path)
                
            if success:
                pkg_meta.status = PackageStatus.TESTING
                self.logger.info(f"✅ INGEST: Successfully ingested {package_name}")
            else:
                pkg_meta.status = PackageStatus.FAILED
                pkg_meta.error_log.append("Failed to create sandbox environment")
                self.logger.error(f"❌ INGEST: Failed to ingest {package_name}")
                
        except Exception as e:
            pkg_meta.status = PackageStatus.FAILED
            pkg_meta.error_log.append(f"Ingestion error: {str(e)}")
            self.logger.error(f"❌ INGEST: Error ingesting {package_name}: {e}")
            
        return pkg_meta
        
    def _create_container_sandbox(self, pkg_meta: PackageMetadata, sandbox_path: Path) -> bool:
        """Create containerized sandbox for package testing"""
        try:
            # Create Dockerfile for sandbox
            dockerfile = sandbox_path / "Dockerfile"
            package_spec = f"{pkg_meta.name}=={pkg_meta.version}" if pkg_meta.version else pkg_meta.name
            
            dockerfile_content = f"""
FROM {self.config['container_image']}

# Install package in isolated environment
RUN pip install --no-cache-dir {package_spec}

# Create test workspace
WORKDIR /test
COPY test_script.py /test/

# Default command runs validation
CMD ["python", "/test/test_script.py"]
"""
            dockerfile.write_text(dockerfile_content)
            
            # Create basic test script
            test_script = sandbox_path / "test_script.py"
            if pkg_meta.test_script and Path(pkg_meta.test_script).exists():
                shutil.copy(pkg_meta.test_script, test_script)
            else:
                test_script.write_text(self._generate_default_test(pkg_meta.name))
                
            # Build container
            build_cmd = [
                self.container_engine, 'build',
                '-t', f'snakepit-test-{pkg_meta.sandbox_id}',
                str(sandbox_path)
            ]
            
            result = subprocess.run(build_cmd, capture_output=True, text=True, timeout=120)
            
            if result.returncode == 0:
                pkg_meta.success_log.append("Container sandbox created successfully")
                return True
            else:
                pkg_meta.error_log.append(f"Container build failed: {result.stderr}")
                return False
                
        except Exception as e:
            pkg_meta.error_log.append(f"Container sandbox creation failed: {str(e)}")
            return False
            
    def _create_venv_sandbox(self, pkg_meta: PackageMetadata, sandbox_path: Path) -> bool:
        """Create virtual environment sandbox for package testing"""
        try:
            venv_path = sandbox_path / "venv"
            
            # Create virtual environment
            subprocess.run([sys.executable, '-m', 'venv', str(venv_path)], 
                         check=True, capture_output=True)
            
            # Get pip path
            if sys.platform == "win32":
                pip_path = venv_path / "Scripts" / "pip"
                python_path = venv_path / "Scripts" / "python"
            else:
                pip_path = venv_path / "bin" / "pip"  
                python_path = venv_path / "bin" / "python"
            
            # Install package
            package_spec = f"{pkg_meta.name}=={pkg_meta.version}" if pkg_meta.version else pkg_meta.name
            subprocess.run([str(pip_path), 'install', package_spec], 
                         check=True, capture_output=True, timeout=120)
            
            # Create test script
            test_script = sandbox_path / "test_script.py"
            if pkg_meta.test_script and Path(pkg_meta.test_script).exists():
                shutil.copy(pkg_meta.test_script, test_script)
            else:
                test_script.write_text(self._generate_default_test(pkg_meta.name))
                
            pkg_meta.validation_results['python_path'] = str(python_path)
            pkg_meta.validation_results['sandbox_path'] = str(sandbox_path)
            pkg_meta.success_log.append("Virtual environment sandbox created successfully")
            return True
            
        except subprocess.CalledProcessError as e:
            pkg_meta.error_log.append(f"Venv sandbox creation failed: {e.stderr}")
            return False
        except Exception as e:
            pkg_meta.error_log.append(f"Venv sandbox creation failed: {str(e)}")
            return False
            
    def _generate_default_test(self, package_name: str) -> str:
        """Generate a basic import test for the package"""
        return f'''#!/usr/bin/env python3
"""
Default package validation test for {package_name}
"""
import sys
import traceback

def test_import():
    """Test basic package import"""
    try:
        # Try importing the package
        __import__('{package_name}')
        print(f"✅ Successfully imported {package_name}")
        return True
    except ImportError as e:
        print(f"❌ Failed to import {package_name}: {{e}}")
        return False
    except Exception as e:
        print(f"❌ Unexpected error importing {package_name}: {{e}}")
        traceback.print_exc()
        return False

def test_basic_functionality():
    """Test basic package functionality"""
    try:
        mod = __import__('{package_name}')
        # Check if module has basic attributes
        attrs = dir(mod)
        print(f"📦 Package attributes: {{len(attrs)}} items")
        
        # Look for version info
        for version_attr in ['__version__', 'VERSION', 'version']:
            if hasattr(mod, version_attr):
                version = getattr(mod, version_attr)
                print(f"📌 Version: {{version}}")
                break
                
        return True
    except Exception as e:
        print(f"❌ Functionality test failed: {{e}}")
        return False

if __name__ == "__main__":
    print(f"🧪 Testing package: {package_name}")
    
    import_success = test_import()
    func_success = test_basic_functionality() if import_success else False
    
    if import_success and func_success:
        print("✅ All tests passed!")
        sys.exit(0)
    else:
        print("❌ Tests failed!")
        sys.exit(1)
'''

    def test_collaborate(self, package_name: str) -> bool:
        """
        Phase 2: Test/Collaborate - validate package functionality in sandbox
        
        Args:
            package_name: Name of package to test
            
        Returns:
            True if package passes validation, False otherwise
        """
        if package_name not in self.active_packages:
            self.logger.error(f"Package {package_name} not found in active packages")
            return False
            
        pkg_meta = self.active_packages[package_name]
        self.logger.info(f"🧪 TEST/COLLABORATE: Validating {package_name}")
        
        pkg_meta.status = PackageStatus.COLLABORATING
        pkg_meta.test_time = time.time()
        
        try:
            if self.container_engine:
                success = self._test_in_container(pkg_meta)
            else:
                success = self._test_in_venv(pkg_meta)
                
            if success:
                pkg_meta.status = PackageStatus.APPROVED
                pkg_meta.success_log.append("Package validation successful")
                self.logger.info(f"✅ TEST/COLLABORATE: {package_name} approved for installation")
                return True
            else:
                pkg_meta.status = PackageStatus.FAILED
                self.logger.error(f"❌ TEST/COLLABORATE: {package_name} failed validation")
                return False
                
        except Exception as e:
            pkg_meta.status = PackageStatus.FAILED
            pkg_meta.error_log.append(f"Testing error: {str(e)}")
            self.logger.error(f"❌ TEST/COLLABORATE: Error testing {package_name}: {e}")
            return False
            
    def _test_in_container(self, pkg_meta: PackageMetadata) -> bool:
        """Run validation tests in container"""
        try:
            # Run container with test
            run_cmd = [
                self.container_engine, 'run', '--rm',
                f'snakepit-test-{pkg_meta.sandbox_id}'
            ]
            
            result = subprocess.run(run_cmd, capture_output=True, text=True, 
                                  timeout=self.config['validation_timeout'])
            
            pkg_meta.validation_results['stdout'] = result.stdout
            pkg_meta.validation_results['stderr'] = result.stderr
            pkg_meta.validation_results['returncode'] = result.returncode
            
            if result.returncode == 0:
                pkg_meta.success_log.append("Container validation passed")
                return True
            else:
                pkg_meta.error_log.append(f"Container validation failed: {result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            pkg_meta.error_log.append("Container validation timed out")
            return False
        except Exception as e:
            pkg_meta.error_log.append(f"Container validation error: {str(e)}")
            return False
            
    def _test_in_venv(self, pkg_meta: PackageMetadata) -> bool:
        """Run validation tests in virtual environment"""
        try:
            python_path = pkg_meta.validation_results['python_path']
            sandbox_path = Path(pkg_meta.validation_results['sandbox_path'])
            test_script = sandbox_path / "test_script.py"
            
            # Run test script
            result = subprocess.run([python_path, str(test_script)],
                                  capture_output=True, text=True,
                                  timeout=self.config['validation_timeout'])
            
            pkg_meta.validation_results['stdout'] = result.stdout
            pkg_meta.validation_results['stderr'] = result.stderr
            pkg_meta.validation_results['returncode'] = result.returncode
            
            if result.returncode == 0:
                pkg_meta.success_log.append("Venv validation passed")
                return True
            else:
                pkg_meta.error_log.append(f"Venv validation failed: {result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            pkg_meta.error_log.append("Venv validation timed out")
            return False
        except Exception as e:
            pkg_meta.error_log.append(f"Venv validation error: {str(e)}")
            return False

    def kill_destroy(self, package_name: str) -> bool:
        """
        Phase 3: Kill/Destroy - remove failed packages and cleanup sandbox
        
        Args:
            package_name: Name of package to destroy
            
        Returns:
            True if cleanup successful
        """
        if package_name not in self.active_packages:
            self.logger.warning(f"Package {package_name} not found for destruction")
            return False
            
        pkg_meta = self.active_packages[package_name]
        self.logger.info(f"💀 KILL/DESTROY: Cleaning up {package_name}")
        
        try:
            # Remove container if it exists
            if self.container_engine:
                try:
                    subprocess.run([
                        self.container_engine, 'rmi', '-f',
                        f'snakepit-test-{pkg_meta.sandbox_id}'
                    ], capture_output=True, timeout=30)
                except:
                    pass  # Container might not exist
                    
            # Remove sandbox directory
            sandbox_path = self.sandbox_dir / pkg_meta.sandbox_id
            if sandbox_path.exists():
                shutil.rmtree(sandbox_path)
                
            # Update status and log
            pkg_meta.status = PackageStatus.DESTROYED
            pkg_meta.success_log.append("Package sandbox destroyed")
            
            # Save to history and remove from active
            self._save_package_history(pkg_meta)
            del self.active_packages[package_name]
            
            self.logger.info(f"💀 KILL/DESTROY: Cleaned up {package_name}")
            return True
            
        except Exception as e:
            self.logger.error(f"❌ KILL/DESTROY: Error cleaning up {package_name}: {e}")
            return False

    def conscript_install(self, package_name: str) -> bool:
        """
        Phase 4: Conscript/Install - install approved packages locally
        
        Args:
            package_name: Name of package to install
            
        Returns:
            True if installation successful
        """
        if package_name not in self.active_packages:
            self.logger.error(f"Package {package_name} not found for installation")
            return False
            
        pkg_meta = self.active_packages[package_name]
        
        if pkg_meta.status != PackageStatus.APPROVED:
            self.logger.error(f"Package {package_name} not approved for installation")
            return False
            
        self.logger.info(f"⚔️ CONSCRIPT: Installing {package_name}")
        pkg_meta.install_time = time.time()
        
        try:
            # Use snakepit's existing install mechanism
            package_spec = f"{pkg_meta.name}=={pkg_meta.version}" if pkg_meta.version else pkg_meta.name
            
            if self.config['dry_run']:
                self.logger.info(f"DRY RUN: Would install {package_spec}")
                success = True
            else:
                # Try to find snakepit binary or use pip directly
                snakepit_paths = [
                    './target/release/snakepit',
                    './target/debug/snakepit',
                    'snakepit',  # in PATH
                ]
                
                snakepit_bin = None
                for path in snakepit_paths:
                    try:
                        result = subprocess.run([path, '--version'], 
                                              capture_output=True, timeout=5)
                        if result.returncode == 0:
                            snakepit_bin = path
                            break
                    except (FileNotFoundError, subprocess.TimeoutExpired):
                        continue
                
                if snakepit_bin:
                    # Use snakepit Rust binary
                    self.logger.info(f"Using snakepit binary: {snakepit_bin}")
                    result = subprocess.run([
                        snakepit_bin, 'install', package_spec
                    ], capture_output=True, text=True, timeout=300, 
                       env={**os.environ, 'SNAKEPIT_BYPASS': '1'})
                else:
                    # Fall back to direct pip installation
                    self.logger.info("Snakepit binary not found, using pip directly")
                    result = subprocess.run([
                        sys.executable, '-m', 'pip', 'install', package_spec
                    ], capture_output=True, text=True, timeout=300)
                
                success = result.returncode == 0
                if not success:
                    pkg_meta.error_log.append(f"Installation failed: {result.stderr}")
                    
            if success:
                pkg_meta.status = PackageStatus.CONSCRIPTED
                pkg_meta.success_log.append("Package installed successfully")
                self._update_dependency_graph(pkg_meta)
                self.logger.info(f"✅ CONSCRIPT: Successfully installed {package_name}")
                
                # Cleanup sandbox after successful install
                self.kill_destroy(package_name)
                return True
            else:
                pkg_meta.status = PackageStatus.FAILED
                self.logger.error(f"❌ CONSCRIPT: Failed to install {package_name}")
                return False
                
        except Exception as e:
            pkg_meta.status = PackageStatus.FAILED
            pkg_meta.error_log.append(f"Installation error: {str(e)}")
            self.logger.error(f"❌ CONSCRIPT: Error installing {package_name}: {e}")
            return False

    def _update_dependency_graph(self, pkg_meta: PackageMetadata):
        """Update snakepit.toml with new dependency"""
        try:
            config_path = Path('snakepit.toml')
            if config_path.exists():
                with open(config_path) as f:
                    config = toml.load(f)
            else:
                config = {'dependencies': []}
                
            # Add dependency if not already present
            package_spec = f"{pkg_meta.name}>={pkg_meta.version}" if pkg_meta.version else pkg_meta.name
            if 'dependencies' not in config:
                config['dependencies'] = []
                
            if package_spec not in config['dependencies']:
                config['dependencies'].append(package_spec)
                
                # Write back to file
                with open(config_path, 'w') as f:
                    toml.dump(config, f)
                    
                self.logger.info(f"Updated snakepit.toml with {package_spec}")
                
        except Exception as e:
            self.logger.warning(f"Could not update dependency graph: {e}")

    def _save_package_history(self, pkg_meta: PackageMetadata):
        """Save package metadata to history file"""
        try:
            # Load existing history
            history = []
            if self.history_file.exists():
                with open(self.history_file) as f:
                    history = json.load(f)
                    
            # Add current package
            history.append({
                'name': pkg_meta.name,
                'version': pkg_meta.version,
                'status': pkg_meta.status.value,
                'ingest_time': pkg_meta.ingest_time,
                'test_time': pkg_meta.test_time,
                'install_time': pkg_meta.install_time,
                'sandbox_id': pkg_meta.sandbox_id,
                'error_log': pkg_meta.error_log,
                'success_log': pkg_meta.success_log,
                'validation_results': pkg_meta.validation_results
            })
            
            # Keep only last 1000 entries
            if len(history) > 1000:
                history = history[-1000:]
                
            # Save back to file
            with open(self.history_file, 'w') as f:
                json.dump(history, f, indent=2)
                
        except Exception as e:
            self.logger.warning(f"Could not save package history: {e}")

    def handle_package(self, package_name: str, version: str = "", test_script: Optional[str] = None) -> bool:
        """
        Complete package handling workflow: ingest -> test -> (kill | conscript)
        
        Args:
            package_name: Name of package to handle
            version: Specific version (optional)
            test_script: Custom test script (optional)
            
        Returns:
            True if package was successfully installed, False if rejected
        """
        self.logger.info(f"🐍 Starting Smart Snakepit handling for {package_name}")
        
        # Phase 1: Ingest
        pkg_meta = self.ingest(package_name, version, test_script)
        if pkg_meta.status == PackageStatus.FAILED:
            self.kill_destroy(package_name)
            return False
            
        # Phase 2: Test/Collaborate
        if not self.test_collaborate(package_name):
            self.kill_destroy(package_name)
            return False
            
        # Phase 4: Conscript (Phase 3 happens automatically on success)
        return self.conscript_install(package_name)

    def list_packages(self) -> Dict[str, PackageMetadata]:
        """List all active packages being processed"""
        return self.active_packages.copy()

    def get_package_status(self, package_name: str) -> Optional[PackageStatus]:
        """Get status of a specific package"""
        if package_name in self.active_packages:
            return self.active_packages[package_name].status
        return None

    def cleanup_all(self):
        """Cleanup all active sandboxes (for shutdown)"""
        self.logger.info("Cleaning up all active packages...")
        package_names = list(self.active_packages.keys())
        for package_name in package_names:
            self.kill_destroy(package_name)


if __name__ == "__main__":
    # Example usage
    handler = SnakepitHandler()
    
    # Test with a simple package
    success = handler.handle_package("requests", "2.31.0")
    if success:
        print("✅ Package successfully handled!")
    else:
        print("❌ Package handling failed!")