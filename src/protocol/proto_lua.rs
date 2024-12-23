use crate::{Lua, LuaPush, LuaWrapperValue};
use hcproto::Buffer;
use log::warn;

use super::{LuaWrapperTableValue, SerUtils};

pub struct ProtoLua;

impl ProtoLua {
    pub fn pack_protocol(lua: *mut crate::lua_State, index: i32) -> Option<Buffer> {
        let value = SerUtils::lua_convert_value(lua, index);
        if value.is_none() {
            warn!("pack_protocol failed");
            return None;
        }
        let value = value.unwrap();
        let mut buffer = Buffer::new();
        unwrap_or!(hcproto::encode_msg(&mut buffer, value).ok(), return None);
        if buffer.len() > 0xFFFFFF {
            println!("pack message(lua msg) size > 0xFFFF fail!");
            return None;
        }
        Some(buffer)
    }

    pub fn unpack_protocol(lua: *mut crate::lua_State, buffer: &mut Buffer) -> i32 {
        if let Ok(val) = hcproto::decode_msg(buffer) {
            LuaWrapperTableValue(val).push_to_lua(lua);
            return 1;
        } else {
            return 0;
        }
    }

    pub fn ser_protocol(lua: *mut crate::lua_State, index: i32) -> Option<Buffer> {
        let t = unsafe { crate::lua_type(lua, index) };
        if t == crate::LUA_TNIL {
            return None;
        }
        if t != crate::LUA_TTABLE {
            Lua::lua_error(lua, "类型必须为table");
            return None;
        }
        let value = SerUtils::lua_read_value(lua, index, 0);
        if value.is_none() {
            warn!("pack_protocol failed");
            return None;
        }
        let value = value.unwrap();
        let mut buffer = Buffer::new();
        if let Err(e) = hcproto::encode_msg_map(&mut buffer, value) {
            Lua::lua_error(lua, format!("序列化错误:{:?}", e));
            return None;
        }
        if buffer.len() > 0xFFFFFF {
            Lua::lua_error(lua, "pack message(lua msg) size > 0xFFFF fail!");
            return None;
        }
        Some(buffer)
    }

    pub fn des_protocol(lua: *mut crate::lua_State, buffer: &mut Buffer) -> i32 {
        if let Ok(val) = hcproto::decode_msg_map(buffer) {
            LuaWrapperValue(val).push_to_lua(lua);
            return 1;
        } else {
            return 0;
        }
    }
}
