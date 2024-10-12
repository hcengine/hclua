
#[macro_use]
pub mod sys;
pub use sys::*;

use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::io::prelude::*;
use std::fs::File;

macro_rules! unwrap_or {
    ($expr:expr, $or:expr) => (
        match $expr {
            Some(x) => x,
            None => { $or }
        }
    )
}

pub mod values;
pub mod lua_tables;
pub mod functions;
pub mod userdata;
pub mod tuples;
pub mod rust_tables;
mod hotfix;
mod object;
mod module;

pub use functions::{function0, function1, function2, function3, function4, function5, function6, function7, function8, function9, function10, Function};
pub use userdata::{push_userdata, push_lightuserdata, read_userdata};
pub use lua_tables::LuaTable;
pub use values::RawString;
pub use object::LuaObject;
pub use module::LuaModule;

pub struct Lua {
    lua: *mut lua_State,
    own: bool,
}


pub struct LuaGuard {
    pub lua: *mut lua_State,
    pub size: i32,
}

impl LuaGuard {

    pub fn forget(mut self) -> i32 {
        let size = self.size;
        self.size = 0;
        size
    }

    pub fn empty(&self) -> LuaGuard {
        LuaGuard {
            lua: self.lua,
            size: 0,
        }
    }

    pub fn new_empty(lua: *mut lua_State) -> LuaGuard {
        LuaGuard {
            lua: lua,
            size: 0,
        }
    }

    pub fn new(lua: *mut lua_State, size: i32) -> LuaGuard {
        LuaGuard {
            lua: lua,
            size: size,
        }
    }
}


macro_rules! impl_exec_func {
    ($name:ident, $($p:ident),*) => (
        #[allow(non_snake_case, unused_mut)]
        pub fn $name<Z, $($p),*>(&mut self, func_name : Z, $($p : $p, )*) -> i32 where Z: Borrow<str>, $($p : LuaPush),* {
            let func_name = CString::new(func_name.borrow()).unwrap();
            unsafe {
                let state = self.state();
                lua_getglobal(state, cstr!("error_handle"));
                lua_getglobal(state, func_name.as_ptr());

                let mut index = 0;
                $(
                    index += $p.push_to_lua(self.state());
                )*

                let success = lua_pcall(state, index, 0, -1 * index - 2);
                if success != 0 {
                    let _guard = LuaGuard::new(state, 2);
                    return success;
                }
                lua_pop(state, 1);
                success
            }
        }
    )
}


macro_rules! impl_read_func {
    ($name:ident, $($p:ident),*) => (
        #[allow(non_snake_case, unused_mut)]
        pub fn $name<'a, Z, R, $($p),*>(&'a mut self, func_name : Z, $($p : $p, )*) -> Option<R> where Z: Borrow<str>, R : LuaRead, $($p : LuaPush),* {
            let func_name = CString::new(func_name.borrow()).unwrap();
            unsafe {
                let state = self.state();
                lua_getglobal(state, cstr!("error_handle"));
                lua_getglobal(state, func_name.as_ptr());

                let mut index = 0;
                $(
                    index += $p.push_to_lua(self.state());
                )*

                let success = lua_pcall(state, index, 1, -1 * index - 2);
                if success != 0 {
                    let _guard = LuaGuard::new(state, 2);
                    return None;
                }
                lua_remove(state, -2);
                LuaRead::lua_read_with_pop(state, -1, 1)
            }
        }
    )
}

impl Lua {
    /// Builds a new Lua context.
    ///
    /// # Panic
    ///
    /// The function panics if the underlying call to `luaL_newstate` fails
    /// (which indicates lack of memory).
    pub fn new() -> Lua {
        let lua = unsafe { luaL_newstate() };
        if lua.is_null() {
            panic!("lua_newstate failed");
        }

        // called whenever lua encounters an unexpected error.
        extern "C" fn panic(lua: *mut lua_State) -> libc::c_int {
            let err = unsafe { lua_tostring(lua, -1) };
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8_lossy(&err.to_bytes());
            panic!("PANIC: unprotected error in call to Lua API ({})\n", err);
        }

        extern "C" fn error_handle(lua: *mut lua_State) -> libc::c_int {
            let err = unsafe { lua_tostring(lua, -1) };
            let err = unsafe { CStr::from_ptr(err) };
            let err = String::from_utf8_lossy(&err.to_bytes());
            println!("error:{}", err);
            0
        }

        unsafe { lua_atpanic(lua, panic) };
        let mut lua = Lua {
            lua: lua,
            own: true,
        };
        lua.register("error_handle", error_handle);
        lua
    }

    pub fn state(&mut self) -> *mut lua_State {
        return self.lua;
    }

    pub fn clone(&mut self) -> Lua {
        Lua {
            lua : self.lua,
            own : false,
        }
    }

    pub fn set_own(&mut self, own: bool) {
        self.own = own;
    }

    /// Takes an existing `lua_State` and build a Lua object from it.
    ///
    /// # Arguments
    ///
    ///  * `close_at_the_end`: if true, lua_close will be called on the lua_State on the destructor
    pub fn from_existing_state(lua: *mut lua_State, close_at_the_end: bool) -> Lua {
        Lua {
            lua : lua,
            own: close_at_the_end,
        }
    }

