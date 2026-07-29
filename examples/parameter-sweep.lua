--- @diagnostic disable:undefined-global
--- Parameter sweep: run experiments with different seeds and collect results

local results = {}
local seeds = { 1, 2, 3 }

dstest.config({
    substrate = "docker",
    weights = { pause = 0.5, kill = 0.5 },
})

for i, seed in ipairs(seeds) do
    dstest.info(string.format("=== seed %d ===", seed))

    dstest.config({ seed = seed })

    local s = dstest.setup({
        image = "kennethreitz/httpbin",
        ports = { 8000 + i },
    })

    local faults = {}
    for j = 1, 3 do
        local result = dstest.step()
        if not result.more then
            break
        end
        table.insert(faults, result.fault)
    end

    table.insert(results, { seed = seed, faults = faults })

    dstest.clear(s)
end

dstest.info("=== parameter sweep results ===")
for _, r in ipairs(results) do
    dstest.info(string.format("seed %d: %s", r.seed, table.concat(r.faults, ", ")))
end
