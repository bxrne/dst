--- @diagnostic disable:undefined-global
--- High-precision timing: measure latency during chaos
--- Run: cat examples/timing.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 321,
    weights = {
        ["deprive:cpu"] = 0.5,
        ["deprive:network"] = 0.5,
    },
    step_delay = 100,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- Baseline timing
local start = dstest.clock()
local ok, resp = pcall(dstest.http, s, "GET", "/get")
local elapsed = dstest.clock().nanos - start.nanos

if ok and resp.status == 200 then
    dstest.info(string.format("Baseline latency: %.2fms", elapsed / 1e6))
else
    dstest.warn("Baseline request failed")
end

-- Run with fault injection
local results = dstest.run_steps(5)

for i, r in ipairs(results) do
    dstest.info(string.format("Round %d: %s", r.round, r.fault))
end

-- Measure latency after faults
dstest.clear(s)
start = dstest.clock()
ok, resp = pcall(dstest.http, s, "GET", "/get")
elapsed = dstest.clock().nanos - start.nanos

dstest.info(string.format("Post-fault latency: %.2fms", elapsed / 1e6))

dstest.clear(s)
dstest.info("timing example complete")
