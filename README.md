# gtd-cli

> **Paper captures the moment. The terminal organizes the life.**

A reference implementation for building an agent-based GTD (Getting Things Done) system. Not a turnkey app — a pattern library you can study, fork, and adapt.

**[View the interactive explainer →](https://kevinutecht.github.io/gtd-cli/)**

Built on two engines: a waterproof weekly sheet for daily capture, and a plain-markdown CLI for structured review across five horizons of commitment.

## Quick Links

- [Interactive Overview](https://kevinutecht.github.io/gtd-cli/) — visual explainer of the full system
- [Examples](examples/sample-data/) — sample horizon files (purpose, vision, goals, areas, projects)
- [Skills](https://github.com/kevinutecht/pi-skill-accountability-partner) — MiMo Code integration

## What You Can Learn

- **Data modeling** — model GTD concepts as plain markdown files
- **TUI architecture** — crossterm-based weekly review with vim keybindings
- **Agent integration** — spawning MiMo Code skills from a TUI for AI-powered review
- **Weekly board format** — scored accountability with structured feedback
- **Zero lock-in** — your data lives in `~/data/gtd/` as `.md` files you own forever

## Build It

```bash
git clone https://github.com/kevinutecht/gtd-cli.git
cd gtd-cli
cargo build --release
cp target/release/gtd ~/bin/gtd
```

## License

MIT
