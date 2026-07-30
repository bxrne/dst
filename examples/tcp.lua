--- @diagnostic disable:undefined-global
--- TCP connectivity: test non-HTTP services during chaos
--- Run: cat examples/tcp.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 654,
    weights = {
        ["deprive:network"] = 1.0,
    },
    step_delay = 100,
    http_timeout = 3,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- Test HTTP port (80) connectivity
local conn = dstest.tcp(s, 80)
if conn.connected then
    dstest.info(string.format("Port 80 reachable at %s", conn.addr))
else
    dstest.warn(string.format("Port 80 unreachable: %s", conn.error))
end

-- Inject network fault
local result = dstest.step()
dstest.info(string.format("Injected: %s", result.fault))

-- Test connectivity during fault (should fail)
conn = dstest.tcp(s, 80)
if conn.connected then
    dstest.warn("Port 80 still reachable during network fault!")
else
    dstest.info(string.format("Port 80 correctly blocked: %s", conn.error))
end

-- Clear and verify recovery
dstest.clear(s)
conn = dstest.tcp(s, 80)
if conn.connected then
    dstest.info("Network recovered - port 80 reachable")
else
    dstest.warn(string.format("Network not recovered: %s", conn.error))
end

dstest.clear(s)
dstest.info("tcp example complete")