    pub fn register<I>(&mut self, index : I, func : extern "C" fn(*mut lua_State) -> libc::c_int) -> i32
                    where I: Borrow<str>
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { lua_register(self.state(), index.as_ptr(), func) };
        0
    }

    /// Opens all standard Lua libraries.
    /// This is done by calling `luaL_openlibs`.
    pub fn openlibs(&mut self) {
        unsafe { luaL_openlibs(self.lua) }
    }

    /// Reads the value of a global variable.
    pub fn query<'l, V, I>(&'l mut self, index: I) -> Option<V>
                         where I: Borrow<str>, V: LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe { lua_getglobal(self.lua, index.as_ptr()); }
        LuaRead::lua_read_with_pop(self.state(), -1, 1)
    }


    /// Reads the value of a global variable.
    pub fn queryc<'l, V>(&'l mut self, index: &CString) -> Option<V>
                         where V: LuaRead
    {
        unsafe { lua_getglobal(self.lua, index.as_ptr()); }
        LuaRead::lua_read_with_pop(self.state(), -1, 1)
    }


    /// Modifies the value of a global variable.
    pub fn set<I, V>(&mut self, index: I, value: V)
                         where I: Borrow<str>, for<'a> V: LuaPush
    {
        let index = CString::new(index.borrow()).unwrap();
        value.push_to_lua(self.state());
        unsafe { lua_setglobal(self.lua, index.as_ptr()); }
    }

    /// Modifies the value of a global variable.
    pub fn setc<I, V>(&mut self, index: CString, value: V)
                         where for<'a> V: LuaPush
    {
        value.push_to_lua(self.state());
        unsafe { lua_setglobal(self.lua, index.as_ptr()); }
    }

    pub fn exec_string<'a, I, R>(&'a mut self, index : I) -> Option<R>
                            where I: Borrow<str>, R : LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe {
            let state = self.state();
            lua_getglobal(state, cstr!("error_handle"));
            luaL_loadstring(state, index.as_ptr());
            let success = lua_pcall(state, 0, 1, -2);
            if success != 0 {
                let _guard = LuaGuard::new(self.lua, 2);
                return None;
            }
            lua_remove(state, -2);
            LuaRead::lua_read_with_pop(state, -1, 1)
        }
    }

    pub fn exec_func<'a, I, R>(&'a mut self, index : I) -> Option<R>
                            where I: Borrow<str>, R : LuaRead
    {
        let index = CString::new(index.borrow()).unwrap();
        unsafe {
            let state = self.state();
            let top = lua_gettop(state);
            lua_getglobal(state, index.as_ptr());
            lua_insert(state, -top - 1);
            lua_getglobal(state, cstr!("error_handle"));
            lua_insert(state, -top - 2);
            let success = lua_pcall(state, top, 1, -top-2);
            if success != 0 {
                let _guard = LuaGuard::new(self.lua, 2);
                return None;
            }
            lua_remove(state, -2);
            LuaRead::lua_read_with_pop(state, -1, 1)
        }
    }

    /// Inserts an empty table, then loads it.
    pub fn empty_table<I>(&mut self, index: I) -> LuaTable
                              where I: Borrow<str>
    {
        let index2 = CString::new(index.borrow()).unwrap();
        unsafe { 
            lua_newtable(self.state());
            lua_setglobal(self.state(), index2.as_ptr()); 
        }
        self.query(index).unwrap()
    }

    pub fn add_lualoader(&mut self, func : extern "C" fn(*mut lua_State) -> libc::c_int) -> i32 {
        let state = self.state();
        unsafe {
            let package = cstr!("package");
            #[cfg(any(feature="lua51", feature="luajit"))]
            let searchers = cstr!("loaders");
            #[cfg(not(any(feature="lua51", feature="luajit")))]
            let searchers = cstr!("searchers");
            lua_getglobal(state, package);
            lua_getfield(state, -1, searchers);
            lua_pushcfunction(state, func);
            let mut i = (lua_rawlen(state, -2) + 1) as lua_Integer;
            while i > 2 {
                lua_rawgeti(state, -2, i - 1);                               
                lua_rawseti(state, -3, i);
                i = i - 1;
            }
            lua_rawseti(state, -2, 2);
            // set loaders into package
            lua_setfield(state, -2, searchers);                               
            lua_pop(state, 1);
        }
        0
    }

    pub fn get_top(&mut self) -> i32 {
        unsafe {
            lua_gettop(self.state())
        }
    }

    pub fn set_top(&mut self, top: i32) {
        unsafe {
            lua_settop(self.state(), top)
        }
    }

    pub fn get_luatype(&mut self, index: i32) -> i32 {
        unsafe {
            lua_type(self.state(), index)
        }
    }

    pub fn is_nil(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TNIL
    }

    pub fn is_boolean(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TBOOLEAN
    }

    pub fn is_lightuserdata(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TLIGHTUSERDATA
    }

    pub fn is_number(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TNUMBER
    }

    pub fn is_string(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TSTRING
    }

    pub fn is_table(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TTABLE
    }

    pub fn is_function(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TFUNCTION
    }

    pub fn is_userdata(&mut self, index: i32) -> bool {
        self.get_luatype(index) == LUA_TUSERDATA
    }

    pub fn load_file(&mut self, file_name: &str) -> i32 {
        let mut f = unwrap_or!(File::open(file_name).ok(), return 0);
        let mut buffer = Vec::new();
        let _ = unwrap_or!(f.read_to_end(&mut buffer).ok(), return 0);
        let mut name = file_name.to_string();
        let mut short_name = name.clone();
        let len = name.len();
        if len > 30 {
            short_name = name.drain((len - 30)..).collect();
        }

        let short_name = CString::new(short_name).unwrap();
        let ret = unsafe { luaL_loadbuffer(self.state(), buffer.as_ptr() as *const libc::c_char, buffer.len(), short_name.as_ptr()) };
        if ret != 0 {
            let err_msg : String = unwrap_or!(LuaRead::lua_read(self.state()), return 0);
            let err_detail = CString::new(format!("error loading from file {} :\n\t{}", file_name, err_msg)).unwrap();
            unsafe { luaL_error(self.state(), err_detail.as_ptr()); }
        }
        1
    }

    /// enable hotfix, can update the new func, and the old data will be keep and bind to the new func
    pub fn enable_hotfix(&mut self) {
        hotfix::load_hot_fix(self);
    }

    pub fn exec_gc(&mut self) -> i32 {
        unsafe { lua_gc(self.state(), LUA_GCCOLLECT, 0) as i32 } 
    }

    impl_exec_func!(exec_func0, );
    impl_exec_func!(exec_func1, A);
    impl_exec_func!(exec_func2, A, B);
    impl_exec_func!(exec_func3, A, B, C);
    impl_exec_func!(exec_func4, A, B, C, D);
    impl_exec_func!(exec_func5, A, B, C, D, E);
    impl_exec_func!(exec_func6, A, B, C, D, E, F);
    impl_exec_func!(exec_func7, A, B, C, D, E, F, G);
    impl_exec_func!(exec_func8, A, B, C, D, E, F, G, H);
    impl_exec_func!(exec_func9, A, B, C, D, E, F, G, H, I);
    impl_exec_func!(exec_func10, A, B, C, D, E, F, G, H, I, J);


    impl_read_func!(read_func0, );
    impl_read_func!(read_func1, A);
    impl_read_func!(read_func2, A, B);
    impl_read_func!(read_func3, A, B, C);
    impl_read_func!(read_func4, A, B, C, D);
    impl_read_func!(read_func5, A, B, C, D, E);
    impl_read_func!(read_func6, A, B, C, D, E, F);
    impl_read_func!(read_func7, A, B, C, D, E, F, G);
    impl_read_func!(read_func8, A, B, C, D, E, F, G, H);
    impl_read_func!(read_func9, A, B, C, D, E, F, G, H, I);
    impl_read_func!(read_func10, A, B, C, D, E, F, G, H, I, J);

}

