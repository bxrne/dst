# dstest

[![CI](https://github.com/bxrne/dstest/actions/workflows/ci.yml/badge.svg)](https://github.com/bxrne/dstest/actions/workflows/ci.yml)
[![Release](https://github.com/bxrne/dstest/actions/workflows/release.yml/badge.svg)](https://github.com/bxrne/dstest/actions/workflows/release.yml)

Deterministic Simulation Testing for containerised services.

Write Lua scripts to define, control, and verify chaos experiments on Docker containers with reproducible fault injection.

## Installation

From crates.io:

```bash
cargo install dstest

# Run a script file
dstest < examples/basic.lua

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
cat examples/basic.lua | cargo run

# Or use REPL mode: type your script and press Ctrl+D to run it
cargo run
```

## Overview

dstest lets you write Lua scripts that define test subjects (Docker containers), inject faults (pause, kill, resource deprivation), and verify service resilience. All experiments are deterministic when seeded, making them reproducible across runs.

## Examples

- `basic.lua` - Minimal HTTP checks
- `oracle.lua` - Oracle predicates for automated verification
- `response-time.lua` - Response time validation
- `multi-service.lua` - Fault injection across multiple containers
- `fault-accumulation.lua` - Stacking faults without clearing
- `http-assertions.lua` - Custom HTTP status/body assertions
- `parameter-sweep.lua` - Running experiments with different seeds
- `logs.lua` - Fetching and analyzing container logs
- `inspect.lua` - Container state verification after faults
- `exec.lua` - Running commands inside containers
- `timing.lua` - High-precision latency measurements
- `tcp.lua` - Testing non-HTTP service connectivity

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
