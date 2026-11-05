use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Binary,
    Unary,
    Stack,
    History,
    Constants,
    Program,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Category::Binary,
        Category::Unary,
        Category::Stack,
        Category::History,
        Category::Constants,
        Category::Program,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Category::Binary => "Binary",
            Category::Unary => "Unary",
            Category::Stack => "Stack",
            Category::History => "History",
            Category::Constants => "Constants",
            Category::Program => "Program",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Binary(fn(f64, f64) -> f64),
    Unary(fn(f64) -> f64),
    Constant(f64),
    Swap,
    Clear,
    Drop,
    Duplicate,
    Undo,
    Redo,
    Quit,
}

#[derive(Debug, Clone, Copy)]
pub struct Operation {
    pub name: &'static str,
    pub category: Category,
    pub kind: Kind,
}

impl Operation {
    /// The name shown in the Operations Guide.
    pub fn label(&self) -> &'static str {
        match self.kind {
            Kind::Duplicate => "clone (empty)",
            _ => self.name,
        }
    }
}

const fn op(name: &'static str, category: Category, kind: Kind) -> Operation {
    Operation {
        name,
        category,
        kind,
    }
}

pub static OPERATIONS: &[Operation] = &[
    op("+", Category::Binary, Kind::Binary(|a, b| a + b)),
    op("-", Category::Binary, Kind::Binary(|a, b| a - b)),
    op("*", Category::Binary, Kind::Binary(|a, b| a * b)),
    op("/", Category::Binary, Kind::Binary(|a, b| a / b)),
    op("%", Category::Binary, Kind::Binary(|a, b| a % b)),
    op("^", Category::Binary, Kind::Binary(|a, b| b.powf(a))),
    op("neg", Category::Unary, Kind::Unary(|a| -a)),
    op("abs", Category::Unary, Kind::Unary(f64::abs)),
    op("sqrt", Category::Unary, Kind::Unary(f64::sqrt)),
    op("sin", Category::Unary, Kind::Unary(f64::sin)),
    op("cos", Category::Unary, Kind::Unary(f64::cos)),
    op("tan", Category::Unary, Kind::Unary(f64::tan)),
    op("asin", Category::Unary, Kind::Unary(f64::asin)),
    op("acos", Category::Unary, Kind::Unary(f64::acos)),
    op("atan", Category::Unary, Kind::Unary(f64::atan)),
    op("deg", Category::Unary, Kind::Unary(f64::to_degrees)),
    op("rad", Category::Unary, Kind::Unary(f64::to_radians)),
    op("!", Category::Unary, Kind::Unary(factorial)),
    op("recip", Category::Unary, Kind::Unary(|a| 1.0 / a)),
    op("log10", Category::Unary, Kind::Unary(|a| a.log(10.0))),
    op("logn", Category::Unary, Kind::Unary(f64::ln)),
    op("log2", Category::Unary, Kind::Unary(|a| a.log(2.0))),
    op("swap", Category::Stack, Kind::Swap),
    op("clear", Category::Stack, Kind::Clear),
    op("drop", Category::Stack, Kind::Drop),
    op("", Category::Stack, Kind::Duplicate),
    op("undo", Category::History, Kind::Undo),
    op("redo", Category::History, Kind::Redo),
    op("inf", Category::Constants, Kind::Constant(f64::INFINITY)),
    op("pi", Category::Constants, Kind::Constant(PI)),
    op("quit", Category::Program, Kind::Quit),
];

pub fn lookup(name: &str) -> Option<&'static Operation> {
    OPERATIONS.iter().find(|op| op.name == name)
}

fn factorial(a: f64) -> f64 {
    let n = a.abs().round() as u64;
    let mut result = 1.0;
    for i in 1..=n {
        result *= i as f64;
    }
    result
}

/// What the caller should do after an input has been executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Quit,
}

#[derive(Debug, Default)]
pub struct Calculator {
    stack: Vec<f64>,
    undo: Vec<Vec<f64>>,
    redo: Vec<Vec<f64>>,
}

