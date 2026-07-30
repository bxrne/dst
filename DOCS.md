# dstest Documentation

## Lua API

### Core Functions

#### `dstest.config(options)`
Configures the experiment. Must be called before `dstest.setup()` and `dstest.step()`. The `substrate` and `seed` fields are required.

```lua
dstest.config({
    substrate = "docker",
    seed = 42,
    weights = { pause = 0.5, kill = 0.3, ["deprive:disk"] = 0.2 },
    accumulation = "single",
    http_timeout = 10,
})
```

**Configuration options:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `substrate` | string | Yes | - | Substrate type (currently only `"docker"`) |
| `seed` | number | Yes | - | Random seed for deterministic fault selection |
| `weights` | table | No | See below | Fault type weights for weighted random selection |
| `accumulation` | string | No | `"single"` | Fault accumulation mode (`"single"` or `"accumulate"`) |
| `http_timeout` | number | No | `5` | HTTP request timeout in seconds |
| `http_retries` | number | No | `30` | HTTP retry attempts |
| `http_retry_delay` | number | No | `500` | Delay between HTTP retries (ms) |
| `step_delay` | number | No | `1000` | Delay before applying fault in single mode (ms) |
| `require_seed` | boolean | No | `true` | Require seed in config |

#### `dstest.setup(config)`
Creates a test subject. Returns a subject ID string in the format `docker/<container-id>`. Uses the substrate type configured in `dstest.config()`.

```lua
local subject = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
    volumes = { "/host/path:/container/path:ro" },
    env = { DEBUG = "true", LOG_LEVEL = "info" },
    cmd = { "python", "-m", "httpbin" },
})
```

**Configuration options:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image` | string | Yes | Docker image to pull and run |
| `ports` | table | No | Array of port numbers to expose |
| `volumes` | table | No | Array of bind mount specifications (`host:container[:options]`). Host path must be absolute. |
| `env` | table | No | Key-value table of environment variables |
| `cmd` | table | No | Array of command arguments to override container entrypoint |

#### `dstest.http(subject, method, path)`
Makes an HTTP request to the subject. Returns a table with `status` (number) and `body` (string).

```lua
local resp = dstest.http(subject, "GET", "/get")
if resp.status == 200 then
    dstest.info("request successful")
end
```

#### `dstest.logs(subject, options?)`
Fetches container logs. Returns an array of log entries.

```lua
local logs = dstest.logs(subject, { tail = "50", timestamps = true })
for _, entry in ipairs(logs) do
    if entry.stream == "stderr" then
        dstest.warn(entry.message)
    end
end
```

**Options:**

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `stdout` | boolean | `true` | Include stdout stream |
| `stderr` | boolean | `true` | Include stderr stream |
| `tail` | string | `"all"` | Number of lines (`"50"`, `"all"`) |
| `since` | number | - | UNIX timestamp (seconds) |
| `timestamps` | boolean | `false` | Include timestamps |

#### `dstest.inspect(subject)`
Returns container metadata and state.

```lua
local info = dstest.inspect(subject)
print(info.state, info.pid, info.ip)
assert(info.state == "running", "container should be running")
```

Return fields:
- `state` (string): `"running"`, `"paused"`, `"exited"`, or `"dead"`
- `pid` (number, optional): Process ID
- `ip` (string, optional): Container IP address
- `memory_limit` (number, optional): Memory limit in bytes
- `cpu_quota` (number, optional): CPU quota (0.0-1.0)

#### `dstest.exec(subject, command)`
Runs a command inside the container and returns the result.

```lua
local result = dstest.exec(subject, {"ls", "-la", "/app"})
if result.exit_code == 0 then
    dstest.info("files:\n" .. result.stdout)
else
    dstest.error("exec failed: " .. result.stderr)
end
```

Return fields:
- `exit_code` (number): Command exit code (-1 if unknown)
- `stdout` (string): Standard output
- `stderr` (string): Standard error

#### `dstest.clock()`
Returns high-precision timestamp. Useful for measuring latency.

```lua
local start = dstest.clock()
-- ... do work ...
local elapsed_nanos = dstest.clock().nanos - start.nanos
print(string.format("elapsed: %.2fms", elapsed_nanos / 1e6))
```

Return fields:
- `nanos` (number): Nanoseconds since UNIX epoch
- `micros` (number): Microseconds since UNIX epoch
- `millis` (number): Milliseconds since UNIX epoch
- `secs` (number): Seconds since UNIX epoch

#### `dstest.tcp(subject, port)`
Tests TCP connectivity to a port on the subject.

```lua
-- Test database connectivity
local conn = dstest.tcp(subject, 5432)
if conn.connected then
    dstest.info("PostgreSQL reachable at " .. conn.addr)
