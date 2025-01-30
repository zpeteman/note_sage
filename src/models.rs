// models.rs
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Low
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)] // Added `Clone`
pub struct Task {
    pub id: u32,
    pub description: String,
    pub tags: Vec<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub completed: bool,
}

pub fn save_tasks(active: &[Task], archived: &[Task]) -> std::io::Result<()> {
    let data = serde_json::json!({
        "active": active,
        "archived": archived
    });
    std::fs::write("tasks.json", serde_json::to_string_pretty(&data)?)
}

pub fn load_tasks() -> (Vec<Task>, Vec<Task>) {
    match std::fs::read_to_string("tasks.json") {
        Ok(data) => {
            let parsed: serde_json::Value = serde_json::from_str(&data).unwrap_or_default();
            let active = parsed["active"]
                .as_array()
                .map(|t| serde_json::from_value(serde_json::Value::Array(t.clone())).unwrap())
                .unwrap_or_default();
            let archived = parsed["archived"]
                .as_array()
                .map(|t| serde_json::from_value(serde_json::Value::Array(t.clone())).unwrap())
                .unwrap_or_default();
            (active, archived)
        }
        Err(_) => (Vec::new(), Vec::new()),
    }
}
