# Makefile for deDupl project
# Common development and testing tasks

.PHONY: help install install-dev venv setup-pre-commit test test-unit test-integration lint format type-check clean build docs install-external-deps

# Default target
help:
	@echo "Available targets:"
	@echo "  install       - Install package in production mode"
	@echo "  install-dev   - Install package with development dependencies"
	@echo "  venv          - Create virtual environment (.venv)"
	@echo "  setup-pre-commit - Install pre-commit hooks"
	@echo "  test          - Run all tests"
	@echo "  test-unit     - Run unit tests only"
	@echo "  test-integration - Run integration tests only"
	@echo "  test-coverage - Run tests with coverage report"
	@echo "  lint          - Run code linters"
	@echo "  format        - Format code with black"
	@echo "  type-check    - Run type checking with mypy"
	@echo "  clean         - Remove build artifacts and cache files"
	@echo "  build         - Build distribution packages"
	@echo "  docs          - Generate documentation"
	@echo "  check-deps    - Check external dependencies"
	@echo "  install-external-deps - Show external dependency installation instructions"
	@echo "  pre-commit    - Run pre-commit hooks"

# Virtual environment setup
venv:
	@echo "Checking for virtual environment..."
	@uv venv --quiet 2>nul || echo "Virtual environment already exists"
	@echo "Virtual environment ready at .venv/"

# Installation targets
install: venv
	uv pip install -e .

install-dev: venv
	UV_LINK_MODE=copy uv pip install -e ".[dev,all]"

# Pre-commit setup (separate target)
setup-pre-commit: venv
	@echo "Installing pre-commit hooks..."
	@uv pip install pre-commit
	@uv run pre-commit install || echo "Pre-commit setup failed (optional)"
	@echo "Pre-commit hooks setup complete (if successful)"

# Testing targets
test: venv install-dev
	UV_LINK_MODE=copy uv run python -m pytest tests/ -v

test-unit: venv install-dev
	UV_LINK_MODE=copy uv run python -m pytest tests/unit/ -v

test-integration: venv install-dev
	UV_LINK_MODE=copy uv run python -m pytest tests/integration/ -v -m integration

test-coverage: venv install-dev
	UV_LINK_MODE=copy uv run python -m pytest tests/ --cov=src/dedupl --cov-report=html --cov-report=term-missing

test-audio: venv install-dev
	UV_LINK_MODE=copy uv run python -m pytest tests/unit/test_audio.py -v

test-image: venv install-dev
	UV_LINK_MODE=copy uv run python -m pytest tests/unit/test_image.py -v

test-common: venv install-dev
	UV_LINK_MODE=copy uv run python -m pytest tests/unit/test_common.py -v

# Code quality targets
lint: venv install-dev
	UV_LINK_MODE=copy uv run python -m ruff check src/ tests/
	UV_LINK_MODE=copy uv run python -m black --check src/ tests/

format: venv install-dev
	UV_LINK_MODE=copy uv run python -m black src/ tests/
	UV_LINK_MODE=copy uv run python -m ruff check --fix src/ tests/

type-check: venv install-dev
	UV_LINK_MODE=copy uv run python -m mypy src/dedupl

# Cleaning targets
clean:
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info
	rm -rf src/*.egg-info
	rm -rf .coverage
	rm -rf htmlcov/
	rm -rf .pytest_cache/
	rm -rf .mypy_cache/
	rm -rf .ruff_cache/
	rm -rf .venv/
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -type f -name "*.pyc" -delete
	find . -type f -name "*.pyo" -delete

# Build targets
build: venv install-dev clean
	uv run python -m build

# Documentation
docs:
	@echo "Documentation generation not yet configured"

# Check external dependencies
check-deps: venv install-dev
	@echo "Checking external dependencies..."
	@which ffprobe > /dev/null 2>&1 && echo "✓ ffprobe found" || echo "✗ ffprobe not found (required for audio/video)"
	@which fpcalc > /dev/null 2>&1 && echo "✓ fpcalc found" || echo "✗ fpcalc not found (required for audio)"
	@uv run python -c "import cv2; print('✓ OpenCV installed')" 2>nul || echo "✗ OpenCV not installed (required for video)"
	@uv run python -c "import imagehash; print('✓ imagehash installed')" 2>nul || echo "✗ imagehash not installed (required for images)"
	@uv run python -c "from PIL import Image; print('✓ Pillow installed')" 2>nul || echo "✗ Pillow not installed (required for images/video)"

# Pre-commit hooks
pre-commit: venv install-dev
	uv run python -m pre_commit run --all-files

# Install external dependencies (system-specific)
install-external-deps:
	@echo "Installing external dependencies..."
	@echo "Please install the following manually:"
	@echo "  - ffprobe (from FFmpeg): https://ffmpeg.org/download.html"
	@echo "  - fpcalc (from Chromaprint): https://acoustid.org/chromaprint"
	@echo ""
	@echo "On Windows with chocolatey:"
	@echo "  choco install ffmpeg chromaprint"
	@echo ""
	@echo "On Ubuntu/Debian:"
	@echo "  sudo apt-get install ffmpeg libchromaprint-tools"
	@echo ""
	@echo "On macOS with Homebrew:"
	@echo "  brew install ffmpeg chromaprint"

# Development workflow
dev-setup: venv install-dev check-deps
	@echo "Development environment ready!"
	@echo "To activate manually: .venv\\Scripts\\activate (Windows) or source .venv/bin/activate (Linux/Mac)"
	@echo "Optional: Run 'make setup-pre-commit' to setup git hooks"
	@echo "To install external deps: 'make install-external-deps'"

# Quick test and lint
check: venv format lint type-check test-unit
	@echo "All checks passed!"

# Run example commands (using modules since scripts were removed)
example-audio-dry:
	@echo "Audio deduplication examples require custom scripts using src/dedupl/audio.py module"
	@echo "See src/dedupl/ for available modules"

example-image-dry:
	@echo "Image deduplication examples require custom scripts using src/dedupl/image.py module"
	@echo "See src/dedupl/ for available modules"

example-video-dry:
	@echo "Video deduplication examples require custom scripts using src/dedupl/common.py module"
	@echo "See src/dedupl/ for available modules"