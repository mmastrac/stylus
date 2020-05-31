use std::collections::{BTreeMap, HashMap};
use std::error::Error;

use handlebars::*;
use itertools::Itertools;
use serde_json::value::*;

use crate::config::MonitorDirAxisValue;
use crate::status::*;

pub fn interpolate_monitor(monitor: &MonitorState, s: &str) -> Result<String, Box<dyn Error>> {
    // TODO: avoid creating this handlebars registry every time
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("t", s)?;

    let mut map = BTreeMap::new();
    map.insert("monitor", monitor);
    Ok(handlebars.render("t", &map)?.trim().to_owned())
}

pub fn interpolate_id(
    values: &HashMap<&String, &MonitorDirAxisValue>,
    s: &str,
) -> Result<String, Box<dyn Error>> {
    // TODO: avoid creating this handlebars registry every time
    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("t", s)?;

    Ok(handlebars.render("t", values)?.trim().to_owned())
}

pub fn interpolate_modify(status: &mut MonitorStatus, s: &str) -> Result<(), Box<dyn Error>> {
    let (raw_path, value) = s.splitn(2, '=').next_tuple().ok_or("Invalid expression")?;
    let value: Value = serde_json::from_str(value)?;
    let mut path = raw_path.split('.');
    let pending = status
        .pending
        .get_or_insert_with(MonitorPendingStatus::default);

    match path.next() {
        Some("status") => {}
        _ => return Err(format!("Invalid path: {}", raw_path).into()),
    }
    match path.next() {
        Some("status") => {
            pending.status = Some(serde_json::from_value(value)?);
        }
        Some("description") => {
            pending.description = Some(serde_json::from_value(value)?);
        }
        Some("metadata") => match path.next() {
            Some(s) => {
                let metadata = pending.metadata.get_or_insert_with(HashMap::new);
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

    fn update(s: &'static str) -> Result<MonitorStatus, Box<dyn Error>> {
        let mut status = MonitorStatus {
            status: StatusState::Blank,
            code: 0,
            description: "".to_owned(),
            metadata: Default::default(),
            pending: None,
            css: MonitorCssStatus {
                metadata: Default::default(),
            },
        };

        interpolate_modify(&mut status, s)?;
        Ok(status)
    }

    #[test]
    fn test_modify() -> Result<(), Box<dyn Error>> {
        let status = update("status.status=\"red\"")?;
        assert_eq!(status.pending.unwrap().status.unwrap(), StatusState::Red);
        let status = update("status.description=\"foo\"")?;
        assert_eq!(status.pending.unwrap().description.unwrap(), "foo");
        let status = update("status.metadata.foo=\"bar\"")?;
        let mut map = HashMap::new();
        map.insert("foo".to_owned(), "bar".to_owned());
        assert_eq!(status.pending.unwrap().metadata.unwrap(), map);
        Ok(())
    }
}
