use hclua::{LuaRead, LuaTable};
use hclua_macro::ObjectMacro;

#[derive(ObjectMacro, Default)]
#[hclua_cfg(name = HcTest1)]
#[hclua_cfg(light)]
struct HcTestMacro1 {
    field: u32,
    hc: String,
    #[hclua_skip]
    vec: Vec<u8>,
}


#[derive(ObjectMacro)]
#[hclua_cfg(name = HcTest)]
#[hclua_cfg(light)]
struct HcTestMacro {
    field: u32,
    hc: String,
    #[hclua_skip]
    vec: Vec<u8>,
}

impl Default for HcTestMacro {
    fn default() -> Self {
        Self {
            field: Default::default(),
            hc: Default::default(),
            vec: Default::default(),
        }
    }
}

impl HcTestMacro {
    fn ok(&self) {
        println!("ok!!!!");
    }
}

fn main() {
    let mut lua = hclua::Lua::new_with_limit(102400, None);
    HcTestMacro::register(&mut lua);
    // 直接注册函数注册
    HcTestMacro::object_def(&mut lua, "ok", hclua::function1(HcTestMacro::ok));
    // 闭包注册单参数
    HcTestMacro::object_def(
        &mut lua,
        "call1",
        hclua::function1(|obj: &HcTestMacro| -> u32 { obj.field }),
    );
    // 闭包注册双参数
    HcTestMacro::object_def(
        &mut lua,
        "call2",
        hclua::function2(|obj: &mut HcTestMacro, val: u32| -> u32 { obj.field + val }),
    );
    HcTestMacro::object_static_def(
        &mut lua,
        "sta_run",
        hclua::function0(|| -> String { "test".to_string() }),
    );
    lua.openlibs();
    println!("xxxxxxxxxx");
    let val = "
        print(type(HcTest));
        local v = HcTest.new();
        v:set_from_table({
            hc = \"string\",
            field = 12345,
        })
        print(\"hc\", v.hc)
        print(\"field\", v.field)
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