/// Types that can be given to a Lua context, for example with `lua.set()` or as a return value
/// of a function.
pub trait LuaPush {
    /// Pushes the value on the top of the stack.
    ///
    /// Must return a guard representing the elements that have been pushed.
    ///
    /// You can implement this for any type you want by redirecting to call to
    /// another implementation (for example `5.push_to_lua`) or by calling
    /// `userdata::push_userdata`.
    fn push_to_lua(self, lua: *mut lua_State) -> i32;
}

/// Types that can be obtained from a Lua context.
///
/// Most types that implement `LuaPush` also implement `LuaRead`, but this is not always the case
/// (for example `&'static str` implements `LuaPush` but not `LuaRead`).
pub trait LuaRead: Sized {
    /// Reads the data from Lua.
    fn lua_read(lua: *mut lua_State) -> Option<Self> {
        LuaRead::lua_read_at_position(lua, -1)
    }

    /// Reads the data from Lua at a given position.
    fn lua_read_at_position(lua: *mut lua_State, index: i32) -> Option<Self> {
        LuaRead::lua_read_with_pop(lua, index, 0)
    }

    /// Reads the data from Lua at a given position.
    fn lua_read_with_pop(lua: *mut lua_State, index: i32, pop: i32) -> Option<Self> {
        let _guard = LuaGuard::new(lua, pop);
        LuaRead::lua_read_with_pop_impl(lua, index, pop)
    }

    fn lua_read_with_pop_impl(lua: *mut lua_State, index: i32, pop: i32) -> Option<Self>;

}

impl Drop for Lua {
    fn drop(&mut self) {
        if self.own {
            unsafe { lua_close(self.lua) }
        }
    }
}

impl Drop for LuaGuard {
    fn drop(&mut self) {
        if self.size != 0 {
            unsafe { 
                lua_pop(self.lua, self.size) }
        }
    }
}

#[allow(unused_macros)]
macro_rules! cstr {
    ($s:expr) => {
        concat!($s, "\0") as *const str as *const [::std::os::raw::c_char]
            as *const ::std::os::raw::c_char
    };
}
