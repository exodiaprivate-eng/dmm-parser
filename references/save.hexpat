// Crimson Desert save file — 0x80 header pattern (Phase S1)
//
// Format reference: references/save_notes.md §1.1.
//
// Usage:
//   plcli run -i <savefile> -p references/save.hexpat -v -d
//
// Only the header is patternable in the clear; everything past 0x80
// is ChaCha20 ciphertext and intentionally left as opaque bytes.

#pragma endian little

import std.mem;

struct SaveHeader {
    char    magic[4];          // "SAVE"
    u16     version;           // current saves: 2
    u16     flags;             // observed 0x0080
    u8      reserved_a[10];    // preserved on rewrite, semantics TBD
    u32     uncompressed_size; // post-LZ4 plaintext body size
    u32     payload_size;      // ciphertext size after 0x80
    u8      nonce[16];         // [0..4)=ChaCha20 init counter LE, [4..16)=nonce
    u8      hmac[32];          // HMAC-SHA256(compressed, key)
    u8      reserved_b[54];    // padding to 0x80
};

struct SaveFile {
    SaveHeader  header;
    u8          payload[header.payload_size]; // chacha20(lz4(body))
    // EOF expected immediately after payload.
};

SaveFile save @ 0x00;
