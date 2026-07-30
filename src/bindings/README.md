# bindings

Lua-facing functions exposed to user scripts via the global `dstest` table.

## Architecture

Every namespace is a module implementing the `LuaModule<S>` trait:

```rust
pub trait LuaModule<S: Substrate> {
    fn register(lua: &Lua, dstest: &Table, ctx: &BindingContext<S>) -> mlua::Result<()>;
}
```

`register_all` (in `mod.rs`) wires every module into the `dstest` table at engine
startup. Each module creates its own sub-table (e.g. `dstest.oracle`) and populates
it with functions, or registers functions directly on `dstest` for flat namespaces.

`BindingContext<S>` (defined in [`src/engine/context.rs`](../engine/context.rs)) is the
shared handle every binding clones `Arc`s from: engine state, config, fault tree, oracle,
and the substrate.

## Namespaces

| Namespace | Folder | Lua functions |
|-----------|--------|---------------|
| `dstest.config`, `dstest.setup` | [`core/`](core/README.md) | `config`, `setup` |
| `dstest.step`, `dstest.run_steps`, `dstest.clear`, `dstest.oracle.*` | [`dst/`](dst/README.md) | `step`, `run_steps`, `clear`, `oracle` |
| `dstest.http`, `dstest.tcp` | [`net/`](net/README.md) | `http`, `tcp` |
| `dstest.inspect`, `dstest.logs`, `dstest.exec` | [`subs/`](subs/README.md) | `inspect`, `logs`, `exec` |
| `dstest.pg.*` | [`pg/`](pg/README.md) | `pg.connect`, `pg.query`, `pg.close` |
| `dstest.clock` | [`clock.rs`](clock.rs) | `clock` |
| `dstest.debug/info/warn/error` | [`log.rs`](log.rs) | `debug`, `info`, `warn`, `error` |

## Adding a new binding module

1. Create `src/bindings/<name>/` with a `mod.rs` defining a unit struct (e.g. `pub struct Foo;`).
2. `impl<S: Substrate> LuaModule<S> for Foo` — create a sub-table, register leaf functions, set it on `dstest`.
3. Add `mod <name>;` to [`mod.rs`](mod.rs) and call `foo::Foo::register(lua, dstest, ctx)?;` in `register_all`.
4. If the module has leaf functions, put each in its own file under the folder with a `pub fn register<S: Substrate>(...)`.
5. Update the table above and the relevant per-module README.

## Conventions

- Leaf files each expose a `pub fn register<S: Substrate>(lua, table, ctx)` — the module `mod.rs` calls them and sets the sub-table on `dstest`.
- Bindings clone `Arc` handles out of `ctx` (e.g. `Arc::clone(&ctx.substrate)`) so closures are `'static + Send`.
- Errors return `mlua::Error::RuntimeError(String)`. Prefer `String`/`anyhow` in substrate code and convert at the binding boundary.
- Update `DOCS.md` and the per-module README when changing the Lua API surface.
