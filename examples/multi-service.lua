--- @diagnostic disable:undefined-global
--- Multi-service: inject faults across multiple containers simultaneously

dstest.config({
    substrate = "docker",
    seed = 42,
    weights = { pause = 0.6, kill = 0.4 },
})

local backend = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 8080 },
})

local cache = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 8081 },
})

local function get_host(subject)
    return subject:gsub("docker/", "localhost:")
end

dstest.info("multi-service experiment started")

for i = 1, 5 do
    local result = dstest.step()
    if not result.more then
        break
    end

    dstest.info(string.format("round %d: %s on %s", result.round, result.fault, result.subject))

    if result.fault ~= "pause" and result.fault ~= "kill" then
        local ok1, r1 = pcall(dstest.http, backend, "GET", "/get")
        local ok2, r2 = pcall(dstest.http, cache, "GET", "/get")

        if ok1 and r1.status == 200 and ok2 and r2.status == 200 then
            dstest.debug("both services healthy")
        else
            dstest.warn("service degradation detected")
        end
    end
end

dstest.clear(backend)
dstest.clear(cache)
dstest.info("multi-service experiment complete")
