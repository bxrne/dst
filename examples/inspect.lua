--- @diagnostic disable:undefined-global
--- Container inspection: verify state after faults
--- Run: cat examples/inspect.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 456,
    weights = {
        pause = 0.5,
        kill = 0.5,
    },
    step_delay = 200,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- Initial inspection
local info = dstest.inspect(s)
dstest.info(string.format("Initial state: %s, pid=%s, ip=%s",
    info.state, tostring(info.pid), tostring(info.ip)))

-- Inject a single fault
local result = dstest.step()
dstest.info(string.format("Injected fault: %s", result.fault))

-- Inspect after fault
info = dstest.inspect(s)
dstest.info(string.format("After fault: state=%s, pid=%s",
    info.state, tostring(info.pid)))

-- Verify container recovered
dstest.clear(s)

local ok, resp = pcall(dstest.http, s, "GET", "/get")
if ok and resp.status == 200 then
    dstest.info("Container recovered successfully")
else
    dstest.warn("Container not yet recovered")
end

info = dstest.inspect(s)
dstest.info(string.format("Final state: %s", info.state))

dstest.clear(s)
dstest.info("inspect example complete")
