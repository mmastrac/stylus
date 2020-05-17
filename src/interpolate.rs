use std::collections::BTreeMap;
use std::error::Error;

use handlebars::*;

use crate::status::*;

pub fn interpolate_monitor(monitor: &MonitorState, s: &str) -> Result<String, Box<dyn Error>> {
    // TODO: avoid creating this handlebars registry every time
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("t", s)?;

    let mut map = BTreeMap::new();
    map.insert("monitor", monitor);
    Ok(handlebars.render("t", &map)?.trim().to_owned())
}
