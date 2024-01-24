use std::time::Duration;

use frontend::terminal::TerminalDisplay;

use crate::backend::{Pomodoro, Task};

mod backend;
mod frontend;

fn main() {
    let mut pomodoro = Pomodoro::new(Duration::from_secs(25 * 60), Duration::from_secs(5 * 60));

    pomodoro.task_add(Task::new("Ler cap. de AM", "Ler capítulo 2 de AM."));
    pomodoro.task_add(Task::new(
        "Estudar Hacking",
        "Fazer aulas do curso de hacker ético.",
    ));

    pomodoro.task_complete(0);
    pomodoro.task_add(Task::new("Ler cap. de AM", "Ler capítulo 2 de AM."));

    let mut terminal = TerminalDisplay::new(pomodoro);

    terminal.pomo_loop();
}
