--- @diagnostic disable:undefined-global
--- Logs inspection: fetch and analyze container logs during chaos testing
--- Run: cat examples/logs.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 123,
    weights = {
        pause = 1.0,
    },
    http_retries = 5,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- Inject faults
dstest.run_steps(3)

-- Fetch last 50 lines of logs
local logs = dstest.logs(s, { tail = "50", timestamps = true })
dstest.info(string.format("Fetched %d log entries", #logs))

for _, entry in ipairs(logs) do
    if entry.stream == "stderr" then
        dstest.warn(string.format("[%s] %s", entry.stream, entry.message))
    else
        dstest.debug(string.format("[%s] %s", entry.stream, entry.message))
    end
end

dstest.clear(s)
dstest.info("logs example complete")
