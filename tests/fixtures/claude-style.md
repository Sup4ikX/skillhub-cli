---
name: doc-writer
description: Auto-generate documentation from source code.
---

# Doc Writer

Reads source files and produces Markdown documentation.

## Use this when

You need a README, API reference, or inline docs.

## Steps

- Scan the source tree
- Extract public types, functions, and modules
- Emit one Markdown section per file
- Cross-link related symbols

## Example

```
$ doc-writer src/
Wrote docs/index.md (12 files documented)
```

## Tips

- Run after every refactor
- Use with `--include-private` for internal docs
