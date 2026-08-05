--- @diagnostic disable:undefined-global
--- Test: oracle predicates and invariants during fault injection.

local cfg = dstest.config({
	substrate = "docker",
	seed = 0xCAFE,
	weights = { pause = 0.5, kill = 0.5 },
	accumulation = "single",
	steps = 3,
	step_delay = 500,
	http_retries = 5,
	http_retry_delay = 200,
})

local s = dstest.setup(cfg, {
	image = "kennethreitz/httpbin",
	ports = { 80 },
})

-- Predicate: skip checks for pause/kill (container is down), otherwise verify /get
dstest.dst.oracle.predicate("get_check", function(subject, fault, round)
	if fault == "pause" or fault == "kill" then
		return true
	end
	local ok, resp = pcall(dstest.net.http, subject, "GET", "/get")
	if not ok then
		return { false, "request failed: " .. tostring(resp) }
	end
	if resp.status ~= 200 then
		return { false, "expected 200, got " .. resp.status }
	end
	return true
end)

-- Invariant: always runs
dstest.dst.oracle.invariant("always_true", function()
	return true
end)

local report = dstest.dst.oracle.run(function()
	local results = dstest.dst.run_steps(cfg, 3)
	for _, r in ipairs(results) do
		dstest.info(string.format("round %d: %s", r.round, r.fault))
	end
end)

assert(report.total_checks > 0, "should have run some checks")
dstest.info(string.format("oracle: %d checks, %d passed, %d failed",
	report.total_checks, report.passed_checks, report.failed_checks))

-- The always_true invariant should always pass
-- Predicate may fail during non-fatal faults if HTTP fails
dstest.info("oracle test passed (oracle ran successfully)")
