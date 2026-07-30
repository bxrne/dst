# dst

Fault injection and the oracle verification system. Exposes `dstest.step`,
`dstest.run_steps`, `dstest.clear`, and `dstest.oracle.*`.

## `dstest.step()`

Applies the next fault in the deterministic sequence. Returns a table describing
what happened, or `{ more = false }` when the sequence is exhausted.

```lua
local result = dstest.step()
-- result = { fault = "pause", subject = "docker/abc123", round = 1,
--            total_rounds = 10, remaining = 9, more = true }
```

| Field | Type | Description |
|-------|------|-------------|
| `fault` | string | Fault type applied (`"pause"`, `"kill"`, `"deprive:disk"`, etc.) |
| `subject` | string | Subject ID (`"<substrate>/<id>"`) |
| `round` | number | Current round (1-indexed) |
| `total_rounds` | number | Total rounds in this experiment |
| `remaining` | number | Steps remaining |
| `more` | boolean | Whether more steps are available |
| `oracle` | table | Present only when oracle is enabled (see below) |

In `single` accumulation mode, previous faults are cleared (with `step_delay` ms
wait) before applying the next. In `accumulate` mode, faults stack.

When the oracle is enabled, `step` runs all predicates after applying the fault
and attaches an `oracle` sub-table: `{ passed, total_checks, passed_checks, failed_checks }`.

## `dstest.run_steps(n)`

Runs `n` steps and returns an array of result tables (same shape as `step`).
Stops early if the sequence is exhausted.

```lua
local results = dstest.run_steps(5)
for _, r in ipairs(results) do
    dstest.info("round " .. r.round .. ": " .. r.fault)
end
```

## `dstest.clear(subject)`

Clears all active faults on a subject.

```lua
dstest.clear(subject)
```

## Oracle

Automated verification of system properties during fault injection. Predicates are
checked after each `step`/`run_steps` round; invariants are checked continuously.

### `dstest.oracle.predicate(name, fn)`

Registers a health predicate checked after each fault. The function receives
`(subject, fault, round)` and returns `true` or `{ passed, message? }`.

```lua
dstest.oracle.predicate("http_health", function(subject, fault, round)
    if fault == "pause" or fault == "kill" then return true end
    local ok, resp = pcall(dstest.http, subject, "GET", "/health")
    if not ok or resp.status ~= 200 then
        return { false, "health check failed" }
    end
    return true
end)
```

### `dstest.oracle.invariant(name, fn)`

Registers an invariant checked throughout the experiment. Returns `true` or
`{ passed, message? }`.

### `dstest.oracle.run(fn)`

Runs a function with the oracle enabled, returns a report. Automatically enables
checking during `step()`/`run_steps()` and disables it after.

```lua
local report = dstest.oracle.run(function()
    dstest.run_steps(10)
end)

if not report.passed then
    for _, f in ipairs(report.failures) do
        dstest.warn(f.type .. " '" .. f.name .. "': " .. f.error)
    end
    error("oracle verification failed")
end
```

### `dstest.oracle.enable()` / `disable()` / `reset()`

Manual control of oracle checking state. `enable` resets the report. `reset`
clears recorded results without changing the enabled flag.

### `dstest.oracle.report()`

Returns the current report without modifying state.

### Report shape

| Field | Type | Description |
|-------|------|-------------|
| `passed` | boolean | Overall pass/fail |
| `total_checks` | number | Total checks performed |
| `passed_checks` | number | Checks that passed |
| `failed_checks` | number | Checks that failed |
| `failures` | array | List of failure records |

### Failure record

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | `"predicate"` or `"invariant"` |
| `name` | string | Name of the failed check |
| `round` | number | Round number (predicates only) |
| `fault` | string | Fault applied (predicates only) |
| `subject` | string | Subject ID (predicates only) |
| `error` | string | Error message |
