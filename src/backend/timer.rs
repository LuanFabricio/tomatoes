use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Timer {
    pub current_time: Duration,
    pub initial_time: Duration,
    pub extra_time: Duration,
}

impl Timer {
    pub fn new(initial_time: Duration) -> Self {
        Self {
            current_time: initial_time,
            extra_time: Duration::ZERO,
            initial_time,
        }
    }

    pub fn to_string(&self) -> String {
        let duration = self.current_time;

        let secs = duration.as_secs() % 60;
        let mins = (duration.as_secs_f32() - (secs as f32)) / 60f32;

        let mut r = format!("{:02}:{:02}", mins, secs);

        if self.current_time.is_zero() {
            let duration = self.extra_time;

            let secs = duration.as_secs() % 60;
            let mins = (duration.as_secs_f32() - (secs as f32)) / 60f32;
            r += format!("(+{:02}:{:02})", mins, secs).as_str();
        }

        r
    }

    const TIME_DECREASE: Duration = Duration::from_secs(1);
    pub fn forward(&mut self) {
        if self.current_time.is_zero() {
            self.extra_time += Self::TIME_DECREASE;
        } else {
            self.current_time -= Self::TIME_DECREASE;
        }
    }

    pub fn reset(&mut self) {
        self.current_time = self.initial_time.clone();
        self.extra_time = Duration::ZERO;
    }

    pub fn transfer_extra_timer_to(&self, t: &mut Timer) {
        let converted_secs = self.extra_time.as_secs_f64() / self.initial_time.as_secs_f64();
        t.reset();
        let t_secs = t.initial_time.as_secs_f64() * (1_f64 + converted_secs);
        t.current_time = Duration::from_secs(t_secs.round() as u64);
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
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

            const SECONDS: u64 = 64;
            #[test]
            fn should_format_to_string_as_min_secs() {
                let timer = Timer::new(Duration::from_secs(SECONDS));

                let expected_formatted = format!("{:02}:{:02}", SECONDS / 60, SECONDS % 60);

                assert_eq!(timer.to_string(), expected_formatted);
            }

            #[test]
            fn should_show_extra_time_if_have_one() {
                let mut timer = Timer::new(Duration::from_secs(SECONDS));

                // Simulating time advance
                for _ in 0..SECONDS + 5 {
                    timer.forward();
                }

                let expecpted_formatted = format!(
                    "{:02}:{:02}(+{:02}:{:02})",
                    timer.current_time.as_secs() / 60,
                    timer.current_time.as_secs() % 60,
                    timer.extra_time.as_secs() / 60,
                    timer.extra_time.as_secs() % 60,
                );

                assert_eq!(timer.to_string(), expecpted_formatted);
            }
        }

        mod forward {
            use super::*;

            #[test]
            fn should_get_current_and_initial_time_difference() {
                let mut timer = Timer::new(Duration::from_secs(5));
                for _ in 0..10 {
                    timer.forward();
                }

                assert_eq!(timer.extra_time, Duration::from_secs(5));
            }
        }

        mod reset {
            use super::*;

            #[test]
            fn should_set_current_equals_initial_time_and_zero_to_extra_time() {
                let mut timer = Timer::new(Duration::from_secs(5));
                for _ in 0..10 {
                    timer.forward();
                }

                assert_ne!(timer.extra_time, Duration::ZERO);
                assert_ne!(timer.current_time, timer.initial_time);

                timer.reset();
                assert_eq!(timer.extra_time, Duration::ZERO);
                assert_eq!(timer.current_time, timer.initial_time);
            }
        }

        mod transfer_extra_timer_to {
            use super::*;

            #[test]
            fn should_transfer_extra_timer_to_anoter_timer() {
                const INITIAL_TIMER1: u64 = 60;
                let mut timer1 = Timer::new(Duration::from_secs(INITIAL_TIMER1));
                timer1.extra_time = Duration::from_secs(INITIAL_TIMER1); // 2x to next timer;

                const INITIAL_TIMER2: u64 = 42;
                let mut timer2 = Timer::new(Duration::from_secs(INITIAL_TIMER2));

                timer1.transfer_extra_timer_to(&mut timer2);
                assert_eq!(timer2.current_time, timer2.initial_time * 2);
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
