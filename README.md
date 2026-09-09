# gtd-cli

> **Paper captures the moment. The terminal organizes the life.**

A reference implementation for building an agent-based GTD (Getting Things Done) system. Not a turnkey app — a pattern library you can study, fork, and adapt.

**[View the interactive explainer →](https://kevinutecht.github.io/gtd-cli/)**

Built on two engines: a waterproof weekly sheet for daily capture, and a plain-markdown CLI for structured review across five horizons of commitment.

## Quick Links

- [Interactive Overview](https://kevinutecht.github.io/gtd-cli/) — visual explainer of the full system
- [Examples](examples/sample-data/) — sample horizon files (purpose, vision, goals, areas, projects)
- Local AI — built-in `llama-agent` integration for accountability and brainstorming

## What You Can Learn

- **Data modeling** — model GTD concepts as plain markdown files
- **TUI architecture** — crossterm-based weekly review with vim keybindings
- **Agent integration** — running local `llama-agent` prompts from a TUI for AI-powered review
- **Weekly board format** — scored accountability with structured feedback
- **Zero lock-in** — your data lives in `~/data/gtd/` as `.md` files you own forever

## Build It

```bash
git clone https://github.com/kevinutecht/gtd-cli.git
cd gtd-cli
cargo build --release
```

## Local AI review helpers

The review TUI runs `llama-agent` itself for both AI actions: `p` creates a Coach's Call and `g`
rewrites `brainstorm.md`. It constructs a prompt from the appropriate GTD Markdown files, validates
the response, and writes the result locally. Configure `LLAMA_MODEL`, `LLAMA_CLI`, and
`LLAMA_CONTEXT` as needed. `LLAMA_CLI` defaults to `llama-agent` on your `PATH`; `LLAMA_MODEL`
must point to an instruct-model `.gguf` file. The app passes `--single-turn` so chat-template models exit after generating one response rather than
entering `llama-agent`'s interactive `>` prompt. It also runs with reasoning disabled and agent
resources isolated, which makes the local model return the structured response the TUI validates.

For example:

```bash
export LLAMA_CLI="$HOME/tools/llama-agent/build/bin/llama-agent"
export LLAMA_MODEL="$HOME/tools/models/your-model.gguf"
export LLAMA_CONTEXT=16384
```

## License

MIT
