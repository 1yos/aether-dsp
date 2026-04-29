//! Proc-macro crate for the Aether Node Development Kit.
//! Uses syn v2 API.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Fields, Lit,
};

#[proc_macro_attribute]
pub fn aether_node(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let name_str = name.to_string();

    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(f) => &f.named,
            _ => panic!("#[aether_node] requires named fields"),
        },
        _ => panic!("#[aether_node] can only be applied to structs"),
    };

    let mut param_defs: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut param_defaults: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut state_defaults: Vec<proc_macro2::TokenStream> = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let mut is_param = false;
        let mut param_name = field_name.to_string();
        let mut min = 0.0f32;
        let mut max = 1.0f32;
        let mut default = 0.0f32;

        for attr in &field.attrs {
            if attr.path().is_ident("param") {
                is_param = true;
                // Parse key=value pairs inside #[param(...)]
                let _ = attr.parse_nested_meta(|meta| {
                    let key = meta.path.get_ident()
                        .map(|i| i.to_string())
                        .unwrap_or_default();
                    let value: Expr = meta.value()?.parse()?;
                    match key.as_str() {
                        "name" => {
                            if let Expr::Lit(el) = &value {
                                if let Lit::Str(s) = &el.lit {
                                    param_name = s.value();
                                }
                            }
                        }
                        "min" => {
                            if let Expr::Lit(el) = &value {
                                if let Lit::Float(f) = &el.lit {
                                    min = f.base10_parse().unwrap_or(0.0);
                                }
                            }
                        }
                        "max" => {
                            if let Expr::Lit(el) = &value {
                                if let Lit::Float(f) = &el.lit {
                                    max = f.base10_parse().unwrap_or(1.0);
                                }
                            }
                        }
                        "default" => {
                            if let Expr::Lit(el) = &value {
                                if let Lit::Float(f) = &el.lit {
                                    default = f.base10_parse().unwrap_or(0.0);
                                }
                            }
                        }
                        _ => {}
                    }
                    Ok(())
                });
            }
        }

        if is_param {
            param_defs.push(quote! {
                ::aether_ndk::ParamDef {
                    name: #param_name,
                    min: #min,
                    max: #max,
                    default: #default,
                }
            });
            param_defaults.push(quote! { #field_name: #default });
        } else {
            state_defaults.push(quote! { #field_name: Default::default() });
        }
    }

    let param_count = param_defs.len();
    let all_defaults: Vec<_> = param_defaults.iter().chain(state_defaults.iter()).collect();

    // Strip #[param] attrs from the struct
    let mut clean_input = input.clone();
    if let Data::Struct(ref mut s) = clean_input.data {
        if let Fields::Named(ref mut f) = s.fields {
            for field in f.named.iter_mut() {
                field.attrs.retain(|a| !a.path().is_ident("param"));
            }
        }
    }

    let expanded = quote! {
        #clean_input

        impl #name {
            pub const PARAM_COUNT: usize = #param_count;

            pub fn param_defs() -> &'static [::aether_ndk::ParamDef] {
                static DEFS: &[::aether_ndk::ParamDef] = &[#(#param_defs),*];
                DEFS
            }
        }

        impl Default for #name {
            fn default() -> Self {
                Self { #(#all_defaults),* }
            }
        }

        impl ::aether_ndk::AetherNodeMeta for #name {
            fn type_name() -> &'static str { #name_str }
            fn param_defs() -> &'static [::aether_ndk::ParamDef] {
                #name::param_defs()
            }
        }
    };

    TokenStream::from(expanded)
}
