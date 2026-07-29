--- @diagnostic disable:undefined-global
--- HTTP assertions: custom checks for status codes, headers, and body content

dstest.config({
    substrate = "docker",
    seed = 42,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

local function assert_status(path, expected)
    local ok, resp = pcall(dstest.http, s, "GET", path)
    if not ok then
        dstest.warn(string.format("%s: request failed", path))
        return false
    end
    if resp.status ~= expected then
        dstest.warn(string.format("%s: expected %d, got %d", path, expected, resp.status))
        return false
    end
    dstest.debug(string.format("%s: OK (%d)", path, resp.status))
    return true
end

local function assert_body_contains(path, expected)
    local ok, resp = pcall(dstest.http, s, "GET", path)
    if not ok then
        dstest.warn(string.format("%s: request failed", path))
        return false
    end
    if not resp.body:find(expected, 1, true) then
        dstest.warn(string.format("%s: body missing '%s'", path, expected))
        return false
    end
    dstest.debug(string.format("%s: body contains '%s'", path, expected))
    return true
end

dstest.info("HTTP assertion tests")

assert_status("/get", 200)
assert_status("/status/200", 200)
assert_status("/status/404", 404)
assert_status("/status/500", 500)
assert_body_contains("/get", "\"headers\"")

dstest.clear(s)
dstest.info("HTTP assertions complete")
