--- @diagnostic disable:undefined-global
--- dstest - Config example with volumes, env, and command overrides

dstest.config({
    substrate = "docker",
    seed = 42,
})

local s = dstest.setup({
    image = "nginx:alpine",
    ports = { 80 },
    volumes = { "/tmp/test:/usr/share/nginx/html:ro" },
    env = { NGINX_PORT = "8080" },
    cmd = { "nginx", "-g", "daemon off;" },
})

local resp = dstest.http(s, "GET", "/")

dstest.info("nginx container running with custom config")

dstest.clear(s)
dstest.info("config example complete")
