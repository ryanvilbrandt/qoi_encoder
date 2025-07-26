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

class EncodeResult(ctypes.Structure):
    _fields_ = [
        ("ptr", ctypes.c_void_p),
        ("len", ctypes.c_size_t),
        ("error", ctypes.c_uint8),
    ]

lib.encode.argtypes = [
    ctypes.c_void_p,  # data
    ctypes.c_size_t,  # width
    ctypes.c_size_t,  # height
    ctypes.c_size_t,   # channels
]
lib.encode.restype = EncodeResult

lib.free_encoded.argtypes = [ctypes.c_void_p, ctypes.c_size_t]
lib.free_encoded.restype = None

# Load test_image.bmp and print pixel bytes
img_path = os.path.join(os.path.dirname(__file__), "test_image.bmp")
img = Image.open(img_path)
pixels = img.tobytes()
width, height = img.size
channels = len(img.getbands())
print("Pixel bytes:", bytearray(pixels).hex(" ").upper())

result = lib.encode(
    ctypes.c_char_p(pixels),
    width,
    height,
    channels,
)
if result.error != 0 or not result.ptr or result.len <= 0:
    raise RuntimeError(f"Encoding failed, error code: {result.error}")
encoded_bytes = ctypes.string_at(result.ptr, result.len)
print("Encoded bytes:", encoded_bytes.hex(" ").upper())
lib.free_encoded(result.ptr, result.len)
print("Python integration test passed.")
