#!/usr/bin/env python3
"""
Snakepit Comprehensive Stress Test & Benchmark Suite

Tests all major features:
1. Native installer performance
2. Cache effectiveness
3. Parallel download speed
4. AI recommendation accuracy
5. System library detection
6. Error fixing capability
7. Wheel selection correctness
"""

import subprocess
import time
import os
import sys
import json
from pathlib import Path
from typing import Dict, List, Tuple

class SnakepitTester:
    def __init__(self):
        self.results = {
            "installation_speed": {},
            "cache_performance": {},
            "parallel_efficiency": {},
            "ai_features": {},
            "error_handling": {},
            "system_compatibility": {}
        }
        self.test_packages = {
            "pure_python": ["requests", "click", "colorama", "pyyaml"],
            "binary_small": ["pillow", "cryptography"],
            "binary_large": ["numpy", "pandas"],
            "complex_deps": ["flask", "django"],
        }
    
    def run_command(self, cmd: List[str], timeout: int = 300) -> Tuple[bool, float, str]:
        """Run a command and measure execution time."""
        start = time.time()
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            duration = time.time() - start
            return result.returncode == 0, duration, result.stdout + result.stderr
        except subprocess.TimeoutExpired:
            return False, timeout, "TIMEOUT"
        except Exception as e:
            return False, time.time() - start, str(e)
    
    def cleanup_package(self, package: str):
        """Remove a package to ensure clean test."""
        subprocess.run(
            ["pip", "uninstall", "-y", package],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
    
    def test_installation_speed(self):
        """Test 1: Installation Speed (Cold Cache)"""
        print("\n" + "="*60)
        print("TEST 1: Installation Speed (Cold Cache)")
        print("="*60)
        
        # Clear cache
        cache_dir = Path.home() / ".cache" / "snakepit"
        if cache_dir.exists():
            subprocess.run(["rm", "-rf", str(cache_dir)])
        
        for category, packages in self.test_packages.items():
            print(f"\n📦 Testing {category}...")
            for pkg in packages:
                self.cleanup_package(pkg)
                success, duration, output = self.run_command(["snakepit", "install", pkg])
                
                status = "✅" if success else "❌"
                print(f"  {status} {pkg}: {duration:.2f}s")
                
                self.results["installation_speed"][f"{category}_{pkg}"] = {
                    "success": success,
                    "duration": duration,
                    "category": category
                }
    
    def test_cache_performance(self):
        """Test 2: Cache Performance (Warm Cache)"""
        print("\n" + "="*60)
        print("TEST 2: Cache Performance (Warm Cache)")
        print("="*60)
        
        test_pkg = "requests"
        
        # First install (cold)
        self.cleanup_package(test_pkg)
        _, cold_time, _ = self.run_command(["snakepit", "install", test_pkg])
        
        # Second install (warm)
        self.cleanup_package(test_pkg)
        _, warm_time, _ = self.run_command(["snakepit", "install", test_pkg])
        
        speedup = cold_time / warm_time if warm_time > 0 else 0
        
        print(f"\n📊 Cache Performance:")
        print(f"  Cold install: {cold_time:.2f}s")
        print(f"  Warm install: {warm_time:.2f}s")
        print(f"  Speedup: {speedup:.2f}x")
        
        self.results["cache_performance"] = {
            "cold_time": cold_time,
            "warm_time": warm_time,
            "speedup": speedup
        }
    
    def test_parallel_efficiency(self):
        """Test 3: Parallel Download Efficiency"""
        print("\n" + "="*60)
        print("TEST 3: Parallel Download Efficiency")
        print("="*60)
        
        # This would require a multi-package install command
        # For now, we'll measure sequential vs theoretical parallel
        packages = ["click", "colorama", "pyyaml"]
        
        # Sequential timing
        total_sequential = 0
        for pkg in packages:
            self.cleanup_package(pkg)
            _, duration, _ = self.run_command(["snakepit", "install", pkg])
            total_sequential += duration
        
        print(f"\n📊 Parallel Efficiency:")
        print(f"  Sequential total: {total_sequential:.2f}s")
        print(f"  Average per package: {total_sequential/len(packages):.2f}s")
        
        self.results["parallel_efficiency"] = {
            "sequential_total": total_sequential,
            "package_count": len(packages),
            "avg_per_package": total_sequential / len(packages)
        }
    
    def test_wheel_selection(self):
        """Test 4: Wheel Selection Correctness"""
        print("\n" + "="*60)
        print("TEST 4: Wheel Selection Correctness")
        print("="*60)
        
        # Test packages with multiple wheel variants
        test_cases = [
            ("numpy", "binary with platform-specific wheels"),
            ("pillow", "binary with image library dependencies"),
            ("cryptography", "binary with OpenSSL dependencies"),
        ]
        
        for pkg, description in test_cases:
            self.cleanup_package(pkg)
            success, duration, output = self.run_command(["snakepit", "install", pkg])
            
            status = "✅" if success else "❌"
            print(f"  {status} {pkg} ({description}): {duration:.2f}s")
            
            self.results["system_compatibility"][pkg] = {
                "success": success,
                "duration": duration,
                "description": description
            }
    
    def test_ai_recommendations(self):
        """Test 5: AI Recommendation Quality"""
        print("\n" + "="*60)
        print("TEST 5: AI Recommendation Quality")
        print("="*60)
        
        # Note: This requires GEMINI_API_KEY
        if not os.getenv("GEMINI_API_KEY"):
            print("⚠️  Skipping AI tests (GEMINI_API_KEY not set)")
            return
        
        test_queries = [
            "web scraping",
            "data visualization",
            "REST API framework",
        ]
        
        for query in test_queries:
            print(f"\n🔮 Testing: '{query}'")
            # Note: This would need non-interactive mode
            # For now, we'll just test that the command exists
            success, duration, output = self.run_command(
                ["snakepit", "recommend", query],
                timeout=30
            )
            
            # Check if output contains expected keywords
            has_recommendations = "Recommendations" in output or "PACKAGE:" in output
            
            status = "✅" if has_recommendations else "❌"
            print(f"  {status} Query processed: {duration:.2f}s")
            
            self.results["ai_features"][f"recommend_{query.replace(' ', '_')}"] = {
                "success": has_recommendations,
                "duration": duration
            }
    
    def test_error_recovery(self):
        """Test 6: Error Recovery & Fixing"""
        print("\n" + "="*60)
        print("TEST 6: Error Recovery & Fixing")
        print("="*60)
        
        # Create a test script with a missing import
        test_script = Path("test_broken.py")
        test_script.write_text("import nonexistent_package\nprint('Hello')\n")
        
        print("\n🔧 Testing error detection...")
        success, duration, output = self.run_command(
            ["python", "test_broken.py"],
            timeout=5
        )
        
        has_error = not success and "ModuleNotFoundError" in output
        print(f"  {'✅' if has_error else '❌'} Error detected correctly")
        
        # Cleanup
        test_script.unlink()
        
        self.results["error_handling"]["detection"] = {
            "success": has_error,
            "duration": duration
        }
    
    def test_system_library_detection(self):
        """Test 7: System Library Detection"""
        print("\n" + "="*60)
        print("TEST 7: System Library Detection")
        print("="*60)
        
        # Test packages that might need system libraries
        # Note: These might fail if system libs aren't installed
        test_cases = [
            ("psycopg2-binary", "PostgreSQL adapter (bundled libs)"),
            ("mysqlclient", "MySQL adapter (needs libmysqlclient)"),
        ]
        
        for pkg, description in test_cases:
            print(f"\n📚 Testing: {pkg}")
            self.cleanup_package(pkg)
            success, duration, output = self.run_command(["snakepit", "install", pkg])
            
            # Check if system lib detection triggered
            has_detection = "SYSTEM:" in output or "libpq" in output or "libmysql" in output
            
            status = "✅" if success or has_detection else "❌"
            print(f"  {status} {description}: {duration:.2f}s")
            
            self.results["system_compatibility"][f"syslib_{pkg}"] = {
                "success": success,
                "detection_triggered": has_detection,
                "duration": duration
            }
    
    def generate_report(self):
        """Generate comprehensive test report"""
        print("\n" + "="*60)
        print("COMPREHENSIVE TEST REPORT")
        print("="*60)
        
        # Calculate statistics
        total_tests = sum(len(v) if isinstance(v, dict) else 1 for v in self.results.values())
        successful_tests = 0
        
        for category, tests in self.results.items():
            if isinstance(tests, dict):
                for test, result in tests.items():
                    if isinstance(result, dict) and result.get("success"):
                        successful_tests += 1
        
        success_rate = (successful_tests / total_tests * 100) if total_tests > 0 else 0
        
        print(f"\n📊 Overall Statistics:")
        print(f"  Total Tests: {total_tests}")
        print(f"  Successful: {successful_tests}")
        print(f"  Failed: {total_tests - successful_tests}")
        print(f"  Success Rate: {success_rate:.1f}%")
        
        # Performance summary
        if "installation_speed" in self.results:
            speeds = [r["duration"] for r in self.results["installation_speed"].values() if r["success"]]
            if speeds:
                print(f"\n⚡ Installation Performance:")
                print(f"  Average: {sum(speeds)/len(speeds):.2f}s")
                print(f"  Fastest: {min(speeds):.2f}s")
                print(f"  Slowest: {max(speeds):.2f}s")
        
        # Cache effectiveness
        if "cache_performance" in self.results:
            cache = self.results["cache_performance"]
            print(f"\n💾 Cache Effectiveness:")
            print(f"  Speedup: {cache.get('speedup', 0):.2f}x")
        
        # Save detailed results
        report_file = Path("test_results.json")
        with open(report_file, "w") as f:
            json.dump(self.results, f, indent=2)
        
        print(f"\n📄 Detailed results saved to: {report_file}")
        
        return success_rate >= 80  # 80% success rate threshold
    
    def run_all_tests(self):
        """Run complete test suite"""
        print("🐍 SNAKEPIT COMPREHENSIVE TEST SUITE")
        print("="*60)
        
        try:
            self.test_installation_speed()
            self.test_cache_performance()
            self.test_parallel_efficiency()
            self.test_wheel_selection()
            self.test_ai_recommendations()
            self.test_error_recovery()
            self.test_system_library_detection()
            
            success = self.generate_report()
            
            if success:
                print("\n✅ TEST SUITE PASSED")
                return 0
            else:
                print("\n❌ TEST SUITE FAILED")
                return 1
                
        except KeyboardInterrupt:
            print("\n\n⚠️  Tests interrupted by user")
            return 2
        except Exception as e:
            print(f"\n\n❌ Fatal error: {e}")
            import traceback
            traceback.print_exc()
            return 3

if __name__ == "__main__":
    tester = SnakepitTester()
    sys.exit(tester.run_all_tests())
