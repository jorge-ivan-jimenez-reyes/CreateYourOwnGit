import os
import struct

os.makedirs(".git", exist_ok=True)

with open(".git/index", "wb") as f:
    f.write(b'DIRC')                          # Magic
    f.write(struct.pack(">I", 2))             # Version
    f.write(struct.pack(">I", 1))             # One entry

    f.write(b'\x00' * 62)                     # Fake metadata
    f.write(b'\x00' * 20)                     # Fake SHA-1
    f.write(b'hola.txt')                      # File name
    f.write(b'\x00')                          # Null terminator

    total_len = 62 + 20 + len("hola.txt") + 1
    padding = (8 - (total_len % 8)) % 8
    f.write(b'\x00' * padding)

print(" hola.txt")
