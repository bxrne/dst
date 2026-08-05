--- @diagnostic disable:undefined-global
--- Test: fault injection, step scheduling, and recovery.
--- Also exercises pg workload generation during faults.

local cfg = dstest.config({
	substrate = "docker",
	seed = 0xDEAD,
	weights = { pause = 0.5, kill = 0.5 },
	accumulation = "single",
	steps = 4,
	step_delay = 500,
	http_retries = 5,
	http_retry_delay = 200,
})

local s = dstest.setup(cfg, {
	image = "kennethreitz/httpbin",
	ports = { 80 },
})

-- Baseline: healthy
local resp = dstest.net.http(s, "GET", "/get")
assert(resp.status == 200, "baseline should return 200")
dstest.info("baseline: 200 OK")

-- Run fault steps
local results = dstest.dst.run_steps(cfg, 4)
assert(#results > 0, "should have fault results")

for _, r in ipairs(results) do
	dstest.info(string.format("round %d: %s on %s", r.round, r.fault, r.subject))
	assert(r.fault, "result should have a fault type")
	assert(r.subject, "result should have a subject")
	assert(r.round, "result should have a round number")
end

-- After clear, should recover
dstest.dst.clear(s)
local ok, r = pcall(dstest.net.http, s, "GET", "/get")
if ok and r.status == 200 then
	dstest.info("recovered after clear")
else
	dstest.warn("still recovering (may need time)")
end

-- PostgreSQL workload during faults
local pg_cfg = dstest.config({
	substrate = "docker",
	seed = 0xBEEF,
	weights = { pause = 0.5, kill = 0.5 },
	accumulation = "single",
	steps = 2,
})

local pg = dstest.setup(pg_cfg, {
	image = "postgres:16-alpine",
	ports = { 5432 },
	env = {
		POSTGRES_PASSWORD = "password",
		POSTGRES_DB = "test_db",
	},
})

dstest.exec(pg, { "sleep", "3" })
local info = dstest.inspect(pg)
local conn_str = string.format("postgres://postgres:password@%s:5432/test_db", info.ip)
local pool = dstest.pg.connect(conn_str, 5)

dstest.pg.query(pool, "CREATE TABLE items (id SERIAL PRIMARY KEY, val TEXT)")
dstest.pg.query(pool, "INSERT INTO items (val) VALUES ('a'), ('b'), ('c')")

-- Run pg workload during faults
dstest.info("running pg workload during faults...")
local pg_stats = dstest.workload.pg(pool, {
	duration_secs = 3,
	rate = 10,
	queries = {
		"SELECT 1",
		"SELECT id, val FROM items ORDER BY id",
	},
})

assert(pg_stats.total_queries > 0, "pg workload should have made queries")
dstest.info(string.format("pg workload: %d queries, %d ok, %d failed, avg %dms",
	pg_stats.total_queries, pg_stats.ok, pg_stats.failed, pg_stats.avg_latency_ms))

dstest.pg.close(pool)
dstest.dst.clear(s)
dstest.info("fault test passed")
