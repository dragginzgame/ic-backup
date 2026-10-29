//! Decode the parent-owned journey configuration once before building any plans.
use ic_backup::model::operation_plan::PlanContextRecord;
use serde_json::Value;
use std::{io, path::Path};

pub(super) struct Config {
    pub context: PlanContextRecord,
    pub target: String,
    pub endpoint: String,
    pub key: Vec<u8>,
}

impl Config {
    pub(super) fn read(root: &Path) -> io::Result<Self> {
        let value: Value = serde_json::from_slice(&super::read(&root.join("config.json"))?)?;
        let field = |name: &str| value.get(name).cloned().unwrap_or(Value::Null);
        Ok(Self {
            context: serde_json::from_value(field("context"))?,
            target: serde_json::from_value(field("target"))?,
            endpoint: serde_json::from_value(field("endpoint"))?,
            key: serde_json::from_value(field("key"))?,
        })
    }
}
