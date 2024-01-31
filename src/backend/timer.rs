use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::time::Duration;

#[derive(Clone, Copy)]
pub struct Timer {
    pub current_time: Duration,
    pub initial_time: Duration,
}

impl Timer {
    pub fn new(initial_time: Duration) -> Self {
        Self {
            current_time: initial_time,
            initial_time,
        }
    }

    pub fn to_string(&self) -> String {
        let duration = self.current_time;

        let secs = duration.as_secs() % 60;
        let mins = (duration.as_secs_f32() - (secs as f32)) / 60f32;

        format!("{:02}:{:02}", mins, secs)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TimerType {
    Focus,
    Rest,
    Transitioning(Box<TimerType>),
}

impl TimerType {
    pub fn to_string(&self) -> String {
        match self {
            TimerType::Focus => String::from("Focus"),
            TimerType::Rest => String::from("Rest"),
            TimerType::Transitioning(s) => {
                let next_mode = match s.deref() {
                    TimerType::Transitioning(_) => unreachable!(),
                    _ => s.to_string(),
                };
                format!("Transitioning({next_mode})",)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    mod timer {
        use super::*;
        mod new {
            use super::*;

            #[test]
            fn should_initialize_current_time_with_initial_time() {
                const INITIAL_TIME_SECS: u64 = 4200;
                let initial_time = Duration::from_secs(INITIAL_TIME_SECS);
                let timer = Timer::new(initial_time.clone());

                assert_eq!(timer.initial_time, initial_time);
                assert_eq!(timer.current_time, initial_time);
                assert_eq!(timer.current_time, timer.initial_time);
            }
        }

        mod to_string {
            use super::*;

            #[test]
            fn should_format_to_string_as_min_secs() {
                const SECONDS: u64 = 64;
                let timer = Timer::new(Duration::from_secs(SECONDS));

                let expected_formatted = format!("{:02}:{:02}", SECONDS / 60, SECONDS % 60);

                assert_eq!(timer.to_string(), expected_formatted);
            }
        }
    }

    mod timer_type {
        use super::*;

        mod to_string {
            use super::*;

            #[test]
            fn should_return_string_based_on_enum() {
                let focus_type = TimerType::Focus;
                let rest_type = TimerType::Rest;
                let transioning_to_focus_type =
                    TimerType::Transitioning(Box::new(TimerType::Focus));
                let transioning_to_rest_type = TimerType::Transitioning(Box::new(TimerType::Rest));

                let expected_focus = "Focus";
                let expected_rest = "Rest";
                let expected_transioning_to_focus = "Transitioning(Focus)";
                let expected_transioning_to_rest = "Transitioning(Rest)";

                assert_eq!(focus_type.to_string(), expected_focus);
                assert_eq!(rest_type.to_string(), expected_rest);
                assert_eq!(
                    transioning_to_focus_type.to_string(),
                    expected_transioning_to_focus
                );
                assert_eq!(
                    transioning_to_rest_type.to_string(),
                    expected_transioning_to_rest
                );
            }

            #[should_panic]
            #[test]
            fn should_crash_if_timertype_transitioning_have_a_transitioning() {
                let transitioning_loop = TimerType::Transitioning(Box::new(
                    TimerType::Transitioning(Box::new(TimerType::Focus)),
                ));

                let _ = transitioning_loop.to_string();
            }
        }
    }
}
