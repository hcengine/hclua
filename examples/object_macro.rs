use hclua_macro::HelloMacro;

#[derive(HelloMacro, Default)]
struct Xx {
    #[field]
    field: u32,
    fieldxx: u32,
    #[field]
    aabbfieldxx11: u32,
}

impl Xx {
    fn ok(&self) {
        println!("ok!!!!");
    }
}

fn main() {
    let mut lua = hclua::Lua::new();
    let xx = Xx::default();
    xx.hello_macro();
    xx.ok();

    Xx::register_field();

    
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
