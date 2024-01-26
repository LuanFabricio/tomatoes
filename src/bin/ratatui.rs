use std::time::Duration;

use tomatoes::backend::{Pomodoro, Task};
use tomatoes::frontend::tui_ratatui::TuiRatatuiDisplay;

fn main() {
    let mut pomodoro = Pomodoro::new(Duration::from_secs(25 * 60), Duration::from_secs(5 * 60));

    // pomodoro.task_add(Task::new("Ler cap. de AM", "Ler capítulo 2 de AM."));
    // pomodoro.task_add(Task::new(
    //     "Ler artigo",
    //     "Reler do Hinton, sobre destilamento de conhecimento.",
    // ));
    // pomodoro.task_add(Task::new(
    //     "Estudar Hacking",
    //     "Fazer aulas do curso de hacker ético.",
    // ));

    // pomodoro.task_add(Task::new("Ler cap. de AM", "Ler capítulo 2 de AM."));

    // pomodoro.task_complete(0);
    // pomodoro.task_complete(3);

    // let mut terminal = TerminalDisplay::new(pomodoro);
    // terminal.pomo_loop();

    let _ = pomodoro.load();
    let res = pomodoro.save();

    println!("Res: {res:?}");
    // let mut tui = TuiRatatuiDisplay::new(pomodoro).expect("Failt to create TUI");
    // tui.pomo_loop().expect("Not fail!");
}
