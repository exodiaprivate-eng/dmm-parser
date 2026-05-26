// SPDX-License-Identifier: LicenseRef-CDMTL-1.0
// Copyright (c) 2026 RicePaddySoftware. All Rights Reserved.
// Licensed under CDMTL v1.0 - see LICENSE.txt
// https://github.com/exodiaprivate-eng/dmm-parser

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use dmm_parser::binary::pamt::{Compression, CryptoType, PackMeta};
use dmm_parser::binary::papgt::PackGroupTreeMeta;
use dmm_parser::binary::paz;

const EXPECTED_FILES: [&str; 12] = [
    "0000/0.pamt",
    "0009/0.pamt",
    "0013/0.pamt",
    "0013/0.paz",
    "0015/0.pamt",
    "0016/0.pamt",
    "0016/0.paz",
    "0032/0.pamt",
    "0032/0.paz",
    "meta/0.papgt",
    "meta/0.pathc",
    "meta/0.paver",
];

const PATHC_SIZE: usize = 6_850_216;
const PAVER_SIZE: usize = 10;

fn sample_root() -> Option<PathBuf> {
    let Some(root) = std::env::var_os("DMM_PARSER_SAMPLE_ROOT") else {
        eprintln!("SKIP: DMM_PARSER_SAMPLE_ROOT not set");
        return None;
    };

    if root.is_empty() {
        eprintln!("SKIP: DMM_PARSER_SAMPLE_ROOT is empty");
        return None;
    }

    let root = PathBuf::from(root);
    assert!(
        root.is_dir(),
        "DMM_PARSER_SAMPLE_ROOT is not a directory: {}",
        root.display()
    );
    Some(root)
}

fn sample_path(root: &Path, rel: &str) -> PathBuf {
    rel.split('/').fold(root.to_path_buf(), |path, part| path.join(part))
}

fn read_sample(root: &Path, rel: &str) -> Vec<u8> {
    let path = sample_path(root, rel);
    fs::read(&path).unwrap_or_else(|err| panic!("cannot read {}: {}", path.display(), err))
}

fn read_papgt(root: &Path) -> PackGroupTreeMeta {
    let data = read_sample(root, "meta/0.papgt");
    PackGroupTreeMeta::parse(&data).expect("meta/0.papgt should parse")
}

fn group_checksum(papgt: &PackGroupTreeMeta, group: &str) -> u32 {
    papgt
        .entries
        .iter()
        .find(|entry| entry.group_name == group)
        .unwrap_or_else(|| panic!("PAPGT should contain group {}", group))
        .entry
        .pack_meta_checksum
}

fn read_pamt(root: &Path, group: &str) -> PackMeta {
    let papgt = read_papgt(root);
    let checksum = group_checksum(&papgt, group);
    let data = read_sample(root, &format!("{}/0.pamt", group));
    PackMeta::parse(&data, Some(checksum))
        .unwrap_or_else(|err| panic!("{}/0.pamt should parse with PAPGT checksum: {}", group, err))
}

fn collect_relative_files(root: &Path) -> BTreeSet<String> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeSet<String>) {
        for entry in fs::read_dir(dir).unwrap_or_else(|err| {
            panic!("cannot list {}: {}", dir.display(), err);
        }) {
            let entry = entry.unwrap_or_else(|err| panic!("cannot read directory entry: {}", err));
            let path = entry.path();
            let file_type = entry
                .file_type()
                .unwrap_or_else(|err| panic!("cannot inspect {}: {}", path.display(), err));

            if file_type.is_dir() {
                visit(root, &path, out);
            } else if file_type.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .unwrap_or_else(|err| panic!("cannot strip prefix for {}: {}", path.display(), err))
                    .components()
                    .map(|component| component.as_os_str().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join("/");
                out.insert(rel);
            } else {
                panic!("unexpected non-file entry under sample root: {}", path.display());
            }
        }
    }

    let mut files = BTreeSet::new();
    visit(root, root, &mut files);
    files
}

