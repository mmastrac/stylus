extern crate cargo_emit;
extern crate glob;
extern crate sheller;

use cargo_emit::{rerun_if_changed, rustc_cfg, warning};
use std::path::Path;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let compiled_script = root.join("src/compiled/stylus.js");
    let compiled_css = root.join("src/compiled/stylus.css");

    let features = std::env::var("CARGO_CFG_FEATURE").unwrap_or_default();
    let from_source_always = features.contains("from-source-always");
    let from_source_auto = features.contains("from-source-auto");

    let files_exist = compiled_script.exists() && compiled_css.exists();

    if from_source_always || (from_source_auto && !files_exist) {
        use glob::glob;
        use sheller::try_run;

        warning!("Building Stylus from source, requires deno and sass");

        for entry in glob("web/**/*")? {
            if let Ok(entry) = entry {
                if entry.is_file() {
                    rerun_if_changed!(entry.to_string_lossy());
                }
            }
        }

        let out_dir = std::env::var_os("OUT_DIR").unwrap().into_string().unwrap();

        try_run!("deno install --config web/deno.json")?;
        try_run!(
            r#"deno bundle --config web/deno.json --minify --platform browser \
            --output {out_dir}/stylus.js --sourcemap=external web/src/app.tsx"#
        )?;
        try_run!("gzip -9 --force {out_dir}/stylus.js.map")?;
        
        // Inline CSS imports manually
        let css_content = inline_css_imports("web/src/style.css")?;
        fs::write(format!("{out_dir}/stylus.css"), css_content)?;
    } else {
        rerun_if_changed!("src/compiled/stylus.js");
        rerun_if_changed!("src/compiled/stylus.css");

        if !files_exist {
            warning!("compiled/stylus.js and/or compiled/stylus.css not found");
            warning!(" ** Run with --features=from-source-auto or --features=from-source-always");
            return Err("compiled/stylus.js and/or compiled/stylus.css not found".into());
        }

        rustc_cfg!("use_files");
    }

    Ok(())
}

fn inline_css_imports(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let mut result = String::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("@import") {
            // Parse @import 'path' or @import "path"
            if let Some(import_path) = extract_import_path(trimmed) {
                let base_dir = Path::new(file_path).parent().unwrap();
                let full_path = base_dir.join(&import_path);
                
                if full_path.exists() {
                    // Recursively inline imports from the imported file
                    let imported_content = inline_css_imports(&full_path.to_string_lossy())?;
                    result.push_str(&format!("/* Inlined from {} */\n", import_path));
                    result.push_str(&imported_content);
                    result.push('\n');
                } else {
                    // Keep the original import if file doesn't exist
                    result.push_str(line);
                    result.push('\n');
                }
            } else {
                // Keep malformed imports as-is
                result.push_str(line);
                result.push('\n');
            }
        } else {
            // Keep non-import lines as-is
            result.push_str(line);
            result.push('\n');
        }
    }
    
    Ok(result)
}

fn extract_import_path(import_line: &str) -> Option<String> {
    // Handle @import 'path' or @import "path"
    if let Some(start) = import_line.find('\'') {
        if let Some(end) = import_line[start + 1..].find('\'') {
            return Some(import_line[start + 1..start + 1 + end].to_string());
        }
    }
    
    if let Some(start) = import_line.find('"') {
        if let Some(end) = import_line[start + 1..].find('"') {
            return Some(import_line[start + 1..start + 1 + end].to_string());
        }
    }
    
    None
}
