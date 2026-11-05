use rpn_calc::calculator::{Calculator, Outcome};
use rpn_calc::persistence;

#[test]
fn session_survives_save_and_reload() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stack.txt");

    let mut calc = Calculator::new();
    for input in ["3", "4", "+", "pi", "2", "undo"] {
        assert_eq!(calc.execute(input), Outcome::Continue);
    }
    persistence::save(&path, calc.stack()).unwrap();

    let mut reloaded = Calculator::with_stack(persistence::load(&path).unwrap());
    assert_eq!(reloaded.stack(), &[7.0, std::f64::consts::PI]);
    reloaded.execute("swap");
    assert_eq!(reloaded.stack(), &[std::f64::consts::PI, 7.0]);
}

#[test]
fn quit_ends_a_session_without_changing_the_stack() {
    let mut calc = Calculator::new();
    calc.execute("5");
    assert_eq!(calc.execute("quit"), Outcome::Quit);
    assert_eq!(calc.stack(), &[5.0]);
}
