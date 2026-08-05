--- @diagnostic disable:undefined-global
--- Test: config registration, setup, and subject creation.

local cfg = dstest.config({
	substrate = "docker",
	seed = 42,
	weights = { pause = 0.5, kill = 0.5 },
	accumulation = "single",
	steps = 2,
})

assert(type(cfg) == "string", "config should return a handle string")
dstest.info("config handle: " .. cfg)

local s = dstest.setup(cfg, {
	image = "kennethreitz/httpbin",
	ports = { 80 },
})

assert(type(s) == "string", "setup should return a subject id")
assert(s:find("^docker/"), "subject id should start with docker/")
dstest.info("subject id: " .. s)

-- Inspect the subject
local info = dstest.inspect(s)
assert(info.state == "running", "container should be running")
assert(info.ip, "container should have an IP")
dstest.info(string.format("state=%s ip=%s", info.state, info.ip))

-- HTTP should work
local resp = dstest.net.http(s, "GET", "/get")
assert(resp.status == 200, "httpbin /get should return 200")
dstest.info("HTTP GET /get returned " .. resp.status)

dstest.dst.clear(s)
dstest.info("config test passed")