impl Calculator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_stack(stack: Vec<f64>) -> Self {
        Self {
            stack,
            ..Self::default()
        }
    }

    pub fn stack(&self) -> &[f64] {
        &self.stack
    }

    pub fn execute(&mut self, input: &str) -> Outcome {
        if let Ok(value) = input.parse::<f64>() {
            self.change(|stack| stack.push(value));
            return Outcome::Continue;
        }
        let Some(operation) = lookup(input) else {
            return Outcome::Continue;
        };
        let len = self.stack.len();
        match operation.kind {
            Kind::Binary(f) if len >= 2 => self.change(|stack| {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(f(a, b));
            }),
            Kind::Unary(f) if len >= 1 => self.change(|stack| {
                let a = stack.pop().unwrap();
                stack.push(f(a));
            }),
            Kind::Constant(value) => self.change(|stack| stack.push(value)),
            Kind::Swap if len >= 2 => self.change(|stack| stack.swap(len - 1, len - 2)),
            Kind::Duplicate if len >= 1 => self.change(|stack| stack.push(stack[len - 1])),
            Kind::Clear => {
                self.undo.push(self.stack.clone());
                self.stack.clear();
            }
            Kind::Drop if len >= 1 => {
                self.undo.push(self.stack.clone());
                self.stack.pop();
            }
            Kind::Undo => {
                if let Some(previous) = self.undo.pop() {
                    self.redo.push(std::mem::replace(&mut self.stack, previous));
                }
            }
            Kind::Redo => {
                if let Some(next) = self.redo.pop() {
                    self.undo.push(std::mem::replace(&mut self.stack, next));
                }
            }
            Kind::Quit => return Outcome::Quit,
            _ => {}
        }
        Outcome::Continue
    }

    fn change(&mut self, apply: impl FnOnce(&mut Vec<f64>)) {
        self.undo.push(self.stack.clone());
        apply(&mut self.stack);
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_stack(inputs: &[&str], expected: &[f64]) {
        let mut calc = Calculator::new();
        for input in inputs {
            calc.execute(input);
        }
        assert_eq!(calc.stack(), expected, "inputs: {inputs:?}");
    }

    #[test]
    fn numeric_input_pushes_a_value() {
        assert_stack(&["10", "95.678"], &[10.0, 95.678]);
    }

    #[test]
    fn binary_operations() {
        assert_stack(&["10", "95.678", "+"], &[105.678]);
        assert_stack(&["10", "4", "-"], &[6.0]);
        assert_stack(&["2", "3", "*"], &[6.0]);
        assert_stack(&["10", "2", "/"], &[5.0]);
        assert_stack(&["10", "3", "%"], &[1.0]);
        assert_stack(&["17", "5", "%"], &[2.0]);
    }

    #[test]
    fn exponent_raises_top_to_second_value() {
        assert_stack(&["2", "3", "^"], &[9.0]);
        assert_stack(&["4", "5", "^"], &[625.0]);
    }

    #[test]
    fn binary_operation_with_too_few_values_does_nothing() {
        assert_stack(&["+"], &[]);
        assert_stack(&["1", "+"], &[1.0]);
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn unary_operations() {
        assert_stack(&["10", "neg"], &[-10.0]);
        assert_stack(&["-5", "abs"], &[5.0]);
        assert_stack(&["16", "sqrt"], &[4.0]);
        assert_stack(&["0", "sin"], &[0.0]);
        assert_stack(&["9", "sin"], &[0.4121184852417566]);
        assert_stack(&["0", "cos"], &[1.0]);
        assert_stack(&["5", "cos"], &[0.28366218546322625]);
        assert_stack(&["0", "tan"], &[0.0]);
        assert_stack(&["6", "tan"], &[-0.29100619138474915]);
        assert_stack(&["1", "asin"], &[1.5707963267948966]);
        assert_stack(&["0.6", "asin"], &[0.6435011087932844]);
        assert_stack(&["1", "acos"], &[0.0]);
        assert_stack(&["0.7", "acos"], &[0.7953988301841436]);
        assert_stack(&["1", "atan"], &[0.7853981633974483]);
        assert_stack(&["5", "atan"], &[1.373400766945016]);
        assert_stack(&["3.141592653589793", "deg"], &[180.0]);
        assert_stack(&["1", "deg"], &[57.29577951308232]);
        assert_stack(&["180", "rad"], &[3.141592653589793]);
        assert_stack(&["95", "rad"], &[1.6580627893946132]);
        assert_stack(&["4", "recip"], &[0.25]);
        assert_stack(&["100", "log10"], &[2.0]);
        assert_stack(&["50", "log10"], &[1.6989700043360185]);
        assert_stack(&["2.718281828459045", "logn"], &[1.0]);
        assert_stack(&["50", "logn"], &[3.912023005428146]);
        assert_stack(&["8", "log2"], &[3.0]);
        assert_stack(&["50", "log2"], &[5.643856189774724]);
    }

    #[test]
    fn unary_operation_on_empty_stack_does_nothing() {
        assert_stack(&["neg"], &[]);
        assert_stack(&["!"], &[]);
    }

    #[test]
    fn factorial() {
        assert_stack(&["5", "!"], &[120.0]);
        assert_stack(&["0", "!"], &[1.0]);
        assert_stack(&["-5", "!"], &[120.0]);
        assert_stack(&["4.6", "!"], &[120.0]);
    }

    #[test]
    fn factorial_stays_exact_up_to_170_and_overflows_at_171() {
        let mut calc = Calculator::new();
        calc.execute("170");
        calc.execute("!");
        assert_eq!(calc.stack(), &[7.257415615307994e306]);
        assert_stack(&["171", "!"], &[f64::INFINITY]);
    }

    #[test]
    fn stack_commands() {
        assert_stack(&["10", "5", "swap"], &[5.0, 10.0]);
        assert_stack(&["10", "5", "clear"], &[]);
        assert_stack(&["10", "5", "drop"], &[10.0]);
        assert_stack(&["10", "2", ""], &[10.0, 2.0, 2.0]);
    }

    #[test]
    fn stack_commands_with_too_few_values_do_nothing() {
        assert_stack(&["1", "swap"], &[1.0]);
        assert_stack(&["drop"], &[]);
        assert_stack(&[""], &[]);
        assert_stack(&["clear"], &[]);
    }

    #[test]
    fn constants() {
        assert_stack(&["inf"], &[f64::INFINITY]);
        assert_stack(&["pi"], &[std::f64::consts::PI]);
    }

    #[test]
    fn undo_and_redo_restore_snapshots() {
        assert_stack(&["3", "7", "+", "undo"], &[3.0, 7.0]);
        assert_stack(&["3", "7", "+", "undo", "redo"], &[10.0]);
        assert_stack(&["3", "7", "+", "undo", "undo", "undo"], &[]);
        assert_stack(&["3", "clear", "undo"], &[3.0]);
        assert_stack(&["3", "4", "drop", "undo"], &[3.0, 4.0]);
    }

    #[test]
    fn a_new_change_clears_redo() {
        assert_stack(&["3", "7", "+", "undo", "1", "redo"], &[3.0, 7.0, 1.0]);
    }

    #[test]
    fn undo_and_redo_with_empty_history_do_nothing() {
        assert_stack(&["undo"], &[]);
        assert_stack(&["redo"], &[]);
        assert_stack(&["1", "redo"], &[1.0]);
    }

    #[test]
    fn unknown_input_does_nothing() {
        assert_stack(&["1", "foo"], &[1.0]);
        assert_stack(&["1", "foo", "undo"], &[]);
    }

    #[test]
    fn quit_reports_quit_and_leaves_stack_unchanged() {
        let mut calc = Calculator::with_stack(vec![1.0]);
        assert_eq!(calc.execute("quit"), Outcome::Quit);
        assert_eq!(calc.stack(), &[1.0]);
    }

    #[test]
    fn other_input_reports_continue() {
        let mut calc = Calculator::new();
        assert_eq!(calc.execute("1"), Outcome::Continue);
        assert_eq!(calc.execute("neg"), Outcome::Continue);
    }

    #[test]
    fn starts_from_an_existing_stack() {
        let mut calc = Calculator::with_stack(vec![2.0, 3.0]);
        calc.execute("*");
        assert_eq!(calc.stack(), &[6.0]);
    }

    fn is_accepted(name: &str) -> bool {
        let mut calc = Calculator::new();
        for input in ["2", "-0.5", "7", "undo"] {
            calc.execute(input);
        }
        let before = calc.stack().to_vec();
        let outcome = calc.execute(name);
        outcome == Outcome::Quit || calc.stack() != before.as_slice()
    }

    #[test]
    fn every_table_entry_is_accepted_by_execute() {
        for operation in OPERATIONS {
            assert!(is_accepted(operation.name), "{:?}", operation.name);
        }
    }

    #[test]
    fn accepted_command_names_are_exactly_the_table() {
        let commands = [
            "+", "-", "*", "/", "%", "^", "neg", "abs", "sqrt", "sin", "cos", "tan", "asin",
            "acos", "atan", "deg", "rad", "!", "recip", "log10", "logn", "log2", "swap", "clear",
            "drop", "", "undo", "redo", "inf", "pi", "quit",
        ];
        let mut table: Vec<_> = OPERATIONS.iter().map(|op| op.name).collect();
        let mut expected = commands.to_vec();
        table.sort();
        expected.sort();
        assert_eq!(table, expected);

        for name in ["foo", "dup", "exit", "q", "PI", "+ "] {
            assert!(!is_accepted(name), "{name:?}");
        }
    }

    #[test]
    fn every_category_has_operations() {
        for category in Category::ALL {
            assert!(OPERATIONS.iter().any(|op| op.category == category));
        }
    }

    #[test]
    fn division_by_zero_gives_infinity() {
        assert_stack(&["10", "0", "/"], &[f64::INFINITY]);
        assert_stack(&["-10", "0", "/"], &[f64::NEG_INFINITY]);
    }

    #[test]
    fn very_big_and_very_small_values() {
        assert_stack(
            &["1e21", "1e21", "*"],
            &[1_000_000_000_000_000_000_000_000_000_000_000_000_000_000.0],
        );
        assert_stack(&["1.0", "1.0", "*"], &[1.0]);
    }
}
