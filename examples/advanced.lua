--- @diagnostic disable:undefined-global
--- dstest - Advanced example with custom weights and fault stepping

dstest.config({
    substrate = "docker",
    seed = 12345,
    weights = { pause = 0.5, kill = 0.3, ["deprive:disk"] = 0.2 },
    accumulation = "single",
    http_timeout = 10,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

dstest.info("running fault injection experiment")

while true do
    local result = dstest.step()
    if not result.more then
        dstest.info("experiment complete")
        break
    end

    dstest.info(string.format("round %d: %s on %s", result.round, result.fault, result.subject))

    local ok, resp = pcall(dstest.http, s, "GET", "/get")
    if ok and resp.status == 200 then
        dstest.debug("service healthy after fault")
    else
        dstest.warn("service degraded after fault")
    end
end

dstest.clear(s)
dstest.info("experiment finished, faults cleared")
