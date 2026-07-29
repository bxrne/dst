---
name: dstest
description: Deterministic simulation testing for containerized services. Write Lua scripts to inject chaos (pause, kill, resource deprivation) into Docker containers with reproducible, seeded fault injection. Use when writing chaos experiments, testing service resilience, or debugging distributed systems.
license: MIT
metadata:
  author: bxrne
  version: "0.1.0"
---

# dstest

dstest is a deterministic chaos testing framework for Docker containers. Write Lua scripts that inject faults and verify system resilience.

## Quick Start

```bash
# Run an example
cat examples/basic.lua | cargo run

# Build and install
cargo build --release
cargo install --path .
```

## Key Commands

| Command | Purpose |
|---------|---------|
| `cargo doc --open` | Open API documentation |
| `cat examples/basic.lua \| cargo run` | Run a script via stdin |
| `dstest < script.lua` | Run script (after install) |
| `cargo test` | Run test suite |
| `cargo clippy -- -D warnings` | Lint check |

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

Call `dstest.config()` first to set substrate and seed. Then `dstest.setup()` uses those settings.

```lua
dstest.config({
    substrate = "docker",      -- Required: only "docker" supported
    seed = 42,                 -- Required: random seed for determinism
    weights = {                -- Optional: fault weights (default below)
        pause = 0.35,
        kill = 0.25,
        ["deprive:disk"] = 0.10,
        ["deprive:network"] = 0.10,
        ["deprive:memory"] = 0.10,
        ["deprive:cpu"] = 0.10,
    },
    accumulation = "single",   -- "single" (default) or "accumulate"
    http_timeout = 5,          -- HTTP timeout in seconds
    http_retries = 30,         -- HTTP retry attempts
    http_retry_delay = 500,    -- Delay between retries (ms)
    step_delay = 1000,         -- Delay before fault (ms)
})
```

## Core API

```lua
-- Create a subject (container)
-- Substrate type comes from dstest.config({ substrate = "docker" })
local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
    volumes = { "/absolute/host/path:/container:ro" },  -- must be absolute
    env = { DEBUG = "true" },
    cmd = { "python", "-m", "httpbin" },
})

-- Inject faults
local result = dstest.step()           -- Single fault
local results = dstest.run_steps(5)    -- Multiple faults

-- HTTP requests
local resp = dstest.http(s, "GET", "/get")
print(resp.status, resp.body)

-- Clear faults
dstest.clear(s)
```

## Oracle (Automated Verification)

```lua
dstest.oracle.predicate("health_check", function(subject, fault, round)
    if fault == "pause" or fault == "kill" then
        return true  -- Skip during these faults
    end
    local resp = dstest.http(subject, "GET", "/health")
    return resp.status == 200
end)

local report = dstest.oracle.run(function()
    dstest.run_steps(10)
end)

print(report.passed, report.passed_checks, report.failed_checks)
```

## Common Patterns

### Health Check Loop
```lua
while true do
    local result = dstest.step()
    if not result.more then break end
    
    if result.fault ~= "pause" and result.fault ~= "kill" then
        local ok, resp = pcall(dstest.http, s, "GET", "/get")
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
local backend = dstest.setup({ image = "myapp/backend", ports = { 8080 } })
local cache = dstest.setup({ image = "redis", ports = { 6379 } })

dstest.run_steps(10)
dstest.clear(backend)
dstest.clear(cache)
```

## Determinism

Same seed = identical fault sequence:

```lua
dstest.config({ seed = 42 })
local r1 = dstest.run_steps(5)

dstest.config({ seed = 42 })
local r2 = dstest.run_steps(5)

-- r1 and r2 have identical faults in identical order
```

## Logging

```lua
dstest.debug("verbose details")
dstest.info("normal progress")
dstest.warn("something concerning")
dstest.error("failure occurred")
```

## Requirements

- Docker daemon running
- Rust 1.85+ (uses 2024 edition)

## Examples in Repo

| File | Demonstrates |
|------|--------------|
| `basic.lua` | Minimal HTTP checks |
| `oracle.lua` | Predicate verification |
| `response-time.lua` | Latency validation |
| `multi-service.lua` | Multiple containers |
| `fault-accumulation.lua` | Stacking faults |
| `http-assertions.lua` | Status/body checks |
| `parameter-sweep.lua` | Multiple seeds |

## Writing Scripts

Scripts are Lua and read from stdin. Use `pcall` for error handling since HTTP may fail during faults:

```lua
local ok, resp = pcall(dstest.http, s, "GET", "/get")
if not ok then
    dstest.warn("request failed: " .. tostring(resp))
end
```

## Debugging

1. Check Docker is running: `docker ps`
2. View container logs: `docker logs <container-id>`
3. Run with verbose logging: scripts use `dstest.debug()` for details
4. Check examples directory for working patterns
