from pathlib import Path
from dedupl.image import compute_perceptual_hash
import time
import sys

# Locate the tests directory dynamically
BASE_DIR = Path(__file__).resolve().parent
TEST_IMAGE_DIR = BASE_DIR / "stress_images"

# Collect all image files recursively
IMAGE_EXTS = {".jpg", ".jpeg", ".png", ".webp", ".bmp", ".gif", ".tif", ".tiff", ".heic", ".heif"}

images = [
    p for p in TEST_IMAGE_DIR.rglob("*")
    if p.suffix.lower() in IMAGE_EXTS
]

if not images:
    print("No images found in tests/unit.")
    sys.exit(1)

print(f"Found {len(images)} images for benchmarking")

start = time.perf_counter()

for img in images:
    compute_perceptual_hash(img)

end = time.perf_counter()

total_time = end - start
avg_time = total_time / len(images)

print(f"Total time: {total_time:.4f} seconds")
print(f"Average per image: {avg_time:.6f} seconds")