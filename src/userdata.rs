use std::any::{Any, TypeId};
use std::mem;
use std::ptr;

use crate::{sys, LuaPush, LuaRead, LuaTable};

// Called when an object inside Lua is being dropped.
#[inline]
extern "C" fn destructor_wrapper<T>(lua: *mut sys::lua_State) -> libc::c_int {
    unsafe {
        let obj = sys::lua_touserdata(lua, -1);
        ptr::drop_in_place(obj as *mut T);
        0
    }
}

/// Pushes an object as a user data.
///
/// In Lua, a user data is anything that is not recognized by Lua. When the script attempts to
/// copy a user data, instead only a reference to the data is copied.
///
/// The way a Lua script can use the user data depends on the content of the **metatable**, which
/// is a Lua table linked to the object.
///
/// # Arguments
///
///  - `metatable`: Function that fills the metatable of the object.
///
pub fn push_userdata<'a, T, F>(data: T, lua: *mut sys::lua_State, mut metatable: F) -> i32
where
    F: FnMut(LuaTable),
    T: 'a + Any,
{
    let typeid = format!("{:?}", TypeId::of::<T>());
    let lua_data_raw =
        unsafe { sys::lua_newuserdata(lua, mem::size_of::<T>() as libc::size_t) };

    // creating a metatable
    unsafe {
        ptr::write(lua_data_raw as *mut _, data);

        sys::lua_newtable(lua);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        sys::lua_settable(lua, -3);

        // index "__gc" call the object's destructor
        {
            "__gc".push_to_lua(lua);

            sys::lua_pushcfunction(lua, destructor_wrapper::<T>);

            sys::lua_settable(lua, -3);
        }

        // calling the metatable closure
        {
            metatable(LuaRead::lua_read(lua).unwrap());
        }

        sys::lua_setmetatable(lua, -2);
    }

    1
}

/// Pushes an object as a user data.
///
/// In Lua, a user data is anything that is not recognized by Lua. When the script attempts to
/// copy a user data, instead only a reference to the data is copied.
///
/// The way a Lua script can use the user data depends on the content of the **metatable**, which
/// is a Lua table linked to the object.
///
/// # Arguments
///
///  - `metatable`: Function that fills the metatable of the object.
///
pub fn push_lightuserdata<'a, T, F>(
    data: &'a mut T,
    lua: *mut sys::lua_State,
    mut metatable: F,
) -> i32
where
    F: FnMut(LuaTable),
    T: 'a + Any,
{
    let typeid = format!("{:?}", TypeId::of::<T>());
    unsafe {
        sys::lua_pushlightuserdata(lua, mem::transmute(data));
    };

    // creating a metatable
    unsafe {
        sys::lua_newtable(lua);

        // index "__typeid" corresponds to the hash of the TypeId of T
        "__typeid".push_to_lua(lua);
        typeid.push_to_lua(lua);
        sys::lua_settable(lua, -3);

        // calling the metatable closure
        {
            metatable(LuaRead::lua_read(lua).unwrap());
        }

        sys::lua_setmetatable(lua, -2);
    }

    1
}

///
pub fn read_userdata<'t, 'c, T>(lua: *mut sys::lua_State, index: i32) -> Option<&'t mut T>
where
    T: 'static + Any,
{
    unsafe {
        let expected_typeid = format!("{:?}", TypeId::of::<T>());
        if sys::lua_isuserdata(lua, index) == 0 {
            return None;
        }
        let data_ptr = sys::lua_touserdata(lua, index);
        if data_ptr.is_null() {
            return None;
        }
        if sys::lua_getmetatable(lua, index) == 0 {
            return None;
        }
        
        "__typeid".push_to_lua(lua);
        sys::lua_gettable(lua, -2);
        match <String as LuaRead>::lua_read(lua) {
            Some(ref val) if val == &expected_typeid => {}
            _ => {
                sys::lua_pop(lua, 2);
                return None;
            }
        }
        sys::lua_pop(lua, 2);
        Some(mem::transmute(data_ptr))
    }
}
