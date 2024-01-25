use std::{
    io::{self, stdout, Stdout},
    time::{Duration, SystemTime},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    style::Stylize,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

use crate::backend::{Pomodoro, TimerType};

#[derive(Debug, PartialEq, Eq)]
enum Area {
    Timer,
    TasksNotCompleted,
    TasksCompleted,
}

pub struct TuiRatatuiDisplay {
    pomodoro: Pomodoro,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_close: bool,
    current_area: Area,
    selected_row: usize,
    pause: bool,
    space_timeout: SystemTime,
}

impl TuiRatatuiDisplay {
    pub fn new(pomodoro: Pomodoro) -> Result<Self, io::Error> {
        let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
        Ok(Self {
            pomodoro,
            terminal,
            should_close: false,
            pause: false,
            current_area: Area::Timer,
            selected_row: 0,
            space_timeout: SystemTime::now(),
        })
    }

    pub fn display(&mut self) -> io::Result<()> {
        let pomo_mode = match self.pomodoro.get_mode() {
            TimerType::Focus => "Focus",
            TimerType::Rest => "Rest",
        };

        // TODO: Refactor
        self.terminal.draw(|frame| {
            let frame_area = frame.size();
            let mut timer_area = frame_area.clone();
            timer_area.height = (timer_area.height >> 1) - 15;

            let mut timer_widget = Paragraph::new(self.pomodoro.timer_to_string())
                .block(Block::default().title(pomo_mode).borders(Borders::ALL));

            if self.current_area == Area::Timer {
                timer_widget = timer_widget.blue();
            }
            frame.render_widget(timer_widget, timer_area);

            let not_completed_tasks = self.pomodoro.task_get_by_complete(false);
            let mut not_completed_tasks_string = String::new();

            for (i, task) in not_completed_tasks.iter().enumerate() {
                if self.current_area == Area::TasksNotCompleted && i == self.selected_row {
                    not_completed_tasks_string +=
                        format!("[*] {}: {}", task.name, task.description).as_str();
                } else {
                    not_completed_tasks_string +=
                        format!("[ ] {}: {}", task.name, task.description).as_str();
                }
                not_completed_tasks_string += "\n";
            }

            let mut task_area = timer_area.clone();
            task_area.y = timer_area.y + timer_area.height;
            let mut task_widget = Paragraph::new(not_completed_tasks_string)
                .block(Block::default().title("TODO:").borders(Borders::ALL));

            if self.current_area == Area::TasksNotCompleted {
                task_widget = task_widget.blue();
            }

            frame.render_widget(task_widget, task_area);

            let completed_tasks = self.pomodoro.task_get_by_complete(true);
            let mut completed_tasks_string = String::new();

            for (i, task) in completed_tasks.iter().enumerate() {
                if self.current_area == Area::TasksCompleted && i == self.selected_row {
                    completed_tasks_string +=
                        format!("[*] {}: {}", task.name, task.description).as_str();
                } else {
                    completed_tasks_string +=
                        format!("[x] {}: {}", task.name, task.description).as_str();
                }
                completed_tasks_string += "\n";
            }

            let mut done_task_area = task_area.clone();
            done_task_area.y = task_area.y + task_area.height;

            let mut completed_tasks_widget = Paragraph::new(completed_tasks_string)
                .block(Block::default().title("DONE:").borders(Borders::ALL));

            if self.current_area == Area::TasksCompleted {
                completed_tasks_widget = completed_tasks_widget.blue();
            }

            frame.render_widget(completed_tasks_widget, done_task_area);
        })?;

        Ok(())
    }

    pub fn pomo_loop(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let _ = stdout().execute(EnterAlternateScreen)?;

        let mut next_count = SystemTime::now();
        let one_sec = Duration::from_secs(1);
        while !self.should_close {
            while next_count.elapsed().unwrap() <= one_sec {
                self.handle_events()?;
                if self.should_close {
                    break;
                }
            }
            next_count = SystemTime::now();

            if !self.pause {
                let _ = self.display();
                self.pomodoro.foward();
            }

            self.handle_events()?;
        }

        disable_raw_mode()?;
        let _ = stdout().execute(LeaveAlternateScreen)?;

        Ok(())
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match (key.code, key.kind) {
                    (KeyCode::Esc, KeyEventKind::Press) => {
                        self.should_close = true;
                    }
                    (KeyCode::Char(' '), KeyEventKind::Press) => {
                        const SPACE_DELAY: Duration = Duration::from_secs(2);
                        if let Ok(time_elapsed) = self.space_timeout.elapsed() {
                            if time_elapsed < SPACE_DELAY {
                                self.space_timeout = SystemTime::now();
                                return Ok(());
                            }
                        }
                        match self.current_area {
                            Area::Timer => {
                                self.pause = !self.pause;
                            }
                            Area::TasksNotCompleted => {
                                self.pomodoro.task_complete(self.selected_row);
                            }
                            Area::TasksCompleted => {
                                self.pomodoro.task_not_complete(self.selected_row);
                            }
                        }
                    }
                    (KeyCode::Down, KeyEventKind::Press) => match self.current_area {
                        Area::Timer => self.current_area = Area::TasksNotCompleted,
                        Area::TasksNotCompleted => {
                            if self.selected_row + 1
                                < self.pomodoro.task_get_by_complete(false).len()
                            {
                                self.selected_row += 1;
                            } else {
                                self.current_area = Area::TasksCompleted;
                                self.selected_row = 0;
                            }
                        }
                        Area::TasksCompleted => {
                            if self.selected_row + 1
                                < self.pomodoro.task_get_by_complete(true).len()
                            {
                                self.selected_row += 1;
                            } else {
                                self.current_area = Area::Timer;
                                self.selected_row = 0;
                            }
                        }
                    },
                    (KeyCode::Up, KeyEventKind::Press) => match self.current_area {
                        Area::Timer => {
                            self.current_area = Area::TasksCompleted;
                            self.selected_row =
                                (self.pomodoro.task_get_by_complete(true).len() - 1).max(0);
                        }
                        Area::TasksNotCompleted => {
                            if self.selected_row > 0 {
                                self.selected_row -= 1;
                            } else {
                                self.current_area = Area::Timer;
                                self.selected_row = 0;
                            }
                        }
                        Area::TasksCompleted => {
                            if self.selected_row > 0 {
                                self.selected_row -= 1;
                            } else {
                                self.current_area = Area::TasksNotCompleted;
                                self.selected_row =
                                    (self.pomodoro.task_get_by_complete(false).len() - 1).max(0);
                            }
                        }
                    },
                    // TODO: Add task add feature
                    // TODO: Add task update completed status feature
                    // TODO: Add task remove feature
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
