# dedupl - Command Reference

## Quick Start (Recommended)

Use Make commands for the best development experience:

```bash
# Setup development environment
make dev-setup

# Run safe examples (dry-run mode)
make example-audio-dry
make example-image-dry
make example-video-dry

# Development tasks
make test           # Run tests
make lint           # Check code quality
make check-deps     # Verify dependencies
```

## All Make Commands

```bash
make help           # Show all available commands

# Installation
make install        # Production install
make install-dev    # Development install
make dev-setup      # Complete setup

# Testing
make test           # All tests
make test-unit      # Unit tests only
make test-coverage  # With coverage
make test-audio     # Audio tests
make test-image     # Image tests

# Code Quality
make lint           # Check style
make format         # Auto-format
make type-check     # Type checking
make check          # All quality checks

# Examples (safe)
make example-audio-dry
make example-image-dry
make example-video-dry

# Maintenance
make clean          # Clean artifacts
make build          # Build package
```

---

## Direct Module Usage

**Note:** The legacy scripts have been removed. Use the modular components from `src/dedupl/` to build custom workflows:

### Audio Files Deduplication

#### Using Make for Audio

```bash
# Safe preview
make example-audio-dry

# Test audio functionality
make test-audio
```

#### Using Audio Module Directly

```python
# Example usage of src/dedupl/audio.py
from src.dedupl.audio import AudioMeta, generate_fingerprint
from src.dedupl.common import walk_files_by_extension

# Your custom audio deduplication script here
```

#### Creating Custom Audio Scripts

Create your own script using the audio module:

```python
#!/usr/bin/env python3
from src.dedupl import audio, common
from tqdm import tqdm

# Build your custom audio deduplication workflow
```

---

### Image Files Deduplication

#### Using Make for Images

```bash
# Safe preview
make example-image-dry

# Test image functionality
make test-image
```

#### Using Image Module Directly

##### Install dependencies

```bash
# Install required packages
uv pip install pillow imagehash tqdm pillow-heif
```

##### Create custom image script

```python
#!/usr/bin/env python3
from src.dedupl.image import ImageMeta, compute_perceptual_hash
from src.dedupl.common import walk_files_by_extension
from tqdm import tqdm

# Build your custom image deduplication workflow
```

### Video Files Deduplication

#### Using Make for Videos

```bash
# Safe preview
make example-video-dry
```

#### Using Common Module for Videos

##### Install dependencies

```bash
# Install required packages
uv pip install opencv-python-headless pillow imagehash tqdm
```

##### Create custom video script

```python
#!/usr/bin/env python3
from src.dedupl.common import walk_files_by_extension, execute_command
from tqdm import tqdm
import cv2

# Build your custom video deduplication workflow
# Video functionality can be implemented using common utilities
# and following patterns from audio/image modules
```

