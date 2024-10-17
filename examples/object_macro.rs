
use hclua_macro::ObjectMacro;

#[derive(ObjectMacro, Default)]
#[hclua_cfg(name = CCCC)]
#[hclua_cfg(light)]
struct TestMacro {
    #[hclua_field]
    field: u32,
    #[hclua_field]
    aabbfieldxx11: u32,
    #[hclua_field]
    kk: String,
}

impl TestMacro {
    fn ok(&self) {
        println!("ok!!!!");
    }
}


fn main() {
    let mut lua = hclua::Lua::new();
    let mut xx = TestMacro::default();
    xx.kk = "ok".to_string();
    xx.ok();

    TestMacro::register(&mut lua);
    TestMacro::object_def(&mut lua, "xxx", hclua::function1(|obj: &mut TestMacro| -> u32 {
        obj.field
    }));
    lua.openlibs();

    
    let val = "
        print(aaa);
        print(\"cccxxxxxxxxxxxxxxx\");
        print(type(CCCC));
        local v = CCCC();
        print(\"xxx\", v:xxx())
        print(\"kkkk\", v.kk)
        v.kk = \"dddsss\";
        print(\"kkkk ok get_kk\", v:get_kk())
        v.kk = \"aa\";
        print(\"new kkkkk\", v.kk)
        v:set_kk(\"dddddd\");
        print(\"new kkkkk1\", v.kk)
        print(\"attemp\", v.kk1)
        print(\"vvvvv\", v:xxx())
        print(\"kkkk get_kk\", v.get_kk())
    ";

    let _: Option<()> = lua.exec_string(val);
}
