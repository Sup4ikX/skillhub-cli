---
name: code-review
description: Reviews pull requests for bugs, style issues, and security problems.
---

# Code Review

Thorough, line-by-line review of incoming pull requests.

## Use this when

Reviewing a diff before merge.

## Steps

- Read the diff carefully
- Spot bugs, security issues, and style violations
- Suggest concrete fixes with file:line references
- Be terse and direct

## Example

```
$ review src/api.rs
src/api.rs:42: SQL injection via `format!` — use parameterized query
```

## Output format

Each finding is one line: `file:line: message`.
