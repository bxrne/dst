# dstest

Deterministic Simulation Testing for containerised services.

Write Lua scripts to define, control, and verify chaos experiments on Docker containers with reproducible fault injection.

## Installation

From crates.io:

```bash
cargo install dstest
```

Or build from source:

```bash
git clone https://github.com/bxrne/dstest
cd dstest
cargo build --release
```

## Quick Start

```bash
cat examples/basic.lua | cargo run
```

## Overview

dstest lets you write Lua scripts that define test subjects (Docker containers), inject faults (pause, kill, resource deprivation), and verify service resilience. All experiments are deterministic when seeded, making them reproducible across runs.

## Examples

- `basic.lua` - Minimal setup with HTTP checks
- `coroutine.lua` - User-controlled fault injection with coroutines
- `response-time.lua` - Response time validation with oracle predicates

## Documentation

See [DOCS.md](DOCS.md) for the full Lua API reference.

## Requirements

- Docker daemon running
- Rust 1.85+ (uses 2024 edition)

## License

MIT
