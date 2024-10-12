//! Low level bindings to Lua 5.4/5.3/5.2/5.1 (including LuaJIT) and Roblox Luau.

#![allow(non_camel_case_types, non_snake_case, dead_code)]
#![allow(clippy::missing_safety_doc)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::os::raw::c_int;

#[macro_use]
pub mod macros;


#[cfg(any(feature = "lua54", feature = "lua53", feature = "lua52"))]
#[doc(hidden)]
pub const LUA_MAX_UPVALUES: c_int = 255;

#[cfg(any(feature = "lua51", feature = "luajit"))]
#[doc(hidden)]
pub const LUA_MAX_UPVALUES: c_int = 60;

// I believe `luaL_traceback` < 5.4 requires this much free stack to not error.
// 5.4 uses `luaL_Buffer`
#[doc(hidden)]
pub const LUA_TRACEBACK_STACK: c_int = 11;

#[cfg(any(feature = "lua54", doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "lua54")))]
pub mod lua54;

#[cfg(any(feature = "lua53", doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "lua53")))]
pub mod lua53;

#[cfg(any(feature = "lua52", doc))]
#[cfg_attr(docsrs, doc(cfg(feature = "lua52")))]
pub mod lua52;

#[cfg(any(feature = "lua51", feature = "luajit", doc))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "lua51", feature = "luajit"))))]
pub mod lua51;


#[cfg(any(feature = "lua54", doc))]
pub use lua54::*;

#[cfg(any(feature = "lua53", doc))]
pub use lua53::*;

#[cfg(any(feature = "lua52", doc))]
pub use lua52::*;

#[cfg(any(feature = "lua51", feature = "luajit", doc))]
pub use lua51::*;