fn assert_expected_chunks_exist(root: &Path, group: &str, pamt: &PackMeta) {
    let group_dir = sample_path(root, group);
    for chunk in &pamt.chunks {
        let paz_path = group_dir.join(format!("{}.paz", chunk.id));
        let metadata = fs::metadata(&paz_path)
            .unwrap_or_else(|err| panic!("cannot stat {}: {}", paz_path.display(), err));
        assert_eq!(
            metadata.len(),
            chunk.size as u64,
            "{} chunk {} size mismatch",
            group,
            chunk.id
        );
    }
}

fn file_count(pamt: &PackMeta) -> usize {
    pamt.directories.iter().map(|dir| dir.files.len()).sum()
}

fn flag_validation_gaps(group: &str, pamt: &PackMeta) -> Vec<String> {
    let mut gaps = Vec::new();

    for (idx, raw_file) in pamt.raw_files.iter().enumerate() {
        let compression_nibble = raw_file.flags & 0x0f;
        let crypto_nibble = raw_file.flags >> 4;

        if compression_nibble > 4 {
            gaps.push(format!(
                "{} raw file #{} has unknown compression nibble {} in flags 0x{:02x}",
                group, idx, compression_nibble, raw_file.flags
            ));
        }
        if crypto_nibble > 3 {
            gaps.push(format!(
                "{} raw file #{} has unknown crypto nibble {} in flags 0x{:02x}",
                group, idx, crypto_nibble, raw_file.flags
            ));
        }
    }

    for dir in &pamt.directories {
        for file in &dir.files {
            let full_path = if dir.path.is_empty() {
                file.name.clone()
            } else {
                format!("{}/{}", dir.path, file.name)
            };

            match file.file.compression {
                Compression::QuickLz => gaps.push(format!(
                    "{} {} uses unsupported QuickLz compression",
                    group, full_path
                )),
                Compression::Partial => gaps.push(format!(
                    "{} {} decoded to unexpected Compression::Partial",
                    group, full_path
                )),
                Compression::None | Compression::Lz4 | Compression::Zlib => {}
            }

            match file.file.crypto {
                CryptoType::Ice => gaps.push(format!(
                    "{} {} uses unsupported Ice crypto",
                    group, full_path
                )),
                CryptoType::Aes => gaps.push(format!(
                    "{} {} uses unsupported Aes crypto",
                    group, full_path
                )),
                CryptoType::None | CryptoType::ChaCha20 => {}
            }
        }
    }

    gaps
}

fn assert_no_flag_validation_gaps(group: &str, pamt: &PackMeta) {
    let gaps = flag_validation_gaps(group, pamt);
    assert!(
        gaps.is_empty(),
        "VALIDATION GAP: unsupported or unknown flags observed:\n{}",
        gaps.join("\n")
    );
}

fn extract_group_entries(root: &Path, group: &str, pamt: &PackMeta) {
    assert_no_flag_validation_gaps(group, pamt);

    let group_dir = sample_path(root, group);
    for dir in &pamt.directories {
        for file in &dir.files {
            let extracted = paz::extract_file(
                &group_dir,
                file,
                &dir.path,
                &pamt.header.encrypt_info.encrypt_info,
            )
            .unwrap_or_else(|err| {
                let full_path = if dir.path.is_empty() {
                    file.name.clone()
                } else {
                    format!("{}/{}", dir.path, file.name)
                };
                panic!("{} extraction failed for {}: {}", group, full_path, err);
            });

            assert_eq!(
                extracted.len(),
                file.file.uncompressed_size as usize,
                "{}:{} extracted size mismatch",
                group,
                file.name
            );
        }
    }
}

fn hex_prefix(data: &[u8], count: usize) -> String {
    data.iter()
        .take(count)
        .map(|byte| format!("{:02x}", byte))
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn sample_bundle_contains_only_expected_paths() {
    let Some(root) = sample_root() else {
        return;
    };

    let actual = collect_relative_files(&root);
    let expected = EXPECTED_FILES
        .into_iter()
        .map(String::from)
        .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected, "sample bundle should contain only approved files");
}

