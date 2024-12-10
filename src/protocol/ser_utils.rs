use crate::{lua_State, LuaRead};
use hcproto::Value;
use std::collections::HashMap;
use std::{ptr, str};

pub struct SerUtils;

impl SerUtils {
    pub fn read_str_to_vec(lua: *mut lua_State, index: i32) -> Option<Vec<u8>> {
        let mut size: libc::size_t = 0;
        let c_str_raw = unsafe { crate::lua_tolstring(lua, index, &mut size) };
        if c_str_raw.is_null() {
            return None;
        }
        unsafe {
            let mut dst: Vec<u8> = Vec::with_capacity(size);
            ptr::copy(c_str_raw as *mut u8, dst.as_mut_ptr(), size);
            dst.set_len(size);
            Some(dst)
        }
    }

    pub fn lua_read_value(lua: *mut lua_State, index: i32, stack: u32) -> Option<Value> {
        if stack > 100 {
            return None;
        };
        unsafe {
            let t = crate::lua_type(lua, index);
            let value = match t {
                crate::LUA_TBOOLEAN => {
                    let val: bool =
                        unwrap_or!(LuaRead::lua_read_at_position(lua, index), return None);
                    Some(Value::from(val))
                }
                crate::LUA_TNUMBER => {
                    let val: f64 =
                        unwrap_or!(LuaRead::lua_read_at_position(lua, index), return None);
                    if val - val.floor() > 0.00001 {
                        Some(Value::from(val as f64))
                    } else {
                        Some(Value::from(val as i64))
                    }
                }
                crate::LUA_TSTRING => {
                    let mut dst = unwrap_or!(Self::read_str_to_vec(lua, index), return None);

                    if dst.len() > 4
                        && dst[0] == 140
                        && dst[1] == 150
                        && dst[2] == 141
                        && dst[3] == 151
                    {
                        dst.drain(0..4);
                        return Some(Value::from(dst));
                    }

                    if let Some(val) = str::from_utf8(&dst).ok() {
                        Some(Value::Str(val.to_string()))
                    } else {
                        Some(Value::from(dst))
                    }
                }
                crate::LUA_TTABLE => {
                    if !crate::lua_istable(lua, index) {
                        return None;
                    }
                    let len = crate::lua_rawlen(lua, index);
                    if len > 0 {
                        let mut val: Vec<Value> = Vec::new();
                        for i in 1..(len + 1) {
                            crate::lua_pushnumber(lua, i as f64);
                            let new_index = if index < 0 { index - 1 } else { index };
                            crate::lua_gettable(lua, new_index);
                            let sub_val = SerUtils::lua_read_value(lua, -1, stack + 1);
                            if sub_val.is_none() {
                                return None;
                            }
                            val.push(sub_val.unwrap());
                            crate::lua_pop(lua, 1);
                        }
                        Some(Value::from(val))
                    } else {
                        let mut val: HashMap<Value, Value> = HashMap::new();
                        crate::lua_pushnil(lua);
                        let t = if index < 0 { index - 1 } else { index };

                        while crate::lua_istable(lua, t) && crate::lua_next(lua, t) != 0 {
                            let sub_val = unwrap_or!(
                                SerUtils::lua_read_value(lua, -1, stack + 1),
                                return None
                            );
                            let value = if crate::lua_isnumber(lua, -2) != 0 {
                                let idx: u32 =
                                    unwrap_or!(LuaRead::lua_read_at_position(lua, -2), return None);
                                Value::from(idx)
                            } else {
                                let key: String =
                                    unwrap_or!(LuaRead::lua_read_at_position(lua, -2), return None);
                                Value::from(key)
                            };
                            val.insert(value, sub_val);
                            crate::lua_pop(lua, 1);
                        }
                        Some(Value::from(val))
                    }
                }
                _ => Some(Value::Nil),
            };
            value
        }
    }

    pub fn lua_convert_value(lua: *mut lua_State, index: i32) -> Option<Vec<Value>> {
        let size = if index < 0 {
            -index
        } else {
            unsafe { crate::lua_gettop(lua) - index + 1 }
        };
        let is_neg = index < 0;

        let mut val: Vec<Value> = Vec::new();
        for i in 0..size {
            let sub_val =
                SerUtils::lua_read_value(lua, if is_neg { index - i } else { i + index }, 0);
            if sub_val.is_none() {
                return None;
            }
            val.push(sub_val.unwrap());
        }
        Some(val)
    }
}
