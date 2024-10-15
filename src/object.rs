use libc::c_char;
use std::{
    any::{Any, TypeId},
    ffi::CString,
    marker::PhantomData,
    mem, ptr,
};

use crate::{
    lua_State, lua_call, lua_getfield, lua_pop, lua_pushvalue, push_lightuserdata, sys, Lua,
    LuaPush, LuaRead, LuaTable,
};

// Called when an object inside Lua is being dropped.
#[inline]
extern "C" fn destructor_wrapper<T>(lua: *mut sys::lua_State) -> libc::c_int {
    unsafe {
        let obj = sys::lua_touserdata(lua, -1);
        ptr::drop_in_place(obj as *mut T);
        0
    }
}

extern "C" fn constructor_wrapper<T>(lua: *mut sys::lua_State) -> libc::c_int
where
    T: Default + Any,
{
    let t = T::default();
    let lua_data_raw = unsafe { sys::lua_newuserdata(lua, mem::size_of::<T>() as libc::size_t) };
    unsafe {
        ptr::write(lua_data_raw as *mut _, t);
    }
    let typeid = CString::new(format!("{:?}", TypeId::of::<T>())).unwrap();
    unsafe {
        sys::lua_getglobal(lua, typeid.as_ptr());
        sys::lua_setmetatable(lua, -2);
    }
    1
}

fn get_metatable_base_key<T: Any>() -> CString {
    CString::new(format!("{:?}", TypeId::of::<T>())).unwrap()
}

fn get_metatable_real_key<T: Any>() -> CString {
    CString::new(format!("{:?}_real", TypeId::of::<T>())).unwrap()
}

fn check_is_field_key(name: &str) -> CString {
    CString::new(format!("{}__isfield", name)).unwrap()
}

fn check_set_field_key(name: &str) -> CString {
    CString::new(format!("{}__set", name)).unwrap()
}

extern "C" fn index_metatable<'a, T>(lua: *mut sys::lua_State) -> libc::c_int
where
    T: Default + Any,
    &'a mut T: LuaRead,
{
    if let Some(key) = String::lua_read_with_pop(lua, 2, 0) {
        let typeid = get_metatable_real_key::<T>();
        unsafe {
            sys::lua_getglobal(lua, typeid.as_ptr());
            let check_key = check_is_field_key(&key);
            let t = lua_getfield(lua, -1, check_key.as_ptr());
            lua_pop(lua, 1);
            let is_field = t == sys::LUA_TBOOLEAN;
            let key = CString::new(key).unwrap();
            let t = lua_getfield(lua, -1, key.as_ptr());
            if t != sys::LUA_TFUNCTION || !is_field {
                return 1;
            }
            lua_pushvalue(lua, 1);
            lua_call(lua, 1, 1);
            1
        }
    } else {
        0
    }
}

extern "C" fn newindex_metatable<'a, T>(lua: *mut sys::lua_State) -> libc::c_int
where
    T: Default + Any,
    &'a mut T: LuaRead,
{
    println!("newindex_metatable!!!!!!!!!!!!!!!!");
    if let Some(mut key) = String::lua_read_with_pop(lua, 2, 0) {
        key.push_str("__set");
        let typeid = get_metatable_real_key::<T>();
        unsafe {
            sys::lua_getglobal(lua, typeid.as_ptr());
            let key = CString::new(key).unwrap();
            let t = lua_getfield(lua, -1, key.as_ptr());
            if t != sys::LUA_TFUNCTION {
                return 0;
            }
            lua_pushvalue(lua, 1);
            lua_pushvalue(lua, 3);
            lua_call(lua, 2, 1);
            1
        }
    } else {
        0
    }
}

// constructor direct create light object,
// in rust we alloc the memory, avoid copy the memory
// in lua we get the object, we must free the memory
extern "C" fn constructor_light_wrapper<T>(lua: *mut sys::lua_State) -> libc::c_int
where
    T: Default + Any,
{
    let t = Box::into_raw(Box::new(T::default()));
    push_lightuserdata(unsafe { &mut *t }, lua, |_| {});

    let typeid = get_metatable_base_key::<T>();
    unsafe {
        sys::lua_getglobal(lua, typeid.as_ptr());
        sys::lua_setmetatable(lua, -2);
    }
    1
}

pub struct LuaObject<'a, T>
where
    T: Default + Any,
    &'a mut T: LuaRead,
{
    lua: *mut lua_State,
    light: bool,
    name: &'static str,
    marker: PhantomData<&'a T>,
}

