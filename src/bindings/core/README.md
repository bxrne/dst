# core

Experiment configuration and subject setup. Exposes `dstest.config` and `dstest.setup`
(flat on the `dstest` table).

## `dstest.config(options) -> handle`

Registers a named experiment configuration and returns its **handle** (a string).
The handle links every `dstest.setup()` to a config — and through it, to a
substrate — so multiple configs can coexist (e.g. different seeds, weights, or
in future, different substrates).

```lua
local docker_config = dstest.config({
    name = "docker_config",          -- optional; auto-generated if omitted
    substrate = "docker",
    seed = 42,
    weights = { pause = 0.5, kill = 0.3, ["deprive:disk"] = 0.2 },
    accumulation = "single",
    steps = 10,
    http_timeout = 10,
})

local s = dstest.setup(docker_config, { image = "kennethreitz/httpbin", ports = { 80 } })
```

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `name` | string | No | `config_N` | Handle name; must be unique |
| `substrate` | string | Yes | - | Substrate name; must match the engine's compiled substrate (`"docker"`) |
| `seed` | number | Yes | - | Random seed for deterministic fault selection (also seeds Lua's `math.random`) |
| `weights` | table | No | [default weights](../../../DOCS.md#default-weights) | Fault-type weights; normalized to sum to 1.0 |
| `accumulation` | string | No | `"single"` | `"single"` (clear before each fault) or `"accumulate"` (stack) |
| `steps` | number | No | `10` | Total fault steps in this config's schedule |
| `http_timeout` | number | No | `5` | HTTP request timeout in seconds |
| `http_retries` | number | No | `30` | HTTP retry attempts |
| `http_retry_delay` | number | No | `500` | Delay between HTTP retries (ms) |
| `step_delay` | number | No | `1000` | Delay before applying fault in single mode (ms) |
| `require_seed` | boolean | No | `true` | Require seed before `step()`/`run_steps()` |

Each call creates a fresh config from defaults — there is no global mutable
config. Handles are deterministic: unnamed configs are `config_1`, `config_2`, …
in registration order.

## `dstest.setup(config_handle, options)`

Creates a test subject (container) under the given config. Returns a subject ID
string `"<substrate>/<id>"` (e.g. `"docker/abc123"`).

```lua
local subject = dstest.setup(docker_config, {
    image = "kennethreitz/httpbin",
    ports = { 80 },
    volumes = { "/host/path:/container/path:ro" },
    env = { DEBUG = "true", LOG_LEVEL = "info" },
    cmd = { "python", "-m", "httpbin" },
})
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image` | string | Yes | Container image to pull and run (pin by digest for reproducibility) |
| `ports` | table | No | Container ports to expose; host side is **ephemeral** (Docker-assigned), first port's mapping is used for `http`/`tcp` |
| `volumes` | table | No | Array of bind mounts (`host:container[:options]`). Host path must be absolute. |
| `env` | table | No | Key-value table of environment variables |
| `cmd` | table | No | Array of command arguments overriding the entrypoint |

Containers are named `dstest-<config>-<n>` and labelled `dstest.managed=true`;
a name collision with a stale dstest container is cleaned up automatically.

> **Note:** subjects created *after* the first `dstest.dst.step()` for a config
> are not part of that config's fault schedule (the schedule snapshots its
> subject set on first step).
