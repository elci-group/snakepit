#!/bin/bash
# Snakepit Universal Wrapper - Intercepts ALL Python package installation methods
# Covers: pip, pip3, python -m pip, easy_install, setup.py, poetry, pipenv, conda, etc.

SNAKEPIT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNAKEPIT_CLI="$SNAKEPIT_DIR/snakepit_cli.py"

# Check if we should intercept
should_intercept() {
    [[ "$SNAKEPIT_BYPASS" != "1" ]] && [[ -f "$SNAKEPIT_CLI" ]]
}

# Extract package name from various formats
extract_package_name() {
    local arg="$1"
    
    # Remove version specifiers: package==1.0, package>=1.0, package[extra]
    echo "$arg" | sed -E 's/([^=<>~!\[]+).*/\1/' | tr -d '[:space:]'
}

# Route installation through snakepit
route_install() {
    local method="$1"
    shift
    local packages=("$@")
    
    echo "🐍 Snakepit: Intercepting $method installation..."
    
    for pkg in "${packages[@]}"; do
        pkg_name=$(extract_package_name "$pkg")
        
        # Skip if it's a URL, file path, or VCS
        if [[ "$pkg" =~ ^(https?|git|file|\./).*$ ]] || [[ -f "$pkg" ]]; then
            echo "⚠️  Skipping $pkg (direct source installation)"
            continue
        fi
        
        # Extract version
        if [[ "$pkg" =~ ^[^=<>~!]+==(.+)$ ]]; then
            version="${BASH_REMATCH[1]}"
            python3 "$SNAKEPIT_CLI" install "$pkg_name" --version "$version" ${SNAKEPIT_VERBOSE:+--verbose}
        else
            python3 "$SNAKEPIT_CLI" install "$pkg_name" ${SNAKEPIT_VERBOSE:+--verbose}
        fi
        
        if [[ $? -ne 0 ]]; then
            echo "❌ Failed to install $pkg_name through snakepit"
            return 1
        fi
    done
    
    return 0
}

# ============================================================================
# PIP WRAPPERS
# ============================================================================

