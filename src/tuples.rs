
use crate::{LuaPush, LuaRead, sys, lua_State};

macro_rules! tuple_impl {
    ($ty:ident) => (
        impl<$ty> LuaPush for ($ty,) where $ty: LuaPush {
            fn push_to_lua(self, lua: *mut sys::lua_State) -> i32 {
                self.0.push_to_lua(lua)
            }

            fn box_push_to_lua(self: Box<Self>, lua: *mut lua_State) -> i32
            {
                (*self).push_to_lua(lua)
            }
        }

        impl<$ty> LuaRead for ($ty,) where $ty: LuaRead {
            fn lua_read_with_pop_impl(lua: *mut sys::lua_State, index: i32, _pop: i32) -> Option<($ty,)> {
                LuaRead::lua_read_at_position(lua, index).map(|v| (v,))
            }
        }
    );

    ($first:ident, $($other:ident),+) => (
        #[allow(non_snake_case)]
        impl<$first: LuaPush, $($other: LuaPush),+>
            LuaPush for ($first, $($other),+)
        {
            fn push_to_lua(self, lua: *mut sys::lua_State) -> i32 {
                match self {
                    ($first, $($other),+) => {
                        let mut total = $first.push_to_lua(lua);

                        $(
                            total += $other.push_to_lua(lua);
                        )+

                        total
                    }
                }
            }

            fn box_push_to_lua(self: Box<Self>, lua: *mut lua_State) -> i32
            {
                (*self).push_to_lua(lua)
            }
        }

        // TODO: what if T or U are also tuples? indices won't match
        #[allow(unused_assignments)]
        #[allow(non_snake_case)]
        impl<$first: LuaRead, $($other: LuaRead),+>
            LuaRead for ($first, $($other),+)
        {
            fn lua_read_with_pop_impl(lua: *mut sys::lua_State, index: i32, _pop: i32) -> Option<($first, $($other),+)> {
                let mut i = index;
                let $first: $first = match LuaRead::lua_read_at_position(lua, i) {
                    Some(v) => v,
                    None => return None
                };

                i += 1;
                // if index > 0 {
                //     i += 1;
                // } else {
                //     i -= 1;
                // }
                

                $(
                    let $other: $other = match LuaRead::lua_read_at_position(lua, i) {
                        Some(v) => v,
                        None => return None
                    };
                    i += 1;
                    // if index > 0 {
                    //     i += 1;
                    // } else {
                    //     i -= 1;
                    // }
                )+

                Some(($first, $($other),+))

            }
        }

        tuple_impl!($($other),+);
    );
}

tuple_impl!(A, B, C, D, E, F, G, H, I, J, K, L, M);
