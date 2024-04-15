# NeoApi-rs
This is a thin layer over mlua to easily call neovim functions in rust. For people that want to write neovim plugins in Rust.

## How to setup your Rust Neovim plugin
```rust
Cargo new --lib (your-plugin-name)
cd (your-plugin-name)
cargo add --git https://github.com/norlock/neo-api-rs
cargo add serde serde_json lazy-static (tokio or any other async crate) 
```

Add crate-type in the Cargo.toml:
```toml
[lib]
crate-type = ["dylib"]
```

Create lua directory and symlink to it
```shell
mkdir lua
touch lua/(your-plugin-name).lua
cd lua
ln -s ../target/release/lib(your-plugin-name).so (your-plugin-name).so
```

Now to build:
```shell
Cargo build --release
```

## How to build your plugin
There is an example you can follow on:
[nvim-traveller-rs](https://github.com/norlock/nvim-traveller-rs)

Basically you can write everything in Rust. You write a global state as static. 
After the init where you need to use blocking_write, you can use async blocks and await thanks to mlua async feature.

```rust
lazy_static! {
    pub static ref CONTAINER: AppContainer = AppContainer::default();
}

#[mlua::lua_module]
fn nvim_traveller_rs(lua: &Lua) -> LuaResult<LuaTable> {
    let module = lua.create_table()?;

    let mut app = CONTAINER.0.blocking_write();

    module.set(
        "some_function",
        lua.create_async_function(open_navigation)?,
    )?;

    Ok(module)
}
```

In lua file: (your-plugin-name).lua:
```lua
return require("your-plugin-name");
```

In your config point to your plugin (e.g. Lazy): 
```lua
...
{ dir = '/path/to/nvim-traveller-rs' },
...
```

And to use:
```lua
local plugin_name = require('your-plugin-name')

vim.keymap.set('n', '<leader>p', plugin_name.some_function, {})
```
