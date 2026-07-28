--- @diagnostic disable:undefined-global
--- Coroutine example: user-controlled fault injection with custom health checks
--- Shows how to interleave fault injection with your own verification logic

dstest.config({
    substrate = "docker",
    seed = 123,
    accumulation = "single",
    step_delay = 100,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

local function run_experiment(steps)
    return coroutine.create(function()
        for i = 1, steps do
            local result = dstest.step()
            if not result or not result.more then
                return nil
            end
            coroutine.yield(result)
        end
    end)
end

dstest.info("starting coroutine-based experiment")

local co = run_experiment(5)
while true do
    local ok, result = coroutine.resume(co)
    if not ok or not result then
        dstest.info("experiment complete")
        break
    end
    
    dstest.info(string.format("applied fault: %s", result.fault))
    
    local start = os.clock()
    local resp = dstest.http(s, "GET", "/get")
    local elapsed = os.clock() - start
    
    if resp.status == 200 then
        dstest.info(string.format("service healthy (%.3fs)", elapsed))
    else
        dstest.warn(string.format("service degraded: status=%d", resp.status))
    end
end

dstest.clear(s)
dstest.info("cleanup complete")
