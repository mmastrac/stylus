use crate::config::*;
use crate::interpolate::*;
use crate::status::*;

pub fn generate_css_for_state(config: &CssConfig, status: &Status) -> String {
    let mut css = format!("/* Generated at {:?} */\n", chrono::Utc::now().to_rfc3339());
    for monitor in &status.monitors {
        css += "\n";
        let mut monitor = monitor.write();

        // Build the css from cache
        let mut cache = monitor.css.take();
        css += cache.get_or_insert_with(|| generate_css_for_monitor(config, &monitor));
        monitor.css = cache;
    }
    css
}

pub fn generate_css_for_monitor(config: &CssConfig, monitor: &MonitorState) -> String {
    let mut css = format!("/* {} */\n", monitor.id);

    css += format!("\n/* Default rules */\n").as_str();
    css += format!("[data-monitor-id={:?}] {{\n", monitor.id).as_str();
    css += format!("  --monitor-id: {:?};\n", monitor.id).as_str();
    if let Some(status) = monitor.status.status {
        css += format!(
            "  --monitor-status: {};\n",
            status.to_string().to_lowercase()
        )
        .as_str();
    }
    css += format!("  --monitor-code: {:?};\n", monitor.status.code).as_str();
    css += format!(
        "  --monitor-description: {:?};\n",
        monitor.status.description
    )
    .as_str();
    for (k, v) in monitor.status.metadata.iter() {
        if k.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            css += format!("  --monitor-metadata-{k}: {};\n", v).as_str();
        }
    }
    css += format!("}}\n").as_str();

    for rule in &config.rules {
        css += &interpolate_monitor(
            &monitor.id,
            &monitor.config,
            &monitor.status,
            &rule.selectors,
        )
        .unwrap_or_else(|_| "/* failed */".into());
        css += " {\n";
        css += &interpolate_monitor(
            &monitor.id,
            &monitor.config,
            &monitor.status,
            &rule.declarations,
        )
        .unwrap_or_else(|_| "/* failed */".into());
        css += "\n}\n\n";
    }
    for child in monitor.children.iter() {
        for rule in &config.rules {
            css += &interpolate_monitor(child.0, &monitor.config, &child.1.status, &rule.selectors)
                .unwrap_or_else(|_| "/* failed */".into());
            css += " {\n";
            css += &interpolate_monitor(
                child.0,
                &monitor.config,
                &child.1.status,
                &rule.declarations,
            )
            .unwrap_or_else(|_| "/* failed */".into());
            css += "\n}\n\n";
        }
    }
    css
}
