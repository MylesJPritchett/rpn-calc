use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let app_result = App::new().run(terminal);
    ratatui::restore();
    app_result
}

/// App holds the state of the application
struct App {
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    stack: Vec<f64>,
}

enum InputMode {
    Normal,
    Editing,
}

impl App {
    const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Editing,
            stack: Vec::new(),
            character_index: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn process_input(&mut self) {
        if let Ok(num) = self.input.parse::<f64>() {
            self.stack.push(num);
        } else {
            match self.input.as_str() {
                "+" => self.perform_operation(|a, b| a + b),
                "-" => self.perform_operation(|a, b| a - b),
                "/" => self.perform_operation(|a, b| a / b),
                "*" => self.perform_operation(|a, b| a * b),
                "" => self.perform_clone(),
                "%" => self.perform_operation(|a, b| a % b),
                "^" => self.perform_power_operation(),
                "neg" => self.perform_negation(),
                "abs" => self.perform_absolution(),
                "sqrt" => self.perform_square_root(),
                "sin" => self.perform_sin(),
                "cos" => self.perform_cos(),
                "tan" => self.perform_tan(),
                "asin" => self.perform_asin(),
                "acos" => self.perform_acos(),
                "atan" => self.perform_atan(),
                "!" => self.perform_factorial(),
                "recip" => self.perform_reciprocal(),
                "log10" => self.perform_logarithm_10(),
                "logn" => self.perform_logarithm_n(),
                "log2" => self.perform_logarithm_2(),
                "swap" => self.perform_swap(),
                "clear" => self.perform_clear(),
                "drop" => self.perform_drop(),
                "?" => self.show_operators(),
                _ => (),
            }
        }
        self.input.clear();
        self.reset_cursor();
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => {
                            return Ok(());
                        }
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.process_input(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(1),
        ]);
        let [help_area, input_area, messages_area] = vertical.areas(frame.area());

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
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"));
        frame.render_widget(input, input_area);
        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + self.character_index as u16 + 1,
                // Move one line down, from the border to the input line
                input_area.y + 1,
            )),
        }

        let stack: Vec<ListItem> = self
            .stack
            .iter()
            .rev()
            .enumerate()
            .map(|(i, m)| {
                let content = Line::from(Span::raw(format!("{i}: {m}")));
                ListItem::new(content)
            })
            .collect();
        let stack = List::new(stack).block(Block::bordered().title("Stack"));
        frame.render_widget(stack, messages_area);
    }

    fn perform_negation(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        let a = self.stack.pop().unwrap();
        let result = -a;
        self.stack.push(result);
    }

    fn perform_absolution(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        let a = self.stack.pop().unwrap();
        let result = a.abs();
        self.stack.push(result);
    }

    fn perform_square_root(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        let a = self.stack.pop().unwrap();
        let result = a.sqrt();
        self.stack.push(result);
    }

    fn perform_operation(&mut self, operation: fn(f64, f64) -> f64) {
        if self.stack.len() < 2 {
            return;
        }
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        let result = operation(a, b);
        self.stack.push(result);
    }

    fn perform_power_operation(&mut self) {
        if self.stack.len() < 2 {
            return;
        }
        let exponent = self.stack.pop().unwrap();
        let base = self.stack.pop().unwrap();
        let result = base.powf(exponent);
        self.stack.push(result);
    }

    fn perform_clone(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        let a = self.stack.pop().unwrap();
        self.stack.push(a);
        self.stack.push(a);
    }

    fn perform_sin(&self) {
        todo!()
    }

    fn perform_cos(&self) {
        todo!()
    }

    fn perform_tan(&self) {
        todo!()
    }

    fn perform_asin(&self) {
        todo!()
    }

    fn perform_acos(&self) {
        todo!()
    }

    fn perform_atan(&self) {
        todo!()
    }

    fn perform_factorial(&mut self) {
        if self.stack.is_empty() {
            return;
        }

        let a = self.stack.pop().unwrap();
        let abs_a = a.abs();
        let result = abs_a * abs_a;
        self.stack.push(result);
    }

    fn perform_reciprocal(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        let a = self.stack.pop().unwrap();
        let result = 1.0 / a;
        self.stack.push(result);
    }

    fn perform_logarithm_10(&self) {
        todo!()
    }

    fn perform_logarithm_n(&self) {
        todo!()
    }

    fn perform_logarithm_2(&self) {
        todo!()
    }

    fn perform_swap(&mut self) {
        if self.stack.len() < 2 {
            return;
        }
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(b);
        self.stack.push(a);
    }

    fn perform_clear(&mut self) {
        self.stack.truncate(0)
    }

    fn perform_drop(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        self.stack.pop().unwrap();
    }

    fn show_operators(&self) {
        todo!()
    }
}
