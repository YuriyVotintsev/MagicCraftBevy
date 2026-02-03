use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, DeriveInput, Data, Fields, Attribute, Meta, Expr, Lit, Type, GenericArgument, PathArguments,
    spanned::Spanned,
};

fn parse_raw_default(attrs: &[Attribute]) -> Option<TokenStream2> {
    for attr in attrs {
        if attr.path().is_ident("raw") {
            let nested = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated
            ).ok()?;

            for meta in nested {
                if let Meta::NameValue(nv) = meta {
                    if nv.path.is_ident("default") {
                        return Some(expr_to_tokens(&nv.value));
                    }
                }
            }
        }
    }
    None
}

fn is_activator(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("activator"))
}

fn expr_to_tokens(expr: &Expr) -> TokenStream2 {
    match expr {
        Expr::Lit(lit) => {
            match &lit.lit {
                Lit::Float(f) => {
                    let value: f64 = f.base10_parse().unwrap();
                    quote! { #value }
                }
                Lit::Int(i) => {
                    let value: i64 = i.base10_parse().unwrap();
                    quote! { #value }
                }
                Lit::Bool(b) => {
                    let value = b.value;
                    quote! { #value }
                }
                Lit::Str(s) => {
                    let value = s.value();
                    quote! { #value }
                }
                _ => quote! { #expr },
            }
        }
        Expr::Unary(unary) => {
            if matches!(unary.op, syn::UnOp::Neg(_)) {
                let inner = expr_to_tokens(&unary.expr);
                quote! { -#inner }
            } else {
                quote! { #expr }
            }
        }
        _ => quote! { #expr },
    }
}

fn is_option_param_value(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return is_param_value_type(inner_ty);
                    }
                }
            }
        }
    }
    false
}

fn is_param_value_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "ParamValue";
        }
    }
    false
}

fn is_bool_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "bool";
        }
    }
    false
}

fn is_generic_option(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return !is_param_value_type(inner_ty);
                    }
                }
            }
        }
    }
    false
}

#[proc_macro_derive(GenerateRaw, attributes(activator, raw))]
pub fn derive_generate_raw(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let raw_name = format_ident!("{}Raw", name);

    let is_activator = is_activator(&input.attrs);

    match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    generate_for_named_struct(name, &raw_name, fields, is_activator)
                }
                Fields::Unit => {
                    generate_for_unit_struct(name, &raw_name, is_activator)
                }
                Fields::Unnamed(_) => {
                    syn::Error::new(
                        input.span(),
                        "GenerateRaw does not support tuple structs"
                    ).to_compile_error().into()
                }
            }
        }
        _ => {
            syn::Error::new(
                input.span(),
                "GenerateRaw can only be derived for structs"
            ).to_compile_error().into()
        }
    }
}

fn generate_for_named_struct(
    name: &syn::Ident,
    raw_name: &syn::Ident,
    fields: &syn::FieldsNamed,
    _is_activator: bool,
) -> TokenStream {
    let mut raw_fields = Vec::new();
    let mut resolve_fields = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let default_value = parse_raw_default(&field.attrs);

        if is_param_value_type(field_ty) {
            if let Some(default) = default_value {
                raw_fields.push(quote! {
                    #[serde(default)]
                    pub #field_name: Option<crate::abilities::ParamValueRaw>
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.as_ref()
                        .map(|v| crate::abilities::resolve_param_value(v, stat_registry))
                        .unwrap_or(crate::abilities::ParamValue::Float(#default as f32))
                });
            } else {
                raw_fields.push(quote! {
                    pub #field_name: crate::abilities::ParamValueRaw
                });

                resolve_fields.push(quote! {
                    #field_name: crate::abilities::resolve_param_value(&self.#field_name, stat_registry)
                });
            }
        } else if is_option_param_value(field_ty) {
            raw_fields.push(quote! {
                #[serde(default)]
                pub #field_name: Option<crate::abilities::ParamValueRaw>
            });

            resolve_fields.push(quote! {
                #field_name: self.#field_name.as_ref()
                    .map(|v| crate::abilities::resolve_param_value(v, stat_registry))
            });
        } else if is_bool_type(field_ty) {
            raw_fields.push(quote! {
                #[serde(default)]
                pub #field_name: Option<bool>
            });

            let resolve_expr = if let Some(default) = default_value {
                quote! {
                    self.#field_name.unwrap_or(#default)
                }
            } else {
                quote! {
                    self.#field_name.unwrap_or(false)
                }
            };

            resolve_fields.push(quote! {
                #field_name: #resolve_expr
            });
        } else if is_generic_option(field_ty) {
            raw_fields.push(quote! {
                #[serde(default)]
                pub #field_name: #field_ty
            });

            resolve_fields.push(quote! {
                #field_name: self.#field_name.clone()
            });
        } else {
            raw_fields.push(quote! {
                #[serde(default)]
                pub #field_name: Option<#field_ty>
            });

            let resolve_expr = if let Some(default) = default_value {
                quote! {
                    self.#field_name.clone().unwrap_or(#default)
                }
            } else {
                quote! {
                    self.#field_name.clone()
                        .expect(concat!("Field '", stringify!(#field_name), "' is required"))
                }
            };

            resolve_fields.push(quote! {
                #field_name: #resolve_expr
            });
        }
    }

    let output = quote! {
        #[derive(Debug, Clone, serde::Deserialize)]
        pub struct #raw_name {
            #(#raw_fields,)*
        }

        impl #raw_name {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry) -> #name {
                #name {
                    #(#resolve_fields,)*
                }
            }
        }
    };

    output.into()
}

fn generate_for_unit_struct(
    name: &syn::Ident,
    raw_name: &syn::Ident,
    _is_activator: bool,
) -> TokenStream {
    let output = quote! {
        #[derive(Debug, Clone, serde::Deserialize, Default)]
        pub struct #raw_name {}

        impl #raw_name {
            pub fn resolve(&self, _stat_registry: &crate::stats::StatRegistry) -> #name {
                #name
            }
        }
    };

    output.into()
}
