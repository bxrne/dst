--- @diagnostic disable:undefined-global
--- Virtual disk faults via dm-flakey: errors, corruption, snapshot/restore.
--- Requires root on the host (loop devices + device-mapper).
--- Run: cat examples/storage.lua | cargo run

local cfg = dstest.config({
    substrate = "docker",
    seed = 0xDEAD,
    weights = { kill = 0.5, ["deprive:disk"] = 0.3, ["deprive:cpu"] = 0.2 },
    accumulation = "accumulate",
})

local s = dstest.setup(cfg, {
    image = "alpine:3.20",
    cmd = { "sleep", "300" },
    storage = { flaky = true, mount = "/data", size_mb = 64 },
})

-- Write some data we can verify later
dstest.exec(s, { "sh", "-c", "echo important > /data/file.txt && sync" })

-- Snapshot the clean state
local snap = dstest.storage.snapshot(s)

-- Inject EIO on all I/O and verify the subject sees errors
dstest.storage.error(s, true)
local r = dstest.exec(s, { "cat", "/data/file.txt" })
if r.exit_code ~= 0 then
    dstest.info("I/O error injected as expected: " .. r.stderr)
else
    dstest.warn("expected I/O error during storage.error(true)")
end

-- Flip some bytes on disk
dstest.storage.corrupt(s, 8)
local r2 = dstest.exec(s, { "cat", "/data/file.txt" })
if r2.stdout ~= "important\n" then
    dstest.info("data corruption detected")
end

-- Restore to the clean snapshot and verify
dstest.storage.restore(s, snap)
dstest.storage.error(s, false)
local r3 = dstest.exec(s, { "cat", "/data/file.txt" })
assert(r3.stdout == "important\n", "restored data mismatch")

dstest.info("storage fault test complete")