else
    dstest.warn("PostgreSQL unreachable: " .. conn.error)
end
```

Return fields:
- `connected` (boolean): Whether connection succeeded
- `addr` (string): Address tested (`host:port`)
- `error` (string, optional): Error message if connection failed

### Fault Injection

#### `dstest.step()`
Applies the next fault in the sequence. Returns a table describing what happened, or `{more = false}` when complete.

```lua
while true do
    local result = dstest.step()
    if not result.more then
        break
    end
    -- Verify service still works
    local resp = dstest.http(subject, "GET", "/get")
    dstest.info("status=" .. resp.status .. " after fault=" .. result.fault)
end
```

Return table fields:
- `fault` (string): The fault type applied
- `subject` (string): The subject ID (format: `docker/<container-id>`)
- `round` (number): Current round number (1-indexed)
- `total_rounds` (number): Total rounds in this experiment
- `remaining` (number): Steps remaining
- `more` (boolean): Whether more steps are available

#### `dstest.run_steps(n)`
Runs multiple steps and returns an array of results.

```lua
local results = dstest.run_steps(5)
for i, result in ipairs(results) do
    dstest.info("round " .. result.round .. ": " .. result.fault)
end
```

#### `dstest.clear(subject)`
Clears all active faults on a subject.

```lua
dstest.clear(subject)
```

### Logging

- `dstest.debug(msg)` - Debug-level message
- `dstest.info(msg)` - Info-level message
- `dstest.warn(msg)` - Warning-level message
- `dstest.error(msg)` - Error-level message

### Oracle

The oracle mechanism provides automated verification of system properties during fault injection experiments.

#### `dstest.oracle.predicate(name, fn)`
Registers a health predicate that is checked after each fault. The function receives `(subject, fault, round)` and returns `true` or `{passed, message?}`.

```lua
dstest.oracle.predicate("http_health", function(subject, fault, round)
    local resp = dstest.http(subject, "GET", "/health")
    if resp.status ~= 200 then
        return {false, "health check failed: " .. resp.status}
    end
    return true
end)
```

#### `dstest.oracle.invariant(name, fn)`
Registers an invariant that is checked continuously throughout the experiment. Returns `true` or `{passed, message?}`.

```lua
dstest.oracle.invariant("response_time", function()
    local start = os.clock()
    dstest.http(s, "GET", "/get")
    local elapsed = os.clock() - start
    if elapsed > 1.0 then
        return {false, "response time exceeded 1s: " .. elapsed}
    end
    return true
end)
```

#### `dstest.oracle.run(fn)`
Runs a function with the oracle enabled, returning a report. This automatically enables oracle checking during `step()` and `run_steps()`.

```lua
local report = dstest.oracle.run(function()
    dstest.run_steps(10)
end)

if not report.passed then
    for _, failure in ipairs(report.failures) do
        dstest.warn(failure.type .. " '" .. failure.name .. "' failed: " .. failure.error)
    end
    error("oracle verification failed")
end
```

Report fields:
- `passed` (boolean): Overall pass/fail status
- `total_checks` (number): Total number of checks performed
- `passed_checks` (number): Number of passed checks
- `failed_checks` (number): Number of failed checks
- `failures` (array): List of failure records

Failure record fields:
- `type` (string): Either `"predicate"` or `"invariant"`
- `name` (string): The name of the check that failed
- `round` (number, optional): The round number (predicates only)
- `fault` (string, optional): The fault type applied (predicates only)
- `subject` (string, optional): The subject ID (predicates only)
- `error` (string): The error message

#### `dstest.oracle.enable()`
Enables oracle checking and resets the report. Oracle predicates and invariants will run automatically after each `step()` call.

#### `dstest.oracle.disable()`
Disables oracle checking.

#### `dstest.oracle.report()`
Returns the current oracle report without modifying state.

#### `dstest.oracle.reset()`
Resets the oracle report, clearing all recorded results.

## Default Weights

| Fault | Weight |
|-------|--------|
| `pause` | 0.35 |
| `kill` | 0.25 |
| `deprive:disk` | 0.10 |
| `deprive:network` | 0.10 |
| `deprive:memory` | 0.10 |
| `deprive:cpu` | 0.10 |

### Accumulation Modes

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

- `basic.lua` - Minimal setup with HTTP checks
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
