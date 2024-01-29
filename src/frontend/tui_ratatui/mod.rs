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
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph},
    Terminal,
};

use crate::backend::{Pomodoro, Task};

const COL_SIZE: usize = 2;

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
    selected_col: usize,
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
            selected_col: 0,
            space_timeout: SystemTime::now(),
            new_task_buffer: String::new(),
        })
    }

    pub fn display(&mut self) -> io::Result<()> {
        let height = self.terminal.size().ok().unwrap().height;
        // Timer
        let timer_widget = Self::create_timer_widget(
            &self.pomodoro,
            &self.current_area,
            height >> 5,
            self.selected_col,
        );

        // Completed tasks
        let not_completed_widget = Self::create_not_completed_widget(
            &self.pomodoro,
            &self.current_area,
            self.selected_row,
        );

        // Completed tasks
        let completed_widget =
            Self::create_completed_widget(&self.pomodoro, &self.current_area, self.selected_row);

        // Task add section
        let task_add_widget =
            Self::create_add_task(&self.current_area, self.new_task_buffer.clone());

        self.terminal.draw(|frame| {
            let frame_area = frame.size();
            let mut timer_area = frame_area.clone();
            timer_area.height = (timer_area.height >> 1) - 15;
            frame.render_widget(timer_widget, timer_area);

            let mut task_area = timer_area.clone();
            task_area.y = timer_area.y + timer_area.height;
            frame.render_widget(not_completed_widget, task_area);

            let mut done_task_area = task_area.clone();
            done_task_area.y = task_area.y + task_area.height;

            frame.render_widget(completed_widget, done_task_area);

            if let Some(task_add_widget) = task_add_widget {
                let mut task_add_area = done_task_area.clone();
                task_add_area.height /= 2;
                task_add_area.y = done_task_area.y + done_task_area.height;

                frame.render_widget(task_add_widget, task_add_area);
            }
        })?;

        Ok(())
    }

    fn create_timer_widget<'a>(
        pomodoro: &'a Pomodoro,
        current_area: &'a Area,
        height: u16,
        selected_col: usize,
    ) -> Paragraph<'a> {
        let mut styles = vec![
            Style::default().bg(Color::Gray),
            Style::default().bg(Color::Gray),
        ];
        styles[selected_col] = styles[selected_col].fg(Color::Red);

        let pomo_string = pomodoro.timer_to_string();
        let pomo_display: Vec<Line<'_>> = vec![
            Span::from(pomo_string).into(),
            vec![
                Span::styled("⏵⏸︎ ", styles[0]),
                Span::styled("⏭ ", styles[1]),
            ]
            .into(),
        ];

        let pomo_mode = pomodoro.timer_mode_string();

        let mut widget = Paragraph::new(pomo_display).block(
            Block::default()
                .title(pomo_mode)
                .borders(Borders::ALL)
                .padding(Padding::new(0, 0, height, 0)),
        );
        if *current_area == Area::Timer {
            widget = widget.blue();
        }

        widget.alignment(ratatui::layout::Alignment::Center)
    }

    fn create_not_completed_widget<'a>(
        pomodoro: &'a Pomodoro,
        current_area: &'a Area,
        selected_row: usize,
    ) -> Paragraph<'a> {
        let not_completed_tasks = pomodoro.task_get_by_complete(false);
        let mut not_completed_tasks_vec: Vec<Line<'_>> = vec![];

        for (i, task) in not_completed_tasks.iter().enumerate() {
            let task_line = if *current_area == Area::TasksNotCompleted && i == selected_row {
                (
                    format!("[*] {}: {}", task.name, task.description),
                    Style::default().add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    format!("[ ] {}: {}", task.name, task.description),
                    Style::default(),
                )
            };

            not_completed_tasks_vec.push(Span::styled(task_line.0 + "\n", task_line.1).into());
        }

        let mut task_widget = Paragraph::new(not_completed_tasks_vec)
            .block(Block::default().title("TODO:").borders(Borders::ALL));
        if *current_area == Area::TasksNotCompleted {
            task_widget = task_widget.blue();
        }

        task_widget
    }

    fn create_completed_widget<'a>(
        pomodoro: &'a Pomodoro,
        current_area: &'a Area,
        selected_row: usize,
    ) -> Paragraph<'a> {
        let completed_tasks = pomodoro.task_get_by_complete(true);
        let mut completed_tasks_vec: Vec<Line<'_>> = vec![];

        for (i, task) in completed_tasks.iter().enumerate() {
            let task_line = if *current_area == Area::TasksCompleted && i == selected_row {
                (
                    format!("[*] {}: {}", task.name, task.description),
                    Style::default().add_modifier(Modifier::BOLD),
                )
            } else {
                (
                    format!("[x] {}: {}", task.name, task.description),
                    Style::default(),
                )
            };

            completed_tasks_vec.push(Span::styled(task_line.0 + "\n", task_line.1).into());
        }

        let mut widget = Paragraph::new(completed_tasks_vec)
            .block(Block::default().title("DONE:").borders(Borders::ALL));

        if *current_area == Area::TasksCompleted {
            widget = widget.blue();
        }

        widget
    }

    fn create_add_task<'a>(
        current_area: &'a Area,
        new_task_buffer: String,
    ) -> Option<Paragraph<'a>> {
        if *current_area == Area::TaskAdd {
            let widget = Paragraph::new(new_task_buffer)
                .block(Block::default().title("Task add").borders(Borders::ALL))
                .blue();
            return Some(widget);
        }
        None
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
                    (KeyCode::Esc, KeyEventKind::Press) => match self.current_area {
                        Area::TaskAdd => {
                            self.selected_row = 0;
                            self.current_area = Area::Timer;
                            self.new_task_buffer = String::new();
                        }
                        _ => self.should_close = true,
                    },
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
                                match self.selected_col {
                                    0 => {
                                        self.pause = !self.pause;
                                    }
                                    1 => {
                                        self.pomodoro.next_mode();
                                    }
                                    _ => {}
                                }
                                // self.pause = !self.pause;
                                // self.selected_col = 0;
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
                    (KeyCode::Left, KeyEventKind::Press) => match self.current_area {
                        Area::Timer => {
                            if self.selected_col == 0 {
                                self.selected_col = COL_SIZE - 1;
                            } else {
                                self.selected_col -= 1;
                            }
                        }
                        _ => {}
                    },
                    (KeyCode::Right, KeyEventKind::Press) => match self.current_area {
                        Area::Timer => {
                            self.selected_col += 1;
                            self.selected_col = self.selected_col % COL_SIZE;
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
                            self.selected_row = 0;
                        }
                        _ => {}
                    },
                    (KeyCode::Backspace, KeyEventKind::Press) => match self.current_area {
                        Area::TaskAdd => {
                            let mut buffer_str = self.new_task_buffer.chars();
                            let _ = buffer_str.next_back();

                            self.new_task_buffer = buffer_str.collect();
                        }
                        _ => {}
                    },
                    // TODO: Add task remove feature
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
