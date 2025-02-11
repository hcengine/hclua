use crate::{
    impl_box_push, lua_State, lua_pushnil, sys, LuaPush, LuaRead, LuaWrapperValue, ProtoLua,
    WrapSerde,
};
use hcproto::Value;
use libc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, ffi::CString, fmt::Debug, net::SocketAddr, ptr};

pub struct RawString(pub Vec<u8>);

pub struct WrapperObject<T>(pub T);

macro_rules! integer_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { sys::lua_pushinteger(lua, self as sys::lua_Integer) };
                1
            }
            fn box_push_to_lua(self: Box<Self>, lua: *mut lua_State) -> i32
            {
                (*self).push_to_lua(lua)
            }
        }

        impl LuaPush for &$t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { sys::lua_pushinteger(lua, *self as sys::lua_Integer) };
                1
            }

            fn box_push_to_lua(self: Box<Self>, lua: *mut lua_State) -> i32
            {
                (*self).push_to_lua(lua)
            }
        }

        impl LuaRead for $t {
            fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<$t> {
                let mut success = 0;
                let val = unsafe { sys::lua_tointegerx(lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

integer_impl!(i8);
integer_impl!(i16);
integer_impl!(i32);
integer_impl!(i64);
integer_impl!(u8);
integer_impl!(u16);
integer_impl!(u32);
integer_impl!(u64);
integer_impl!(usize);

macro_rules! numeric_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { sys::lua_pushnumber(lua, self as f64) };
                1
            }
            fn box_push_to_lua(self: Box<Self>, lua: *mut lua_State) -> i32
            {
                (*self).push_to_lua(lua)
            }
        }

        impl LuaPush for &$t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { sys::lua_pushnumber(lua, *self as f64) };
                1
            }

            fn box_push_to_lua(self: Box<Self>, lua: *mut lua_State) -> i32
            {
                (*self).push_to_lua(lua)
            }
        }

        impl LuaRead for $t {
            fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<$t> {
                let mut success = 0;
                let val = unsafe { sys::lua_tonumberx(lua, index, &mut success) };
                match success {
                    0 => None,
                    _ => Some(val as $t)
                }
            }
        }
    );
);

numeric_impl!(f32);
numeric_impl!(f64);

impl LuaPush for &String {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        if let Some(value) = CString::new(&self[..]).ok() {
            unsafe { sys::lua_pushstring(lua, value.as_ptr()) };
            1
        } else {
            unsafe { sys::lua_pushstring(lua, cstr!("UNVAILED STRING")) };
            1
        }
    }
    impl_box_push!();
}

impl LuaPush for String {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        (&self).push_to_lua(lua)
    }
    impl_box_push!();
}

impl LuaRead for String {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<String> {
        if unsafe { sys::lua_isstring(lua, index) == 0 } {
            return None;
        }
        let mut size = 0;
        let data = unsafe { sys::lua_tolstring(lua, index, &mut size) };
        let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, size) };
        match std::str::from_utf8(bytes) {
            Ok(v) => Some(v.to_string()),
            Err(_) => Some("raw binary string...".to_string()),
        }
    }
}

impl LuaPush for &CString {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushstring(lua, self.as_ptr()) };
        1
    }
    impl_box_push!();
}

impl LuaPush for CString {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        (&self).push_to_lua(lua)
    }
    impl_box_push!();
}

impl LuaRead for CString {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<CString> {
        let mut size = 0;
        let data = unsafe { sys::lua_tolstring(lua, index, &mut size) };
        let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, size) };
        match std::str::from_utf8(bytes) {
            Ok(v) => CString::new(v).ok(),
            Err(_) => None,
        }
    }
}

impl<'s> LuaPush for &'s str {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        if let Some(value) = CString::new(&self[..]).ok() {
            unsafe { sys::lua_pushstring(lua, value.as_ptr()) };
            1
        } else {
            unsafe { sys::lua_pushstring(lua, cstr!("UNVAILED STRING")) };
            1
        }
    }
    impl_box_push!();
}

impl LuaPush for bool {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushboolean(lua, self.clone() as libc::c_int) };
        1
    }
    impl_box_push!();
}

impl LuaPush for &bool {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushboolean(lua, self.clone() as libc::c_int) };
        1
    }
    impl_box_push!();
}

