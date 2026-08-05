--- @diagnostic disable:undefined-global
--- Proxied network faults between two subjects: latency, loss, partitions.
--- Run: cat examples/link.lua | cargo run

local cfg = dstest.config({
    substrate = "docker",
    seed = 0xBEEF,
    weights = { pause = 0.5, ["deprive:memory"] = 0.3, kill = 0.2 },
    accumulation = "single",
})

local server = dstest.setup(cfg, {
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

local client = dstest.setup(cfg, {
    image = "curlimages/curl:latest",
    cmd = { "sleep", "300" },
    depends = { server },
})

-- Impair the link from client to server
local link = dstest.net.link(client, server, 80)
link:latency(100, 30)   -- 100ms base + 30ms jitter
link:loss(0.1)          -- 10% packet loss

-- Measure latency under impairment
local start = dstest.clock()
local ok, resp = pcall(dstest.net.http, client, "GET", "/get")
local elapsed = (dstest.clock().nanos - start.nanos) / 1e6

if ok then
    dstest.info(string.format("response: status=%d latency=%.0fms", resp.status, elapsed))
else
    dstest.warn("request failed: " .. tostring(resp))
end

-- Heal the link and re-measure
link:heal()
local start2 = dstest.clock()
local ok2, resp2 = pcall(dstest.net.http, client, "GET", "/get")
local elapsed2 = (dstest.clock().nanos - start2.nanos) / 1e6

if ok2 then
    dstest.info(string.format("after heal: status=%d latency=%.0fms", resp2.status, elapsed2))
end

dstest.dst.clear(server)
dstest.info("link example complete")
