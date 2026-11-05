mod input_line;

use std::path::PathBuf;

use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use rpn_calc::calculator::{Calculator, Category, Outcome, OPERATIONS};
use rpn_calc::persistence;

use input_line::InputLine;

pub struct App {
    calculator: Calculator,
    /// Where the Stack is saved after each change, or `None` if saving is disabled
    /// because the existing file couldn't be loaded.
    stack_path: Option<PathBuf>,
    input: InputLine,
    input_mode: InputMode,
    status: Option<String>,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    pub fn new() -> Self {
        Self::with_stack_path(persistence::default_path())
    }

    /// Loads the Stack from `stack_path` and saves it there after every change. If the file
    /// exists but can't be loaded, the session starts empty and never saves, so the file
    /// isn't overwritten.
    pub fn with_stack_path(stack_path: PathBuf) -> Self {
        let (stack, stack_path, status) = match persistence::load(&stack_path) {
            Ok(stack) => (stack, Some(stack_path), None),
            Err(error) => (
                Vec::new(),
                None,
                Some(format!(
                    "couldn't load stack: {error}; changes won't be saved"
                )),
            ),
        };
        Self {
            calculator: Calculator::with_stack(stack),
            stack_path,
            input: InputLine::default(),
            input_mode: InputMode::Editing,
            status,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    self.status = None;
                }
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => {
                            if self.submit() == Outcome::Quit {
                                return Ok(());
                            }
                        }
                        KeyCode::Char(c) => self.input.insert(c),
                        KeyCode::Backspace => self.input.delete_before_cursor(),
                        KeyCode::Left => self.input.move_left(),
                        KeyCode::Right => self.input.move_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn submit(&mut self) -> Outcome {
        match self.calculator.execute(self.input.text()) {
            Ok(Outcome::Quit) => Outcome::Quit,
            Ok(Outcome::Continue) => {
                self.input.clear();
                if let Some(path) = &self.stack_path {
                    if let Err(error) = persistence::save(path, self.calculator.stack()) {
                        self.status = Some(format!("couldn't save stack: {error}"));
                    }
                }
                Outcome::Continue
            }
            Err(error) => {
                self.status = Some(error.to_string());
                Outcome::Continue
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, status_area, main_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to push the Value or run the Operation".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        frame.render_widget(Paragraph::new(text), help_area);

        let input = Paragraph::new(self.input.text())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        if let InputMode::Editing = self.input_mode {
            #[allow(clippy::cast_possible_truncation)]
            frame.set_cursor_position(Position::new(
                input_area.x + self.input.cursor() as u16 + 1,
                input_area.y + 1,
            ));
        }

        if let Some(status) = &self.status {
            let status = Paragraph::new(status.as_str()).style(Style::default().fg(Color::Red));
            frame.render_widget(status, status_area);
        }

        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [stack_area, operations_area] = horizontal.areas(main_area);

        let stack: Vec<ListItem> = self
            .calculator
            .stack()
            .iter()
            .rev()
            .enumerate()
            .map(|(i, value)| ListItem::new(Line::from(Span::raw(format!("{i}: {value}")))))
            .collect();
        let stack = List::new(stack).block(Block::bordered().title("Stack"));
        frame.render_widget(stack, stack_area);

        let operations_guide =
            Paragraph::new(operations_guide()).block(Block::bordered().title("Operations Guide"));
        frame.render_widget(operations_guide, operations_area);
    }
}

fn operations_guide() -> Text<'static> {
    let heading = Style::default()
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::UNDERLINED);
    let mut lines = Vec::new();
    for category in Category::ALL {
        lines.push(Line::from(Span::styled(category.label(), heading)));
        let labels: Vec<&str> = OPERATIONS
            .iter()
            .filter(|operation| operation.category() == category)
            .map(|operation| operation.label())
            .collect();
        for chunk in labels.chunks(6) {
            lines.push(Line::from(format!("  {}", chunk.join(" "))));
        }
        lines.push(Line::from(""));
    }
    Text::from(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn submit(app: &mut App, text: &str) -> Outcome {
        text.chars().for_each(|c| app.input.insert(c));
        app.submit()
    }

    #[test]
    fn failed_save_shows_a_status_message_and_the_session_continues() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("stack.txt");
        let mut app = App::with_stack_path(path.clone());
        // A directory can't be opened as a file, so saving to it fails.
        std::fs::create_dir(&path).unwrap();

        assert_eq!(submit(&mut app, "5"), Outcome::Continue);
        let status = app.status.as_deref().unwrap_or_default();
        assert!(status.starts_with("couldn't save stack: "), "{status:?}");

        assert_eq!(submit(&mut app, "dup"), Outcome::Continue);
        assert_eq!(app.calculator.stack(), &[5.0, 5.0]);
    }

    #[test]
    fn failed_load_shows_a_status_message_and_disables_saving() {
        let dir = tempfile::tempdir().unwrap();
        // A directory at the stack path can't be read as a file, so loading fails.
        let path = dir.path().join("stack.txt");
        std::fs::create_dir(&path).unwrap();
        let mut app = App::with_stack_path(path.clone());

        let status = app.status.clone().unwrap_or_default();
        assert!(status.starts_with("couldn't load stack: "), "{status:?}");
        assert!(status.ends_with("; changes won't be saved"), "{status:?}");

        app.status = None;
        assert_eq!(submit(&mut app, "5"), Outcome::Continue);
        assert_eq!(app.status, None);
        assert_eq!(app.calculator.stack(), &[5.0]);
        assert!(path.is_dir());
    }
}
