#!/usr/bin/env python3
"""
Test script for dependency resolver using your example output
"""

from dependency_resolver import DependencyResolver

# Your actual pip output from the command
test_output = """
ERROR: pip's dependency resolver does not currently take into account all the packages that are installed. This behaviour is the source of the following dependency conflicts.
twisted 25.5.0 requires constantly>=15.1, which is not installed.
twisted 25.5.0 requires hyperlink>=17.1.1, which is not installed.
twisted 25.5.0 requires incremental>=24.7.0, which is not installed.
twisted 25.5.0 requires zope-interface>=5, which is not installed.
flask 3.1.2 requires werkzeug>=3.1.0, which is not installed.
flask 3.1.2 requires blinker>=1.9.0, but you have blinker 1.4 which is incompatible.
flask 3.1.2 requires markupsafe>=2.1.1, but you have markupsafe 2.0.1 which is incompatible.
Successfully installed automat-25.4.16 itsdangerous-2.2.0
"""

def main():
    print("=" * 70)
    print("Testing Snakepit Dependency Resolver")
    print("=" * 70)
    print("\n📋 Simulating pip output with conflicts...\n")
    
    # Create resolver
    resolver = DependencyResolver()
    
    # Parse conflicts
    print("🔍 Parsing conflicts...")
    conflicts = resolver.parse_pip_output(test_output)
    
    print(f"\n✅ Found {len(conflicts)} conflicts:\n")
    for i, conflict in enumerate(conflicts, 1):
        print(f"{i}. {conflict.package}")
        print(f"   Type: {conflict.conflict_type}")
        print(f"   Required by: {conflict.required_by}")
        print(f"   Spec: {conflict.required_spec}")
        if conflict.current_version:
            print(f"   Current version: {conflict.current_version}")
        print()
    
    # Create resolution plan
    print("🔧 Creating resolution plan...")
    resolution = resolver.create_resolution_plan(conflicts)
    
    if resolution.success:
        print("✅ Resolution plan created successfully!\n")
        print(f"📦 Packages to install: {len(resolution.packages_to_install)}")
        for pkg in resolution.packages_to_install:
            print(f"   • {pkg}")
        
        print(f"\n⬆️  Packages to upgrade: {len(resolution.packages_to_upgrade)}")
        for pkg in resolution.packages_to_upgrade:
            print(f"   • {pkg}")
        
        print(f"\n📋 Resolution order:")
        for i, pkg in enumerate(resolution.resolution_order, 1):
            print(f"   {i}. {pkg}")
        
        print("\n" + "=" * 70)
        print("DRY RUN - What would be executed:")
        print("=" * 70)
        for pkg in resolution.resolution_order:
            print(f"pip install --upgrade {pkg}")
        
    else:
        print("❌ Failed to create resolution plan")
    
    print("\n" + "=" * 70)
    print("Test complete!")
    print("=" * 70)

if __name__ == "__main__":
    main()
