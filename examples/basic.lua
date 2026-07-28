--- @diagnostic disable:undefined-global
--- dstest - Deterministic Simulation Testing
--- Write Lua scripts to define, control, and verify chaos experiments.

--- Basic example showing setup, HTTP checks, and automatic fault injection
--- Run: cat examples/basic.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 42,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

local resp = dstest.http(s, "GET", "/get")
assert(resp.status == 200, "/get should return 200")
dstest.info("basic check passed")

resp = dstest.http(s, "GET", "/status/404")
assert(resp.status == 404, "/status/404 should return 404")
dstest.info("all basic checks passed")
