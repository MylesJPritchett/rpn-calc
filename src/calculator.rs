use std::f64::consts::PI;
use std::fmt;

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

impl Kind {
    /// How many Values must be on the Stack for this Operation to run.
    fn operands(self) -> usize {
        match self {
            Kind::Binary(_) | Kind::Swap => 2,
            Kind::Unary(_) | Kind::Drop | Kind::Duplicate => 1,
            _ => 0,
        }
    }
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
            Kind::Duplicate => "dup (empty)",
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
    op("^", Category::Binary, Kind::Binary(f64::powf)),
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
    op("log10", Category::Unary, Kind::Unary(f64::log10)),
    op("logn", Category::Unary, Kind::Unary(f64::ln)),
    op("log2", Category::Unary, Kind::Unary(f64::log2)),
    op("swap", Category::Stack, Kind::Swap),
    op("clear", Category::Stack, Kind::Clear),
    op("drop", Category::Stack, Kind::Drop),
    op("dup", Category::Stack, Kind::Duplicate),
    op("undo", Category::History, Kind::Undo),
    op("redo", Category::History, Kind::Redo),
    op("inf", Category::Constants, Kind::Constant(f64::INFINITY)),
    op("pi", Category::Constants, Kind::Constant(PI)),
    op("quit", Category::Program, Kind::Quit),
];

/// Looks up an Operation by name. Empty input is an alias for `dup`.
pub fn lookup(name: &str) -> Option<&'static Operation> {
    let name = if name.is_empty() { "dup" } else { name };
    OPERATIONS.iter().find(|op| op.name == name)
}

/// 171! and above overflow `f64`.
const MAX_FACTORIAL: f64 = 170.0;

fn factorial(a: f64) -> f64 {
    let n = a.abs().round();
    if n.is_nan() {
        return f64::NAN;
    }
    if n > MAX_FACTORIAL {
        return f64::INFINITY;
    }
    (1..=n as u64).map(|i| i as f64).product()
}

/// What the caller should do after an input has been executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Quit,
}

/// Why an input was rejected. A rejected input leaves the Stack and History unchanged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    UnknownCommand(String),
    NotEnoughOperands {
        op: &'static str,
        needed: usize,
        have: usize,
    },
    NothingToUndo,
    NothingToRedo,
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalcError::UnknownCommand(input) => write!(f, "unknown command: {input}"),
            CalcError::NotEnoughOperands { op, needed, have } => {
                let values = if *needed == 1 { "value" } else { "values" };
                write!(f, "{op} needs {needed} {values}, stack has {have}")
            }
            CalcError::NothingToUndo => write!(f, "nothing to undo"),
            CalcError::NothingToRedo => write!(f, "nothing to redo"),
        }
    }
}

