--- @diagnostic disable:undefined-global
--- Virtual clock injection: pin a subject to a fixed epoch, advance it, and
--- verify the subject's own perception of time follows.
--- Run: cat examples/clock.lua | cargo run

local cfg = dstest.config({ substrate = "docker", seed = 0xDEAD })

local s = dstest.setup(cfg, {
	image = "kennethreitz/httpbin",
	ports = { 80 },
	-- Opt into a virtual clock pinned to 2020-09-13T12:26:40Z.
	clock = { virtual = true, start_epoch = 1600000000 },
})

-- The subject's own clock reports the pinned epoch (it doesn't advance on
-- its own — the manual clock is frozen until we move it).
local r = dstest.exec(s, { "date", "-u", "+%Y-%m-%dT%H:%M:%SZ" })
dstest.info("subject time: " .. r.stdout)

local vc = dstest.clock.virtual(s)
dstest.info(string.format("vc:now() = %d ms", vc:now().millis))

-- Advance by 1 hour; the subject sees the new time immediately.
vc:advance(3600 * 1000)
local r2 = dstest.exec(s, { "date", "-u", "+%Y-%m-%dT%H:%M:%SZ" })
dstest.info("after +1h: " .. r2.stdout)

-- Advance by 30 days to test lease/renewal scenarios.
vc:advance(30 * 24 * 3600 * 1000)
local r3 = dstest.exec(s, { "date", "-u", "+%Y-%m-%dT%H:%M:%SZ" })
dstest.info("after +30d: " .. r3.stdout)

dstest.info("clock example complete")
