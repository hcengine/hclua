use std::collections::HashMap;
use libc;
use std::hash::Hash;
use hcproto::Value;
use crate::{LuaPush, lua_State};
/// the wrapper for push to lua

#[derive(PartialEq, Clone)]
pub struct LuaWrapperValue(pub Value);

impl Eq for LuaWrapperValue {
}

impl Hash for LuaWrapperValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(&self.0).hash(state);
    }
}



impl LuaPush for LuaWrapperValue {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        match self.0 {
            Value::Nil => ().push_to_lua(lua),
            Value::Bool(val) => val.push_to_lua(lua),
            Value::U8(val) => val.push_to_lua(lua),
            Value::I8(val) => val.push_to_lua(lua),
            Value::U16(val) => val.push_to_lua(lua),
            Value::I16(val) => val.push_to_lua(lua),
            Value::U32(val) => val.push_to_lua(lua),
            Value::I32(val) => val.push_to_lua(lua),
            Value::U64(val) => val.push_to_lua(lua),
            Value::I64(val) => val.push_to_lua(lua),
            Value::Varint(val) => val.push_to_lua(lua),
            Value::F32(val) => val.push_to_lua(lua),
            Value::F64(val) => val.push_to_lua(lua),
            Value::Str(val) => val.push_to_lua(lua),
            Value::Raw(val) => {
                unsafe {
                    let mut pre = vec![140, 150, 141, 151];
                    pre.extend(val);
                    crate::lua_pushlstring(lua, pre.as_ptr() as *const libc::c_char, pre.len())
                };
                1
            }
            Value::Map(mut val) => {
                let mut wrapper_val: HashMap<LuaWrapperValue, LuaWrapperValue> = HashMap::new();
                for (k, v) in val.drain() {
                    wrapper_val.insert(LuaWrapperValue(k), LuaWrapperValue(v));
                }
                wrapper_val.push_to_lua(lua)
            }
            Value::Arr(mut val) => {
                let mut wrapper_val: Vec<LuaWrapperValue> = vec![];
                for v in val.drain(..) {
                    wrapper_val.push(LuaWrapperValue(v));
                }
                wrapper_val.push_to_lua(lua)
            }
        }
    }
}

pub struct LuaWrapperVecValue(pub Vec<Value>);
impl LuaPush for LuaWrapperVecValue {
    fn push_to_lua(mut self, lua: *mut lua_State) -> i32 {
        let mut index = 0;
        for v in self.0.drain(..) {
            index = LuaWrapperValue(v).push_to_lua(lua);
        }
        index
    }
}


pub struct LuaWrapperTableValue(pub Vec<Value>);
impl LuaPush for LuaWrapperTableValue {
    fn push_to_lua(mut self, lua: *mut lua_State) -> i32 {
        unsafe {
            crate::lua_newtable(lua);
            for (i, v) in self.0.drain(..).enumerate() {
                crate::lua_pushnumber(lua, (i + 1) as f64);
                LuaWrapperValue(v).push_to_lua(lua);
                crate::lua_settable(lua, -3);
            }
        }
        1
    }
}
