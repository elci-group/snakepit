#!/usr/bin/env python3
"""
Snakepit Validation Framework

Advanced package validation system with:
- Custom test templates
- Security scanning
- Dependency analysis  
- Performance testing
- Compatibility checks

Author: adminx
"""

import os
import sys
import json
import subprocess
import tempfile
import time
import importlib.util
import ast
import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple, Any, Set
from dataclasses import dataclass
from enum import Enum
import logging


class ValidationLevel(Enum):
    """Validation strictness levels"""
    BASIC = "basic"          # Import test only
    STANDARD = "standard"    # Import + basic functionality  
    COMPREHENSIVE = "comprehensive"  # Full test suite
    SECURITY = "security"    # Security-focused validation
    PERFORMANCE = "performance"  # Performance benchmarking


@dataclass
class ValidationResult:
    """Result of package validation"""
    package_name: str
    level: ValidationLevel
    passed: bool
    score: float  # 0.0 to 1.0
    duration: float
    test_results: Dict[str, Any]
    warnings: List[str]
    errors: List[str]
    security_issues: List[str] = None
    performance_metrics: Dict[str, float] = None


class ValidationFramework:
    """Advanced package validation framework"""
    
    def __init__(self, logger: Optional[logging.Logger] = None):
        self.logger = logger or logging.getLogger(__name__)
        self.test_templates = {}
        self.security_patterns = self._load_security_patterns()
        self._load_test_templates()
        
    def _load_security_patterns(self) -> Dict[str, List[str]]:
        """Load security patterns to check for"""
        return {
            'dangerous_imports': [
                r'import\s+os\.system',
                r'import\s+subprocess\.',
                r'from\s+subprocess\s+import',
                r'import\s+eval',
                r'import\s+exec',
                r'__import__\s*\(',
            ],
            'network_access': [
                r'import\s+urllib',
                r'import\s+requests',
                r'import\s+socket',
                r'import\s+http\.',
                r'from\s+urllib',
            ],
            'file_system': [
                r'open\s*\(',
                r'file\s*\(',
                r'import\s+shutil',
                r'os\.remove',
                r'os\.unlink',
                r'pathlib\.',
            ],
            'eval_patterns': [
                r'eval\s*\(',
                r'exec\s*\(',
                r'compile\s*\(',
                r'__import__\s*\(',
            ]
        }
        
    def _load_test_templates(self):
        """Load test templates for different package types"""
        
        # Basic import test template
        self.test_templates['basic'] = '''
def test_basic_import(package_name):
    """Test basic package import"""
    try:
        __import__(package_name)
        return True, "Import successful"
    except ImportError as e:
        return False, f"Import failed: {e}"
    except Exception as e:
        return False, f"Unexpected error: {e}"
'''

        # Web framework test template
        self.test_templates['web'] = '''
def test_web_framework(package_name):
    """Test web framework functionality"""
    results = {}
    
    # Basic import
    try:
        module = __import__(package_name)
        results['import'] = True
    except Exception as e:
        results['import'] = False
        results['import_error'] = str(e)
        return results
        
    # Check for common web framework attributes
    web_attrs = ['app', 'route', 'run', 'Flask', 'Django', 'FastAPI']
    found_attrs = [attr for attr in web_attrs if hasattr(module, attr)]
    results['web_attributes'] = found_attrs
    
    # Try creating basic app (if applicable)
    try:
        if hasattr(module, 'Flask'):
            app = module.Flask(__name__)
            results['app_creation'] = True
        elif hasattr(module, 'FastAPI'):
            app = module.FastAPI()
            results['app_creation'] = True
        else:
            results['app_creation'] = 'not_applicable'
    except Exception as e:
        results['app_creation_error'] = str(e)
        
    return results
'''

        # Data science test template
        self.test_templates['data'] = '''
def test_data_package(package_name):
    """Test data science package functionality"""
    results = {}
    
    try:
        module = __import__(package_name)
        results['import'] = True
        
        # Check for common data science attributes
        data_attrs = ['DataFrame', 'array', 'Series', 'numpy', 'pandas']
        found_attrs = [attr for attr in data_attrs if hasattr(module, attr)]
        results['data_attributes'] = found_attrs
        
        # Try basic operations
        if package_name == 'numpy':
            arr = module.array([1, 2, 3])
            results['array_creation'] = len(arr) == 3
        elif package_name == 'pandas':
            df = module.DataFrame({'a': [1, 2], 'b': [3, 4]})
            results['dataframe_creation'] = len(df) == 2
            
    except Exception as e:
        results['import'] = False
        results['error'] = str(e)
        
    return results
'''

        # Security test template
        self.test_templates['security'] = '''
def test_security_aspects(package_name, source_files):
    """Test package for security issues"""
    results = {
        'dangerous_patterns': [],
        'network_access': [],
        'file_system_access': [],
        'eval_usage': []
    }
    
    # Scan source files for security patterns
    for file_path in source_files:
        try:
            with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                content = f.read()
                
            # Check for dangerous patterns
            if 'os.system' in content or 'subprocess' in content:
                results['dangerous_patterns'].append(f"System calls in {file_path}")
            
            if any(pattern in content for pattern in ['urllib', 'requests', 'socket']):
                results['network_access'].append(f"Network access in {file_path}")
                
            if any(pattern in content for pattern in ['open(', 'file(', 'shutil']):
                results['file_system_access'].append(f"File system access in {file_path}")
                
            if any(pattern in content for pattern in ['eval(', 'exec(', '__import__']):
                results['eval_usage'].append(f"Dynamic code execution in {file_path}")
                
        except Exception as e:
            results['scan_errors'] = results.get('scan_errors', [])
            results['scan_errors'].append(f"Error scanning {file_path}: {e}")
            
    return results
'''

    def generate_test_script(self, 
                           package_name: str, 
                           validation_level: ValidationLevel,
                           custom_tests: Optional[List[str]] = None) -> str:
        """Generate comprehensive test script for package"""
        
        # Detect package type for appropriate templates
        package_type = self._detect_package_type(package_name)
        
        script_parts = [
            "#!/usr/bin/env python3",
            '"""Generated test script for package validation"""',
            "",
            "import sys",
            "import time",
            "import json",
            "import traceback",
            "from pathlib import Path",
            "",
            f"PACKAGE_NAME = '{package_name}'",
            f"VALIDATION_LEVEL = '{validation_level.value}'",
            "",
        ]
        
        # Add test functions based on validation level and package type
        if validation_level in [ValidationLevel.BASIC, ValidationLevel.STANDARD]:
            script_parts.append(self.test_templates['basic'])
            
        if validation_level in [ValidationLevel.STANDARD, ValidationLevel.COMPREHENSIVE]:
            if package_type == 'web':
                script_parts.append(self.test_templates['web'])
            elif package_type == 'data':
                script_parts.append(self.test_templates['data'])
                
        if validation_level in [ValidationLevel.SECURITY, ValidationLevel.COMPREHENSIVE]:
            script_parts.append(self.test_templates['security'])
            
        # Add custom test functions
        if custom_tests:
            script_parts.extend(custom_tests)
            
        # Add main execution logic
        main_script = f'''
def run_validation():
    """Main validation function"""
    results = {{
        'package_name': PACKAGE_NAME,
        'validation_level': VALIDATION_LEVEL,
        'start_time': time.time(),
        'tests': {{}},
        'warnings': [],
        'errors': [],
        'overall_score': 0.0
    }}
    
    test_count = 0
    passed_tests = 0
    
    # Basic import test
    try:
        print(f"🧪 Testing basic import of {{PACKAGE_NAME}}")
        passed, message = test_basic_import(PACKAGE_NAME)
        results['tests']['basic_import'] = {{'passed': passed, 'message': message}}
        test_count += 1
        if passed:
            passed_tests += 1
            print(f"✅ Basic import: {{message}}")
        else:
            print(f"❌ Basic import: {{message}}")
    except Exception as e:
        results['errors'].append(f"Basic import test error: {{e}}")
        print(f"❌ Basic import test failed: {{e}}")
        
    # Additional tests based on validation level
    if VALIDATION_LEVEL in ['standard', 'comprehensive']:
        try:
            module = __import__(PACKAGE_NAME)
            
            # Version check
            version_attrs = ['__version__', 'VERSION', 'version']
            version = None
            for attr in version_attrs:
                if hasattr(module, attr):
                    version = getattr(module, attr)
                    break
                    
            if version:
                results['tests']['version_info'] = {{'passed': True, 'version': str(version)}}
                print(f"📌 Package version: {{version}}")
            else:
                results['warnings'].append("No version information found")
                
            # Attribute count check
            attrs = [attr for attr in dir(module) if not attr.startswith('_')]
            results['tests']['attribute_count'] = {{'passed': True, 'count': len(attrs)}}
            print(f"📦 Public attributes: {{len(attrs)}}")
            
            test_count += 2
            passed_tests += 2
            
        except Exception as e:
            results['errors'].append(f"Extended testing error: {{e}}")
            
    # Performance test for comprehensive validation
    if VALIDATION_LEVEL in ['performance', 'comprehensive']:
        try:
            print("⚡ Running performance tests")
            start_time = time.time()
            
            # Import time measurement
            import_start = time.time()
            __import__(PACKAGE_NAME)
            import_time = time.time() - import_start
            
            results['tests']['import_performance'] = {{
                'passed': import_time < 5.0,  # Fail if import takes >5s
                'import_time': import_time
            }}
            
            print(f"⏱️ Import time: {{import_time:.3f}}s")
            test_count += 1
            if import_time < 5.0:
                passed_tests += 1
                
        except Exception as e:
            results['errors'].append(f"Performance test error: {{e}}")
    
    # Calculate overall score
    if test_count > 0:
        results['overall_score'] = passed_tests / test_count
    
    results['end_time'] = time.time()
    results['duration'] = results['end_time'] - results['start_time']
    results['passed_tests'] = passed_tests
    results['total_tests'] = test_count
    
    return results

if __name__ == "__main__":
    try:
        print(f"🐍 Starting validation for {{PACKAGE_NAME}} ({{VALIDATION_LEVEL}} level)")
        results = run_validation()
        
        print(f"\\n📊 Validation Results:")
        print(f"   Score: {{results['overall_score']:.2f}} ({{results['passed_tests']}}/{{results['total_tests']}} tests passed)")
        print(f"   Duration: {{results['duration']:.2f}}s")
        
        if results['warnings']:
            print(f"   Warnings: {{len(results['warnings'])}}")
            
        if results['errors']:
            print(f"   Errors: {{len(results['errors'])}}")
            for error in results['errors']:
                print(f"     • {{error}}")
        
        # Exit with appropriate code
        if results['overall_score'] >= 0.8:
            print("✅ Package validation PASSED")
            sys.exit(0)
        else:
            print("❌ Package validation FAILED")
            sys.exit(1)
            
    except Exception as e:
        print(f"❌ Validation script error: {{e}}")
        traceback.print_exc()
        sys.exit(1)
'''
        
        script_parts.append(main_script)
        return "\n".join(script_parts)
        
    def _detect_package_type(self, package_name: str) -> str:
        """Detect package type based on name patterns"""
        web_patterns = ['flask', 'django', 'fastapi', 'tornado', 'pyramid', 'bottle']
        data_patterns = ['pandas', 'numpy', 'scipy', 'sklearn', 'matplotlib', 'seaborn']
        ml_patterns = ['tensorflow', 'torch', 'keras', 'xgboost', 'lightgbm']
        
        name_lower = package_name.lower()
        
        if any(pattern in name_lower for pattern in web_patterns):
            return 'web'
        elif any(pattern in name_lower for pattern in data_patterns):
            return 'data'
        elif any(pattern in name_lower for pattern in ml_patterns):
            return 'ml'
        else:
            return 'general'
            
    def validate_package(self, 
                        package_name: str,
                        package_path: str,
                        validation_level: ValidationLevel = ValidationLevel.STANDARD,
                        timeout: int = 120) -> ValidationResult:
        """Run comprehensive package validation"""
        
        start_time = time.time()
        
        # Generate test script
        test_script = self.generate_test_script(package_name, validation_level)
        
        # Create temporary test file
        with tempfile.NamedTemporaryFile(mode='w', suffix='.py', delete=False) as f:
            f.write(test_script)
            test_file = f.name
            
        try:
            # Run validation in package environment
            if sys.platform == "win32":
                python_exe = Path(package_path) / "Scripts" / "python.exe"
            else:
                python_exe = Path(package_path) / "bin" / "python"
                
            # Execute validation script
            result = subprocess.run(
                [str(python_exe), test_file],
                capture_output=True,
                text=True,
                timeout=timeout
            )
            
            duration = time.time() - start_time
            
            # Parse results from output
            validation_result = ValidationResult(
                package_name=package_name,
                level=validation_level,
                passed=result.returncode == 0,
                score=self._extract_score_from_output(result.stdout),
                duration=duration,
                test_results=self._parse_test_output(result.stdout),
                warnings=self._extract_warnings_from_output(result.stdout),
                errors=self._extract_errors_from_output(result.stderr)
            )
            
            # Add security scan if requested
            if validation_level in [ValidationLevel.SECURITY, ValidationLevel.COMPREHENSIVE]:
                security_results = self._security_scan(package_path)
                validation_result.security_issues = security_results
                
            return validation_result
            
        except subprocess.TimeoutExpired:
            return ValidationResult(
                package_name=package_name,
                level=validation_level,
                passed=False,
                score=0.0,
                duration=time.time() - start_time,
                test_results={},
                warnings=[],
                errors=[f"Validation timed out after {timeout}s"]
            )
            
        except Exception as e:
            return ValidationResult(
                package_name=package_name,
                level=validation_level,
                passed=False,
                score=0.0,
                duration=time.time() - start_time,
                test_results={},
                warnings=[],
                errors=[f"Validation error: {str(e)}"]
            )
            
        finally:
            # Cleanup temporary files
            try:
                os.unlink(test_file)
            except:
                pass
                
    def _extract_score_from_output(self, output: str) -> float:
        """Extract validation score from test output"""
        try:
            # Look for score pattern in output
            score_pattern = r'Score:\s+([\d.]+)'
            match = re.search(score_pattern, output)
            if match:
                return float(match.group(1))
        except:
            pass
        return 0.0
        
    def _parse_test_output(self, output: str) -> Dict[str, Any]:
        """Parse test results from output"""
        results = {}
        
        # Extract test results using patterns
        patterns = {
            'import_time': r'Import time:\s+([\d.]+)s',
            'version': r'Package version:\s+(.+)',
            'attributes': r'Public attributes:\s+(\d+)'
        }
        
        for key, pattern in patterns.items():
            match = re.search(pattern, output)
            if match:
                try:
                    if key == 'attributes':
                        results[key] = int(match.group(1))
                    elif key == 'import_time':
                        results[key] = float(match.group(1))
                    else:
                        results[key] = match.group(1)
                except:
                    pass
                    
        return results
        
    def _extract_warnings_from_output(self, output: str) -> List[str]:
        """Extract warnings from test output"""
        warnings = []
        for line in output.split('\n'):
            if 'warning' in line.lower() or '⚠️' in line:
                warnings.append(line.strip())
        return warnings
        
    def _extract_errors_from_output(self, error_output: str) -> List[str]:
        """Extract errors from stderr"""
        errors = []
        if error_output:
            for line in error_output.split('\n'):
                if line.strip():
                    errors.append(line.strip())
        return errors
        
    def _security_scan(self, package_path: str) -> List[str]:
        """Perform security scan on package files"""
        security_issues = []
        
        # Find Python files in package
        package_dir = Path(package_path)
        python_files = list(package_dir.rglob("*.py"))
        
        for file_path in python_files:
            try:
                with open(file_path, 'r', encoding='utf-8', errors='ignore') as f:
                    content = f.read()
                    
                # Check security patterns
                for category, patterns in self.security_patterns.items():
                    for pattern in patterns:
                        if re.search(pattern, content, re.IGNORECASE):
                            security_issues.append(f"{category}: {pattern} found in {file_path.name}")
                            
            except Exception as e:
                self.logger.warning(f"Could not scan {file_path}: {e}")
                
        return security_issues

    def create_custom_test(self, test_name: str, test_function: str):
        """Add custom test template"""
        self.test_templates[test_name] = test_function
        
    def get_validation_report(self, result: ValidationResult) -> str:
        """Generate formatted validation report"""
        report_lines = [
            f"📊 Validation Report: {result.package_name}",
            "=" * 50,
            f"Validation Level: {result.level.value}",
            f"Overall Score: {result.score:.2f}/1.0",
            f"Status: {'✅ PASSED' if result.passed else '❌ FAILED'}",
            f"Duration: {result.duration:.2f}s",
            ""
        ]
        
        if result.test_results:
            report_lines.append("🧪 Test Results:")
            for test, details in result.test_results.items():
                if isinstance(details, dict) and 'passed' in details:
                    status = "✅" if details['passed'] else "❌"
                    report_lines.append(f"   {status} {test}: {details.get('message', 'OK')}")
                else:
                    report_lines.append(f"   • {test}: {details}")
            report_lines.append("")
            
        if result.warnings:
            report_lines.append("⚠️ Warnings:")
            for warning in result.warnings:
                report_lines.append(f"   • {warning}")
            report_lines.append("")
            
        if result.errors:
            report_lines.append("❌ Errors:")
            for error in result.errors:
                report_lines.append(f"   • {error}")
            report_lines.append("")
            
        if result.security_issues:
            report_lines.append("🔒 Security Issues:")
            for issue in result.security_issues:
                report_lines.append(f"   • {issue}")
            report_lines.append("")
            
        return "\n".join(report_lines)


# Example usage and testing
if __name__ == "__main__":
    framework = ValidationFramework()
    
    # Test script generation
    test_script = framework.generate_test_script("requests", ValidationLevel.COMPREHENSIVE)
    print("Generated test script:")
    print(test_script[:500] + "..." if len(test_script) > 500 else test_script)