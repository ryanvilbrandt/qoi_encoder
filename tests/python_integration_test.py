import ctypes
import os
import sys
from PIL import Image

if sys.platform.startswith("win"):
    lib_name = "qoi_encoder.dll"
elif sys.platform.startswith("linux"):
    lib_name = "libqoi_encoder.so"
elif sys.platform.startswith("darwin"):
    lib_name = "libqoi_encoder.dylib"
else:
    raise RuntimeError(f"Unsupported platform: {sys.platform}")

lib_path = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    "target",
    "release",
    lib_name,
)
lib = ctypes.CDLL(lib_path)

# Load test_image.bmp and print pixel bytes
img_path = os.path.join(os.path.dirname(__file__), "test_image.bmp")
img = Image.open(img_path)
pixels = img.tobytes()
print("Pixel bytes:", pixels)

result = lib.encode(pixels, len(pixels))
assert result == 127, f"Expected 127, got {result}"
print("Python integration test passed.")
