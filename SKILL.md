---
name: dstest
description: Deterministic simulation testing for containerized services. Write Lua scripts to inject chaos (pause, kill, resource deprivation) into Docker containers with reproducible, seeded fault injection. Use when writing chaos experiments, testing service resilience, or debugging distributed systems.
license: MIT
metadata:
  author: bxrne
  version: "0.2.0"
---

# dstest

dstest is a deterministic chaos testing framework for Docker containers. Write Lua scripts that inject faults and verify system resilience.

## Quick Start

```bash
# Run an example
cat examples/httpbin.lua | cargo run

# Build and install
cargo build --release
cargo install --path .
```

## Key Commands

| Command | Purpose |
|---------|---------|
| `cat examples/httpbin.lua \| cargo run` | Run a script via stdin |
| `dstest < script.lua` | Run script (after install) |
| `cargo test` | Run test suite |
| `cargo clippy -- -D warnings` | Lint gate (CI-enforced) |
| `cargo doc --open` | Open API docs |

## Available Faults

| Fault | Effect |
|-------|--------|
| `pause` | Freeze container (cgroups) |
| `kill` | Kill container (SIGKILL) |
| `deprive:disk` | Throttle disk I/O to 1MB/s |
| `deprive:network` | Disconnect from bridge network |
| `deprive:memory` | Halve memory limit (min 64MB) |
| `deprive:cpu` | Limit CPU to 20% quota |

## Configuration

Call `dstest.config()` first — it returns a **handle** — then pass that handle
as the first argument to `dstest.setup()`. Full field reference:
[`src/bindings/core/README.md`](src/bindings/core/README.md).

```lua
local docker_config = dstest.config({
    substrate = "docker",      -- Required: must match the engine's compiled substrate
    seed = 42,                 -- Required: random seed for determinism
    weights = {                -- Optional: fault weights (defaults below)
        pause = 0.35,
        kill = 0.25,
        ["deprive:disk"] = 0.10,
        ["deprive:network"] = 0.10,
        ["deprive:memory"] = 0.10,
        ["deprive:cpu"] = 0.10,
    },
    accumulation = "single",   -- "single" (default) or "accumulate"
    steps = 10,                -- Total fault steps in the schedule
    http_timeout = 5,          -- HTTP timeout in seconds
    http_retries = 30,         -- HTTP retry attempts
    http_retry_delay = 500,    -- Delay between retries (ms)
    step_delay = 1000,         -- Delay before fault in single mode (ms)
})
```

## Core API

```lua
local s = dstest.setup(docker_config, {
    image = "kennethreitz/httpbin",
    ports = { 80 },
    volumes = { "/absolute/host/path:/container:ro" },
    env = { DEBUG = "true" },
    cmd = { "python", "-m", "httpbin" },
})

-- Fault injection (namespaced under dstest.dst)
local result = dstest.dst.step()        -- Single fault (or step(cfg) with multiple configs)
local results = dstest.dst.run_steps(5) -- Multiple faults (or run_steps(cfg, 5))
dstest.dst.clear(s)                     -- Clear active faults

-- HTTP and TCP (namespaced under dstest.net)
local resp = dstest.net.http(s, "GET", "/get")

-- Container introspection (flat on dstest)
local info = dstest.inspect(s)
local logs = dstest.logs(s, { tail = "50" })
local exec_result = dstest.exec(s, {"ls", "-la", "/app"})
```

For the full API reference, see the per-module READMEs linked from
[`src/bindings/README.md`](src/bindings/README.md).

## Oracle (Automated Verification)

```lua
dstest.dst.oracle.predicate("health_check", function(subject, fault, round)
    if fault == "pause" or fault == "kill" then return true end
    local ok, resp = pcall(dstest.net.http, subject, "GET", "/health")
    return ok and resp.status == 200
end)

local report = dstest.dst.oracle.run(function()
    dstest.dst.run_steps(10)
end)

print(report.passed, report.passed_checks, report.failed_checks)
```

Full oracle reference: [`src/bindings/dst/README.md`](src/bindings/dst/README.md).

## Common Patterns

### Health Check Loop
```lua
while true do
    local result = dstest.dst.step()
    if not result.more then break end

    if result.fault ~= "pause" and result.fault ~= "kill" then
        local ok, resp = pcall(dstest.net.http, s, "GET", "/get")
        if ok and resp.status == 200 then
            dstest.info("healthy")
        else
            dstest.warn("unhealthy")
        end
    end
end
```

### Multi-Service Testing
```lua
local backend = dstest.setup(docker_config, { image = "myapp/backend", ports = { 8080 } })
local cache = dstest.setup(docker_config, { image = "redis", ports = { 6379 } })

dstest.dst.run_steps(10)
dstest.dst.clear(backend)
dstest.dst.clear(cache)
```

## Determinism

Same seed = identical fault sequence. Register two configs with the same seed
and both produce the same schedule for their subjects:

```lua
local cfg1 = dstest.config({ substrate = "docker", seed = 42 })
local s1 = dstest.setup(cfg1, { image = "kennethreitz/httpbin", ports = { 80 } })
local r1 = dstest.dst.run_steps(cfg1, 5)
-- re-running the whole script with seed 42 yields the identical schedule
```

Oracle failures make the process exit with code `2` (script errors `1`,
infra errors `3`) — CI fails without explicit `error()` calls.

## Logging

```lua
dstest.debug("verbose details")
dstest.info("normal progress")
dstest.warn("something concerning")
dstest.error("failure occurred")
```

## Examples

| File | Demonstrates |
|------|--------------|
| [`examples/httpbin.lua`](examples/httpbin.lua) | HTTP analysis: GET/POST, status/body assertions, latency |
| [`examples/pg.lua`](examples/pg.lua) | PostgreSQL: connect, create table, insert, query, close |
| [`examples/oracle.lua`](examples/oracle.lua) | Fault injection with oracle predicates and invariants |

## Writing Scripts

Scripts are Lua and read from stdin. Use `pcall` for error handling since HTTP may
fail during faults:

```lua
local ok, resp = pcall(dstest.net.http, s, "GET", "/get")
if not ok then
    dstest.warn("request failed: " .. tostring(resp))
end
```

## Requirements

- Docker daemon running
- Rust 1.85+ (uses 2024 edition)
