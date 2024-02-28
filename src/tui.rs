use std::{error::Error, io::Stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

pub struct TuiCrossterm {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl TuiCrossterm {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let stdout = std::io::stdout();
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;

        Ok(Self { terminal })
    }
    fn initialize_panic_handler() {
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)
                .unwrap();
            crossterm::terminal::disable_raw_mode().unwrap();
            original_hook(panic_info);
        }));
    }

    pub fn setup(&mut self) -> Result<&mut Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
        TuiCrossterm::initialize_panic_handler();
        let mut stdout = std::io::stdout();

        // stdout.queue(Clear(ClearType::All))?;

        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        Ok(&mut self.terminal)
    }

    fn tear_down(&mut self) -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;

        Ok(())
    }
}

impl Drop for TuiCrossterm {
    fn drop(&mut self) {
        let _ = self.tear_down();
    }
}
