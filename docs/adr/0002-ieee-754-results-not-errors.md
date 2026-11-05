# IEEE 754 results instead of errors for maths

## Context

Some inputs are outside an Operation's domain: `10 0 /`, `-1 sqrt`, `1e20 !`. The calculator could reject them with an error, or return whatever `f64` gives.

## Decision

Operations always produce a Value, following IEEE 754: `10 0 /` gives `inf` and `-1 sqrt` gives `NaN`. Errors are kept for input the calculator can't act on: an unknown command, too few operands, or nothing to undo or redo.

Factorial follows the same rules. `!` rounds `|x|` to the nearest whole number and multiplies in `f64`, which is exact up to 170!. Anything above 170 gives `inf` straight away, without looping, and `NaN` gives `NaN`. There's no gamma function.

## Consequences

- Every maths Operation is a plain `fn(f64) -> f64` or `fn(f64, f64) -> f64`, with no error path.
- `inf` and `NaN` can sit on the Stack and are saved like any other Value. The result itself is the feedback, so there's no Status message for domain problems.
