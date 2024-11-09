
use hclua::{LuaTable, Lua};

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
    println!("xxxxxxxx");
    let val = r#"
        print("xxxxxx11x");
        local a = require("rust_module");
        
        print("aaaaaaaaa  %d", a.id);
    "#;
    let _: Option<()> = lua.exec_string(val);
}
