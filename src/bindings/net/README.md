# net

HTTP and TCP networking against subjects. Exposes `dstest.http` and `dstest.tcp`.

## `dstest.http(subject, method, path)`

Makes an HTTP request to a subject's mapped host port. Returns a table with
`status` (number) and `body` (string). Retries on connection failure using
`http_retries` / `http_retry_delay` from config.

```lua
local resp = dstest.http(subject, "GET", "/get")
if resp.status == 200 then
    dstest.info("request successful")
end
```

The host is resolved from the first port in `dstest.setup({ ports = { ... } })`.
Timeout is `http_timeout` seconds per attempt.

| Return field | Type | Description |
|--------------|------|-------------|
| `status` | number | HTTP status code |
| `body` | string | Response body |

Wrap in `pcall` — requests can fail during faults (e.g. network deprivation):

```lua
local ok, resp = pcall(dstest.http, subject, "GET", "/get")
if not ok then dstest.warn("request failed: " .. tostring(resp)) end
```

## `dstest.tcp(subject, port)`

Opens a TCP connection to a port on the subject. Returns `(conn, err)` — a
connection userdata on success, or `nil` and an error string on failure.

```lua
local conn, err = dstest.tcp(subject, 6379)
if not conn then
    dstest.warn("connect failed: " .. tostring(err))
    return
end
conn:send("PING\r\n")
local line = conn:recv_line()
conn:close()
```

The host IP is extracted from the subject's mapped host (first port in `setup`).
Timeout is `http_timeout` seconds.

### Connection methods

| Method | Description |
|--------|-------------|
| `conn:send(data)` | Send a string |
| `conn:recv(n)` | Read up to `n` bytes (returns `nil` on EOF) |
| `conn:recv_line()` | Read until `\n` (returns `nil` on EOF) |
| `conn:recv_until(delim)` | Read until a delimiter string (returns `nil` on EOF) |
| `conn:close()` | Close both directions |
| `conn:addr()` | Return the remote address string |
| `conn:set_timeout(secs)` | Set read/write timeout in seconds |
| `conn:set_nodelay(bool)` | Enable/disable TCP_NODELAY |
