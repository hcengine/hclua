mod proto_lua;
mod ser_utils;
mod wrapper;

use std::fmt::{Debug, Pointer};

pub use proto_lua::ProtoLua;
pub use ser_utils::SerUtils;
pub use wrapper::{LuaWrapperTableValue, LuaWrapperValue, LuaWrapperVecValue};


pub struct WrapSerde<T> {
    pub value: T,
}

impl<T: Debug> Debug for WrapSerde<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}

impl<T> WrapSerde<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}
