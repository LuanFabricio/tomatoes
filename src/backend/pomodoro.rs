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
    play_sound_alarm: bool,
}

impl Pomodoro {
    pub fn new(focus_time: Duration, rest_time: Duration) -> Self {
        Self {
            focus: Timer::new(focus_time),
            rest: Timer::new(rest_time),
            tasks: vec![],
            timer: TimerType::Focus,
            play_sound_alarm: true,
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

    const ONE_SEC: Duration = Duration::from_secs(1);
    pub fn forward(&mut self) -> Duration {
        match &self.timer {
            TimerType::Focus => {
                self.focus.current_time -= Self::ONE_SEC;

                if self.focus.current_time == Duration::ZERO {
                    let mut new_timer = TimerType::Rest;
                    if self.play_sound_alarm {
                        new_timer = TimerType::Transitioning(Box::new(new_timer));
                    }
                    self.timer = new_timer;
                    self.focus.current_time = self.focus.initial_time
                }
                self.focus.current_time
            }
            TimerType::Rest => {
                self.rest.current_time -= Self::ONE_SEC;

                if self.rest.current_time == Duration::ZERO {
                    let mut new_timer = TimerType::Focus;
                    if self.play_sound_alarm {
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

                if self.play_sound_alarm && std::env::var("ENV") != Ok("TEST".to_string()) {
                    self.alarm_play();
                }

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

    pub fn alarm_disable(&mut self) {
        self.play_sound_alarm = false;
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

#[cfg(test)]
mod test {
    use super::*;

    const FOCUS_TIME: Duration = Duration::from_secs(15);
    const REST_TIME: Duration = Duration::from_secs(15);

    mod new {
        use super::*;

        #[test]
        fn should_initialize_in_focus_time() {
            let pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);
            assert_eq!(pomodoro.timer, TimerType::Focus);
        }

        #[test]
        fn should_initalize_timers() {
            let pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);

            let focus_timer = Timer::new(FOCUS_TIME);
            let rest_timer = Timer::new(REST_TIME);

            assert_eq!(pomodoro.focus, focus_timer);
            assert_eq!(pomodoro.rest, rest_timer);
        }

        #[test]
        fn should_initialize_with_the_play_sound_alarm_equals_true() {
            let pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);
            assert_eq!(pomodoro.play_sound_alarm, true);
        }

        #[test]
        fn should_initialize_with_the_task_vec_empty() {
            let pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);
            assert_eq!(pomodoro.tasks.is_empty(), true);
        }
    }

    mod forward {
        use super::*;

        #[test]
        fn should_advance_current_time_one_second_by_timer_type() {
            let mut pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);

            pomodoro.timer = TimerType::Focus;
            pomodoro.forward();
            let new_focus_time = FOCUS_TIME - Duration::from_secs(1);
            assert_eq!(pomodoro.focus.current_time, new_focus_time);

            pomodoro.timer = TimerType::Rest;
            pomodoro.forward();
            let new_rest_time = REST_TIME - Duration::from_secs(1);
            assert_eq!(pomodoro.rest.current_time, new_rest_time);
        }

        #[test]
        fn should_reset_timer_when_is_done() {
            let mut pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);
            // Disable alarm transition.
            pomodoro.alarm_disable();

            pomodoro.focus.current_time = Duration::from_secs(1);
            pomodoro.forward();
            assert_eq!(pomodoro.focus.current_time, pomodoro.focus.initial_time);

            assert_eq!(pomodoro.timer, TimerType::Rest);
            pomodoro.rest.current_time = Duration::from_secs(1);
            pomodoro.forward();
            assert_eq!(pomodoro.rest.current_time, pomodoro.rest.initial_time);
        }

        #[test]
        fn should_exit_transition() {
            std::env::set_var("ENV", "TEST");
            let mut pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);

            pomodoro.timer = TimerType::Transitioning(Box::new(TimerType::Focus));
            pomodoro.forward();
            assert_eq!(pomodoro.timer, TimerType::Focus);

            pomodoro.timer = TimerType::Transitioning(Box::new(TimerType::Rest));
            pomodoro.forward();
            assert_eq!(pomodoro.timer, TimerType::Rest);
        }

        #[test]
        fn should_not_transition_if_timer_is_disabled() {
            let mut pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);
            pomodoro.alarm_disable();

            pomodoro.focus.current_time = Duration::from_secs(1);
            pomodoro.forward();
            assert_eq!(pomodoro.timer, TimerType::Rest);

            pomodoro.rest.current_time = Duration::from_secs(1);
            pomodoro.forward();
            assert_eq!(pomodoro.timer, TimerType::Focus);
        }

        #[should_panic]
        #[test]
        fn should_crash_if_is_transitioning_to_anoter_transition() {
            std::env::set_var("ENV", "TEST");
            let mut pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);

            pomodoro.timer = TimerType::Transitioning(Box::new(TimerType::Transitioning(
                Box::new(TimerType::Focus),
            )));
            pomodoro.forward();
        }
    }

    mod alarm_disable {
        use super::*;

        #[test]
        fn should_set_play_sound_alarm_to_false() {
            let mut pomodoro = Pomodoro::new(FOCUS_TIME, REST_TIME);
            assert_eq!(pomodoro.play_sound_alarm, true);

            pomodoro.alarm_disable();
            assert_eq!(pomodoro.play_sound_alarm, false);
        }
    }
}
