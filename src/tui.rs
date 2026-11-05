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
    stack_path: PathBuf,
    input: InputLine,
    input_mode: InputMode,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    pub fn new() -> Self {
        let stack_path = persistence::default_path();
        let stack = persistence::load(&stack_path).unwrap_or_default();
        Self {
            calculator: Calculator::with_stack(stack),
            stack_path,
            input: InputLine::default(),
            input_mode: InputMode::Editing,
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => self.input_mode = InputMode::Editing,
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => {
                            if self.submit()? == Outcome::Quit {
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

    fn submit(&mut self) -> Result<Outcome> {
        let outcome = self.calculator.execute(self.input.text());
        if outcome == Outcome::Quit {
            return Ok(outcome);
        }
        self.input.clear();
        persistence::save(&self.stack_path, self.calculator.stack())?;
        Ok(outcome)
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, main_area] = vertical.areas(frame.area());

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
                    " to add the number to stack or perform operation".into(),
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
            .filter(|op| op.category == category)
            .map(|op| op.label())
            .collect();
        for chunk in labels.chunks(6) {
            lines.push(Line::from(format!("  {}", chunk.join(" "))));
        }
        lines.push(Line::from(""));
    }
    Text::from(lines)
}