impl LuaRead for bool {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<bool> {
        if unsafe { !sys::lua_isboolean(lua, index) } {
            return None;
        }

        Some(unsafe { sys::lua_toboolean(lua, index) != 0 })
    }
}

impl LuaPush for () {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushnil(lua) };
        1
    }
    impl_box_push!();
}

impl LuaPush for &() {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushnil(lua) };
        1
    }
    impl_box_push!();
}

impl LuaRead for () {
    fn lua_read_with_pop_impl(_: *mut lua_State, _: i32, _pop: i32) -> Option<()> {
        Some(())
    }
}

impl LuaPush for &RawString {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushlstring(lua, self.0.as_ptr() as *const i8, self.0.len()) };
        1
    }
    impl_box_push!();
}

impl LuaPush for RawString {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushlstring(lua, self.0.as_ptr() as *const i8, self.0.len()) };
        1
    }
    impl_box_push!();
}

impl LuaRead for RawString {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<RawString> {
        let mut size: libc::size_t = 0;
        let c_str_raw = unsafe { sys::lua_tolstring(lua, index, &mut size) };
        if c_str_raw.is_null() {
            return None;
        }

        unsafe {
            let mut dst: Vec<u8> = Vec::with_capacity(size);
            ptr::copy(c_str_raw as *mut u8, dst.as_mut_ptr(), size);
            dst.set_len(size);
            Some(RawString(dst))
        }
    }
}

impl<T: LuaPush> LuaPush for Option<T> {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        if let Some(v) = self {
            v.push_to_lua(lua)
        } else {
            unsafe { lua_pushnil(lua) };
            1
        }
    }
    impl_box_push!();
}

impl<T: LuaPush> LuaPush for Box<T> {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        (*self).push_to_lua(lua)
    }
    impl_box_push!();
}

impl<'a, T> LuaPush for &'a Option<T>
where
    &'a T: LuaPush,
{
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        if let Some(v) = self {
            v.push_to_lua(lua)
        } else {
            unsafe { lua_pushnil(lua) };
            1
        }
    }

    impl_box_push!();
}

impl LuaPush for SocketAddr {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        format!("{}", self).push_to_lua(lua)
    }

    impl_box_push!();
}

impl<T: LuaRead> LuaRead for Option<T> {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, pop: i32) -> Option<Self> {
        Some(T::lua_read_with_pop_impl(lua, index, pop))
    }
}

impl<T: LuaPush, E: Debug> LuaPush for Result<T, E> {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        match self {
            Ok(t) => t.push_to_lua(lua),
            Err(e) => {
                crate::Lua::lua_error(lua, format!("序列化错误:{:?}", e));
                unreachable!()
            }
        }
    }
}

impl<T: Serialize> LuaPush for WrapSerde<T> {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        let mut buffer = unwrap_or!(hcproto::to_buffer(&self.value).ok(), return 0);
        if let Ok(mut val) = hcproto::decode_msg(&mut buffer) {
            let mut map = HashMap::new();
            while val.len() >= 2 {
                let v = val.pop().unwrap();
                let k = val.pop().unwrap();
                map.insert(k, v);
            }
            LuaWrapperValue(Value::Map(map)).push_to_lua(lua);
            return 1;
        } else {
            return 0;
        }
    }

    impl_box_push!();
}

impl<'a, T: Deserialize<'a>> LuaRead for WrapSerde<T> {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _: i32) -> Option<Self> {
        let buffer = unwrap_or!(ProtoLua::ser_protocol(lua, index), return None);
        let ret = hcproto::from_buffer(buffer);
        match ret {
            Err(e) => {
                crate::Lua::lua_error(lua, format!("序列化错误:{:?}", e));
                return None;
            }
            Ok(v) => return Some(WrapSerde::new(v)),
        }
    }
}

impl<'a, T: 'static> LuaRead for WrapperObject<T> {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<Self> {
        println!("aaaaaaa WrapperObject aaaaaaaaa");
        let obj: Option<&mut T> = crate::userdata::read_wrapper_light_userdata(lua, index);
        println!("bbbbbbb WrapperObject bbbbbbbb");
        match obj {
            Some(v) => unsafe {
                let v = Box::from_raw(v);
                Some(WrapperObject(*v))
            },
            None => None,
        }
    }
}
