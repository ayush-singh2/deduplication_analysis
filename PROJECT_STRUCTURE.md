# Project Structure Documentation

## Overview

The deDupl project has been refactored with a modern, maintainable structure following Python best practices.

## Directory Layout

```sh
dedupl/
├── src/                          # Source code
│   └── dedupl/                   # Main package
│       ├── __init__.py           # Package initialization
│       ├── common.py             # Shared utilities
│       ├── audio.py              # Audio deduplication logic
│       └── image.py              # Image deduplication logic
│
├── tests/                       # Test suite
│   ├── __init__.py
│   ├── unit/                    # Unit tests
│   │   ├── test_common.py       # Tests for common utilities
│   │   ├── test_audio.py        # Tests for audio module
│   │   └── test_image.py        # Tests for image module
│   ├── integration/             # Integration tests
│   └── fixtures/                # Test data and fixtures
│
├── .github/                     # GitHub configuration
│   └── workflows/
│       ├── ci.yml               # CI/CD workflow
│       ├── claude-code-review.yml # Claude code review workflow
│       └── claude.yml           # Claude workflow
│
├── pyproject.toml               # Project configuration and dependencies
├── pytest.ini                   # Pytest configuration
├── Makefile                     # Development tasks
├── README.md                    # User documentation
├── CLAUDE.md                    # Claude AI context
├── IMPROVEMENT_SUMMARY.md       # Improvement documentation
├── PROJECT_STRUCTURE.md         # This file
├── command.md                   # Command examples
├── GITHUB_SETUP.md             # GitHub setup instructions
├── TROUBLESHOOTING.md          # Troubleshooting guide
├── .python-version             # Python version specification
├── uv.lock                     # Dependency lock file
└── .gitignore                  # Git ignore patterns
```

## Key Components

### Source Package (`src/dedupl/`)

#### `common.py`

Core utilities shared across all deduplication modules:

- `DeduplicationConfig`: Configuration management
- `DuplicateStats`: Statistics tracking
- Security utilities: Path validation, command sanitization
- File operations: SHA-1 hashing, directory walking
- External tool management: Dependency checking, command execution
- Progress and logging utilities

#### `audio.py`

Audio deduplication using Chromaprint fingerprinting:

- `AudioMeta`: Audio file metadata
- `FingerprintEntry`: Fingerprint with metadata
- `generate_fingerprint()`: Chromaprint integration
- `probe_ffprobe()`: FFmpeg metadata extraction
- `select_best_quality()`: Intelligent duplicate selection
- `group_duplicates()`: Fingerprint-based grouping

#### `image.py`

Image deduplication using perceptual hashing:

- `ImageMeta`: Image file metadata
- `compute_perceptual_hash()`: pHash generation
- `group_by_exact_hash()`: SHA-1 based grouping
- `group_by_perceptual_hash()`: Visual similarity grouping
- `select_best_quality()`: Resolution-based selection

**Note**: Video deduplication module is planned for future development.

### Test Suite (`tests/`)

#### Unit Tests

Comprehensive test coverage for all modules:

- **test_common.py**: 40+ tests covering utilities, security, and configuration
- **test_audio.py**: 25+ tests for audio fingerprinting and metadata
- **test_image.py**: 30+ tests for image hashing and grouping

Test categories:

- Path security validation
- Command execution safety
- File operations
- Dependency checking
- Quality selection algorithms
- Duplicate grouping logic
- Error handling

### Development Tools

#### Makefile Commands

```bash
make install        # Install package
make install-dev    # Install with dev dependencies
make test          # Run all tests
make test-coverage # Generate coverage report
make lint          # Check code style
make format        # Auto-format code
make type-check    # Run mypy type checking
make check-deps    # Verify external dependencies
make clean         # Remove build artifacts
```

#### CI/CD Pipeline (GitHub Actions)

- **ci.yml**: Main CI/CD pipeline with linting, testing, and coverage
- **claude-code-review.yml**: Automated code review using Claude AI
- **claude.yml**: Claude AI integration workflow
- **Linting**: Black, Ruff, Mypy
- **Testing**: Python 3.8-3.12 on Linux, Windows, macOS
- **Coverage**: Automated coverage reporting
- **Build**: Distribution package creation

## Design Principles

### 1. Separation of Concerns

- Core logic separated from CLI interfaces
- Shared utilities in common module
- Media-specific logic in dedicated modules

### 2. Security First

- Path traversal prevention
- Command injection protection
- Symlink detection
- Input validation

### 3. Comprehensive Testing

- Unit tests for all functions
- Mocked external dependencies
- Edge case coverage
- Security test cases

### 4. Type Safety

- Complete type hints
- Mypy strict mode
- Runtime validation

### 5. Error Handling

- Detailed logging
- Graceful degradation
- User-friendly messages
- Debug information

## Development Workflow

### Setting Up Development Environment

```bash
# Clone repository
git clone <repo-url>
cd DeDuplicationCodes_python

# Install with dev dependencies
make install-dev

# Check external dependencies
make check-deps

# Run tests
make test
```

### Adding New Features

1. Create feature branch
2. Write tests first (TDD)
3. Implement feature
4. Run full test suite
5. Check code quality
6. Submit pull request

### Testing Guidelines

- Write unit tests for all new functions
- Mock external dependencies
- Test error conditions
- Verify security constraints
- Check performance impact

## Future Enhancements

### Planned Features

1. Video deduplication module development
2. GUI interface
3. Database backend for large collections
4. Cloud storage support
5. Parallel processing improvements
6. Machine learning-based similarity detection

### Technical Debt

1. Add integration tests
2. Improve performance profiling
3. Add benchmarking suite
4. Enhance documentation
5. Complete test coverage for edge cases

## Dependencies

### Python Requirements

- **Core**: Python 3.8+
- **Progress**: tqdm
- **Images**: Pillow, imagehash, pillow-heif
- **Video**: OpenCV, numpy
- **Testing**: pytest, pytest-cov, pytest-mock
- **Quality**: black, ruff, mypy

### External Tools

- **ffprobe**: Audio/video metadata (FFmpeg)
- **fpcalc**: Audio fingerprinting (Chromaprint)

## Contributing

### Code Style

- Black formatting (line length: 100)
- Type hints required
- Docstrings for public functions
- Security validation for file operations

### Testing Requirements

- Minimum 80% code coverage
- All tests must pass
- Security tests required for file operations
- Performance tests for algorithms

### Review Process

1. Automated CI checks
2. Code review required
3. Security review for file operations
4. Documentation updates

## License

MIT License - See LICENSE file for details
