use std::io::{Read, Write};

use super::Task;

pub struct PomoFile;

impl PomoFile {
    pub fn load() -> std::io::Result<Vec<Task>> {
        let mut file = std::fs::File::open(".data/tasks")?;
        let mut task_string = String::new();

        let _ = file.read_to_string(&mut task_string)?;

        let tasks_data: Vec<&str> = task_string.as_str().split("\n").collect();

        let mut tasks: Vec<Task> = vec![];
        for task_data in tasks_data {
            if task_data.len() == 0 {
                continue;
            }

            let task_data: Vec<&str> = task_data.split(":").collect();
            let task = Task::new(task_data[0], task_data[1]);
            tasks.push(task);
        }

        Ok(tasks)
    }

    pub fn save(tasks: Vec<Task>) -> std::io::Result<()> {
        let _ = Self::create_data_folder();

        if let Ok(mut file) = std::fs::File::create(".data/tasks") {
            for task in tasks {
                file.write_all(format!("{}:{}\n", task.name, task.description).as_bytes())?;
            }
        }

        Ok(())
    }

    fn create_data_folder() -> std::io::Result<()> {
        std::fs::create_dir(".data")?;
        Ok(())
    }
}
