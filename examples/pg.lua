--- @diagnostic disable:undefined-global

dstest.config({
	substrate = "docker",
	seed = 42,
})

local s = dstest.setup({
	image = "postgres:16-alpine",
	ports = { 5432 },
	env = {
		POSTGRES_PASSWORD = "password",
		POSTGRES_DB = "test_db"
	}
})

-- Get runtime metadata containing the live container IP address
local info = dstest.inspect(s)
dstest.info("Target Container bridge IP: " .. tostring(info.ip))

-- Basic delay loop using exec to give Postgres time to generate cluster files
dstest.info("Waiting for database boot cycle...")
dstest.exec(s, { "sleep", "3" })

-- Connect directly to the internal container IP address instead of localhost
local conn_str = string.format("postgres://postgres:password@%s:5432/test_db", info.ip)
dstest.info("Connecting to target database string: " .. conn_str)

local pool = dstest.pg.connect(conn_str, 5)
dstest.info("Database connection established!")

local res = dstest.pg.query(pool, "SELECT 1 as active")
dstest.info("Execution sample count: " .. #res)

dstest.pg.close(pool)