#[test]
fn parse_current_meta_papgt() {
    let Some(root) = sample_root() else {
        return;
    };

    let papgt = read_papgt(&root);
    assert!(!papgt.entries.is_empty(), "PAPGT should contain entries");

    let groups = papgt
        .entries
        .iter()
        .map(|entry| entry.group_name.as_str())
        .collect::<BTreeSet<_>>();
    for group in ["0000", "0009", "0013", "0015", "0016", "0032"] {
        assert!(groups.contains(group), "PAPGT should contain group {}", group);
    }

    println!("PAPGT entries: {}", papgt.entries.len());
}

#[test]
fn roundtrip_current_meta_papgt() {
    let Some(root) = sample_root() else {
        return;
    };

    let data = read_sample(&root, "meta/0.papgt");
    let papgt = PackGroupTreeMeta::parse(&data).expect("meta/0.papgt should parse");
    let written = papgt.to_bytes().expect("meta/0.papgt should serialize");

    assert_eq!(written, data, "PAPGT roundtrip should be byte-for-byte stable");
}

#[test]
fn parse_tiny_0016_pamt_with_papgt_checksum() {
    let Some(root) = sample_root() else {
        return;
    };

    let pamt = read_pamt(&root, "0016");
    assert_expected_chunks_exist(&root, "0016", &pamt);
    assert!(!pamt.chunks.is_empty(), "0016 should contain chunks");
    assert!(!pamt.directories.is_empty(), "0016 should contain directories");
    assert!(file_count(&pamt) > 0, "0016 should contain files");

    println!(
        "0016: chunks={}, directories={}, files={}, raw_files={}",
        pamt.chunks.len(),
        pamt.directories.len(),
        file_count(&pamt),
        pamt.raw_files.len()
    );
}

#[test]
fn extract_tiny_0016_entries() {
    let Some(root) = sample_root() else {
        return;
    };

    let pamt = read_pamt(&root, "0016");
    extract_group_entries(&root, "0016", &pamt);
}

#[test]
fn parse_0013_pamt_with_papgt_checksum() {
    let Some(root) = sample_root() else {
        return;
    };

    let pamt = read_pamt(&root, "0013");
    assert_expected_chunks_exist(&root, "0013", &pamt);
    assert!(!pamt.chunks.is_empty(), "0013 should contain chunks");
    assert!(!pamt.directories.is_empty(), "0013 should contain directories");
    assert!(file_count(&pamt) > 0, "0013 should contain files");

    println!(
        "0013: chunks={}, directories={}, files={}, raw_files={}",
        pamt.chunks.len(),
        pamt.directories.len(),
        file_count(&pamt),
        pamt.raw_files.len()
    );
}

#[test]
fn extract_0013_entries_smoke() {
    let Some(root) = sample_root() else {
        return;
    };

    let pamt = read_pamt(&root, "0013");
    extract_group_entries(&root, "0013", &pamt);
}

#[test]
fn parse_0032_pamt_with_papgt_checksum() {
    let Some(root) = sample_root() else {
        return;
    };

    let pamt = read_pamt(&root, "0032");
    assert_expected_chunks_exist(&root, "0032", &pamt);
    assert!(!pamt.chunks.is_empty(), "0032 should contain chunks");
    assert!(!pamt.directories.is_empty(), "0032 should contain directories");
    assert!(file_count(&pamt) > 0, "0032 should contain files");

    println!(
        "0032: chunks={}, directories={}, files={}, raw_files={}",
        pamt.chunks.len(),
        pamt.directories.len(),
        file_count(&pamt),
        pamt.raw_files.len()
    );
}

#[test]
fn extract_0032_entries_smoke() {
    let Some(root) = sample_root() else {
        return;
    };

    let pamt = read_pamt(&root, "0032");
    extract_group_entries(&root, "0032", &pamt);
}

