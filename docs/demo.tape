# Generates docs/screenshot.png and docs/demo.gif.
# Run from the repo root: vhs docs/demo.tape

Output docs/demo.gif

Require cargo

Set Shell bash
Set FontSize 18
Set Width 1200
Set Height 700
Set Padding 20
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms

# Build, and use a throwaway data directory so the real Stack is neither shown nor overwritten.
Hide
Type "cargo build --release --quiet && export XDG_DATA_HOME=$(mktemp -d) && clear"
Enter
Wait@120s
Type "./target/release/rpn-calc"
Enter
Sleep 1s
Show

Type "12" Enter
Type "30" Enter
Type "+" Enter
Sleep 500ms
Type "2" Enter
Type "^" Enter
Sleep 500ms
Type "sqrt" Enter
Sleep 500ms
Type "5" Enter
Type "!" Enter
Sleep 500ms
Type "fact" Enter
Sleep 2s
Backspace 4
Sleep 300ms
Type "pi" Enter
Sleep 1s

Screenshot docs/screenshot.png
Sleep 1s

Hide
Type "quit" Enter
