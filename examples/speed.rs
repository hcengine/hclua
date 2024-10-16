use hclua::{LuaRead, LuaPush};

struct Xx {
    kk: String,
    nn: String,
}

macro_rules! test {
    ($obj: expr, $name: ident) => {
        $obj.$name = format!("____{} ___ {}", stringify!($name), "aaa");
        $obj.nn = "xxx".to_string();
    };
}
// macro_rules! add_field {
//     ($obj: expr, $name: ident, $t: ty, $field_type: ty) => {
//         let tname = format!("{:?}", TypeId::of::<T>());
//         let mut lua = Lua::from_existing_state(self.lua, false);
//         match lua.query::<LuaTable, _>(tname) {
//             Some(mut table) => {
//                 match table.query::<LuaTable, _>("__index") {
//                     Some(mut index) => {
//                         index.set(name, param);
//                     }
//                     None => {
//                         let mut index = table.empty_table("__index");
//                         index.set(name, param);
//                     }
//                 };
//             }
//             None => (),
//         };
//         self
//     };
// }

fn main() {
    let mut xx = Xx { kk: String::new(), nn: String::new() };
    test!(xx, kk);
    let mut lua = hclua::Lua::new();
    lua.openlibs();
    let val = r"
        local start = os.time();
        local sum = 0;
        for i = 0, 10000000000 do
            sum = sum + i;
        end
        print(os.time() - start);
    ";
    let _: Option<()> = lua.exec_string(val);
}