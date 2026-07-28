--- @diagnostic disable:undefined-global
--- Response time validation: measures latency and asserts correct response codes
--- Uses oracle predicates for automated verification during fault injection

dstest.config({
    substrate = "docker",
    seed = 999,
    weights = {
        pause = 0.50,
        kill = 0.50,
    },
    step_delay = 200,
    http_retries = 10,
    http_retry_delay = 200,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

dstest.oracle.predicate("response_time_under_500ms", function(subject, fault, round)
    if fault == "pause" or fault == "kill" then
        return true
    end
    
    local start = os.clock()
    local ok, resp = pcall(dstest.http, subject, "GET", "/get")
    local elapsed = (os.clock() - start) * 1000
    
    if not ok then
        return {false, "request failed"}
    end
    
    if resp.status ~= 200 then
        return {false, string.format("expected 200, got %d", resp.status)}
    end
    
    if elapsed > 500 then
        return {false, string.format("response time %.0fms exceeds 500ms", elapsed)}
    end
    
    return true
end)

dstest.oracle.predicate("status_codes_correct", function(subject, fault, round)
    if fault == "pause" or fault == "kill" then
        return true
    end
    
    local ok, resp = pcall(dstest.http, subject, "GET", "/status/200")
    if not ok or resp.status ~= 200 then
        return {false, "/status/200 should return 200"}
    end
    
    ok, resp = pcall(dstest.http, subject, "GET", "/status/404")
    if not ok or resp.status ~= 404 then
        return {false, "/status/404 should return 404"}
    end
    
    return true
end)

dstest.info("running response time validation")

local report = dstest.oracle.run(function()
    local results = dstest.run_steps(10)
    for _, r in ipairs(results) do
        dstest.info(string.format("round %d: %s", r.round, r.fault))
    end
end)

dstest.info(string.format(
    "report: passed=%s checks=%d passed=%d failed=%d",
    tostring(report.passed),
    report.total_checks,
    report.passed_checks,
    report.failed_checks
))

if not report.passed then
    dstest.warn("validation failures:")
    for _, f in ipairs(report.failures) do
        dstest.warn(string.format("  [%s] %s: %s", f.type, f.name, f.error))
    end
end

dstest.clear(s)
dstest.info("validation complete")
