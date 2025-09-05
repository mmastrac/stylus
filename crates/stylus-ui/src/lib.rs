#[cfg(use_files)]
pub const STYLUS_JAVASCRIPT: &str = include_str!("compiled/stylus.js");
#[cfg(use_files)]
pub const STYLUS_JAVASCRIPT_MAP: &[u8] = include_bytes!("compiled/stylus.js.map.gz");
#[cfg(use_files)]
pub const STYLUS_CSS: &str = include_str!("compiled/stylus.css");

#[cfg(not(use_files))]
pub const STYLUS_JAVASCRIPT: &str = include_str!(concat!(env!("OUT_DIR"), "/stylus.js"));
#[cfg(not(use_files))]
pub const STYLUS_JAVASCRIPT_MAP: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/stylus.js.map.gz"));
#[cfg(not(use_files))]
pub const STYLUS_CSS: &str = include_str!(concat!(env!("OUT_DIR"), "/stylus.css"));

pub const STYLUS_HTML: &str = include_str!("../web/index-compiled.html");

pub const STYLUS_LOGO: &str = include_str!("../web/stylus.svg");
