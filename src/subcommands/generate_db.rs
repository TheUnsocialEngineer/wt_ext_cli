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

pub fn build_tank_db(input_folder: &str, output_file: &str) -> io::Result<()> {
    let blk_regex = Regex::new(r"weapon_presets\s*\{([\s\S]*?)\}").unwrap();
    let name_regex = Regex::new(r#"name:t\s*=\s*"([^"]+)""#).unwrap();

    let key_regex = Regex::new(r"(?m)^\s*([A-Za-z0-9_]+)\s*\{").unwrap();
    let calibre_regex = Regex::new(r"(?i)\d+mm").unwrap();
    let keyword_regex = Regex::new(
        r"(?i)(?:^|_)(?:API|APIT|APDS|APHE|APCBC|APC|AP|HEAT|HESH|HE|ATGM|VT|CN|NATO|USSR|USA|SW)(?:_|$)"
    ).unwrap();

    let mut tanks: Vec<Value> = Vec::new();

    for entry in fs::read_dir(input_folder)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name().into_string().unwrap_or_default();
        let stem = path.file_stem().unwrap_or_default().to_string_lossy().to_string();

        let mut presets: Vec<String> = Vec::new();
        let mut ammos_set: HashSet<String> = HashSet::new();

        match path.extension().and_then(|e| e.to_str()) {
            Some("blk") => {
                let mut content = String::new();
                File::open(&path)?.read_to_string(&mut content)?;

                // Extract presets
                if let Some(block) = blk_regex.captures(&content).and_then(|cap| cap.get(1)) {
                    for name_cap in name_regex.captures_iter(block.as_str()) {
                        if let Some(preset) = name_cap.get(1).map(|m| m.as_str()) {
                            presets.push(preset.to_string());
                        }
                    }
                }

                // Extract ammo
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
                }
            }
            Some("blkx") => {
                let mut content = String::new();
                File::open(&path)?.read_to_string(&mut content)?;

                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                    // Presets
                    if let Some(presets_val) = data.get("weapon_presets").and_then(|wp| wp.get("preset")) {
                        match presets_val {
                            Value::Object(obj) => {
                                if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                                    presets.push(name.to_string());
                                }
                            }
                            Value::Array(arr) => {
                                for p in arr {
                                    if let Some(name) = p.get("name").and_then(|n| n.as_str()) {
                                        presets.push(name.to_string());
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    // Ammo
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
            }
            _ => {}
        }

        // Build tank entry
        if !presets.is_empty() || !ammos_set.is_empty() {
            let mut ammos: Vec<String> = ammos_set.into_iter().collect();
            ammos.sort();

            let tank_entry = json!({
                "ID": stem,
                "name": format!("{} (USA)", stem), // placeholder country logic
                "country": "USA",                  // TODO: detect dynamically
                "weapons_default": presets,
                "ammo_amount": 25,                 // placeholder
                "image": format!("{}.png", stem),
                "role": "Medium tank",             // placeholder
                "ammo": ammos
            });

            tanks.push(tank_entry);
        }
    }

    let out_str = serde_json::to_string_pretty(&tanks)?;
    let mut out = File::create(output_file)?;
    out.write_all(out_str.as_bytes())?;
    println!("\nTank DB complete. Output written to {}", output_file);
    Ok(())
}
