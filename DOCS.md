# dstest Documentation

## Usage

dstest reads a Lua script from stdin. Pipe in a script file or type interactively.

```bash
# Run a script file
dstest < examples/httpbin.lua
cat examples/httpbin.lua | dstest

# REPL mode: run dstest with no redirect, type your script, press Ctrl+D to execute
dstest
```

In REPL mode, dstest reads until EOF (Ctrl+D), then executes the entire script.
Logging output is printed to stderr as the script runs.

## Lua API

The global `dstest` table is the entry point. Function reference lives in the
per-module READMEs under [`src/bindings/`](src/bindings/README.md):

| Namespace | Functions | Reference |
|-----------|-----------|-----------|
| `dstest.config`, `dstest.setup` | experiment config, create subjects | [`src/bindings/core/README.md`](src/bindings/core/README.md) |
| `dstest.step`, `dstest.run_steps`, `dstest.clear` | fault injection | [`src/bindings/dst/README.md`](src/bindings/dst/README.md) |
| `dstest.oracle.*` | predicates, invariants, reports | [`src/bindings/dst/README.md`](src/bindings/dst/README.md) |
| `dstest.http`, `dstest.tcp` | HTTP requests, raw TCP | [`src/bindings/net/README.md`](src/bindings/net/README.md) |
| `dstest.inspect`, `dstest.logs`, `dstest.exec` | container introspection | [`src/bindings/subs/README.md`](src/bindings/subs/README.md) |
| `dstest.pg.connect`, `dstest.pg.query`, `dstest.pg.close` | PostgreSQL | [`src/bindings/pg/README.md`](src/bindings/pg/README.md) |
| `dstest.clock` | high-precision timestamps | [`src/bindings/clock.rs`](src/bindings/clock.rs) |
| `dstest.debug`, `dstest.info`, `dstest.warn`, `dstest.error` | logging | [`src/bindings/log.rs`](src/bindings/log.rs) |

## Default Weights

| Fault | Weight |
|-------|--------|
| `pause` | 0.35 |
| `kill` | 0.25 |
| `deprive:disk` | 0.10 |
| `deprive:network` | 0.10 |
| `deprive:memory` | 0.10 |
| `deprive:cpu` | 0.10 |

## Accumulation Modes

- `single`: Each subject can have only one active fault. Previous faults are cleared before applying new ones.
- `accumulate`: Multiple faults can stack on the same subject.

## Fault Types

| Fault | Effect |
|-------|--------|
| `pause` | Freezes the container (cgroups freeze) |
| `kill` | Kills the container (SIGKILL) |
| `deprive:disk` | Throttles disk I/O to 1MB/s, 50% blkio weight |
| `deprive:network` | Disconnects from bridge network (no internet) |
| `deprive:memory` | Reduces memory limit to 50% of current (min 64MB) |
| `deprive:cpu` | Limits CPU to 20% quota |

## Determinism

Same seed produces identical fault sequences:

```lua
dstest.config({ substrate = "docker", seed = 42 })
local s = dstest.setup({ image = "kennethreitz/httpbin", ports = { 80 } })
local results1 = dstest.run_steps(10)

dstest.config({ substrate = "docker", seed = 42 })
local s = dstest.setup({ image = "kennethreitz/httpbin", ports = { 80 } })
local results2 = dstest.run_steps(10)

-- results1 and results2 have identical fault sequences
```

## Examples

| File | Demonstrates |
|------|--------------|
| [`examples/httpbin.lua`](examples/httpbin.lua) | HTTP analysis: GET/POST, status/body assertions, latency timing |
| [`examples/pg.lua`](examples/pg.lua) | PostgreSQL lifecycle: connect, create table, insert, query, close |
| [`examples/oracle.lua`](examples/oracle.lua) | Fault injection with oracle predicates and invariants |
