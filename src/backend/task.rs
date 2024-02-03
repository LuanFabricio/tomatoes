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

        Self::new(name, description)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod new {
        use super::*;

        const TASK_NAME: &str = "Task name";
        const TASK_DESCRIPTION: &str = "Task description";
        #[test]
        fn should_create_a_task_with_name_and_description() {
            let task = Task::new(TASK_NAME, TASK_DESCRIPTION);

            assert_eq!(task.name, TASK_NAME);
            assert_eq!(task.description, TASK_DESCRIPTION);
        }

        #[test]
        fn should_create_a_task_with_completed_equals_false() {
            let task = Task::new(TASK_NAME, TASK_DESCRIPTION);

            assert_eq!(task.completed, false);
        }
    }

    mod from_str {
        use super::*;
        const TASK_NAME: &str = "Task name";
        const TASK_DESCRIPTION: &str = "Task description";

        #[test]
        fn should_create_a_task_with_a_string() {
            let task_str = TASK_NAME.to_string() + ":" + TASK_DESCRIPTION;
            let task = Task::from_str(task_str.as_str());

            assert_eq!(task.name, TASK_NAME);
            assert_eq!(task.description, TASK_DESCRIPTION);
        }

        #[test]
        fn should_handle_str_without_description() {
            let task_str = TASK_NAME.to_string();
            let task = Task::from_str(task_str.as_str());

            assert_eq!(task.name, TASK_NAME);
            assert_eq!(task.description, "");
        }
    }
}
