
use hclua_macro::ObjectMacro;

#[derive(ObjectMacro, Default)]
#[hclua_cfg(name = HcTest)]
#[hclua_cfg(light)]
struct HcTestMacro {
    #[hclua_field]
    field: u32,
    #[hclua_field]
    hc: String,
}

impl HcTestMacro {
    fn ok(&self) {
        println!("ok!!!!");
    }
}


fn main() {
    let mut lua = hclua::Lua::new();
    HcTestMacro::register(&mut lua);
    // 直接注册函数注册
    HcTestMacro::object_def(&mut lua, "ok", hclua::function1(HcTestMacro::ok));
    // 闭包注册单参数
    HcTestMacro::object_def(&mut lua, "call1", hclua::function1(|obj: &HcTestMacro| -> u32 {
        obj.field
    }));
    // 闭包注册双参数
    HcTestMacro::object_def(&mut lua, "call2", hclua::function2(|obj: &mut HcTestMacro, val: u32| -> u32 {
        obj.field + val
    }));
    HcTestMacro::object_static_def(&mut lua, "sta_run", hclua::function0(|| -> String {
        "test".to_string()
    }));
    lua.openlibs();
    
    let val = "
        print(type(HcTest));
        local v = HcTest.new();
        print(\"call ok\", v:ok())
        print(\"call1\", v:call1())
        print(\"call2\", v:call2(2))
        print(\"kkkk\", v.hc)
        v.hc = \"dddsss\";
        print(\"kkkk ok get_hc\", v:get_hc())
        v.hc = \"aa\";
        print(\"new kkkkk\", v.hc)
        v:set_hc(\"dddddd\");
        print(\"new kkkkk1\", v.hc)
        print(\"attemp\", v.hc1)
        print(\"static run\", HcTest.sta_run())
        HcTest.del(v);
    ";
    let _: Option<()> = lua.exec_string(val);
}
