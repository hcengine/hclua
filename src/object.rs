use lazy_static::lazy_static;
use libc::c_char;
use std::collections::HashSet;
use std::{
    any::{Any, TypeId},
    ffi::CString,
    marker::PhantomData,
    mem, ptr,
    sync::RwLock,
};

use crate::{
    luaL_error, lua_State, lua_call, lua_error, lua_getfield, lua_gettop, lua_pushvalue,
    push_lightuserdata, sys, Lua, LuaPush, LuaRead, LuaTable,
};

lazy_static! {
    static ref FIELD_CHECK: RwLock<HashSet<(TypeId, &'static str)>> = RwLock::new(HashSet::new());
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
    pub fn is_field(name: &str) -> bool {
        let val = FIELD_CHECK.read().unwrap();
        val.contains(&(TypeId::of::<T>(), name))
    }

    pub fn set_field(name: &'static str) {
        let mut val = FIELD_CHECK.write().unwrap();
        val.insert((TypeId::of::<T>(), name));
    }

    fn get_metatable_base_key() -> CString {
        CString::new(format!("{:?}", TypeId::of::<T>())).unwrap()
    }

    fn get_metatable_real_key() -> CString {
        CString::new(format!("{:?}_real", TypeId::of::<T>())).unwrap()
    }

    fn get_set_field_key(name: &str) -> CString {
        CString::new(format!("{}__set", name)).unwrap()
    }

    /// 元表的index操作, 处理字段及函数的映射
    extern "C" fn index_metatable(lua: *mut sys::lua_State) -> libc::c_int {
        unsafe {
            if lua_gettop(lua) < 2 {
                let value = CString::new(format!("index field must use 2 top")).unwrap();
                return luaL_error(lua, value.as_ptr());
            }
        }
        if let Some(key) = String::lua_read_with_pop(lua, 2, 0) {
            let typeid = Self::get_metatable_real_key();
            unsafe {
                sys::lua_getglobal(lua, typeid.as_ptr());
                let is_field = LuaObject::is_field(&*key);
                let key = CString::new(key).unwrap();
                let t = lua_getfield(lua, -1, key.as_ptr());
                if !is_field {
                    if t == sys::LUA_TFUNCTION {
                        return 1;
                    } else {
                        return 1;
                    }
                }
                lua_pushvalue(lua, 1);
                lua_call(lua, 1, 1);
                1
            }
        } else {
            0
        }
    }

    extern "C" fn newindex_metatable(lua: *mut sys::lua_State) -> libc::c_int {
        if let Some(mut key) = String::lua_read_with_pop(lua, 2, 0) {
            if !LuaObject::is_field(&*key) {
                let value = CString::new(format!("key {key} not a field")).unwrap();
                unsafe {
                    return luaL_error(lua, value.as_ptr());
                }
            }
            key.push_str("__set");
            let typeid = Self::get_metatable_real_key();
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

    extern "C" fn constructor_wrapper(lua: *mut sys::lua_State) -> libc::c_int {
        let t = T::default();
        let lua_data_raw =
            unsafe { sys::lua_newuserdata(lua, mem::size_of::<T>() as libc::size_t) };
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

    // constructor direct create light object,
    // in rust we alloc the memory, avoid copy the memory
    // in lua we get the object, we must free the memory
    extern "C" fn constructor_light_wrapper(lua: *mut sys::lua_State) -> libc::c_int {
        let t = Box::into_raw(Box::new(T::default()));
        push_lightuserdata(unsafe { &mut *t }, lua, |_| {});

        let typeid = Self::get_metatable_base_key();
        unsafe {
            sys::lua_getglobal(lua, typeid.as_ptr());
            sys::lua_setmetatable(lua, -2);
        }
        1
    }

    #[inline]
    extern "C" fn destructor_light_wrapper(lua: *mut sys::lua_State) -> libc::c_int
    where
        &'a mut T: LuaRead,
    {
        let msg: &mut T = unwrap_or!(crate::LuaRead::lua_read_at_position(lua, 1), return 0);
        unsafe {
            sys::lua_pushnil(lua);
            sys::lua_setmetatable(lua, 1);
        }
        let _msg = unsafe { Box::from_raw(msg) };
        0
    }

    #[inline]
    extern "C" fn destructor_bad_wrapper(lua: *mut sys::lua_State) -> libc::c_int {
        unsafe {
            "usedata object must not del, beacuse it belong to lua gc".push_to_lua(lua);
            lua_error(lua)
        }
    }

    #[inline]
    extern "C" fn destructor_wrapper(lua: *mut sys::lua_State) -> libc::c_int {
        unsafe {
            let obj = sys::lua_touserdata(lua, -1);
            ptr::drop_in_place(obj as *mut T);
            0
        }
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
        let typeid = Self::get_metatable_base_key();
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

                    sys::lua_pushcfunction(self.lua, Self::destructor_wrapper);

                    sys::lua_settable(self.lua, -3);
                }

                "__index".push_to_lua(self.lua);
                sys::lua_pushcfunction(self.lua, Self::index_metatable);
                // sys::lua_newtable(self.lua);
                sys::lua_rawset(self.lua, -3);

                "__newindex".push_to_lua(self.lua);
                sys::lua_pushcfunction(self.lua, Self::newindex_metatable);
                sys::lua_rawset(self.lua, -3);

                sys::lua_setglobal(self.lua, typeid.as_ptr() as *const c_char);

                let typeid = Self::get_metatable_real_key();
                sys::lua_newtable(self.lua);
                sys::lua_setglobal(self.lua, typeid.as_ptr());
                false
            },
        }
    }

    pub fn ensure_table(&mut self) {
        let name = CString::new(self.name).unwrap();
        let mut lua = Lua::from_existing_state(self.lua, false);
        if lua.queryc::<LuaTable>(&name).is_none() {
            unsafe {
                sys::lua_newtable(self.lua);
                // index "__name" corresponds to the hash of the TypeId of T
                "new".push_to_lua(self.lua);
                if self.light {
                    sys::lua_pushcfunction(self.lua, Self::constructor_light_wrapper);
                } else {
                    sys::lua_pushcfunction(self.lua, Self::constructor_wrapper);
                }
                sys::lua_settable(self.lua, -3);

                "del".push_to_lua(self.lua);
                if self.light {
                    sys::lua_pushcfunction(self.lua, Self::destructor_light_wrapper);
                } else {
                    sys::lua_pushcfunction(self.lua, Self::destructor_bad_wrapper);
                }
                sys::lua_settable(self.lua, -3);

                sys::lua_setglobal(self.lua, name.as_ptr() as *const c_char);
            }
        }
    }

    pub fn create(&mut self) -> &mut LuaObject<'a, T> {
        self.ensure_matetable();
        self.ensure_table();
        self
    }

    pub fn add_object_method_get<P>(lua: &mut Lua, name: &str, param: P)
    where
        P: LuaPush,
    {
        let typeid = Self::get_metatable_real_key();
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
        let typeid = Self::get_metatable_real_key();
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.set(Self::get_set_field_key(name), param);
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

    pub fn mark_field(&mut self, name: &'static str) -> &mut LuaObject<'a, T> {
        Self::set_field(name);
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
        let typeid = Self::get_metatable_real_key();
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.set(name, param);
            }
            None => (),
        };
    }

    pub fn static_def<P>(&mut self, name: &str, param: P) -> &mut LuaObject<'a, T>
    where
        P: LuaPush,
    {
        self.ensure_table();
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(self.name) {
            Some(mut table) => {
                table.set(name, param);
            }
            None => (),
        };
        self
    }

    pub fn register(
        &mut self,
        name: &str,
        func: extern "C" fn(*mut sys::lua_State) -> libc::c_int,
    ) -> &mut LuaObject<'a, T> {
        let mut lua = Lua::from_existing_state(self.lua, false);
        Self::object_register(&mut lua, name, func);
        self
    }

    pub fn object_register(
        lua: &mut Lua,
        name: &str,
        func: extern "C" fn(*mut sys::lua_State) -> libc::c_int,
    ) {
        let typeid = Self::get_metatable_real_key();
        match lua.queryc::<LuaTable>(&typeid) {
            Some(mut table) => {
                table.register(name, func);
            }
            None => (),
        };
    }

    pub fn static_register(
        &mut self,
        name: &str,
        func: extern "C" fn(*mut sys::lua_State) -> libc::c_int,
    ) {
        self.ensure_table();
        let mut lua = Lua::from_existing_state(self.lua, false);
        match lua.query::<LuaTable, _>(self.name) {
            Some(mut table) => {
                table.register(name, func);
            }
            None => (),
        };
    }
    
}

#[macro_export]
macro_rules! add_object_field {
    ($userdata: expr, $name: ident, $t: ty, $field_type: ty) => {
        $userdata.add_method_get(
            &format!("{}", stringify!($name)),
            hclua::function1(|obj: &mut $t| -> &$field_type { &obj.$name }),
        );
        $userdata.add_method_set(
            stringify!($name),
            hclua::function2(|obj: &mut $t, val: $field_type| {
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

pub struct WrapObject<T>(pub T);

impl<'a, T> LuaRead for WrapObject<T>
where
    T: Clone + 'a,
    &'a mut T: LuaRead,
{
    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, pop: i32) -> Option<Self> {
        let v: Option<&mut T> = LuaRead::lua_read_with_pop_impl(lua, index, pop);
        v.map(|v| WrapObject(v.clone()))
    }
}
