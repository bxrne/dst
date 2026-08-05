# dstest

[![CI](https://github.com/bxrne/dstest/actions/workflows/ci.yml/badge.svg)](https://github.com/bxrne/dstest/actions/workflows/ci.yml)
[![Release](https://github.com/bxrne/dstest/actions/workflows/release.yml/badge.svg)](https://github.com/bxrne/dstest/actions/workflows/release.yml)
![Tag](https://img.shields.io/github/v/tag/bxrne/dstest?include_prereleases&sort=semver&style=flat)

Deterministic Simulation Testing for containerised services.

Write Lua scripts to define, control, and verify chaos experiments on Docker containers with reproducible fault injection.

## Installation

From crates.io:

```bash
cargo install dstest

# Run a script file
dstest < examples/httpbin.lua

# Or use REPL mode: type your script interactively, press Ctrl+D to execute
dstest
```

Or build from source:

```bash
git clone https://github.com/bxrne/dstest
cd dstest
cargo build --release
```

## Quick Start

```bash
# Run a script file
cat examples/httpbin.lua | cargo run

# Or use REPL mode: type your script and press Ctrl+D to run it
cargo run
```

## Overview

dstest lets you write Lua scripts that define test subjects (Docker containers), inject faults (pause, kill, resource deprivation), and verify service resilience. All experiments are deterministic when seeded, making them reproducible across runs.

## Examples

- `httpbin.lua` - HTTP analysis: status/body assertions, latency timing, fault recovery
- `pg.lua` - PostgreSQL lifecycle: connect, create table, insert, query, close
- `oracle.lua` - Fault injection with oracle predicates and invariants
- `clock.lua` - Virtual clock injection: pin, advance, verify

## Documentation

See [DOCS.md](DOCS.md) for the full Lua API reference.

## AI Assistant Support

This repo includes an AI skill (`SKILL.md`) that teaches assistants how to work with dstest.

**To use with your agent:**

```bash
# Claude Code / Opencode
cp SKILL.md ~/.config/opencode/skills/dstest/SKILL.md

# Other agents (e.g., ~/.agents/skills/)
mkdir -p ~/.agents/skills/dstest
cp SKILL.md ~/.agents/skills/dstest/SKILL.md
```

Then instruct your assistant to "use the dstest skill" when writing or debugging chaos experiments.

## Requirements

- Docker daemon running
- Rust 1.85+ (uses 2024 edition)

## License

MIT
