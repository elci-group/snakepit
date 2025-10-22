"""
Snakepit Site Customization Module

This module is loaded automatically by Python on startup when placed in the
site-packages directory. It intercepts pip imports and subprocess calls to
route them through snakepit's smart handler.

Installation:
    ln -s /path/to/snakepit_sitecustomize.py $(python3 -c "import site; print(site.getsitepackages()[0])")/sitecustomize.py

Environment Variables:
    SNAKEPIT_INTERCEPT=0    Disable snakepit interception
    SNAKEPIT_VERBOSE=1      Enable verbose logging
"""

import sys
import os
import builtins

# Only intercept if not explicitly disabled
SNAKEPIT_ENABLED = os.environ.get('SNAKEPIT_INTERCEPT', '1') == '1'
SNAKEPIT_VERBOSE = os.environ.get('SNAKEPIT_VERBOSE', '0') == '1'

# Get snakepit directory
SNAKEPIT_DIR = os.path.dirname(os.path.abspath(__file__))

def log_verbose(message):
    """Log message if verbose mode enabled"""
    if SNAKEPIT_VERBOSE:
        print(f"[Snakepit] {message}", file=sys.stderr)


if SNAKEPIT_ENABLED:
    # Store original __import__
    _original_import = builtins.__import__
    
    def snakepit_import_hook(name, *args, **kwargs):
        """
        Custom import hook that intercepts pip module imports
        """
        # Import normally first
        module = _original_import(name, *args, **kwargs)
        
        # If importing pip.main, wrap it
        if name == 'pip' or name.startswith('pip.'):
            log_verbose(f"Intercepted pip import: {name}")
            
            # Check if this is a pip install operation
            if hasattr(module, 'main') and callable(module.main):
                original_main = module.main
                
                def wrapped_main(args=None):
                    """Wrapped pip main that routes through snakepit"""
                    if args is None:
                        args = sys.argv[1:]
                    
                    # Check if this is an install command
                    if isinstance(args, list) and len(args) > 0 and args[0] == 'install':
                        log_verbose(f"Intercepting pip install: {args}")
                        
                        # Try to use snakepit wrapper
                        wrapper_path = os.path.join(SNAKEPIT_DIR, 'snakepit-pip-wrapper.sh')
                        if os.path.exists(wrapper_path):
                            log_verbose(f"Routing to snakepit wrapper: {wrapper_path}")
                            import subprocess
                            result = subprocess.run([wrapper_path] + args, check=False)
                            return result.returncode
                    
                    # Fall back to original pip main
                    return original_main(args)
                
                # Replace the main function
                module.main = wrapped_main
        
        return module
    
    # Install the import hook
    builtins.__import__ = snakepit_import_hook
    log_verbose("Snakepit import hook installed")
    
    # Also intercept subprocess calls to pip
    try:
        import subprocess
        _original_run = subprocess.run
        _original_call = subprocess.call
        _original_popen = subprocess.Popen
        
        def should_intercept_command(args):
            """Check if command should be intercepted"""
            if not args:
                return False
            
            # Convert args to list if it's a string
            if isinstance(args, str):
                args = args.split()
            
            # Check for pip commands
            if len(args) > 0:
                cmd = os.path.basename(str(args[0]))
                if cmd in ['pip', 'pip3', 'pip3.11', 'pip3.12']:
                    if len(args) > 1 and args[1] == 'install':
                        return True
            
            return False
        
        def get_snakepit_command(args):
            """Convert pip command to snakepit wrapper command"""
            wrapper_path = os.path.join(SNAKEPIT_DIR, 'snakepit-pip-wrapper.sh')
            if isinstance(args, str):
                # String command
                return args.replace('pip ', f'{wrapper_path} ', 1)
            else:
                # List command
                return [wrapper_path] + list(args[1:])
        
        def wrapped_run(*args, **kwargs):
            """Wrapped subprocess.run"""
            if should_intercept_command(args[0] if args else kwargs.get('args')):
                cmd = args[0] if args else kwargs.get('args')
                log_verbose(f"Intercepting subprocess.run: {cmd}")
                new_cmd = get_snakepit_command(cmd)
                if args:
                    args = (new_cmd,) + args[1:]
                else:
                    kwargs['args'] = new_cmd
            return _original_run(*args, **kwargs)
        
        def wrapped_call(*args, **kwargs):
            """Wrapped subprocess.call"""
            if should_intercept_command(args[0] if args else kwargs.get('args')):
                cmd = args[0] if args else kwargs.get('args')
                log_verbose(f"Intercepting subprocess.call: {cmd}")
                new_cmd = get_snakepit_command(cmd)
                if args:
                    args = (new_cmd,) + args[1:]
                else:
                    kwargs['args'] = new_cmd
            return _original_call(*args, **kwargs)
        
        class WrappedPopen(_original_popen):
            """Wrapped Popen class"""
            def __init__(self, args, *pargs, **kwargs):
                if should_intercept_command(args):
                    log_verbose(f"Intercepting Popen: {args}")
                    args = get_snakepit_command(args)
                super().__init__(args, *pargs, **kwargs)
        
        subprocess.run = wrapped_run
        subprocess.call = wrapped_call
        subprocess.Popen = WrappedPopen
        
        log_verbose("Snakepit subprocess hooks installed")
    
    except Exception as e:
        log_verbose(f"Failed to install subprocess hooks: {e}")


# Mark that snakepit is active
os.environ['SNAKEPIT_ACTIVE'] = '1'
