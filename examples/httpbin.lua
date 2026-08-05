--- @diagnostic disable:undefined-global
--- HTTP analysis against httpbin: status assertions, body inspection, latency.
--- Run: cat examples/httpbin.lua | cargo run

local docker_config = dstest.config({
    substrate = "docker",
    seed = 42,
})

local s = dstest.setup(docker_config, {
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- Baseline latency
local start = dstest.clock()
local resp = dstest.net.http(s, "GET", "/get")
local elapsed_ms = (dstest.clock().nanos - start.nanos) / 1e6

assert(resp.status == 200, "/get should return 200")
dstest.info(string.format("baseline: %d bytes in %.2fms", #resp.body, elapsed_ms))

-- Status code endpoints
for _, code in ipairs({ 200, 404, 500 }) do
    local r = dstest.net.http(s, "GET", "/status/" .. code)
    assert(r.status == code, string.format("/status/%d should return %d, got %d", code, code, r.status))
    dstest.debug(string.format("  /status/%d -> %d", code, r.status))
end

-- POST with body
local post = dstest.net.http(s, "POST", "/post")
assert(post.status == 200, "/post should return 200")
assert(post.body:find('"data"', 1, true), "/post body should echo data")
dstest.info("POST /post echoed data")

-- Inject one fault and verify recovery
local result = dstest.dst.step()
dstest.info(string.format("injected: %s (round %d)", result.fault, result.round))

dstest.dst.clear(s)

local ok, r = pcall(dstest.net.http, s, "GET", "/get")
if ok and r.status == 200 then
    dstest.info("recovered after fault clear")
else
    dstest.warn("not yet recovered")
end

dstest.dst.clear(s)
dstest.info("httpbin example complete")
