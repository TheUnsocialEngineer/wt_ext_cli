use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use regex::Regex;
use serde_json::{Value, json};

/// Extracts the full `modifications { ... }` block from a .blk file by balancing braces.
fn extract_modifications_block(content: &str) -> Option<String> {
    if let Some(start) = content.find("modifications") {
        if let Some(open_brace) = content[start..].find('{') {
            let mut depth = 0;
            let mut block = String::new();
            for c in content[start + open_brace + 1..].chars() {
                if c == '{' {
                    depth += 1;
                    block.push(c);
                } else if c == '}' {
                    if depth == 0 {
                        return Some(block);
                    } else {
                        depth -= 1;
                        block.push(c);
                    }
                } else {
                    block.push(c);
                }
            }
        }
    }
    None
}

pub fn extract_ammo_from_blk(input_folder: &str, output_file: &str) -> io::Result<()> {
    let mut result = json!({ "tanks": [] });

    // Capture identifiers followed by `{` at the start of a line
    let key_regex = Regex::new(r"(?m)^\s*([A-Za-z0-9_]+)\s*\{").unwrap();

    // Calibre regex: e.g. 25mm, 125mm, 410mm
    let calibre_regex = Regex::new(r"(?i)\d+mm").unwrap();

    // Ammo keyword regex (must appear as a token: start, _, or end)
    let keyword_regex = Regex::new(
        r"(?i)(?:^|_)(?:API|APIT|APDS|APHE|APCBC|APC|AP|HEAT|HESH|HE|ATGM|VT|CN|NATO|USSR|USA|SW)(?:_|$)"
    ).unwrap();

    for entry in fs::read_dir(input_folder)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name().into_string().unwrap_or_default();
        println!("\nProcessing file: {}", filename);

        // Dedupe per file
        let mut ammos_set: HashSet<String> = HashSet::new();

        match path.extension().and_then(|e| e.to_str()) {
            // Handle blkx (JSON)
            Some("blkx") => {
                let mut content = String::new();
                File::open(&path)?.read_to_string(&mut content)?;
                match serde_json::from_str::<Value>(&content) {
                    Ok(data) => {
                        if let Some(mods) = data.get("modifications").and_then(|m| m.as_object()) {
                            for key in mods.keys() {
                                let key = key.trim();
                                if key.ends_with("_ammo_pack") {
                                    continue;
                                }
                                if calibre_regex.is_match(key) || keyword_regex.is_match(key) {
                                    ammos_set.insert(key.to_string());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Failed to parse {} as JSON: {}", filename, e);
                    }
                }
            }

            // Handle blk (raw text)
            Some("blk") => {
                let mut content = String::new();
                File::open(&path)?.read_to_string(&mut content)?;
                if let Some(block_str) = extract_modifications_block(&content) {
                    for cap in key_regex.captures_iter(&block_str) {
                        let key = cap.get(1).map(|m| m.as_str()).unwrap_or("").trim();
                        if key.is_empty() || key.ends_with("_ammo_pack") {
                            continue;
                        }
                        if calibre_regex.is_match(key) || keyword_regex.is_match(key) {
                            ammos_set.insert(key.to_string());
                        }
                    }
                } else {
                    println!("  No modifications block found in blk.");
                }
            }

            _ => {
                println!("  Skipping non-blk/blkx file: {}", filename);
            }
        }

        if !ammos_set.is_empty() {
            let mut ammos: Vec<String> = ammos_set.into_iter().collect();
            ammos.sort();
            result["tanks"].as_array_mut().unwrap().push(json!({ filename: ammos }));
        }
    }

    let out_str = serde_json::to_string_pretty(&result)?;
    let mut out = File::create(output_file)?;
    out.write_all(out_str.as_bytes())?;
    println!("\nAmmo extraction complete. Output written to {}", output_file);
    Ok(())
}
