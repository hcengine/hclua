
use hclua::{add_object_field, lua_State, object_impl, LuaObject, LuaPush, LuaRead};

#[derive(Default)]
struct Xx {
    kk: String,
    nn: String,
}

object_impl!(Xx);

fn main() {
    let mut lua = hclua::Lua::new();
    let mut object = LuaObject::<Xx>::new(lua.state(), "CCCC");
    object.create();
    add_object_field!(object, kk, Xx, String);
    object.add_method_get("xxx", hclua::function1(|obj: &mut Xx| "sss is xxx".to_string()));
    lua.openlibs();

    let val = "
        print(aaa);
        print(\"cccxxxxxxxxxxxxxxx\");
        print(type(CCCC));
        local v = CCCC();
        print(\"vvvvv\", v:xxx())
        print(\"kkkk\", v.kk)
        v.kk = \"aa\";
        print(\"ccccc\", v.kk)
        print(\"vvvvv\", v:xxx())
    ";

    let _: Option<()> = lua.exec_string(val);
}