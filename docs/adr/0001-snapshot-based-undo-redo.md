# Snapshot-based undo/redo

## Context

Every change to the Stack must be reversible with `undo`, and many Operations are lossy (`clear`, `drop`, `%`, `!`, `abs`), so they have no inverse to apply.

## Decision

History stores Snapshots of the whole Stack. Every change pushes a Snapshot onto the undo list and clears the redo list; `undo` and `redo` swap the current Stack with the latest Snapshot. History is unbounded.

## Consequences

- Adding an Operation never needs undo logic, and `clear` and `drop` are undoable like everything else.
- Each step costs memory proportional to the Stack's size. For a calculator Stack that's negligible, so there's no limit.
- A failed Operation records nothing, so `undo` never steps through no-ops.