#[test]
fn parse_large_metadata_pamts_with_papgt_checksum() {
    let Some(root) = sample_root() else {
        return;
    };

    for group in ["0000", "0009", "0015"] {
        let pamt = read_pamt(&root, group);
        assert!(!pamt.chunks.is_empty(), "{} should contain chunks", group);
        assert!(
            !pamt.directories.is_empty(),
            "{} should contain directories",
            group
        );
        assert!(file_count(&pamt) > 0, "{} should contain files", group);

        println!(
            "{} metadata: chunks={}, directories={}, files={}, raw_files={}",
            group,
            pamt.chunks.len(),
            pamt.directories.len(),
            file_count(&pamt),
            pamt.raw_files.len()
        );
    }
}

#[test]
fn compare_0013_0016_pamt_layouts() {
    let Some(root) = sample_root() else {
        return;
    };

    let pamt_0013 = read_pamt(&root, "0013");
    let pamt_0016 = read_pamt(&root, "0016");

    assert_expected_chunks_exist(&root, "0013", &pamt_0013);
    assert_expected_chunks_exist(&root, "0016", &pamt_0016);

    println!(
        "0013 layout: chunks={}, directories={}, files={}, raw_directories={}, raw_files={}",
        pamt_0013.chunks.len(),
        pamt_0013.directories.len(),
        file_count(&pamt_0013),
        pamt_0013.raw_directories.len(),
        pamt_0013.raw_files.len()
    );
    println!(
        "0016 layout: chunks={}, directories={}, files={}, raw_directories={}, raw_files={}",
        pamt_0016.chunks.len(),
        pamt_0016.directories.len(),
        file_count(&pamt_0016),
        pamt_0016.raw_directories.len(),
        pamt_0016.raw_files.len()
    );

    assert_eq!(
        pamt_0013.raw_files.len(),
        file_count(&pamt_0013),
        "0013 raw/resolved file count should match"
    );
    assert_eq!(
        pamt_0016.raw_files.len(),
        file_count(&pamt_0016),
        "0016 raw/resolved file count should match"
    );
}

#[test]
fn survey_unsupported_flags_second_wave_groups() {
    let Some(root) = sample_root() else {
        return;
    };

    let mut gaps = Vec::new();
    for group in ["0013", "0016", "0032", "0000", "0009", "0015"] {
        let pamt = read_pamt(&root, group);
        gaps.extend(flag_validation_gaps(group, &pamt));
    }

    if gaps.is_empty() {
        println!(
            "No unsupported compression/encryption flags observed in 0013/0016/0032/0000/0009/0015 samples"
        );
    } else {
        println!("Unsupported flag observations:\n{}", gaps.join("\n"));
    }

    assert!(
        gaps.is_empty(),
        "VALIDATION GAP: unsupported or unknown flags observed:\n{}",
        gaps.join("\n")
    );
}

#[test]
fn pathc_header_smoke() {
    let Some(root) = sample_root() else {
        return;
    };

    let data = read_sample(&root, "meta/0.pathc");
    assert_eq!(data.len(), PATHC_SIZE, "meta/0.pathc size changed");
    assert!(
        data.iter().any(|byte| *byte != 0),
        "meta/0.pathc should not be all zero bytes"
    );
    println!("PATHC first 32 bytes: {}", hex_prefix(&data, 32));
}

#[test]
fn paver_version_smoke() {
    let Some(root) = sample_root() else {
        return;
    };

    let data = read_sample(&root, "meta/0.paver");
    assert_eq!(data.len(), PAVER_SIZE, "meta/0.paver size changed");
    assert!(
        data.iter().any(|byte| *byte != 0),
        "meta/0.paver should not be all zero bytes"
    );
    println!("PAVER raw bytes: {}", hex_prefix(&data, data.len()));
    println!("PAVER lossy text: {:?}", String::from_utf8_lossy(&data));
}
