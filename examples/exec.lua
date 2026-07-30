--- @diagnostic disable:undefined-global
--- Command execution: run commands inside containers
--- Run: cat examples/exec.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 789,
    weights = {
        pause = 1.0,
    },
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- List files in container
local result = dstest.exec(s, {"ls", "-la", "/app"})
if result.exit_code == 0 then
    dstest.info("Files in /app:")
    dstest.info(result.stdout)
else
    dstest.error("ls failed: " .. result.stderr)
end

-- Check environment variables
result = dstest.exec(s, {"env"})
if result.exit_code == 0 then
    dstest.debug("Environment variables set")
end

-- Inject fault and verify container state
local step_result = dstest.step()
dstest.info(string.format("Injected: %s", step_result.fault))

-- Clear faults before next exec
dstest.clear(s)

-- Run health check inside container
result = dstest.exec(s, {"curl", "-s", "http://localhost:80/status/200"})
dstest.info(string.format("Health check exit_code=%d", result.exit_code))

dstest.clear(s)
dstest.info("exec example complete")
