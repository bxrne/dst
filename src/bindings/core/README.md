# core

Experiment configuration and subject setup. Exposes `dstest.config` and `dstest.setup`
(flat on the `dstest` table).

## `dstest.config(options)`

Configures the experiment. Must be called before `dstest.setup()` and `dstest.dst.step()`.
The `substrate` and `seed` fields are required.

```lua
dstest.config({
    substrate = "docker",
    seed = 42,
    weights = { pause = 0.5, kill = 0.3, ["deprive:disk"] = 0.2 },
    accumulation = "single",
    http_timeout = 10,
})
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `substrate` | string | Yes | - | Substrate name; must match the engine's compiled substrate (`"docker"`) |
| `seed` | number | Yes | - | Random seed for deterministic fault selection |
| `weights` | table | No | [default weights](../../../DOCS.md#default-weights) | Fault-type weights for weighted random selection |
| `accumulation` | string | No | `"single"` | `"single"` (clear before each fault) or `"accumulate"` (stack) |
| `http_timeout` | number | No | `5` | HTTP request timeout in seconds |
| `http_retries` | number | No | `30` | HTTP retry attempts |
| `http_retry_delay` | number | No | `500` | Delay between HTTP retries (ms) |
| `step_delay` | number | No | `1000` | Delay before applying fault in single mode (ms) |
| `require_seed` | boolean | No | `true` | Require seed before `step()`/`run_steps()` |

## `dstest.setup(config)`

Creates a test subject (container). Returns a subject ID string `"<substrate>/<id>"`
(e.g. `"docker/abc123"`). The substrate type comes from `dstest.config()`.

```lua
local subject = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
    volumes = { "/host/path:/container/path:ro" },
    env = { DEBUG = "true", LOG_LEVEL = "info" },
    cmd = { "python", "-m", "httpbin" },
})
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image` | string | Yes | Container image to pull and run |
| `ports` | table | No | Array of port numbers to expose (first is used for `http`/`tcp` host mapping) |
| `volumes` | table | No | Array of bind mounts (`host:container[:options]`). Host path must be absolute. |
| `env` | table | No | Key-value table of environment variables |
| `cmd` | table | No | Array of command arguments overriding the entrypoint |

### Signature quirk

`setup` accepts either `(table)` or `(string, table)` where the string overrides
the substrate name for this subject. In practice the substrate is fixed at compile
time, so the string form is validated against the engine's substrate and errors on
mismatch.
