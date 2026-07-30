--- @diagnostic disable:undefined-global
--- Oracle-driven chaos experiment: predicates + invariants during fault injection.
--- Run: cat examples/oracle.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 999,
    weights = {
        pause = 0.3,
        kill = 0.3,
        ["deprive:memory"] = 0.2,
        ["deprive:cpu"] = 0.2,
    },
    accumulation = "single",
    step_delay = 200,
    http_retries = 10,
    http_retry_delay = 200,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- Predicate: /get must stay healthy during non-fatal faults
dstest.dst.oracle.predicate("get_healthy", function(subject, fault, round)
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

-- Invariant: response time must stay under 500ms
dstest.dst.oracle.invariant("response_time_under_500ms", function()
    local start = dstest.clock()
    local ok, resp = pcall(dstest.net.http, s, "GET", "/get")
    local elapsed = (dstest.clock().nanos - start.nanos) / 1e6
    if not ok then
        return { false, "request failed" }
    end
    if elapsed > 500 then
        return { false, string.format("response time %.0fms exceeds 500ms", elapsed) }
    end
    return true
end)

dstest.info("running oracle experiment")

local report = dstest.dst.oracle.run(function()
    local results = dstest.dst.run_steps(8)
    for _, r in ipairs(results) do
        dstest.info(string.format("round %d: %s on %s", r.round, r.fault, r.subject))
    end
end)

dstest.info(string.format(
    "report: passed=%s total=%d passed=%d failed=%d",
    tostring(report.passed),
    report.total_checks,
    report.passed_checks,
    report.failed_checks
))

if not report.passed then
    dstest.warn("oracle failures:")
    for _, f in ipairs(report.failures) do
        dstest.warn(string.format("  [%s] %s: %s", f.type, f.name, f.error))
    end
end

-- Confirm final container state
local info = dstest.inspect(s)
dstest.info(string.format("final state: %s", info.state))

dstest.dst.clear(s)
dstest.info("oracle example complete")
