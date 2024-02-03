use crate::backend::Pomodoro;

pub struct TerminalDisplay {
    pomodoro: Pomodoro,
}

impl TerminalDisplay {
    pub fn new(pomodoro: Pomodoro) -> Self {
        Self { pomodoro }
    }

    pub fn display(&self) {
        std::process::Command::new("clear")
            .spawn()
            .expect("clear commando failed to start")
            .wait()
            .expect("failed to wait");

        let timer_string = self.pomodoro.to_string();

        let not_completed_tasks = self.pomodoro.task_get_by_complete(false);
        let mut not_completed_string = String::from("");
        for task in not_completed_tasks.into_iter() {
            not_completed_string += format!("[ ] {}: {}\n", task.name, task.description).as_str();
        }

        let completed_tasks = self.pomodoro.task_get_by_complete(true);
        let mut completed_string = String::from("");
        for task in completed_tasks.into_iter() {
            completed_string += format!("[x] {}: {}\n", task.name, task.description).as_str();
        }

        println!("{timer_string}\n\n{not_completed_string}{completed_string}");
    }

    pub fn pomo_loop(&mut self) {
        loop {
            self.pomodoro.forward();
            self.display();

            std::process::Command::new("sleep")
                .arg("1")
                .spawn()
                .expect("sleep commando failed to start")
                .wait()
                .expect("failed to wait");
        }
    }
}
