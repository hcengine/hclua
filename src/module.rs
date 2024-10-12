use crate::{lua_State, sys, Lua, LuaPush, LuaTable};


pub struct LuaModule {
    lua: *mut lua_State,
    name: &'static str,
}

impl LuaModule {
    pub fn new(lua: *mut lua_State, name: &'static str) -> Self {
        LuaModule {
            name,
            lua
        }
    }
    
    pub fn ensure(&self) -> LuaTable {
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(self.name) {
            Some(table) => {
                table
            }
            None => {
                lua.empty_table(self.name)
            },
        }
    }
    
    pub fn def<P>(&self, name: &str, param: P) -> &LuaModule
    where
        P: LuaPush,
    {
        let mut t = self.ensure();
        t.set(name, param);
        self
    }

    pub fn register(
        &self,
        name: &str,
        func: extern "C" fn(*mut sys::lua_State) -> libc::c_int,
    ) -> &LuaModule {
        let mut t = self.ensure();
        t.register(name, func);
        self
    }
}