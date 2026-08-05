--- @diagnostic disable:undefined-global
--- Main test runner for dstest.
---
--- Usage: cat tests/main.lua | cargo run --release
---
--- Loads and runs each test_*.lua file in sequence within the same Lua VM.
--- Each test gets its own config and subjects. Errors are caught and
--- reported without aborting subsequent tests.

local test_files = {
	"tests/test_config.lua",
	"tests/test_fault.lua",
	"tests/test_oracle.lua",
	"tests/test_workload_http.lua",
}

dstest.info(string.format("found %d test files", #test_files))

local passed = 0
local failed = 0

for _, tf in ipairs(test_files) do
	local name = tf:match("test_(.*)%.lua") or tf
	dstest.info(string.format("running %s...", name))

	local content = io.open(tf, "r")
	if not content then
		dstest.warn(string.format("  SKIP %s (file not found)", name))
		goto continue
	end
	local source = content:read("*all")
	content:close()

	local ok, err = pcall(function()
		local fn = load(source, tf)
		if not fn then
			error("could not load test file")
		end
		local result = fn()
		if result == false then
			error("test returned false")
		end
	end)

	if ok then
		dstest.info(string.format("  PASS %s", name))
		passed = passed + 1
	else
		dstest.warn(string.format("  FAIL %s: %s", name, err))
		failed = failed + 1
	end

	::continue::
end

dstest.info(string.format("results: %d passed, %d failed, %d total", passed, failed, #test_files))

if failed > 0 then
	dstest.error("some tests failed")
end