impl std::error::Error for CalcError {}

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

    pub fn execute(&mut self, input: &str) -> Result<Outcome, CalcError> {
        if let Ok(value) = input.parse::<f64>() {
            self.change(|stack| stack.push(value));
            return Ok(Outcome::Continue);
        }
        let Some(operation) = lookup(input) else {
            return Err(CalcError::UnknownCommand(input.to_string()));
        };
        let len = self.stack.len();
        let needed = operation.kind.operands();
        if len < needed {
            return Err(CalcError::NotEnoughOperands {
                op: operation.name,
                needed,
                have: len,
            });
        }
        match operation.kind {
            Kind::Binary(f) => self.change(|stack| {
                let b = stack.pop().unwrap();
                let a = stack.pop().unwrap();
                stack.push(f(a, b));
            }),
            Kind::Unary(f) => self.change(|stack| {
                let a = stack.pop().unwrap();
                stack.push(f(a));
            }),
            Kind::Constant(value) => self.change(|stack| stack.push(value)),
            Kind::Swap => self.change(|stack| stack.swap(len - 1, len - 2)),
            Kind::Duplicate => self.change(|stack| stack.push(stack[len - 1])),
            Kind::Clear => self.change(Vec::clear),
            Kind::Drop => self.change(|stack| {
                stack.pop();
            }),
            Kind::Undo => {
                let previous = self.undo.pop().ok_or(CalcError::NothingToUndo)?;
                self.redo.push(std::mem::replace(&mut self.stack, previous));
            }
            Kind::Redo => {
                let next = self.redo.pop().ok_or(CalcError::NothingToRedo)?;
                self.undo.push(std::mem::replace(&mut self.stack, next));
            }
            Kind::Quit => return Ok(Outcome::Quit),
        }
        Ok(Outcome::Continue)
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
            let _ = calc.execute(input);
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
    fn exponent_raises_second_value_to_top() {
        assert_stack(&["2", "3", "^"], &[8.0]);
        assert_stack(&["4", "5", "^"], &[1024.0]);
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
        assert_stack(&["50", "log10"], &[1.6989700043360187]);
        assert_stack(&["2.718281828459045", "logn"], &[1.0]);
        assert_stack(&["50", "logn"], &[3.912023005428146]);
        assert_stack(&["8", "log2"], &[3.0]);
        assert_stack(&["50", "log2"], &[5.643856189774724]);
    }

    #[test]
    fn logs_of_exact_powers_are_exact() {
        assert_stack(&["1000", "log10"], &[3.0]);
        assert_stack(&["8", "log2"], &[3.0]);
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
        calc.execute("170").unwrap();
        calc.execute("!").unwrap();
        assert_eq!(calc.stack(), &[7.257415615307994e306]);
        assert_stack(&["171", "!"], &[f64::INFINITY]);
    }

    #[test]
    fn factorial_of_huge_or_infinite_values_is_infinity() {
        assert_stack(&["inf", "!"], &[f64::INFINITY]);
        assert_stack(&["1e20", "!"], &[f64::INFINITY]);
        assert_stack(&["170.6", "!"], &[f64::INFINITY]);
    }

    #[test]
    fn factorial_of_nan_is_nan() {
        let mut calc = Calculator::new();
        for input in ["inf", "inf", "-", "!"] {
            calc.execute(input).unwrap();
        }
        assert_eq!(calc.stack().len(), 1);
        assert!(calc.stack()[0].is_nan());
    }

    #[test]
    fn stack_commands() {
        assert_stack(&["10", "5", "swap"], &[5.0, 10.0]);
        assert_stack(&["10", "5", "clear"], &[]);
        assert_stack(&["10", "5", "drop"], &[10.0]);
        assert_stack(&["10", "2", ""], &[10.0, 2.0, 2.0]);
        assert_stack(&["10", "2", "dup"], &[10.0, 2.0, 2.0]);
    }

    #[test]
    fn stack_commands_with_too_few_values_do_nothing() {
        assert_stack(&["1", "swap"], &[1.0]);
        assert_stack(&["drop"], &[]);
        assert_stack(&[""], &[]);
        assert_stack(&["dup"], &[]);
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
    fn clear_clears_redo() {
        assert_stack(&["3", "7", "+", "undo", "clear", "redo"], &[]);
    }

    #[test]
    fn drop_clears_redo() {
        assert_stack(&["3", "7", "+", "undo", "drop", "redo"], &[3.0]);
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
    fn unknown_input_reports_unknown_command() {
        let mut calc = Calculator::new();
        assert_eq!(
            calc.execute("foo"),
            Err(CalcError::UnknownCommand("foo".to_string()))
        );
    }

    #[test]
    fn too_few_values_reports_not_enough_operands() {
        let cases: [(&[&str], &str, usize, usize); 4] = [
            (&["1"], "swap", 2, 1),
            (&["1"], "+", 2, 1),
            (&[], "neg", 1, 0),
            (&[], "drop", 1, 0),
        ];
        for (setup, op, needed, have) in cases {
            let mut calc = Calculator::new();
            for input in setup {
                calc.execute(input).unwrap();
            }
            assert_eq!(
                calc.execute(op),
                Err(CalcError::NotEnoughOperands { op, needed, have })
            );
        }
    }

    #[test]
    fn empty_history_reports_nothing_to_undo_or_redo() {
        let mut calc = Calculator::new();
        assert_eq!(calc.execute("undo"), Err(CalcError::NothingToUndo));
        assert_eq!(calc.execute("redo"), Err(CalcError::NothingToRedo));
        calc.execute("1").unwrap();
        assert_eq!(calc.execute("redo"), Err(CalcError::NothingToRedo));
    }

    #[test]
    fn errors_display_as_status_messages() {
        let cases = [
            (
                CalcError::UnknownCommand("foo".to_string()),
                "unknown command: foo",
            ),
            (
                CalcError::NotEnoughOperands {
                    op: "swap",
                    needed: 2,
                    have: 1,
                },
                "swap needs 2 values, stack has 1",
            ),
            (
                CalcError::NotEnoughOperands {
                    op: "neg",
                    needed: 1,
                    have: 0,
                },
                "neg needs 1 value, stack has 0",
            ),
            (CalcError::NothingToUndo, "nothing to undo"),
            (CalcError::NothingToRedo, "nothing to redo"),
        ];
        for (error, text) in cases {
            assert_eq!(error.to_string(), text);
        }
    }

    #[test]
    fn a_failed_operation_leaves_stack_and_history_unchanged() {
        let mut calc = Calculator::new();
        for input in ["1", "2", "+", "5", "undo"] {
            calc.execute(input).unwrap();
        }
        for input in ["foo", "swap"] {
            assert!(calc.execute(input).is_err(), "{input:?}");
            assert_eq!(calc.stack(), &[3.0], "{input:?}");
        }
        calc.execute("redo").unwrap();
        assert_eq!(calc.stack(), &[3.0, 5.0]);
        calc.execute("undo").unwrap();
        calc.execute("undo").unwrap();
        assert_eq!(calc.stack(), &[1.0, 2.0]);
    }

    #[test]
    fn quit_reports_quit_and_leaves_stack_unchanged() {
        let mut calc = Calculator::with_stack(vec![1.0]);
        assert_eq!(calc.execute("quit"), Ok(Outcome::Quit));
        assert_eq!(calc.stack(), &[1.0]);
    }

    #[test]
    fn other_input_reports_continue() {
        let mut calc = Calculator::new();
        assert_eq!(calc.execute("1"), Ok(Outcome::Continue));
        assert_eq!(calc.execute("neg"), Ok(Outcome::Continue));
    }

    #[test]
    fn starts_from_an_existing_stack() {
        let mut calc = Calculator::with_stack(vec![2.0, 3.0]);
        calc.execute("*").unwrap();
        assert_eq!(calc.stack(), &[6.0]);
    }

    fn is_accepted(name: &str) -> bool {
        let mut calc = Calculator::new();
        for input in ["2", "-0.5", "7", "undo"] {
            calc.execute(input).unwrap();
        }
        let before = calc.stack().to_vec();
        let outcome = calc.execute(name);
        outcome == Ok(Outcome::Quit) || calc.stack() != before.as_slice()
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
            "drop", "dup", "undo", "redo", "inf", "pi", "quit",
        ];
        let mut table: Vec<_> = OPERATIONS.iter().map(|op| op.name).collect();
        let mut expected = commands.to_vec();
        table.sort();
        expected.sort();
        assert_eq!(table, expected);

        for name in ["foo", "exit", "q", "PI", "+ "] {
            assert!(!is_accepted(name), "{name:?}");
        }
    }

    #[test]
    fn empty_input_is_accepted_as_duplicate() {
        assert!(is_accepted(""));
    }

    #[test]
    fn operations_guide_lists_dup() {
        assert!(OPERATIONS.iter().any(|op| op.label().starts_with("dup")));
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
