put your (decrypted) packet binaries here!

then you can run `bin.ps1 inspector`

you can get them by disabling cert pin in the game and then use mitmproxy with a script like this:

```python

"""
Decrypts the game's encrypted gRPC payloads and saves decrypted protobuf
messages to:

    log/<ROUTE>_<TIMESTAMP>_<REQ|RESP>_<INDEX>.bin

Example usage: mitmweb --mode local:hololive-Dreams.exe -s <this_script_file_name>
"""

import hashlib
import struct
import zlib
from datetime import datetime
from pathlib import Path

from cryptography.hazmat.primitives.ciphers import Cipher, algorithms, modes
from mitmproxy import ctx, http

SECRET_KEYWORD = b"BJYZ0w3DJm"
AES_KEY = hashlib.md5(SECRET_KEYWORD).digest()
TARGET_HOST = "as.game-hololive-dreams.com"

LOG_DIR = Path(__file__).resolve().parent / "log"
LOG_DIR.mkdir(parents=True, exist_ok=True)

ctx.log.info(f"Log directory: {LOG_DIR.resolve()}")


def aes_cbc_decrypt(key: bytes, iv: bytes, ciphertext: bytes) -> bytes:
    cipher = Cipher(algorithms.AES(key), modes.CBC(iv))
    decryptor = cipher.decryptor()
    return decryptor.update(ciphertext) + decryptor.finalize()


def decrypt_all_frames(data: bytes) -> list[bytes]:
    results = []
    pos = 0

    while pos < len(data):
        remaining = data[pos:]

        if len(remaining) < 6:
            ctx.log.info(f"Remaining too small ({len(remaining)} bytes)")
            break

        frame_len = struct.unpack(">I", remaining[1:5])[0]

        if frame_len == 0:
            ctx.log.info("Encountered zero-length frame")
            break

        if len(remaining) < 5 + frame_len:
            ctx.log.warning(
                f"Truncated frame: expected {frame_len} bytes, have {len(remaining)-5}"
            )
            break

        msg = remaining[5 : 5 + frame_len]
        pos += 5 + frame_len

        if len(msg) < 6:
            ctx.log.warning("Message too small")
            continue

        hdr_size = msg[0] | (msg[1] << 8)
        hdr_compress = msg[2]
        keylen = msg[3]

        if keylen < 1 or keylen > 128:
            ctx.log.warning(f"Invalid key length: {keylen}")
            continue

        if hdr_size != keylen + 2:
            ctx.log.warning(
                f"Unexpected header size: hdr_size={hdr_size}, keylen={keylen}"
            )
            continue

        total_hdr = hdr_size + 2

        if len(msg) < total_hdr:
            ctx.log.warning(
                f"Header larger than message ({total_hdr} > {len(msg)})"
            )
            continue

        key = msg[4 : 4 + keylen]
        ciphertext = msg[total_hdr:]
        iv = hashlib.md5(SECRET_KEYWORD + key).digest()

        try:
            plaintext = aes_cbc_decrypt(AES_KEY, iv, ciphertext)

            pad = plaintext[-1]
            if 1 <= pad <= 16:
                plaintext = plaintext[:-pad]

            if hdr_compress:
                plaintext = zlib.decompress(plaintext)

            results.append(plaintext)

        except Exception as e:
            ctx.log.error(f"Decrypt failed: {e!r}")

    return results


def save_decrypted(flow: http.HTTPFlow, direction: str):
    content = flow.request.content if direction == "REQ" else flow.response.content

    if not content:
        ctx.log.info(f"{direction}: empty body")
        return

    ctx.log.info(f"{direction}: encrypted body = {len(content)} bytes")

    bodies = decrypt_all_frames(content)

    ctx.log.info(f"{direction}: decrypted messages = {len(bodies)}")

    if not bodies:
        return

    route = (
        flow.request.path.split("?", 1)[0]
        .strip("/")
        .replace("/", "_")
        .replace("\\", "_")
    )

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S_%f")[:-3]

    for i, body in enumerate(bodies):
        filename = (
            f"{route}_{timestamp}_{direction}_{i}.bin"
        )

        path = LOG_DIR / filename

        try:
            path.write_bytes(body)
            ctx.log.info(f"Wrote {len(body)} bytes -> {path}")
        except Exception as e:
            ctx.log.error(f"Failed writing {path}: {e!r}")

def request(flow: http.HTTPFlow):
    ctx.log.info(
        f"REQ {flow.request.method} {flow.request.pretty_url} "
        f"(host={flow.request.host})"
    )

    if TARGET_HOST in flow.request.pretty_url:
        save_decrypted(flow, "REQ")


def response(flow: http.HTTPFlow):
    ctx.log.info(
        f"RESP {flow.request.method} {flow.request.pretty_url} "
        f"(host={flow.request.host})"
    )

    if flow.response and TARGET_HOST in flow.request.pretty_url:
        save_decrypted(flow, "RESP")
```
