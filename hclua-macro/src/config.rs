use std::default::Default;

use quote::ToTokens;
use syn::parse::{Parse, ParseStream};
use syn::{self, parenthesized, Token};

pub struct Config {
    pub name: String,
    pub light: bool,
}

enum ConfigAttrib {
    Name(String),
    Light,
}

const CONFIG_ATTRIBUTE_NAME: &'static str = "hclua_cfg";

impl Config {
    // Parse any additional attributes present after `lru_cache` and return a configuration object
    // created from their contents. Additionally, return any attributes that were not handled here.
    pub fn parse_from_attributes(
        name: String,
        attribs: &[syn::Attribute],
    ) -> syn::Result<Config> {
        let mut parsed_attributes = Vec::new();

        for attrib in attribs {
            let segs = &attrib.path().segments;
            if segs.len() > 0 {
                if segs[0].ident == CONFIG_ATTRIBUTE_NAME {
                    let tokens = attrib.meta.to_token_stream();
                    let parsed = syn::parse2::<ConfigAttrib>(tokens)?;
                    parsed_attributes.push(parsed);
                }
            }
        }

        let mut config: Config = Config {
            name, light: false
        };

        for parsed_attrib in parsed_attributes {
            match parsed_attrib {
                ConfigAttrib::Name(val) => config.name = val,
                ConfigAttrib::Light => config.light = true,
            }
        }

        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            name: String::new(),
            light: false,
        }
    }
}

impl Parse for ConfigAttrib {
    fn parse(input: ParseStream) -> syn::parse::Result<Self> {
        let _name = input.parse::<syn::Ident>()?;
        let content;
        let _paren = parenthesized!(content in input);
        let name = content.parse::<syn::Ident>()?;
        match &name.to_string()[..] {
            "light" => Ok(ConfigAttrib::Light),
            "name" => {
                content.parse::<Token![=]>()?;
                let obj_name = content.parse::<syn::Ident>()?;
                Ok(ConfigAttrib::Name(obj_name.to_string()))
            },
            _ => Err(syn::parse::Error::new(
                name.span(), format!("unrecognized config option '{}'", name.to_string())
            ))
        }
    }
}
