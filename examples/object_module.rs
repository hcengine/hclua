
use hclua::{Lua, LuaTable};

#[hclua::lua_module]
fn rust_module(lua: &mut Lua) -> Option<LuaTable> {
    println!("xxxxxxxxxxxw1");
    let mut table = lua.create_table();
    table.set("id", 1);
    Some(table)
}

fn main() {
    let mut lua = hclua::Lua::new_with_limit(102400, None);
    lua.openlibs();
    luareg_rust_module(lua.state());

    println!("xxxxxxxx");
    let val = r#"
        print("xxxxxx11x");
        local a = require("io");
        print("bbbb");
        local a = require("rust_module");
        print("xxx")
        print("aaaaaaaaa  %d", a.id);

        local ab = require("rust.module");
        print("xxx ab!")
        print("aaaaaaaaa ab! %d", ab.id);
    "#;
    let _: Option<()> = lua.exec_string(val);
}
