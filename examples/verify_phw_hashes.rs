// Verify the hash values bridge_v2 used against the actual engine hash algorithm.
// Crimson Desert uses Jenkins lookup3 (hashlittle) for these string lookups.
// We don't know the seed for game-data hashes, so try common seeds (0, INTEGRITY_SEED).

fn hashlittle(data: &[u8], initval: u32) -> u32 {
    let length = data.len();
    let mut a: u32 = 0xDEADBEEFu32.wrapping_add(length as u32).wrapping_add(initval);
    let mut b: u32 = a;
    let mut c: u32 = a;

    let mut i = 0;
    while i + 12 <= length {
        a = a.wrapping_add(u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]));
        b = b.wrapping_add(u32::from_le_bytes([data[i+4], data[i+5], data[i+6], data[i+7]]));
        c = c.wrapping_add(u32::from_le_bytes([data[i+8], data[i+9], data[i+10], data[i+11]]));

        a = a.wrapping_sub(c); a ^= c.rotate_left(4); c = c.wrapping_add(b);
        b = b.wrapping_sub(a); b ^= a.rotate_left(6); a = a.wrapping_add(c);
        c = c.wrapping_sub(b); c ^= b.rotate_left(8); b = b.wrapping_add(a);
        a = a.wrapping_sub(c); a ^= c.rotate_left(16); c = c.wrapping_add(b);
        b = b.wrapping_sub(a); b ^= a.rotate_left(19); a = a.wrapping_add(c);
        c = c.wrapping_sub(b); c ^= b.rotate_left(4); b = b.wrapping_add(a);

        i += 12;
    }

    let rem = length - i;
    if rem > 0 {
        let mut tail = [0u8; 12];
        tail[..rem].copy_from_slice(&data[i..]);

        if rem >= 12 {
            c = c.wrapping_add(u32::from_le_bytes([tail[8], tail[9], tail[10], tail[11]]));
        } else if rem >= 9 {
            let mask = 0xFFFFFFFFu32 >> (8 * (12 - rem));
            c = c.wrapping_add(u32::from_le_bytes([tail[8], tail[9], tail[10], tail[11]]) & mask);
        }
        if rem >= 8 {
            b = b.wrapping_add(u32::from_le_bytes([tail[4], tail[5], tail[6], tail[7]]));
        } else if rem >= 5 {
            let mask = 0xFFFFFFFFu32 >> (8 * (8 - rem));
            b = b.wrapping_add(u32::from_le_bytes([tail[4], tail[5], tail[6], tail[7]]) & mask);
        }
        if rem >= 4 {
            a = a.wrapping_add(u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]));
        } else if rem >= 1 {
            let mask = 0xFFFFFFFFu32 >> (8 * (4 - rem));
            a = a.wrapping_add(u32::from_le_bytes([tail[0], tail[1], tail[2], tail[3]]) & mask);
        } else {
            return c;
        }
    }

    c ^= b; c = c.wrapping_sub(b.rotate_left(14));
    a ^= c; a = a.wrapping_sub(c.rotate_left(11));
    b ^= a; b = b.wrapping_sub(a.rotate_left(25));
    c ^= b; c = c.wrapping_sub(b.rotate_left(16));
    a ^= c; a = a.wrapping_sub(c.rotate_left(4));
    b ^= a; b = b.wrapping_sub(a.rotate_left(14));
    c ^= b; c = c.wrapping_sub(b.rotate_left(24));

    c
}

fn main() {
    // Bridge_v2 claimed values:
    //   1767116530 — "Player_PHW" appearance (offset 185)
    //   3755051597 — "Player_PHW_Lower" character_prefab_path (offset 189)
    //   3000129643 — "phw_description_player_001" or similar — for skeleton_name slot
    //   2831867940 — "character/model/1_pc/2_phw/phw_01.pab" — for lookup_24 slot
    //   3511542393 — "character/binary/skeletonvariation/.../cd_phw_00_nude_00_0001.pabc" — lookup_25

    let expected = [
        ("Player_PHW",              1767116530u32),
        ("Player_PHM",              0u32),  // for reference, vanilla Kliff value 2343703689 — what is its string?
        ("Player_PHW_Lower",        3755051597u32),
        ("Player_PHM_Lower",        0u32),  // vanilla
        ("phw_description_player_001",                                              3000129643u32),
        ("phm_description_player_001",                                              0u32),  // vanilla
        ("character/model/1_pc/2_phw/phw_01.pab",                                   2831867940u32),
        ("character/model/1_pc/1_phm/phm_01.pab",                                   0u32),  // vanilla
        ("character/binary/skeletonvariation/1_pc/2_phw/nude/cd_phw_00_nude_00_0001.pabc",  3511542393u32),
        ("character/binary/skeletonvariation/1_pc/1_phm/nude/cd_phm_00_nude_00_0001.pabc",  0u32),  // vanilla
    ];

    // Seeds to try
    let seeds: &[u32] = &[0, 0xC5EDE, 0xDEADBEEF, 0x60616263];

    println!("Verifying expected hash values against hashlittle(text, seed) for various seeds:\n");
    for (s, want) in &expected {
        println!("string: {:?}  expected: {}", s, want);
        for &seed in seeds {
            let h = hashlittle(s.as_bytes(), seed);
            let mark = if h == *want && *want != 0 { " ✓ MATCH" } else { "" };
            println!("  seed=0x{:08X}  hash={:10} (0x{:08X}){}", seed, h, h, mark);
        }
        // also try lowercase
        let lower = s.to_lowercase();
        if lower != *s {
            for &seed in seeds {
                let h = hashlittle(lower.as_bytes(), seed);
                let mark = if h == *want && *want != 0 { " ✓ MATCH (lowercase)" } else { "" };
                if !mark.is_empty() {
                    println!("  seed=0x{:08X}  hash={:10} (0x{:08X}){}", seed, h, h, mark);
                }
            }
        }
        println!();
    }

    println!("\n--- Reverse: what does vanilla Kliff appearance_name (2343703689 = 0x8BB21489) come from? ---");
    println!("Expected to be 'Player_PHM' or similar. Trying common candidates...");
    let candidates = ["Player_PHM", "PHM", "kliff_appearance", "Kliff", "player_phm", "PlayerPhm",
        "Player_Kliff", "PlayerKliff", "Kliff_Default", "phm_appearance",
        "_appearanceName", "appearance_name"];
    for c in &candidates {
        for &seed in seeds {
            let h = hashlittle(c.as_bytes(), seed);
            if h == 2343703689 {
                println!("  MATCH: {:?} seed=0x{:08X} → 2343703689", c, seed);
            }
            let l = c.to_lowercase();
            if l != *c {
                let h2 = hashlittle(l.as_bytes(), seed);
                if h2 == 2343703689 {
                    println!("  MATCH (lc): {:?} seed=0x{:08X} → 2343703689", c, seed);
                }
            }
        }
    }
}
