# deDupl

**deDupl** is a Python toolkit for detecting and removing duplicate audio, image, and video files using perceptual and content-based fingerprinting. It helps you free up disk space and organize your media library efficiently.

## Features

- Scans directories for duplicate audio, image, and video files based on content (not just filename or metadata)
- Uses Chromaprint (`fpcalc`) and `ffprobe` for audio, perceptual hashes for images and videos
- Supports recursive directory scanning
- Provides options to preview, move, or delete duplicates
- Fast and efficient comparison using hashing and multithreading

## Quick Start

1. **Clone and setup**:

   ```sh
   git clone https://github.com/humanrightsconnected/dedupl.git
   cd dedupl
   make dev-setup      # Complete setup (creates venv + installs dependencies)
   ```

   Or step by step:

   ```sh
   make venv           # Create virtual environment
   make install-dev    # Install dependencies
   ```

2. **Check dependencies**:

   ```sh
   make check-deps     # Verify external tools are installed
   ```

3. **Test the installation**:

   ```sh
   make test          # Run test suite
   ```

### External Dependencies

The following external tools are required and will be checked by `make check-deps`:

- **fpcalc** (from Chromaprint) - for audio fingerprinting
- **ffprobe** (from FFmpeg) - for audio/video metadata

### Installation Options

```sh
make venv           # Create virtual environment (.venv)
make install        # Production installation
make install-dev    # Development installation
make dev-setup      # Complete development setup (recommended)
```

## Usage

### Quick Examples

Use the modular components to build custom deduplication workflows:

```sh
# Preview audio duplicates (safe)
make example-audio-dry

# Preview image duplicates (safe)
make example-image-dry

# Preview video duplicates (safe)
make example-video-dry
```

See [command.md](./command.md) for detailed usage examples and the **Makefile** for all available development commands.

### Audio Deduplication

**Using Make (Recommended):**

```sh
# Safe preview (dry-run)
make example-audio-dry

# Run tests for audio module
make test-audio
```

**Using the Module:**

Import and use the audio deduplication functionality from `src/dedupl/audio.py` to build custom workflows.

---

### Image Deduplication

**Using Make (Recommended):**

```sh
# Safe preview (dry-run)
make example-image-dry

# Run tests for image module
make test-image
```

**Using the Module:**

Import and use the image deduplication functionality from `src/dedupl/image.py` to build custom workflows.

---

### Video Deduplication

**Using Make (Recommended):**

```sh
# Safe preview (dry-run)
make example-video-dry
```

**Using the Module:**

Video deduplication functionality can be implemented using the common utilities from `src/dedupl/common.py` and following the patterns established in the audio and image modules.

---

## Development

### Available Make Commands

```sh
make help           # Show all available commands
make install        # Production installation
make install-dev    # Development installation
make dev-setup      # Complete environment setup
make check-deps     # Verify external dependencies

# Testing
make test           # Run all tests
make test-unit      # Unit tests only
make test-coverage  # Tests with coverage report
make test-audio     # Audio module tests
make test-image     # Image module tests

# Code Quality
make lint           # Check code style
make format         # Auto-format code
make type-check     # Type checking with mypy
make check          # Run all quality checks

# Examples (safe dry-run mode)
make example-audio-dry
make example-image-dry
make example-video-dry

# Maintenance
make clean          # Remove build artifacts
make build          # Build distribution packages
```

### Requirements

- **Python 3.8+**
- **External tools**: fpcalc (Chromaprint), ffprobe (FFmpeg)
- **Dependencies**: Automatically managed via `make install-dev`
- **Security scanning**: bandit (included in dev dependencies)

Use `make check-deps` to verify all requirements are met.

## Contributing

### Development Workflow

1. **Setup**: `make dev-setup`
2. **Pre-commit hooks**: `make setup-pre-commit` (recommended)
3. **Code**: Make your changes
4. **Test**: `make test`
5. **Quality**: `make check` (format, lint, type-check)
6. **Pre-commit check**: `make pre-commit` (optional - runs automatically on commit)
7. **Commit**: Standard git workflow

### Pre-commit Hooks

This project uses pre-commit hooks to ensure code quality. After setup, they run automatically on every commit.

**Setup (one-time):**

```sh
make setup-pre-commit
```

**Manual execution (if needed):**

```sh
make pre-commit          # Run all hooks on all files
```

**If hooks fail:**

1. Fix the issues reported by the hooks
2. Stage your changes: `git add .`
3. Commit again: `git commit -m "your message"`

**Common hook failures and fixes:**

- **Black formatting**: Run `make format` to auto-fix
- **Ruff linting**: Run `make format` to auto-fix most issues
- **mypy errors**: Fix type annotations in your code
- **Test failures**: Fix failing tests in `tests/unit/`
- **Security issues (bandit)**: Review and fix security vulnerabilities

### Getting Started

Contributions welcome! Use the Makefile for all development tasks:

- `make dev-setup` - Get started
- `make setup-pre-commit` - Setup git hooks (recommended)
- `make test` - Ensure tests pass
- `make check` - Verify code quality
- `make pre-commit` - Run hooks manually

See `make help` for all available commands.

## License

MIT License - See LICENSE file for details.
