use proc_macro::{TokenStream, TokenTree};
use quote_use::quote_use;
use proc_macro2::{self};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Token;
use syn::{
    self, Attribute, DeriveInput, Expr, FnArg, Generics, Ident, ItemFn, ItemStruct, Lit, LitStr, Meta, ReturnType, Visibility
};
use syn::{parse_quote, Token};

use std::io::Result;

use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::parse_macro_input;
mod config;

#[proc_macro_derive(HelloMacro, attributes(field, hclua_cfg))]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    let ItemStruct { ident, fields, attrs, .. } = parse_macro_input!(input);
    let config =  config::Config::parse_from_attributes(ident.to_string(), &attrs[..]).unwrap();
    let functions: Vec<_> = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        if field.attrs.iter().any(|attr| attr.path().is_ident("field")) {
            let get_name = format_ident!("get_{}", field_ident);
            let set_name = format_ident!("set_{}", field_ident);
            let ty = field.ty.clone();
            quote!{
                fn #get_name(&mut self) -> &#ty {
                    println!("aaaa");
                    &self.#field_ident
                }

                fn #set_name(&mut self, val: #ty) {
                    println!("aaaa");
                    self.#field_ident = val;
                }
            }
        } else {
            quote!{}
        }
    }).collect();

    
    let registers: Vec<_> = fields.iter().map(|field| {
        let field_ident = field.ident.clone().unwrap();
        println!("==={:?}", field_ident);
        if field.attrs.iter().any(|attr| attr.path().is_ident("field")) {
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
                hclua::LuaObject::object_mark_field(lua, &stringify!(#field_ident));
            }
        } else {
            quote!{}
        }
    }).collect();

    println!("register!! = {:?}", registers);
    

    // let functions: Vec<_> = depends_on.iter().map(|field| {
    //     let name = format_ident!("{}", field);
    //     quote!{
    //         fn #name(&mut self) {
    //             // ...
    //             println!("aaaa");
    //         }
    //     }
    // }).collect();

    // println!("functions = {:?}", functions);

    let name = config.name;
    let is_light = config.light;
    let gen = quote! {
        impl #ident {
            fn hello_macro(&self) {
                println!("Hello, Macro! My name is {} {}", stringify!(#ident), "a");
            }

            fn register_field(lua: &mut hclua::Lua) {
                println!("register");

                #(#registers)*
            }

            fn register(lua: &mut hclua::Lua) {
                let mut obj = if #is_light {
                    hclua::LuaObject::<#ident>::new(lua.state(), &#name)
                } else {
                    hclua::LuaObject::<#ident>::new(lua.state(), &#name)
                };
                obj.create();

                Self::register_field(lua);
            }

            #(#functions)*
            // expand_fn!(#ident, #(#depends_on), *)
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
    // 构建 Rust 代码所代表的语法树
    // 以便可以进行操作
    // let ast = syn::parse(input).unwrap();
    
    // // 构建 trait 实现
    // impl_hello_macro(&ast)
}

// fn impl_hello_macro(ast: &syn::DeriveInput) -> TokenStream {
//     let name = &ast.ident;
    
//     let field: syn::ItemFn = syn::parse({
//         quote! {
//             fn register_field() {

//             }
//         }
//         .into()
//     })
//     .unwrap();

//     // field
//     //     .block
//     //     .stmts
//     //     .append(quote! {println("aaaaaaaaaaa");}.into());

//     TokenStream::new();
//     let attribs = &ast.attrs;
//     // for attrib in attribs {
//     //     let segs = &attrib.path().segments;
//     //     if segs.len() > 0 {
//     //         if segs[0].ident == CONFIG_ATTRIBUTE_NAME {
//     //             let tokens = attrib.meta.to_token_stream();
//     //             let parsed = syn::parse2::<ConfigAttrib>(tokens)?;
//     //             parsed_attributes.push(parsed);
//     //         }
//     //         else {
//     //             remaining_attributes.push(attrib.clone());
//     //         }
//     //     }
//     // }

//     let g: Vec<_> = attribs
//         .iter()
//         .map(|field| {
//             let field_name = field.meta.path().get_ident().unwrap();
//             quote! { println!("{:?}", #field_name);}
//         })
//         .collect();

//     println!("attribs = {:?}", attribs);
//     let gen = quote! {
//         impl #name {
//             fn hello_macro(&self) {
//                 println!("Hello, Macro! My name is {}", stringify!(#name));
//             }

//             fn register_field() {
//                 println!("register");
//                 (#(#g),*)
//                 println("aaaaa");
//             }
//         }
//     };
//     gen.into()
// }
