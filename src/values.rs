use std::ffi::CString;
use libc;
use crate::{lua_State, lua_pushnil, sys, LuaPush, LuaRead} ;

pub struct RawString(pub Vec<u8>);

macro_rules! integer_impl(
    ($t:ident) => (
        impl LuaPush for $t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { sys::lua_pushinteger(lua, self as sys::lua_Integer) };
                1
            }
        }

        impl LuaPush for &$t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { sys::lua_pushinteger(lua, *self as sys::lua_Integer) };
                1
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
        }

        impl LuaPush for &$t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe { sys::lua_pushnumber(lua, *self as f64) };
                1
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
}

impl LuaPush for String {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        (&self).push_to_lua(lua)
    }
}

impl LuaRead for String {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<String> {
        if unsafe {
            sys::lua_isstring(lua, index) == 0
        } {
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
}

impl LuaPush for CString {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        (&self).push_to_lua(lua)
    }
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
}

impl LuaPush for bool {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushboolean(lua, self.clone() as libc::c_int) };
        1
    }
}

impl LuaPush for &bool {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushboolean(lua, self.clone() as libc::c_int) };
        1
    }
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
}

impl LuaPush for &() {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushnil(lua) };
        1
    }
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
}

impl LuaPush for RawString {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        unsafe { sys::lua_pushlstring(lua, self.0.as_ptr() as *const i8, self.0.len()) };
        1
    }
}

impl LuaRead for RawString {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, _pop: i32) -> Option<RawString> {
        let mut size: libc::size_t = 0;
        let c_str_raw = unsafe { sys::lua_tolstring(lua, index, &mut size) };
        if c_str_raw.is_null() {
            return None;
        }

        let value = unsafe { Vec::from_raw_parts(c_str_raw as *mut u8, size, size) };
        Some(RawString(value))
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
}

impl<'a, T> LuaPush for &'a Option<T>
where &'a T: LuaPush {
    fn push_to_lua(self, lua: *mut lua_State) -> i32 {
        if let Some(v) = self {
            v.push_to_lua(lua)
        } else {
            unsafe { lua_pushnil(lua) };
            1
        }
    }
}

impl<T: LuaRead> LuaRead for Option<T> {
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, pop: i32) -> Option<Self> {
        Some(T::lua_read_with_pop_impl(lua, index, pop))
    }
}
