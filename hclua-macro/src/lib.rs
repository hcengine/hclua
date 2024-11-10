use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{format_ident, quote};
use syn::meta::ParseNestedMeta;
use syn::{self, ItemStruct, parse_macro_input, ItemFn, LitStr, Result};

mod config;

#[proc_macro_derive(ObjectMacro, attributes(hclua_field, hclua_cfg))]
pub fn object_macro_derive(input: TokenStream) -> TokenStream {
    let ItemStruct {
        ident,
        fields,
        attrs,
        ..
    } = parse_macro_input!(input);
    let config = config::Config::parse_from_attributes(ident.to_string(), &attrs[..]).unwrap();
    let functions: Vec<_> = fields
        .iter()
        .map(|field| {
            let field_ident = field.ident.clone().unwrap();
            if field
                .attrs
                .iter()
                .any(|attr| attr.path().is_ident("hclua_field"))
            {
                let get_name = format_ident!("get_{}", field_ident);
                let set_name = format_ident!("set_{}", field_ident);
                let ty = field.ty.clone();
                quote! {
                    fn #get_name(&mut self) -> &#ty {
                        &self.#field_ident
                    }

                    fn #set_name(&mut self, val: #ty) {
                        self.#field_ident = val;
                    }
                }
            } else {
                quote! {}
            }
        })
        .collect();

    let registers: Vec<_> = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        if field.attrs.iter().any(|attr| attr.path().is_ident("hclua_field")) {
            let ty = field.ty.clone();
            let get_name = format_ident!("get_{}", field_ident);
            let set_name = format_ident!("set_{}", field_ident);
            quote!{
                hclua::LuaObject::add_object_method_get(lua, &stringify!(#field_ident), hclua::function1(|obj: &mut #ident| -> &#ty {
                    &obj.#field_ident
                }));
                hclua::LuaObject::add_object_method_set(lua, &stringify!(#field_ident), hclua::function2(|obj: &mut #ident, val: #ty| {
                    obj.#field_ident = val;
                }));
                hclua::LuaObject::object_def(lua, &stringify!(#get_name), hclua::function1(#ident::#get_name));
                hclua::LuaObject::object_def(lua, &stringify!(#set_name), hclua::function2(#ident::#set_name));
                hclua::LuaObject::set_field(&stringify!(#field_ident));
            }
        } else {
            quote!{}
        }
    }).collect();

    let name = config.name;
    let is_light = config.light;
    let gen = quote! {
        impl #ident {
            fn register_field(lua: &mut hclua::Lua) {
                #(#registers)*
            }

            fn register(lua: &mut hclua::Lua) {
                let mut obj = if #is_light {
                    hclua::LuaObject::<#ident>::new_light(lua.state(), &#name)
                } else {
                    hclua::LuaObject::<#ident>::new(lua.state(), &#name)
                };
                obj.create();

                Self::register_field(lua);
            }

            fn object_def<P>(lua: &mut hclua::Lua, name: &str, param: P)
            where
                P: hclua::LuaPush,
            {
                hclua::LuaObject::<#ident>::object_def(lua, name, param);
            }


            fn object_static_def<P>(lua: &mut hclua::Lua, name: &str, param: P)
            where
                P: hclua::LuaPush,
            {
                let mut obj = if #is_light {
                    hclua::LuaObject::<#ident>::new_light(lua.state(), &#name)
                } else {
                    hclua::LuaObject::<#ident>::new(lua.state(), &#name)
                };
                obj.create();
                obj.static_def(name, param);
            }

            #(#functions)*
        }

        impl<'a> hclua::LuaRead for &'a mut #ident {
            fn lua_read_with_pop_impl(
                lua: *mut hclua::lua_State,
                index: i32,
                _pop: i32,
            ) -> Option<&'a mut #ident> {
                hclua::userdata::read_userdata(lua, index)
            }
        }

        impl<'a> hclua::LuaRead for &'a #ident {
            fn lua_read_with_pop_impl(
                lua: *mut hclua::lua_State,
                index: i32,
                _pop: i32,
            ) -> Option<&'a #ident> {
                hclua::userdata::read_userdata(lua, index).map(|v| &*v)
            }
        }

        impl hclua::LuaPush for #ident {
            fn push_to_lua(self, lua: *mut hclua::lua_State) -> i32 {
                unsafe {
                    let obj = std::boxed::Box::into_raw(std::boxed::Box::new(self));
                    hclua::userdata::push_lightuserdata(&mut *obj, lua, |_| {});
                    let typeid =
                        std::ffi::CString::new(format!("{:?}", std::any::TypeId::of::<#ident>()))
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
    gen.into()
}

#[derive(Default)]
struct ModuleAttributes {
    name: Option<Ident>,
}

impl ModuleAttributes {
    fn parse(&mut self, meta: ParseNestedMeta) -> Result<()> {
        if meta.path.is_ident("name") {
            match meta.value() {
                Ok(value) => {
                    self.name = Some(value.parse::<LitStr>()?.parse()?);
                }
                Err(_) => {
                    return Err(meta.error("`name` attribute must have a value"));
                }
            }
        } else {
            return Err(meta.error("unsupported module attribute"));
        }
        Ok(())
    }
}

#[proc_macro_attribute]
pub fn lua_module(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut args = ModuleAttributes::default();
    if !attr.is_empty() {
        let args_parser = syn::meta::parser(|meta| args.parse(meta));
        parse_macro_input!(attr with args_parser);
    }

    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    let module_name = args.name.unwrap_or_else(|| func_name.clone());
    let ext_entrypoint_name = Ident::new(&format!("luaopen_{module_name}"), Span::call_site());
    let ext_register = Ident::new(&format!("luareg_{module_name}"), Span::call_site());
    let name = module_name.to_string();
    

    let wrapped = quote! {
        #func

        #[no_mangle]
        #[link(kind="static")]
        unsafe extern "C" fn #ext_entrypoint_name(state: *mut hclua::lua_State) -> libc::c_int {
            use hclua::LuaPush;
            
            let mut lua = Lua::from_existing_state(state, false);
            if let Some(v) = #func_name(&mut lua) {
                v.push_to_lua(state);
                1
            } else {
                lua.error(format!("load module: {:?} failed", 1));
                0
            }
        }

        pub fn #ext_register(state: *mut hclua::lua_State) {
            unsafe {
                let cstr = std::ffi::CString::new(#name).unwrap();
                let value = cstr.as_ptr();
                hclua::luaL_requiref(state, value, #ext_entrypoint_name, 1 as libc::c_int);
                

                if #name.contains('_') {
                    let new = #name.replace("_", ".");
                    let cstr = std::ffi::CString::new(new).unwrap();
                    let value = cstr.as_ptr();
                    hclua::luaL_requiref(state, value, #ext_entrypoint_name, 1 as libc::c_int);
                }
            }
        }
    };

    wrapped.into()
}