_snakepit_pip_wrapper() {
    if ! should_intercept; then
        command python3 -m pip "$@"
        return $?
    fi
    
    local cmd="$1"
    case "$cmd" in
        install)
            shift
            local packages=()
            local other_args=()
            
            while [[ $# -gt 0 ]]; do
                case "$1" in
                    -r|--requirement)
                        # Handle requirements file
                        if [[ -f "$2" ]]; then
                            echo "🐍 Snakepit: Processing requirements from $2"
                            while IFS= read -r line; do
                                # Skip comments and empty lines
                                [[ "$line" =~ ^#.*$ ]] || [[ -z "$line" ]] && continue
                                packages+=("$line")
                            done < "$2"
                            shift 2
                        else
                            echo "❌ Requirements file not found: $2"
                            return 1
                        fi
                        ;;
                    -e|--editable|-c|--constraint)
                        # Skip editable and constraints for now
                        echo "⚠️  Skipping $1 $2 (not supported by validation)"
                        shift 2
                        ;;
                    -*)
                        other_args+=("$1")
                        shift
                        ;;
                    *)
                        packages+=("$1")
                        shift
                        ;;
                esac
            done
            
            if [[ ${#packages[@]} -gt 0 ]]; then
                route_install "pip" "${packages[@]}"
            else
                echo "⚠️  No packages to install"
                return 1
            fi
            ;;
        *)
            # Pass through non-install commands
            command python3 -m pip "$@"
            ;;
    esac
}

# ============================================================================
# PYTHON -m pip WRAPPER
# ============================================================================

_snakepit_python_wrapper() {
    local args=("$@")
    
    # Check if this is "python -m pip install"
    if [[ "${args[0]}" == "-m" ]] && [[ "${args[1]}" == "pip" ]] && [[ "${args[2]}" == "install" ]]; then
        if should_intercept; then
            # Route through pip wrapper
            _snakepit_pip_wrapper install "${args[@]:3}"
            return $?
        fi
    fi
    
    # Otherwise, run normally
    command python3 "$@"
}

# ============================================================================
# EASY_INSTALL WRAPPER (legacy)
# ============================================================================

_snakepit_easy_install_wrapper() {
    if ! should_intercept; then
        command easy_install "$@"
        return $?
    fi
    
    echo "🐍 Snakepit: Intercepting easy_install (legacy)"
    local packages=()
    
    for arg in "$@"; do
        [[ "$arg" =~ ^- ]] && continue
        packages+=("$arg")
    done
    
    if [[ ${#packages[@]} -gt 0 ]]; then
        route_install "easy_install" "${packages[@]}"
    else
        command easy_install "$@"
    fi
}

# ============================================================================
# SETUP.PY WRAPPER
# ============================================================================

_snakepit_setup_py_wrapper() {
    local setup_file="$1"
    shift
    
    if [[ "$1" == "install" ]] && should_intercept; then
        echo "🐍 Snakepit: Intercepting setup.py install"
        
        # Try to extract package name from setup.py
        local pkg_name=$(python3 -c "
import sys
sys.path.insert(0, '$(dirname "$setup_file")')
try:
    from setuptools import setup
    import setuptools
    # Capture setup() call
    original_setup = setuptools.setup
    name = None
    def capture_setup(**kwargs):
        global name
        name = kwargs.get('name', 'unknown')
    setuptools.setup = capture_setup
    exec(open('$setup_file').read())
    print(name if name else 'unknown')
except:
    print('unknown')
" 2>/dev/null)
        
        if [[ "$pkg_name" != "unknown" ]] && [[ -n "$pkg_name" ]]; then
            echo "📦 Detected package: $pkg_name"
            
            # Validate in sandbox first
            echo "🧪 Validating setup.py installation..."
            SNAKEPIT_AUTO_TEST=0 python3 "$SNAKEPIT_CLI" validate "$pkg_name" 2>/dev/null
            
            if [[ $? -eq 0 ]]; then
                echo "✅ Validation passed, proceeding with installation"
            else
                echo "⚠️  Validation inconclusive, proceeding with caution"
            fi
        fi
    fi
    
    # Run actual setup.py
    command python3 "$setup_file" "$@"
}

# ============================================================================
# POETRY WRAPPER
# ============================================================================

_snakepit_poetry_wrapper() {
    if ! should_intercept; then
        command poetry "$@"
        return $?
    fi
    
    local cmd="$1"
    case "$cmd" in
        add|install)
            echo "🐍 Snakepit: Intercepting poetry $cmd"
            shift
            
            local packages=()
            for arg in "$@"; do
                [[ "$arg" =~ ^- ]] && continue
                packages+=("$arg")
            done
            
            if [[ ${#packages[@]} -gt 0 ]]; then
                # Validate packages first
                for pkg in "${packages[@]}"; do
                    pkg_name=$(extract_package_name "$pkg")
                    python3 "$SNAKEPIT_CLI" validate "$pkg_name" ${SNAKEPIT_VERBOSE:+--verbose}
                    if [[ $? -ne 0 ]]; then
                        echo "❌ Package $pkg_name failed validation"
                        return 1
                    fi
                done
                echo "✅ All packages validated, running poetry $cmd"
            fi
            
            # Run actual poetry command
            command poetry "$cmd" "${packages[@]}"
            ;;
        *)
            command poetry "$@"
            ;;
    esac
}

# ============================================================================
# PIPENV WRAPPER
# ============================================================================

_snakepit_pipenv_wrapper() {
    if ! should_intercept; then
        command pipenv "$@"
        return $?
    fi
    
    local cmd="$1"
    case "$cmd" in
        install)
            echo "🐍 Snakepit: Intercepting pipenv install"
            shift
            
            local packages=()
            for arg in "$@"; do
                [[ "$arg" =~ ^- ]] && continue
                packages+=("$arg")
            done
            
            if [[ ${#packages[@]} -gt 0 ]]; then
                route_install "pipenv" "${packages[@]}"
                return $?
            fi
            
            command pipenv install "$@"
            ;;
        *)
            command pipenv "$@"
            ;;
    esac
}

# ============================================================================
# CONDA WRAPPER
# ============================================================================

_snakepit_conda_wrapper() {
    if ! should_intercept; then
        command conda "$@"
        return $?
    fi
    
    local cmd="$1"
    case "$cmd" in
        install)
            echo "🐍 Snakepit: Intercepting conda install"
            shift
            
            local packages=()
            local skip_next=false
            
            for arg in "$@"; do
                if [[ "$skip_next" == "true" ]]; then
                    skip_next=false
                    continue
                fi
                
                case "$arg" in
                    -n|--name|-p|--prefix|-c|--channel)
                        skip_next=true
                        ;;
                    -*)
                        ;;
                    *)
                        packages+=("$arg")
                        ;;
                esac
            done
            
            if [[ ${#packages[@]} -gt 0 ]]; then
                echo "⚠️  Note: Conda packages validated against PyPI equivalents"
                route_install "conda" "${packages[@]}"
                return $?
            fi
            
            command conda install "$@"
            ;;
        *)
            command conda "$@"
            ;;
    esac
}

# ============================================================================
# PDM WRAPPER
# ============================================================================

_snakepit_pdm_wrapper() {
    if ! should_intercept; then
        command pdm "$@"
        return $?
    fi
    
    local cmd="$1"
    case "$cmd" in
        add)
            echo "🐍 Snakepit: Intercepting pdm add"
            shift
            
            local packages=()
            for arg in "$@"; do
                [[ "$arg" =~ ^- ]] && continue
                packages+=("$arg")
            done
            
            if [[ ${#packages[@]} -gt 0 ]]; then
                route_install "pdm" "${packages[@]}"
                return $?
            fi
            
            command pdm add "$@"
            ;;
        *)
            command pdm "$@"
            ;;
    esac
}

# ============================================================================
# FLIT WRAPPER
# ============================================================================

_snakepit_flit_wrapper() {
    if ! should_intercept; then
        command flit "$@"
        return $?
    fi
    
    local cmd="$1"
    case "$cmd" in
        install)
            echo "🐍 Snakepit: Intercepting flit install"
            # Flit installs current project, validate it
            if [[ -f "pyproject.toml" ]]; then
                local pkg_name=$(python3 -c "
import tomli if 'tomli' in dir() else tomllib
try:
    with open('pyproject.toml', 'rb') as f:
        data = tomli.load(f) if 'tomli' in dir() else __import__('tomllib').load(f)
        print(data.get('project', {}).get('name', 'unknown'))
except:
    print('unknown')
" 2>/dev/null)
                
                if [[ "$pkg_name" != "unknown" ]]; then
                    echo "📦 Validating $pkg_name"
                    python3 "$SNAKEPIT_CLI" validate "$pkg_name" ${SNAKEPIT_VERBOSE:+--verbose} 2>/dev/null || true
                fi
            fi
            ;;
    esac
    
    command flit "$@"
}

# ============================================================================
# HATCH WRAPPER
# ============================================================================

_snakepit_hatch_wrapper() {
    if ! should_intercept; then
        command hatch "$@"
        return $?
    fi
    
    echo "🐍 Snakepit: Monitoring hatch command"
    command hatch "$@"
}

# ============================================================================
# PIP-TOOLS WRAPPERS (pip-sync, pip-compile)
# ============================================================================

_snakepit_pip_sync_wrapper() {
    if ! should_intercept; then
        command pip-sync "$@"
        return $?
    fi
    
    echo "🐍 Snakepit: Intercepting pip-sync"
    
    # Parse requirements from file
    for file in "$@"; do
        [[ ! -f "$file" ]] && continue
        
        echo "📄 Validating packages from $file"
        while IFS= read -r line; do
            [[ "$line" =~ ^#.*$ ]] || [[ -z "$line" ]] && continue
            pkg_name=$(extract_package_name "$line")
            python3 "$SNAKEPIT_CLI" validate "$pkg_name" ${SNAKEPIT_VERBOSE:+--verbose} 2>/dev/null || true
        done < "$file"
    done
    
    command pip-sync "$@"
}

# Export if this script is sourced
if [[ "${BASH_SOURCE[0]}" != "${0}" ]]; then
    export -f _snakepit_pip_wrapper
    export -f _snakepit_python_wrapper
    export -f _snakepit_easy_install_wrapper
    export -f _snakepit_poetry_wrapper
    export -f _snakepit_pipenv_wrapper
    export -f _snakepit_conda_wrapper
    export -f _snakepit_pdm_wrapper
    export -f _snakepit_flit_wrapper
    export -f _snakepit_hatch_wrapper
    export -f _snakepit_pip_sync_wrapper
fi
