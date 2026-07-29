--- @diagnostic disable:undefined-global
--- Fault accumulation: stack faults without clearing between rounds
--- Note: same fault type won't stack (e.g. pause on already-paused container)

dstest.config({
    substrate = "docker",
    seed = 123,
    weights = { pause = 0.33, kill = 0.33, ["deprive:memory"] = 0.34 },
    accumulation = "accumulate",
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

dstest.info("fault accumulation experiment started")

local faults_applied = {}
for i = 1, 5 do
    local result = dstest.step()
    if not result.more then
        dstest.info("experiment complete")
        break
    end
    table.insert(faults_applied, result.fault)
    dstest.info(string.format("round %d: %s (accumulated)", result.round, result.fault))
end

dstest.info(string.format("faults applied: %s", table.concat(faults_applied, ", ")))
dstest.info("clearing all accumulated faults")
dstest.clear(s)
dstest.info("fault accumulation experiment complete")