impl<'a, T> LuaObject<'a, T>
where
    T: Default + Any,
    &'a mut T: LuaRead,
{
    pub fn test() {
        println!("aaa");
    }
    pub fn new(lua: *mut lua_State, name: &'static str) -> LuaObject<'a, T> {
        LuaObject {
            lua,
            light: false,
            name,
            marker: PhantomData,
        }
    }

    pub fn new_light(lua: *mut lua_State, name: &'static str) -> LuaObject<'a, T> {
        LuaObject {
            lua,
            light: true,
            name,
            marker: PhantomData,
        }
    }

    pub fn ensure_matetable(&mut self) -> bool {
        let typeid = get_metatable_base_key::<T>();
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.queryc::<LuaTable>(&typeid) {
            Some(_) => true,
            None => unsafe {
                sys::lua_newtable(self.lua);
                // index "__name" corresponds to the hash of the TypeId of T
                "__typeid".push_to_lua(self.lua);
                (&typeid).push_to_lua(self.lua);
                sys::lua_settable(self.lua, -3);

                // index "__gc" call the object's destructor
                if !self.light {
                    "__gc".push_to_lua(self.lua);

                    sys::lua_pushcfunction(self.lua, destructor_wrapper::<T>);

                    sys::lua_settable(self.lua, -3);
                }

                "__index".push_to_lua(self.lua);
                sys::lua_pushcfunction(self.lua, index_metatable::<T>);
                // sys::lua_newtable(self.lua);
                sys::lua_rawset(self.lua, -3);

                "__newindex".push_to_lua(self.lua);
                sys::lua_pushcfunction(self.lua, newindex_metatable::<T>);
                sys::lua_rawset(self.lua, -3);

                sys::lua_setglobal(self.lua, typeid.as_ptr() as *const c_char);

                let typeid = get_metatable_real_key::<T>();
                sys::lua_newtable(self.lua);
                sys::lua_setglobal(self.lua, typeid.as_ptr());
                false
            },
        }
    }

    pub fn create(&mut self) -> &mut LuaObject<'a, T> {
        self.ensure_matetable();
        unsafe {
            let name = CString::new(self.name).unwrap();
            if self.light {
                sys::lua_pushcfunction(self.lua, constructor_light_wrapper::<T>);
            } else {
                sys::lua_pushcfunction(self.lua, constructor_wrapper::<T>);
            }
            sys::lua_setglobal(self.lua, name.as_ptr());
        }
        self
    }

    pub fn add_object_method_get<P>(lua: &mut Lua, name: &str, param: P)
    where
        P: LuaPush,
    {
        let typeid = get_metatable_real_key::<T>();
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.set(name, param);
            }
            None => (),
        };
    }

    pub fn add_method_get<P>(&mut self, name: &str, param: P) -> &mut LuaObject<'a, T>
    where
        P: LuaPush,
    {
        let mut lua = Lua::from_existing_state(self.lua, false);
        Self::add_object_method_get(&mut lua, name, param);
        self
    }

    
    pub fn add_object_method_set<P>(lua: &mut Lua, name: &str, param: P)
    where
        P: LuaPush,
    {
        let typeid = get_metatable_real_key::<T>();
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.set(check_set_field_key(name), param);
            }
            None => (),
        };
    }

    pub fn add_method_set<P>(&mut self, name: &str, param: P) -> &mut LuaObject<'a, T>
    where
        P: LuaPush,
    {
        let mut lua = Lua::from_existing_state(self.lua, false);
        Self::add_object_method_set(&mut lua, name, param);
        self
    }

    pub fn mark_field(&mut self, name: &str) -> &mut LuaObject<'a, T> {
        let typeid = get_metatable_real_key::<T>();
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.set(check_is_field_key(name), true);
            }
            None => (),
        };
        self
    }

    pub fn def<P>(&mut self, name: &str, param: P) -> &mut LuaObject<'a, T>
    where
        P: LuaPush,
    {
        let mut lua = Lua::from_existing_state(self.lua, false);
        Self::object_def(&mut lua, name, param);
        self
    }
    
    pub fn object_def<P>(lua: &mut Lua, name: &str, param: P)
    where
        P: LuaPush,
    {
        let typeid = get_metatable_real_key::<T>();
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.set(name, param);
            }
            None => (),
        };
    }

    pub fn register(
        &mut self,
        name: &str,
        func: extern "C" fn(*mut sys::lua_State) -> libc::c_int,
    ) -> &mut LuaObject<'a, T> {
        let typeid = get_metatable_real_key::<T>();
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.register(name, func);
            }
            None => (),
        };
        self
    }
}

#[macro_export]
macro_rules! add_object_field {
    ($userdata: expr, $name: ident, $t: ty, $field_type: ty) => {
        $userdata.add_method_get(
            &format!("{}", stringify!($name)),
            hclua::function1(|obj: &mut $t| -> &$field_type {
                println!("aaaa");
                &obj.$name
            }),
        );
        $userdata.add_method_set(
            stringify!($name),
            hclua::function2(|obj: &mut $t, val: $field_type| {
                println!("bbbb {}", val);
                obj.$name = val;
            }),
        );
        $userdata.mark_field(stringify!($name));
    };
}

#[macro_export]
macro_rules! object_impl {
    ($t: ty) => {
        impl<'a> LuaRead for &'a mut $t {
            fn lua_read_with_pop_impl(
                lua: *mut lua_State,
                index: i32,
                _pop: i32,
            ) -> Option<&'a mut $t> {
                hclua::userdata::read_userdata(lua, index)
            }
        }

        impl LuaPush for $t {
            fn push_to_lua(self, lua: *mut lua_State) -> i32 {
                unsafe {
                    let obj = std::boxed::Box::into_raw(std::boxed::Box::new(self));
                    hclua::userdata::push_lightuserdata(&mut *obj, lua, |_| {});
                    let typeid =
                        std::ffi::CString::new(format!("{:?}", std::any::TypeId::of::<$t>()))
                            .unwrap();
                    hclua::lua_getglobal(lua, typeid.as_ptr());
                    if hclua::lua_istable(lua, -1) {
                        hclua::lua_setmetatable(lua, -2);
                    } else {
                        hclua::lua_pop(lua, 1);
                    }
                    1
                }
            }
        }
    };
}
