use std::collections::BTreeMap;
use std::error::Error;

use handlebars::*;
use serde::Serialize;
use serde_json::value::*;

use crate::config::{MonitorDirAxisValue, MonitorDirTestConfig};
use crate::status::*;

pub fn interpolate_monitor(
    id: &str,
    config: &MonitorDirTestConfig,
    status: &MonitorStatus,
    s: &str,
) -> Result<String, Box<dyn Error>> {
    // TODO: avoid creating this handlebars registry every time
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("t", s)?;

    let mut map = BTreeMap::new();
    #[derive(Clone, Debug, Serialize)]
    struct Monitor<'a> {
        id: &'a str,
        config: &'a MonitorDirTestConfig,
        status: &'a MonitorStatus,
    }
    map.insert("monitor", Monitor { id, config, status });
    Ok(handlebars.render("t", &map)?.trim().to_owned())
}

pub fn interpolate_id(
    values: &BTreeMap<String, MonitorDirAxisValue>,
    s: &str,
) -> Result<String, Box<dyn Error>> {
    // TODO: avoid creating this handlebars registry every time
    let mut handlebars = Handlebars::new();
    handlebars.set_strict_mode(true);
    handlebars.register_template_string("t", s)?;

    Ok(handlebars.render("t", values)?.trim().to_owned())
}

pub fn interpolate_modify<'a>(
    mut status: &'a mut MonitorStatus,
    children: &'a mut BTreeMap<String, MonitorChildStatus>,
    s: &str,
) -> Result<(), Box<dyn Error>> {
    let (raw_path, value) = s.split_once('=').ok_or("Invalid expression")?;
    let value: Value = serde_json::from_str(value)?;
    let mut path = raw_path.split('.');

    match path.next() {
        Some("status") => {}
        Some("group") => {
            let part = path.next().ok_or("Missing group child")?;
            status = &mut children
                .entry(part.to_owned())
                .or_insert_with(|| {
                    let mut status = MonitorChildStatus::default();
                    if let Some((_, index)) = part.rsplit_once('-') {
                        if let Ok(index) = index.parse::<i64>() {
                            status
                                .axes
                                .insert("index".to_owned(), MonitorDirAxisValue::Number(index));
                        }
                    }
                    status
                })
                .status;
            if path.next() != Some("status") {
                return Err(format!("Invalid path: {}", raw_path).into());
            }
        }
        _ => return Err(format!("Invalid path: {}", raw_path).into()),
    };

    let pending = status
        .pending
        .get_or_insert_with(MonitorPendingStatus::default);

    match path.next() {
        Some("status") => {
            pending.status = Some(serde_json::from_value(value)?);
        }
        Some("description") => {
            pending.description = Some(serde_json::from_value(value)?);
        }
        Some("metadata") => match path.next() {
            Some(s) => {
                let metadata = pending.metadata.get_or_insert_with(BTreeMap::new);
                drop(metadata.insert(s.to_owned(), serde_json::from_value(value)?))
            }
            _ => return Err(format!("Invalid path: {}", raw_path).into()),
        },
        _ => return Err(format!("Invalid path: {}", raw_path).into()),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::FromIterator;
    use std::sync::Arc;

    fn update(s: &'static str) -> Result<MonitorStatus, Box<dyn Error>> {
        let mut status = Default::default();

        interpolate_modify(&mut status, &mut BTreeMap::new(), s)?;
        Ok(status)
    }

    #[test]
    fn test_interpolate_id() -> Result<(), Box<dyn Error>> {
        let mut values = BTreeMap::new();
        values.insert("index".to_owned(), MonitorDirAxisValue::Number(2));
        assert_eq!(interpolate_id(&values, "port-{{ index }}")?, "port-2");
        Ok(())
    }

    #[test]
    fn test_interpolate_error() -> Result<(), Box<dyn Error>> {
        let mut values = BTreeMap::new();
        values.insert("index".to_owned(), MonitorDirAxisValue::Number(2));
        assert!(interpolate_id(&values, "port-{{ ondex }}").is_err());
        Ok(())
    }

    #[test]
    fn test_replace() -> Result<(), Box<dyn Error>> {
        let config = Default::default();
        let mut status = MonitorStatus::default();
        status.css.metadata = Arc::new(BTreeMap::from_iter(vec![(
            "color".to_owned(),
            "blue".to_owned(),
        )]));
        assert_eq!(
            interpolate_monitor(
                "id",
                &config,
                &status,
                "{{monitor.status.css.metadata.color}}"
            )?,
            "blue"
        );
        Ok(())
    }

    #[test]
    fn test_modify() -> Result<(), Box<dyn Error>> {
        let status = update("status.status=\"red\"")?;
        assert_eq!(status.pending.unwrap().status.unwrap(), StatusState::Red);
        let status = update("status.description=\"foo\"")?;
        assert_eq!(status.pending.unwrap().description.unwrap(), "foo");
        let status = update("status.metadata.foo=\"bar\"")?;
        let mut map = BTreeMap::new();
        map.insert("foo".to_owned(), "bar".to_owned());
        assert_eq!(status.pending.unwrap().metadata.unwrap(), map);
        Ok(())
    }
}
