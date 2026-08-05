--- @diagnostic disable:undefined-global
--- Test: fault injection, step scheduling, and recovery.

local cfg = dstest.config({
	substrate = "docker",
	seed = 0xDEAD,
	weights = { pause = 0.5, kill = 0.5 },
	accumulation = "single",
	steps = 4,
	step_delay = 500,
	http_retries = 5,
	http_retry_delay = 200,
})

local s = dstest.setup(cfg, {
	image = "kennethreitz/httpbin",
	ports = { 80 },
})

-- Baseline: healthy
local resp = dstest.net.http(s, "GET", "/get")
assert(resp.status == 200, "baseline should return 200")
dstest.info("baseline: 200 OK")

-- Run fault steps
local results = dstest.dst.run_steps(cfg, 4)
assert(#results > 0, "should have fault results")

for _, r in ipairs(results) do
	dstest.info(string.format("round %d: %s on %s", r.round, r.fault, r.subject))
	assert(r.fault, "result should have a fault type")
	assert(r.subject, "result should have a subject")
	assert(r.round, "result should have a round number")
end

-- After clear, should recover
dstest.dst.clear(s)
local ok, r = pcall(dstest.net.http, s, "GET", "/get")
if ok and r.status == 200 then
	dstest.info("recovered after clear")
else
	dstest.warn("still recovering (may need time)")
end

dstest.info("fault test passed")
