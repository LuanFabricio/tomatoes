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
        })
    }

    pub fn display(&mut self) -> io::Result<()> {
        let pomo_mode = match self.pomodoro.get_mode() {
            TimerType::Focus => "Focus",
            TimerType::Rest => "Rest",
        };

        self.terminal.draw(|frame| {
            frame.render_widget(
                Paragraph::new(self.pomodoro.timer_to_string())
                    .block(Block::default().title(pomo_mode).borders(Borders::ALL)),
                frame.size(),
            );
        })?;
        Ok(())
    }

    pub fn pomo_loop(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let _ = stdout().execute(EnterAlternateScreen)?;

        let mut next_count = SystemTime::now();
        let one_sec = Duration::from_secs(1);
        while !self.should_close {
            self.handle_events()?;

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
                        self.pause = !self.pause;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }
}
