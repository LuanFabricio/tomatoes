use std::time::Duration;

pub struct Pomodoro {
    focus: Timer,
    rest: Timer,
    tasks: Vec<Task>,
    timer: TimerType,
}

#[derive(Copy, Clone)]
pub enum TimerType {
    Focus,
    Rest,
}

impl Pomodoro {
    pub fn new(focus_time: Duration, rest_time: Duration) -> Self {
        Self {
            focus: Timer::new(focus_time),
            rest: Timer::new(rest_time),
            tasks: vec![],
            timer: TimerType::Focus,
        }
    }

    pub fn foward(&mut self) -> Duration {
        match self.timer {
            TimerType::Focus => {
                self.focus.current_time -= Duration::from_secs(1);

                if self.focus.current_time == Duration::ZERO {
                    self.timer = TimerType::Rest;
                    self.focus.current_time = self.focus.initial_time
                }
                self.focus.current_time
            }
            TimerType::Rest => {
                self.rest.current_time -= Duration::from_secs(1);

                if self.rest.current_time == Duration::ZERO {
                    self.timer = TimerType::Focus;
                    self.rest.current_time = self.rest.initial_time
                }
                self.rest.current_time
            }
        }
    }

    pub fn get_mode(&self) -> TimerType {
        self.timer
    }

    pub fn task_add(&mut self, new_task: Task) {
        self.tasks.push(new_task);
    }

    pub fn task_remove(&mut self, task_index: usize) -> Task {
        self.tasks.remove(task_index)
    }

    pub fn task_complete(&mut self, task_index: usize) {
        for (i, task) in self.tasks.iter_mut().filter(|x| !x.completed).enumerate() {
            if i == task_index {
                task.completed = true;
                break;
            }
        }
    }

    pub fn task_not_complete(&mut self, task_index: usize) {
        for (i, task) in self.tasks.iter_mut().filter(|x| x.completed).enumerate() {
            if i == task_index {
                task.completed = false;
                break;
            }
        }
    }

    pub fn task_get_by_complete(&self, completed: bool) -> Vec<Task> {
        let completed_tasks: Vec<Task> = self
            .tasks
            .clone()
            .into_iter()
            .filter(|task| task.completed == completed)
            .collect();

        completed_tasks
    }

    pub fn to_string(&self) -> String {
        let timer_type = match self.timer {
            TimerType::Focus => String::from("Focus"),
            TimerType::Rest => String::from("Rest"),
        };

        let timer_string = self.timer_to_string();
        format!("{timer_type}: \n\t {timer_string}")
    }

    pub fn timer_to_string(&self) -> String {
        let duration = self.get_current_duration();

        let secs = duration.as_secs() % 60;
        let mins = (duration.as_secs_f32() - (secs as f32)) / 60f32;

        format!("{:02}:{:02}", mins, secs)
    }

    fn get_current_duration(&self) -> Duration {
        match self.timer {
            TimerType::Focus => self.focus.current_time,
            TimerType::Rest => self.rest.current_time,
        }
    }
}

pub struct Timer {
    current_time: Duration,
    initial_time: Duration,
}

impl Timer {
    pub fn new(initial_time: Duration) -> Self {
        Self {
            current_time: initial_time,
            initial_time,
        }
    }
}

#[derive(Debug, Clone)]
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
