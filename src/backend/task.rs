use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub name: String,
    pub description: String,
    pub completed: bool,
}

impl Task {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            completed: false,
        }
    }

    pub fn from_str(s: &str) -> Self {
        let split: Vec<&str> = s.split(':').collect();
        let name = split[0].trim();
        let description = split.get(1).unwrap_or(&"").trim();

        Self {
            name: name.into(),
            description: description.to_string(),
            completed: false,
        }
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formated_task = format!(
            "Task\n\t name: {}\n\tdescription: {}\n\t completed: {}",
            self.name, self.description, self.completed
        );

        write!(f, "{}", formated_task)
    }
}
