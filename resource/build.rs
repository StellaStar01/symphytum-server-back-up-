use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    const MASTER_PROTOS: &str = "../types/protobufs/entity/master";

    println!("cargo::rerun-if-changed={MASTER_PROTOS}");

    let mut files: Vec<PathBuf> = match fs::read_dir(MASTER_PROTOS) {
        Ok(entries) => entries.filter_map(|e| e.ok().map(|e| e.path())).collect(),
        Err(_) => Vec::new(),
    };
    files.sort();

    let mut tables: Vec<(String, String)> = Vec::with_capacity(files.len() + 2);
    for path in &files {
        if path.extension().and_then(|s| s.to_str()) != Some("proto") {
            continue;
        }

        println!("cargo::rerun-if-changed={}", path.display());

        let text = match fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue,
        };

        let mut rest = text.as_str();

        while let Some(rel) = rest.find("message ") {
            rest = &rest[rel + "message ".len()..];

            let name_end = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());

            let name = &rest[..name_end];

            let Some(body_start_rel) = rest[name_end..].find('{') else {
                continue;
            };

            let body_start = name_end + body_start_rel;
            let Some(body_end) = find_matching_brace(&rest[body_start..]) else {
                continue;
            };

            let body = &rest[body_start..=body_start + body_end];
            if body.contains("(options.master.table) = true") {
                tables.push((upper_snake(name), format!("types::entity::master::{name}")));
            }
        }
    }

    let out = format!(
        "master_tables!(\n    {}\n);\n",
        tables
            .iter()
            .map(|(name, full)| format!("({name}, {full})"))
            .collect::<Vec<_>>()
            .join(",\n    ")
    );

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    fs::write(PathBuf::from(out_dir).join("master_tables.rs"), out)
        .expect("write master_tables.rs");
}

fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

// ParkPrintBooth -> PARK_PRINT_BOOTH
// MusicMv -> MUSIC_MV
fn upper_snake(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 12);
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_uppercase());
    }
    out
}
