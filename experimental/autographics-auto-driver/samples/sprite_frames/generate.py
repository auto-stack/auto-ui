"""Deterministic fixtures for the PLAN-004 sprite frames (self-made, tiny).

The sheet is one static PNG with three 32x32 frames, each asymmetric: a red
left half and a blue right half plus a per-frame marker block, so a frame or
flip mistake changes real pixels. The border row is fully transparent, so the
fixture also exercises alpha. Nothing here is 858 candidate art.

    python samples/sprite_frames/generate.py --check
"""
import argparse
import hashlib
import pathlib
import struct
import zlib

HERE = pathlib.Path(__file__).resolve().parent


def png(width, height, pixel):
    """Encode one RGBA8 truecolour PNG without external dependencies."""
    rows = bytearray()
    for y in range(height):
        rows.append(0)  # filter: None
        for x in range(width):
            rows.extend(pixel(x, y))
    def chunk(kind, data):
        return (struct.pack(">I", len(data)) + kind + data
                + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF))
    ihdr = struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr)
            + chunk(b"IDAT", zlib.compress(bytes(rows), 9)) + chunk(b"IEND", b""))


def sheet_pixel(x, y):
    frame = x // 32
    lx = x % 32
    if y == 0:
        return (0, 0, 0, 0)  # transparent top row
    if 4 <= y < 8 and frame * 4 <= lx < frame * 4 + 4:
        return (0, 255, 0, 255)  # per-frame marker
    return (255, 0, 0, 255) if lx < 16 else (0, 0, 255, 255)


def other_pixel(x, y):
    return (0, 255, 255, 255) if x < y else (255, 255, 0, 255)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    outputs = {
        "sheet.png": png(96, 32, sheet_pixel),
        "other.png": png(16, 16, other_pixel),
    }
    ok = True
    for name, data in outputs.items():
        path = HERE / name
        digest = hashlib.sha256(data).hexdigest()
        if args.check:
            actual = hashlib.sha256(path.read_bytes()).hexdigest() if path.exists() else "missing"
            status = "PASS" if actual == digest else "FAIL"
            ok = ok and actual == digest
            print(f"{status} {name} {len(data)} {digest}")
        else:
            path.write_bytes(data)
            print(f"wrote {name} {len(data)} {digest}")
    if not ok:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
