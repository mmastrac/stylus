use crate::status::*;
use handlebars::*;
use std::error::Error;

pub fn interpolate_monitor(monitor: MonitorState, s: String) -> Result<String, Box<dyn Error>> {
    // TODO: avoid creating this handlebars registry every time
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("t", s)?;
    Ok(handlebars.render("t", &monitor)?)
}
