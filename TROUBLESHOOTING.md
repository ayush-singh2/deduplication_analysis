# Troubleshooting Guide

## Virtual Environment Issues

### Error: "pre-commit install failed"

```sh
process_begin: CreateProcess(NULL, pre-commit install, ...) failed.
```

**Solution**: Pre-commit setup is optional and won't prevent development:

```bash
# Basic setup (pre-commit optional)
make dev-setup

# Or setup pre-commit manually later
make setup-pre-commit
```

Pre-commit hooks are useful for code quality but not required for basic development.

### Error: "No virtual environment found"

```sh
error: No virtual environment found; run `uv venv` to create an environment
```

**Solution**: The Makefile now handles this automatically:

```bash
make dev-setup      # Complete setup (creates venv + installs dependencies)
```

Or manually:

```bash
make venv           # Create virtual environment first
make install-dev    # Then install dependencies
```

### Virtual Environment Location

The virtual environment is created in `.venv/` directory:

- **Windows**: `.venv\Scripts\activate`
- **Linux/Mac**: `source .venv/bin/activate`

### Manual Virtual Environment Management

If you prefer manual control:

```bash
# Create virtual environment
uv venv

# Activate (Windows)
.venv\Scripts\activate

# Activate (Linux/Mac)
source .venv/bin/activate

# Install dependencies
uv pip install -e ".[dev,all]"
```

## External Dependencies

### Missing fpcalc or ffprobe

Check what's missing:

```bash
make check-deps
```

**Install missing tools:**

#### Windows

```powershell
# Using chocolatey
choco install ffmpeg chromaprint

# Or download manually:
# FFmpeg: https://ffmpeg.org/download.html
# Chromaprint: https://acoustid.org/chromaprint
```

#### Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install ffmpeg
sudo apt-get install libchromaprint-tools
```

#### macOS

```bash
brew install ffmpeg chromaprint
```

## Testing Issues

### Tests failing with import errors

Ensure virtual environment is activated and dependencies installed:

```bash
make dev-setup      # Complete setup
make test          # Run tests
```

### Module not found errors

The project uses `src/` layout. Ensure you're using the Makefile commands which handle paths correctly:

```bash
make test          # Uses uv run automatically
make lint          # Uses uv run automatically
```

## Performance Issues

### Slow installation

Use `uv` cache and parallel installation:

```bash
# Clear cache if needed
uv cache clean

# Reinstall
make clean
make dev-setup
```

### Memory issues during testing

Run specific test modules:

```bash
make test-unit      # Unit tests only
make test-audio     # Audio tests only
make test-image     # Image tests only
```

## Common Errors

### "Command not found: make"

#### Windows

Install via chocolatey:

```powershell
choco install make
```

Or use PowerShell alternatives:

```powershell
# Instead of make dev-setup
uv venv
uv pip install -e ".[dev,all]"
```

#### WSL (Windows Subsystem for Linux)

```bash
sudo apt-get install build-essential
```

### Path issues on Windows

Use forward slashes or escape backslashes in paths:

```bash
# Good
make example-audio-dry

# For direct script usage, use forward slashes
uv run dedupl_audio_fingerprints.py C:/path/to/music --dry-run
```

### Permission errors

Run terminal as administrator (Windows) or use sudo (Linux/Mac) only if absolutely necessary. Usually not required for virtual environments.

## Development Issues

### Pre-commit hooks failing

```bash
make format         # Auto-fix formatting issues
make lint          # Check remaining issues
make type-check    # Verify type hints
```

### Import errors in IDE

Configure your IDE to use the virtual environment:

- **VSCode**: Select Python interpreter from `.venv/`
- **PyCharm**: Configure project interpreter to point to `.venv/`

## Getting Help

### Debug information

Collect this information when reporting issues:

```bash
# System info
uv --version
python --version
make --version

# Project status
make check-deps
ls -la .venv/      # Check virtual environment

# Test a simple command
make venv
```

### Enable verbose output

For debugging Makefile issues:

```bash
make -n dev-setup   # Show commands without executing
make -d dev-setup   # Debug makefile execution
```

## Quick Fixes

### Reset everything

```bash
make clean          # Remove all build artifacts and .venv
make dev-setup      # Start fresh
```

### Verify installation

```bash
make dev-setup      # Complete setup
make check-deps     # Verify external tools
make test-unit      # Quick test run
make example-audio-dry  # Test with safe example
```
