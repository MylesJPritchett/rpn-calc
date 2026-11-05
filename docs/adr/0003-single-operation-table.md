# A single Operation table drives dispatch and the Operations Guide

## Context

Dispatch used to be a hand-written `match`, and the Operations Guide was a separate hand-written list. The two had to be kept in sync by hand.

## Decision

Every Operation is one entry in a static table (`OPERATIONS` in `src/calculator.rs`) with its name and its kind, e.g. `Binary(fn)`, `Unary(fn)`, `Constant(value)` or `Swap`. `Calculator::execute` looks the input up in the table and checks the operand count from the kind before applying it. Each kind also knows its category, so the TUI renders the Operations Guide by grouping the same table by category without a separate field that could disagree with the kind.

## Consequences

- Adding a maths Operation is a one-line change, and it appears in the Operations Guide automatically.
- Tests check that every table entry is accepted by `execute` and that no other command names are, so the two can't drift.
- Stack commands, History commands and `quit` still need their own kind and a branch in the small `match` over kinds.
