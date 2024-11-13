use core::f64;

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
    undo: Vec<Vec<f64>>,
    redo: Vec<Vec<f64>>,
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
            undo: Vec::new(),
            redo: Vec::new(),
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
            self.push_number(num);
        } else {
            match self.input.as_str() {
                "+" => self.perform_operation(|a, b| a + b),
                "-" => self.perform_operation(|a, b| a - b),
                "/" => self.perform_operation(|a, b| a / b),
                "*" => self.perform_operation(|a, b| a * b),
                "" => self.perform_clone(),
                "%" => self.perform_operation(|a, b| a % b),
                "^" => self.perform_operation(|a, b| b.powf(a)),
                "neg" => self.perform_single_operand_operation(|a| -a),
                "abs" => self.perform_single_operand_operation(|a| a.abs()),
                "sqrt" => self.perform_single_operand_operation(|a| a.sqrt()),
                "sin" => self.perform_single_operand_operation(|a| a.sin()),
                "cos" => self.perform_single_operand_operation(|a| a.cos()),
                "tan" => self.perform_single_operand_operation(|a| a.tan()),
                "asin" => self.perform_single_operand_operation(|a| a.asin()),
                "acos" => self.perform_single_operand_operation(|a| a.acos()),
                "atan" => self.perform_single_operand_operation(|a| a.atan()),
                "deg" => self.perform_single_operand_operation(|a| a.to_degrees()),
                "rad" => self.perform_single_operand_operation(|a| a.to_radians()),
                "!" => self.perform_factorial(),
                "recip" => self.perform_single_operand_operation(|a| 1.0 / a),
                "log10" => self.perform_single_operand_operation(|a| a.log(10.0)),
                "logn" => self.perform_single_operand_operation(|a| a.ln()),
                "log2" => self.perform_single_operand_operation(|a| a.log(2.0)),
                "swap" => self.perform_swap(),
                "clear" => self.perform_clear(),
                "drop" => self.perform_drop(),
                "undo" => self.undo(),
                "redo" => self.redo(),
                "inf" => self.push_infinity(),
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

    fn push_number(&mut self, num: f64) {
        self.undo.push(self.stack.clone());
        self.stack.push(num);
        self.redo.clear();
    }

    fn push_infinity(&mut self) {
        self.undo.push(self.stack.clone());
        self.stack.push(f64::INFINITY);
        self.redo.clear();
    }

    fn undo(&mut self) {
        if let Some(previous_state) = self.undo.pop() {
            // Restore the previous state of the stack.
            //
            self.redo.push(self.stack.clone());
            self.stack = previous_state;
        } else {
            println!("Nothing to undo");
        }
    }

    fn redo(&mut self) {
        if let Some(redo_state) = self.redo.pop() {
            //
            self.undo.push(self.stack.clone());
            self.stack = redo_state;
        } else {
            println!("Nothing to redo");
        }
    }

    fn perform_single_operand_operation<F>(&mut self, operation: F)
    where
        F: FnOnce(f64) -> f64,
    {
        if self.stack.is_empty() {
            return;
        }

        self.undo.push(self.stack.clone()); // Save the current state for undo
        let a = self.stack.pop().unwrap(); // Pop the operand
        let result = operation(a); // Apply the operation
        self.stack.push(result); // Push the result back onto the stack
        self.redo.clear();
    }

    fn perform_operation(&mut self, operation: fn(f64, f64) -> f64) {
        if self.stack.len() < 2 {
            return;
        }
        self.undo.push(self.stack.clone());
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        let result = operation(a, b);
        self.stack.push(result);
        self.redo.clear();
    }

    fn perform_clone(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        self.undo.push(self.stack.clone());
        let a = self.stack.pop().unwrap();
        self.stack.push(a);
        self.stack.push(a);
        self.redo.clear();
    }

    fn perform_factorial(&mut self) {
        if self.stack.is_empty() {
            return;
        }

        self.undo.push(self.stack.clone());
        let a = self.stack.pop().unwrap();
        let abs_a = a.abs();

        fn factorial(n: u64) -> u64 {
            let mut result = 1;
            for i in 1..=n {
                result *= i;
            }
            result
        }

        let rounded_a = abs_a.round() as u64; // Round to the nearest integer and cast to u64

        // Calculate factorial
        let result = factorial(rounded_a);

        self.stack.push(result as f64);
        self.redo.clear();
    }

    fn perform_swap(&mut self) {
        if self.stack.len() < 2 {
            return;
        }
        self.undo.push(self.stack.clone());
        let b = self.stack.pop().unwrap();
        let a = self.stack.pop().unwrap();
        self.stack.push(b);
        self.stack.push(a);
        self.redo.clear();
    }

    fn perform_clear(&mut self) {
        self.undo.push(self.stack.clone());
        self.stack.clear();
    }

    fn perform_drop(&mut self) {
        if self.stack.is_empty() {
            return;
        }
        self.undo.push(self.stack.clone());
        self.stack.pop().unwrap();
    }
}

#[cfg(test)]
mod tests {

    use super::App;

    mod tui {

        use super::App;
        #[test]
        fn cursor_movement_left() {
            let mut app = App::new();
            app.input = String::from("hello");
            app.character_index = 3;
            app.move_cursor_left();
            assert_eq!(app.character_index, 2);
        }

        #[test]
        fn cursor_movement_right() {
            let mut app = App::new();
            app.input = String::from("hello");
            app.character_index = 1;
            app.move_cursor_right();
            assert_eq!(app.character_index, 2);
        }

        #[test]
        fn enter_char() {
            let mut app = App::new();
            app.enter_char('a');
            assert_eq!(app.input, "a");
            assert_eq!(app.character_index, 1);
        }

        #[test]
        fn delete_char() {
            let mut app = App::new();
            app.input = String::from("hello");
            app.character_index = 3;
            app.delete_char();
            assert_eq!(app.input, "helo");
            assert_eq!(app.character_index, 2);
        }
    }

    #[allow(clippy::approx_constant)]
    mod process_input {

        use core::f64;

        use super::App;

        #[test]
        fn addition() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);

            app.input = String::from("95.678");
            app.process_input();
            assert_eq!(app.stack, vec![10.0, 95.678]);

            app.input = String::from("+");
            app.process_input();
            assert_eq!(app.stack, vec![105.678]);
        }

        #[test]
        fn subtraction() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);

            app.input = String::from("4");
            app.process_input();
            assert_eq!(app.stack, vec![10.0, 4.0]);

            app.input = String::from("-");
            app.process_input();
            assert_eq!(app.stack, vec![6.0]);
        }

        #[test]
        fn multiplication() {
            let mut app = App::new();
            app.input = String::from("2");
            app.process_input();
            assert_eq!(app.stack, vec![2.0]);

            app.input = String::from("3");
            app.process_input();
            assert_eq!(app.stack, vec![2.0, 3.0]);

            app.input = String::from("*");
            app.process_input();
            assert_eq!(app.stack, vec![6.0]);
        }

        #[test]
        fn division() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);

            app.input = String::from("2");
            app.process_input();
            assert_eq!(app.stack, vec![10.0, 2.0]);

            app.input = String::from("/");
            app.process_input();
            assert_eq!(app.stack, vec![5.0]);
        }

        #[test]
        fn modulus() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);

            app.input = String::from("3");
            app.process_input();
            assert_eq!(app.stack, vec![10.0, 3.0]);

            app.input = String::from("%");
            app.process_input();
            assert_eq!(app.stack, vec![1.0]);
        }

        #[test]
        fn exponentiation() {
            let mut app = App::new();
            app.input = String::from("2");
            app.process_input();
            assert_eq!(app.stack, vec![2.0]);

            app.input = String::from("3");
            app.process_input();
            assert_eq!(app.stack, vec![2.0, 3.0]);

            app.input = String::from("^");
            app.process_input();
            assert_eq!(app.stack, vec![9.0]);
        }

        #[test]
        fn negation() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);

            app.input = String::from("neg");
            app.process_input();
            assert_eq!(app.stack, vec![-10.0]);
        }

        #[test]
        fn absolute_value() {
            let mut app = App::new();
            app.input = String::from("-5");
            app.process_input();
            assert_eq!(app.stack, vec![-5.0]);

            app.input = String::from("abs");
            app.process_input();
            assert_eq!(app.stack, vec![5.0]);
        }

        #[test]
        fn square_root() {
            let mut app = App::new();
            app.input = String::from("16");
            app.process_input();
            assert_eq!(app.stack, vec![16.0]);

            app.input = String::from("sqrt");
            app.process_input();
            assert_eq!(app.stack, vec![4.0]);
        }

        #[test]
        fn sine() {
            let mut app = App::new();
            app.input = String::from("0");
            app.process_input();
            assert_eq!(app.stack, vec![0.0]);

            app.input = String::from("sin");
            app.process_input();
            assert_eq!(app.stack, vec![0.0]);
        }

        #[test]
        fn cosine() {
            let mut app = App::new();
            app.input = String::from("0");
            app.process_input();
            assert_eq!(app.stack, vec![0.0]);

            app.input = String::from("cos");
            app.process_input();
            assert_eq!(app.stack, vec![1.0]);
        }

        #[test]
        fn tangent() {
            let mut app = App::new();
            app.input = String::from("0");
            app.process_input();
            assert_eq!(app.stack, vec![0.0]);

            app.input = String::from("tan");
            app.process_input();
            assert_eq!(app.stack, vec![0.0]);
        }

        #[test]
        fn arcsine() {
            let mut app = App::new();
            app.input = String::from("1");
            app.process_input();
            assert_eq!(app.stack, vec![1.0]);

            app.input = String::from("asin");
            app.process_input();
            assert_eq!(app.stack, vec![1.5707963267948966]); // ~π/2
        }

        #[test]
        fn arccosine() {
            let mut app = App::new();
            app.input = String::from("1");
            app.process_input();
            assert_eq!(app.stack, vec![1.0]);

            app.input = String::from("acos");
            app.process_input();
            assert_eq!(app.stack, vec![0.0]);
        }

        #[test]
        fn arctangent() {
            let mut app = App::new();
            app.input = String::from("1");
            app.process_input();
            assert_eq!(app.stack, vec![1.0]);

            app.input = String::from("atan");
            app.process_input();
            assert_eq!(app.stack, vec![0.7853981633974483]); // ~π/4
        }

        #[test]
        fn degrees_conversion() {
            let mut app = App::new();
            app.input = String::from("3.141592653589793"); // π
            app.process_input();
            assert_eq!(app.stack, vec![3.141592653589793]);

            app.input = String::from("deg");
            app.process_input();
            assert_eq!(app.stack, vec![180.0]);
        }

        #[test]
        fn radians_conversion() {
            let mut app = App::new();
            app.input = String::from("180");
            app.process_input();
            assert_eq!(app.stack, vec![180.0]);

            app.input = String::from("rad");
            app.process_input();
            assert_eq!(app.stack, vec![3.141592653589793]); // ~π
        }

        #[test]
        fn factorial() {
            let mut app = App::new();
            app.input = String::from("5");
            app.process_input();
            assert_eq!(app.stack, vec![5.0]);

            app.input = String::from("!");
            app.process_input();
            assert_eq!(app.stack, vec![120.0]);
        }

        #[test]
        fn reciprocal() {
            let mut app = App::new();
            app.input = String::from("4");
            app.process_input();
            assert_eq!(app.stack, vec![4.0]);

            app.input = String::from("recip");
            app.process_input();
            assert_eq!(app.stack, vec![0.25]);
        }

        #[test]
        fn log_base_10() {
            let mut app = App::new();
            app.input = String::from("100");
            app.process_input();
            assert_eq!(app.stack, vec![100.0]);

            app.input = String::from("log10");
            app.process_input();
            assert_eq!(app.stack, vec![2.0]);
        }

        #[test]
        fn log_base_natural() {
            let mut app = App::new();
            app.input = String::from("2.718281828459045"); // e
            app.process_input();
            assert_eq!(app.stack, vec![2.718281828459045]);

            app.input = String::from("logn");
            app.process_input();
            assert_eq!(app.stack, vec![1.0]);
        }

        #[test]
        fn log_base_2() {
            let mut app = App::new();
            app.input = String::from("8");
            app.process_input();
            assert_eq!(app.stack, vec![8.0]);

            app.input = String::from("log2");
            app.process_input();
            assert_eq!(app.stack, vec![3.0]);
        }

        #[test]
        fn swap() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            app.input = String::from("5");
            app.process_input();

            app.input = String::from("swap");
            app.process_input();
            assert_eq!(app.stack, vec![5.0, 10.0]);
        }

        #[test]
        fn clear() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            app.input = String::from("5");
            app.process_input();

            app.input = String::from("clear");
            app.process_input();
            assert_eq!(app.stack, vec![]);
        }

        #[test]
        fn drop() {
            let mut app = App::new();
            app.input = String::from("10");
            app.process_input();
            app.input = String::from("5");
            app.process_input();

            app.input = String::from("drop");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);
        }

        #[test]
        fn undo_redo() {
            let mut app = App::new();

            // Push 3.0 to the stack
            app.input = String::from("3");
            app.process_input();
            assert_eq!(app.stack, vec![3.0]);

            // Push 7.0 to the stack
            app.input = String::from("7");
            app.process_input();
            assert_eq!(app.stack, vec![3.0, 7.0]);

            // Perform addition
            app.input = String::from("+");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);

            // Undo the addition
            app.input = String::from("undo");
            app.process_input();
            assert_eq!(app.stack, vec![3.0, 7.0]);

            // Redo the addition
            app.input = String::from("redo");
            app.process_input();
            assert_eq!(app.stack, vec![10.0]);
        }

        #[test]
        fn push_infinity() {
            let mut app = App::new();
            app.input = String::from("inf");
            app.process_input();
            assert_eq!(app.stack, vec![f64::INFINITY])
        }
    }

    mod function_tests {
        use core::f64;

        use super::App;
        #[test]
        fn push_number() {
            let mut app = App::new();
            app.push_number(5.6);
            assert_eq!(app.stack.pop().unwrap(), 5.6);
        }

        #[test]
        fn addition() {
            let mut app = App::new();
            app.push_number(5.0);
            app.push_number(3.0);
            app.perform_operation(|a, b| a + b);
            assert_eq!(app.stack.pop().unwrap(), 8.0);
        }

        #[test]
        fn subtraction() {
            let mut app = App::new();
            app.push_number(10.0);
            app.push_number(4.0);
            app.perform_operation(|a, b| a - b);
            assert_eq!(app.stack.pop().unwrap(), 6.0);
        }

        #[test]
        fn multiplication() {
            let mut app = App::new();
            app.push_number(2.0);
            app.push_number(3.0);
            app.perform_operation(|a, b| a * b);
            assert_eq!(app.stack.pop().unwrap(), 6.0);
        }

        #[test]
        fn division() {
            let mut app = App::new();
            app.push_number(10.0);
            app.push_number(2.0);
            app.perform_operation(|a, b| a / b);
            assert_eq!(app.stack.pop().unwrap(), 5.0);
        }

        #[test]
        fn clone() {
            let mut app = App::new();
            app.push_number(10.0);
            app.push_number(2.0);
            app.perform_clone();
            assert_eq!(app.stack.pop().unwrap(), 2.0);
        }

        #[test]
        fn modulo() {
            let mut app = App::new();
            app.push_number(17.0);
            app.push_number(5.0);
            app.perform_operation(|a, b| a % b);
            assert_eq!(app.stack.pop().unwrap(), 2.0);
        }

        #[test]
        fn exponent() {
            let mut app = App::new();
            app.push_number(4.0);
            app.push_number(5.0);
            app.perform_operation(|a, b| b.powf(a));
            assert_eq!(app.stack.pop().unwrap(), 625.0);
        }

        #[test]
        fn neg() {
            let mut app = App::new();
            app.push_number(4.0);
            app.perform_single_operand_operation(|a| -a);
            assert_eq!(app.stack.pop().unwrap(), -4.0);
        }

        #[test]
        fn abs() {
            let mut app = App::new();
            app.push_number(-4.0);
            app.perform_single_operand_operation(|a| a.abs());
            assert_eq!(app.stack.pop().unwrap(), 4.0);
        }

        #[test]
        fn sqrt() {
            let mut app = App::new();
            app.push_number(9.0);
            app.perform_single_operand_operation(|a| a.sqrt());
            assert_eq!(app.stack.pop().unwrap(), 3.0);
        }

        #[test]
        fn sin() {
            let mut app = App::new();
            app.push_number(9.0);
            app.perform_single_operand_operation(|a| a.sin());
            assert_eq!(app.stack.pop().unwrap(), 0.4121184852417566);
        }

        #[test]
        fn cos() {
            let mut app = App::new();
            app.push_number(5.0);
            app.perform_single_operand_operation(|a| a.cos());
            assert_eq!(app.stack.pop().unwrap(), 0.28366218546322625);
        }

        #[test]
        fn tan() {
            let mut app = App::new();
            app.push_number(6.0);
            app.perform_single_operand_operation(|a| a.tan());
            assert_eq!(app.stack.pop().unwrap(), -0.29100619138474915);
        }

        #[test]
        fn asin() {
            let mut app = App::new();
            app.push_number(0.6);
            app.perform_single_operand_operation(|a| a.asin());
            assert_eq!(app.stack.pop().unwrap(), 0.6435011087932844);
        }

        #[test]
        fn acos() {
            let mut app = App::new();
            app.push_number(0.7);
            app.perform_single_operand_operation(|a| a.acos());
            assert_eq!(app.stack.pop().unwrap(), 0.7953988301841436);
        }

        #[test]
        fn atan() {
            let mut app = App::new();
            app.push_number(5.0);
            app.perform_single_operand_operation(|a| a.atan());
            assert_eq!(app.stack.pop().unwrap(), 1.373400766945016);
        }

        #[test]
        fn convert_to_degrees() {
            let mut app = App::new();
            app.push_number(1.0);
            app.perform_single_operand_operation(|a| a.to_degrees());
            assert_eq!(app.stack.pop().unwrap(), 57.29577951308232);
        }

        #[test]
        fn convert_to_radians() {
            let mut app = App::new();
            app.push_number(95.0);
            app.perform_single_operand_operation(|a| a.to_radians());
            assert_eq!(app.stack.pop().unwrap(), 1.6580627893946132);
        }

        #[test]
        fn factorial() {
            let mut app = App::new();
            app.push_number(5.0);
            app.perform_factorial();
            assert_eq!(app.stack.pop().unwrap(), 120.0);
        }

        #[test]
        fn recipricol() {
            let mut app = App::new();
            app.push_number(4.0);
            app.perform_single_operand_operation(|a| 1.0 / a);
            assert_eq!(app.stack.pop().unwrap(), 0.25);
        }

        #[test]
        fn log10() {
            let mut app = App::new();
            app.push_number(50.0);
            app.perform_single_operand_operation(|a| a.log(10.0));
            assert_eq!(app.stack.pop().unwrap(), 1.6989700043360185);
        }

        #[test]
        fn logn() {
            let mut app = App::new();
            app.push_number(50.0);
            app.perform_single_operand_operation(|a| a.ln());
            assert_eq!(app.stack.pop().unwrap(), 3.912023005428146);
        }

        #[test]
        fn log2() {
            let mut app = App::new();
            app.push_number(50.0);
            app.perform_single_operand_operation(|a| a.log(2.0));
            assert_eq!(app.stack.pop().unwrap(), 5.643856189774724);
        }

        #[test]
        fn push_infinity() {
            let mut app = App::new();
            app.push_infinity();
            assert_eq!(app.stack.pop().unwrap(), f64::INFINITY)
        }

        #[test]
        fn undo_redo() {
            let mut app = App::new();
            app.push_number(3.0);
            app.push_number(7.0);
            app.perform_operation(|a, b| a + b);

            // Verify that the stack has the result of the addition
            assert_eq!(app.stack, vec![10.0]);

            // Undo the addition, should revert to the original stack state
            app.undo();
            assert_eq!(app.stack, vec![3.0, 7.0]);

            // Redo the addition, should return the stack to [10.0]
            app.redo();
            assert_eq!(app.stack, vec![10.0]);
        }

        #[test]
        fn clear() {
            let mut app = App::new();
            app.push_number(42.0);
            app.perform_clear();
            assert!(app.stack.is_empty());
        }

        #[test]
        fn drop() {
            let mut app = App::new();
            app.push_number(5.0);
            app.push_number(10.0);
            app.perform_drop();
            assert_eq!(app.stack.len(), 1);
            assert_eq!(app.stack.pop().unwrap(), 5.0);
        }

        #[test]
        fn swap() {
            let mut app = App::new();
            app.push_number(1.0);
            app.push_number(2.0);
            app.perform_swap();
            assert_eq!(app.stack.pop().unwrap(), 1.0);
            assert_eq!(app.stack.pop().unwrap(), 2.0);
        }
    }

    mod edge_cases {

        use super::App;

        #[test]
        fn divide_pos_by_0() {
            let mut app = App::new();
            app.push_number(10.0);
            app.push_number(0.0);
            app.perform_operation(|a, b| a / b);
            assert_eq!(app.stack.pop().unwrap(), f64::INFINITY);
        }

        #[test]
        fn divide_neg_by_0() {
            let mut app = App::new();
            app.push_number(-10.0);
            app.push_number(0.0);
            app.perform_operation(|a, b| a / b);
            assert_eq!(app.stack.pop().unwrap(), -f64::INFINITY);
        }
    }
}
