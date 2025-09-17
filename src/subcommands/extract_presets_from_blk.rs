use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::Path;
use regex::Regex;
use serde_json::{Value, json};

pub fn extract_presets_from_blk(input_folder: &str, output_file: &str) -> io::Result<()> {
    let mut result = json!({"tanks": []});

    let blk_regex = Regex::new(r"weapon_presets\s*\{([\s\S]*?)\}").unwrap();
    let name_regex = Regex::new(r#"name:t\s*=\s*"([^"]+)""#).unwrap();

    for entry in fs::read_dir(input_folder)? {
        let entry = entry?;
        let path = entry.path();
        let filename = entry.file_name().into_string().unwrap_or_default();

        match path.extension().and_then(|e| e.to_str()) {
            Some("blk") => {
                let mut content = String::new();
                File::open(&path)?.read_to_string(&mut content)?;
                if let Some(block) = blk_regex.captures(&content).and_then(|cap| cap.get(1)) {
                    for name_cap in name_regex.captures_iter(block.as_str()) {
                        let preset = name_cap.get(1).map(|m| m.as_str()).unwrap_or("");
                        result["tanks"].as_array_mut().unwrap().push(json!({filename.clone(): preset}));
                    }
                }
            }
            Some("blkx") => {
                let mut content = String::new();
                File::open(&path)?.read_to_string(&mut content)?;
                if let Ok(data) = serde_json::from_str::<Value>(&content) {
                    if let Some(presets) = data.get("weapon_presets").and_then(|wp| wp.get("preset")) {
                        match presets {
                            Value::Object(obj) => {
                                if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                                    result["tanks"].as_array_mut().unwrap().push(json!({filename: name}));
                                }
                            }
                            Value::Array(arr) => {
                                for p in arr {
                                    if let Some(name) = p.get("name").and_then(|n| n.as_str()) {
                                        result["tanks"].as_array_mut().unwrap().push(json!({filename.clone(): name}));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut out = File::create(output_file)?;
    out.write_all(serde_json::to_string_pretty(&result)?.as_bytes())?;
    Ok(())
}