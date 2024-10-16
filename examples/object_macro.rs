
use hclua_macro::HelloMacro;

#[derive(HelloMacro, Default)]
#[hclua_cfg(CCCC)]
#[hclua_cfg(light)]
struct Xx {
    #[field]
    field: u32,
    #[field]
    aabbfieldxx11: u32,
    #[field]
    kk: String,
}

impl Xx {
    fn ok(&self) {
        println!("ok!!!!");
    }
}


fn main() {
    let mut lua = hclua::Lua::new();
    let mut xx = Xx::default();
    xx.kk = "ok".to_string();
    xx.ok();

    Xx::register(&mut lua);
    hclua::LuaObject::<Xx>::object_def(&mut lua, "xxx", hclua::function1(|obj: &mut Xx| -> u32 {
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