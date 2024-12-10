mod proto_lua;
mod ser_utils;
mod wrapper;

pub use proto_lua::ProtoLua;
pub use ser_utils::SerUtils;
pub use wrapper::{LuaWrapperTableValue, LuaWrapperValue, LuaWrapperVecValue};

pub struct WrapSerde<T> {
    pub value: T,
}

impl<T> WrapSerde<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}
