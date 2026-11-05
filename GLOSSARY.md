# rpn-calc

A Reverse Polish Notation calculator for the terminal: the user pushes Values onto a Stack and applies Operations to them.

## Stack and Values

**Stack**:
The ordered list of Values the user is working on. It is kept between sessions.
_Avoid_: List, buffer, memory

**Top**:
The most recently pushed Value on the Stack, shown at index `0` in the Stack panel.
_Avoid_: Head, last, first

**Value**:
A single number on the Stack, which may be `inf` or `NaN`.
_Avoid_: Number, item, entry

## Operations

**Operation**:
A named command the user types to act on the Stack or its History.
_Avoid_: Function, op, instruction

**Binary Operation**:
An Operation that replaces the Top and the Value below it with one result, e.g. `+` or `^`. The Value below the Top is the left-hand side, so `2 3 ^` is 2³.
_Avoid_: Two-operand operation

**Unary Operation**:
An Operation that replaces the Top with one result, e.g. `sqrt` or `!`.
_Avoid_: Single-operand operation, function

**Stack command**:
An Operation that rearranges or removes Values without computing anything: `swap`, `drop`, `clear` and `dup`.
_Avoid_: Stack operation, stack manipulation

**Constant**:
An Operation that pushes a fixed Value: `pi` or `inf`.

**History command**:
An Operation that moves through History: `undo` or `redo`.

**Duplicate**:
To push a copy of the Top, with `dup` or by submitting an empty line.
_Avoid_: Copy, repeat

## History

**Snapshot**:
A copy of the whole Stack as it was before a change.
_Avoid_: Checkpoint, state, delta

**History**:
The Snapshots that `undo` and `redo` move between. Any change to the Stack records a Snapshot and discards anything `redo` could have restored.
_Avoid_: Log, journal, undo buffer

## Interface

**Operations Guide**:
The panel beside the Stack that lists every Operation by category.
_Avoid_: Help, cheat sheet, menu

**Status message**:
A one-line message saying why the last input did nothing, or that loading or saving the Stack failed. It clears on the next keypress.
_Avoid_: Error, toast, notification
