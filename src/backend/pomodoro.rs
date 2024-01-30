use super::file::PomoFile;
use super::task::*;
use super::timer::*;

use rodio::OutputStream;

use std::{io::BufReader, ops::Deref, time::Duration};

pub struct Pomodoro {
    focus: Timer,
    rest: Timer,
    tasks: Vec<Task>,
    timer: TimerType,
    play_alarm: bool,
}

impl Pomodoro {
    pub fn new(focus_time: Duration, rest_time: Duration) -> Self {
        Self {
            focus: Timer::new(focus_time),
            rest: Timer::new(rest_time),
            tasks: vec![],
            timer: TimerType::Focus,
            play_alarm: true,
        }
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        self.tasks = PomoFile::load()?;
        Ok(())
    }

    pub fn save(&self) -> std::io::Result<()> {
        PomoFile::save(self.task_get_by_complete(false))?;

        Ok(())
    }

    pub fn foward(&mut self) -> Duration {
        let one_sec = Duration::from_secs(1);
        match &self.timer {
            TimerType::Focus => {
                self.focus.current_time -= one_sec;

                if self.focus.current_time == Duration::ZERO {
                    let mut new_timer = TimerType::Rest;
                    if self.play_alarm {
                        new_timer = TimerType::Transitioning(Box::new(new_timer));
                    }
                    self.timer = new_timer;
                    self.focus.current_time = self.focus.initial_time
                }
                self.focus.current_time
            }
            TimerType::Rest => {
                self.rest.current_time -= one_sec;

                if self.rest.current_time == Duration::ZERO {
                    let mut new_timer = TimerType::Focus;
                    if self.play_alarm {
                        new_timer = TimerType::Transitioning(Box::new(new_timer));
                    }
                    self.timer = new_timer;
                    self.rest.current_time = self.rest.initial_time
                }
                self.rest.current_time
            }
            TimerType::Transitioning(s) => {
                let duration = match s.deref() {
                    TimerType::Focus => self.focus.current_time,
                    TimerType::Rest => self.rest.current_time,
                    _ => {
                        unreachable!()
                    }
                };

                self.alarm_play();
                self.timer = s.deref().clone();
                duration
            }
        }
    }

    pub fn alarm_play(&self) {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let file = std::fs::File::open("assets/sounds/clock-alarm-8761.mp3").unwrap();
        let beep = stream_handle.play_once(BufReader::new(file)).unwrap();
        // NOTE: Maybe switch to a non-blocking approach
        beep.sleep_until_end();
    }

    pub fn next_mode(&mut self) {
        self.reset_timer(self.timer.clone());
        match &self.timer {
            TimerType::Focus => {
                self.timer = TimerType::Rest;
            }
            TimerType::Rest => {
                self.timer = TimerType::Focus;
            }
            TimerType::Transitioning(_) => {
                unreachable!()
            }
        }
    }

    pub fn reset_timer(&mut self, timer_type: TimerType) {
        match timer_type {
            TimerType::Rest => self.rest.current_time = self.rest.initial_time,
            TimerType::Focus => self.focus.current_time = self.focus.initial_time,
            TimerType::Transitioning(_) => {
                unreachable!()
            }
        };
    }

    pub fn get_mode(&self) -> TimerType {
        self.timer.clone()
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
        let timer_type = self.timer.to_string();

        let timer_string = self.get_current_timer().to_string();
        format!("{timer_type}: \n\t {timer_string}")
    }

    pub fn get_current_timer(&self) -> Timer {
        match &self.timer {
            TimerType::Focus => self.focus,
            TimerType::Rest => self.rest,
            TimerType::Transitioning(s) => match s.deref() {
                TimerType::Focus => self.focus,
                TimerType::Rest => self.rest,
                _ => unimplemented!(),
            },
        }
    }

    // TODO: Add a extend mode option.
    // TODO: Add a autopause mode option.
}
