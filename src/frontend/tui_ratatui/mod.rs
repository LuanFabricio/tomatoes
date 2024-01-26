use std::{
    io::{self, stdout, Stdout},
    process::exit,
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
    widgets::{canvas::Rectangle, Block, Borders, Paragraph},
    Terminal,
};

use crate::backend::{Pomodoro, Task, TimerType};

#[derive(Debug, PartialEq, Eq)]
enum Area {
    Timer,
    TasksNotCompleted,
    TasksCompleted,
    TaskAdd,
}

pub struct TuiRatatuiDisplay {
    pomodoro: Pomodoro,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    should_close: bool,
    current_area: Area,
    selected_row: usize,
    pause: bool,
    space_timeout: SystemTime,
    new_task_buffer: String,
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
            new_task_buffer: String::new(),
        })
    }

    pub fn display(&mut self) -> io::Result<()> {
        let pomo_mode = match self.pomodoro.get_mode() {
            TimerType::Focus => "Focus",
            TimerType::Rest => "Rest",
        };

        // Timer
        let mut timer_widget = Paragraph::new(self.pomodoro.timer_to_string())
            .block(Block::default().title(pomo_mode).borders(Borders::ALL));

        if self.current_area == Area::Timer {
            timer_widget = timer_widget.blue();
        }

        // Completed tasks
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

        let mut task_widget = Paragraph::new(not_completed_tasks_string)
            .block(Block::default().title("TODO:").borders(Borders::ALL));

        if self.current_area == Area::TasksNotCompleted {
            task_widget = task_widget.blue();
        }

        // Completed tasks
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

        let mut completed_tasks_widget = Paragraph::new(completed_tasks_string)
            .block(Block::default().title("DONE:").borders(Borders::ALL));

        if self.current_area == Area::TasksCompleted {
            completed_tasks_widget = completed_tasks_widget.blue();
        }

        let task_add_widget = if self.current_area == Area::TaskAdd {
            let widget = Paragraph::new(self.new_task_buffer.clone())
                .block(Block::default().title("Task add").borders(Borders::ALL))
                .blue();
            Some(widget)
        } else {
            None
        };

        // TODO: Refactor
        self.terminal.draw(|frame| {
            let frame_area = frame.size();
            let mut timer_area = frame_area.clone();
            timer_area.height = (timer_area.height >> 1) - 15;
            frame.render_widget(timer_widget, timer_area);

            let mut task_area = timer_area.clone();
            task_area.y = timer_area.y + timer_area.height;
            frame.render_widget(task_widget, task_area);

            let mut done_task_area = task_area.clone();
            done_task_area.y = task_area.y + task_area.height;

            frame.render_widget(completed_tasks_widget, done_task_area);

            if let Some(task_add_widget) = task_add_widget {
                let mut task_add_area = done_task_area.clone();
                task_add_area.height /= 2;
                task_add_area.y = done_task_area.y + done_task_area.height;

                frame.render_widget(task_add_widget, task_add_area);
            }
        })?;

        Ok(())
    }

    pub fn pomo_loop(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let _ = stdout().execute(EnterAlternateScreen)?;

        let mut next_count = SystemTime::now();
        let one_sec = Duration::from_secs(1);
        while !self.should_close {
            if !self.pause && next_count.elapsed().unwrap() > one_sec {
                self.pomodoro.foward();
                next_count = SystemTime::now();
            }

            let _ = self.display();
            self.handle_events()?;
        }

        disable_raw_mode()?;
        let _ = stdout().execute(LeaveAlternateScreen)?;

        let _ = self.pomodoro.save();

        Ok(())
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_secs_f64(1f64 / 60f64))? {
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
                            _ => {
                                self.new_task_buffer += " ";
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
                        _ => {}
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
                        _ => {}
                    },
                    (KeyCode::Char('+'), KeyEventKind::Press) => match self.current_area {
                        Area::TaskAdd => {}
                        _ => self.current_area = Area::TaskAdd,
                    },
                    (KeyCode::Char(c), KeyEventKind::Press) => match self.current_area {
                        Area::TaskAdd => {
                            self.new_task_buffer += c.to_string().as_str();
                        }
                        _ => {}
                    },
                    (KeyCode::Enter, KeyEventKind::Press) => match self.current_area {
                        Area::TaskAdd => {
                            let new_task = Task::from_str(self.new_task_buffer.as_str());
                            self.pomodoro.task_add(new_task);

                            self.new_task_buffer = String::new();
                            self.current_area = Area::Timer;
                        }
                        _ => {}
                    },
                    (KeyCode::Backspace, KeyEventKind::Press) => match self.current_area {
                        Area::TaskAdd => {
                            let last_index = (self.new_task_buffer.len() - 1).max(0);
                            self.new_task_buffer.remove(last_index);
                        }
                        _ => {}
                    },
                    // TODO: Add task add feature
                    // TODO: Add task remove feature
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
