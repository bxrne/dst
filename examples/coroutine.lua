--- @diagnostic disable:undefined-global
--- dstest - Coroutine example for user-controlled async-style execution

dstest.config({
    substrate = "docker",
    seed = 999,
    weights = { pause = 0.5, kill = 0.5 },
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

local function background_health_check()
    while true do
        local ok, resp = pcall(dstest.http, s, "GET", "/health")
        if ok and resp.status == 200 then
            dstest.debug("background health check passed")
        else
            dstest.warn("background health check failed")
        end
        coroutine.yield()
    end
end

local health_co = coroutine.create(background_health_check)

local function fault_injector()
    while true do
        local result = dstest.step()
        if not result.more then
            dstest.info("no more faults")
            return
        end
        dstest.info(string.format("fault applied: %s", result.fault))
        coroutine.yield(result)
    end
end

local fault_co = coroutine.create(fault_injector)

dstest.info("starting coroutine-based experiment")

for i = 1, 10 do
    local ok, result = coroutine.resume(fault_co)
    if not ok or not result then
        break
    end

    coroutine.resume(health_co)
    
    local ok, resp = pcall(dstest.http, s, "GET", "/get")
    if ok and resp.status == 200 then
        dstest.info("health check passed")
    else
        dstest.warn("health check failed")
    end
end

dstest.clear(s)
dstest.info("coroutine experiment complete")
