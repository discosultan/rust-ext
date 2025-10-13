use std::{
    cmp::max,
    io::{self, Stderr},
    ops::{Deref, DerefMut},
    panic::{set_hook, take_hook},
};

use ratatui::{crossterm, layout::Constraint, prelude::CrosstermBackend};

fn crossterm_enter() -> io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;
    Ok(())
}

fn crossterm_exit() -> io::Result<()> {
    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}

pub struct Terminal {
    terminal: ratatui::Terminal<CrosstermBackend<Stderr>>,
}

impl Terminal {
    pub fn run() -> io::Result<Self> {
        // Set custom hook to exit crossterm before printing panic output. Note
        // that for some reason panic output is not correctly printed if we rely
        // solely on the Terminal Drop impl for exiting crossterm, even though
        // it should be run first during the panic unwinding process.
        let original_hook = take_hook();
        set_hook(Box::new(move |panic_info| {
            let _ = crossterm_exit();
            original_hook(panic_info);
        }));
        crossterm_enter()?;
        let terminal = ratatui::Terminal::new(CrosstermBackend::new(std::io::stderr()))?;
        Ok(Self { terminal })
    }
}

impl Deref for Terminal {
    type Target = ratatui::Terminal<CrosstermBackend<Stderr>>;

    fn deref(&self) -> &Self::Target {
        &self.terminal
    }
}

impl DerefMut for Terminal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.terminal
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = crossterm_exit();
    }
}

pub fn calc_widths<'a, const N: usize>(
    rows: impl IntoIterator<Item = &'a [impl AsRef<str> + 'a; N]>,
) -> [Constraint; N] {
    rows.into_iter()
        .fold([Constraint::Length(0); N], calc_widths_row)
}

pub fn calc_widths_single<'a, const N: usize>(
    row: &'a [impl AsRef<str> + 'a; N],
) -> [Constraint; N] {
    calc_widths_row([Constraint::Length(0); N], row)
}

pub fn calc_widths_row<'a, const N: usize>(
    mut acc: [Constraint; N],
    row: &'a [impl AsRef<str> + 'a; N],
) -> [Constraint; N] {
    for i in 0..N {
        let Constraint::Length(len) = acc[i] else {
            unreachable!()
        };
        acc[i] = Constraint::Length(max(
            len,
            u16::try_from(row[i].as_ref().len()).unwrap_or(u16::MAX),
        ));
    }
    acc
}
