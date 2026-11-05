# Plain-text Stack persistence in the platform data directory

## Context

The Stack was saved to `stack.txt` in whatever directory the app was launched from, so the user's work was scattered or lost, and the tests overwrote each other's file.

## Decision

After every successful change, the Stack is saved as one Value per line to `rpn-calc/stack.txt` in the platform data directory, resolved with the `dirs` crate. If the platform has no data directory, `stack.txt` in the current directory is used. We kept plain text over JSON or a database because the Stack is a single list of numbers.

## Consequences

- The file is human-readable and editable. Loading skips lines that don't parse and treats a missing file as an empty Stack, so startup never fails.
- History isn't saved, so `undo` can't reach past the start of a session.
- A failed save shows a Status message instead of quitting.
- `load` and `save` take an explicit path, so tests use a temp directory and never touch the real Stack.
