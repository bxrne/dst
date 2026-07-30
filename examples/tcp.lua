--- @diagnostic disable:undefined-global
--- TCP send/recv: test line-oriented protocols during chaos
--- Run: cat examples/tcp.lua | cargo run

dstest.config({
    substrate = "docker",
    seed = 654,
    weights = {
        ["deprive:network"] = 1.0,
    },
    step_delay = 100,
    http_timeout = 3,
})

local s = dstest.setup({
    image = "kennethreitz/httpbin",
    ports = { 80 },
})

-- Open a TCP connection
local conn, err = dstest.tcp(s, 80)
if not conn then
    dstest.warn(string.format("connect failed: %s", tostring(err)))
    return
end

dstest.info(string.format("connected to %s", conn:addr()))

-- Send a raw HTTP request
conn:send("GET /get HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")

-- Read status line
local line = conn:recv_line()
if line then
    dstest.info(string.format("status: %s", line:gsub("[\r\n]+$", "")))
else
    dstest.warn("connection closed before response")
    return
end

-- Read response headers
while true do
    line = conn:recv_line()
    if not line or line == "\r\n" or line == "\n" then
        break
    end
    dstest.debug(string.format("header: %s", line:gsub("[\r\n]+$", "")))
end

-- Read response body (up to 4096 bytes)
local body = conn:recv(4096)
if body then
    dstest.info(string.format("body: %d bytes", #body))
end

conn:close()
dstest.info("connection closed")

-- Inject network fault
local result = dstest.step()
dstest.info(string.format("Injected: %s", result.fault))

-- Try to connect during fault (should fail)
conn, err = dstest.tcp(s, 80)
if not conn then
    dstest.info(string.format("correctly rejected during fault: %s", tostring(err)))
else
    dstest.warn("connection should have failed during network fault!")
    conn:close()
end

-- Clear and verify recovery
dstest.clear(s)
conn, err = dstest.tcp(s, 80)
if conn then
    dstest.info("network recovered - connection established")
    conn:close()
else
    dstest.warn(string.format("network not recovered: %s", tostring(err)))
end

dstest.clear(s)
dstest.info("tcp example complete")
