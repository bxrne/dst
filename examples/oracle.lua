--- @diagnostic disable:undefined-global
--- dstest - Oracle example with predicates and invariants

dstest.config({
    substrate = "docker",
    seed = 999,
    weights = { pause = 0.5, kill = 0.5 },
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

dstest.oracle.predicate("get_endpoint", function(subject, fault, round)
    if fault == "pause" or fault == "kill" then
        return true
    end
    local ok, resp = pcall(dstest.http, subject, "GET", "/get")
    if not ok then
        return { false, "request failed: " .. tostring(resp) }
    end
    if resp.status ~= 200 then
        return { false, "status " .. resp.status }
    end
    return true
end)

dstest.oracle.predicate("status_404", function(subject, fault, round)
    if fault == "pause" or fault == "kill" then
        return true
    end
    local ok, resp = pcall(dstest.http, subject, "GET", "/status/404")
    if not ok then
        return { false, "request failed: " .. tostring(resp) }
    end
    if resp.status ~= 404 then
        return { false, "expected 404, got " .. resp.status }
    end
    return true
end)

dstest.info("running oracle experiment")

local report = dstest.oracle.run(function()
    local results = dstest.run_steps(5)
    for _, r in ipairs(results) do
        dstest.info(string.format("round %d: %s", r.round, r.fault))
    end
end)

dstest.info(string.format("oracle report: passed=%s total=%d passed=%d failed=%d",
    report.passed, report.total_checks, report.passed_checks, report.failed_checks))

if not report.passed then
    dstest.warn("oracle failures detected:")
    for _, f in ipairs(report.failures) do
        dstest.warn(string.format("  [%s] %s:%s %s", f.type, f.name, 
            f.round and (" round=" .. f.round) or "", f.error))
    end
end

dstest.clear(s)
dstest.info("oracle experiment complete")
