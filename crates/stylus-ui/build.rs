extern crate cargo_emit;
extern crate glob;
extern crate sheller;

use cargo_emit::{rerun_if_changed, rustc_cfg, warning};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let compiled_script = root.join("compiled/stylus.js");
    let compiled_css = root.join("compiled/stylus.css");

    let files_exist = compiled_script.exists() && compiled_css.exists();

    if cfg!(feature = "from-source-always") || (cfg!(feature = "from-source-auto") && !files_exist)
    {
        use glob::glob;
        use sheller::try_run;

        warning!("Building Stylus from source, requires deno");

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
        try_run!("gzip -9 {out_dir}/stylus.js.map")?;
        std::fs::copy("web/src/style.css", format!("{out_dir}/stylus.css"))?;
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
