--- @diagnostic disable:undefined-global
--- dstest - Deterministic Simulation Testing
--- Basic example: minimal setup, HTTP checks, automatic fault injection

dstest.config({
    substrate = "docker",
    seed = 42,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

local resp = dstest.http(s, "GET", "/get")
assert(resp.status == 200, "expected 200, got " .. resp.status)
dstest.info("basic check passed")

resp = dstest.http(s, "GET", "/status/404")
assert(resp.status == 404, "expected 404, got " .. resp.status)
dstest.info("all checks passed")
