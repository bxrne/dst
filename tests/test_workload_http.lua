--- @diagnostic disable:undefined-global
--- Test: HTTP workload generation against httpbin.

local cfg = dstest.config({
	substrate = "docker",
	seed = 42,
	weights = { pause = 0.5, kill = 0.5 },
	accumulation = "single",
	steps = 2,
})

local s = dstest.setup(cfg, {
	image = "kennethreitz/httpbin",
	ports = { 80 },
})

-- OpenAPI-driven workload from real httpbin spec
local project_dir = os.getenv("PWD") or "."
local openapi_path = project_dir .. "/tests/httpbin.openapi.yaml"

dstest.info("running OpenAPI-driven workload...")
local stats = dstest.workload.http(s, {
	duration_secs = 5,
	rate = 5,
	openapi = openapi_path,
})

assert(stats.total_requests > 0, "should have made requests")
assert(stats.ok > 0, "should have successful requests")
assert(stats.avg_latency_ms > 0, "should have measured latency")
dstest.info(string.format("openapi workload: %d requests, %d ok, %d failed, avg %dms",
	stats.total_requests, stats.ok, stats.failed, stats.avg_latency_ms))

-- Per-method breakdown
assert(stats.breakdown, "should have per-method breakdown")
dstest.info("breakdown:")
for key, bt in pairs(stats.breakdown) do
	dstest.info(string.format("  %s: ok=%d failed=%d", key, bt.ok, bt.failed))
end

-- Run faults during workload
dstest.info("running faults during workload...")
local results = dstest.dst.run_steps(cfg, 2)
for _, r in ipairs(results) do
	dstest.info(string.format("  fault: %s on round %d", r.fault, r.round))
end

dstest.dst.clear(s)
dstest.info("http workload test passed")
