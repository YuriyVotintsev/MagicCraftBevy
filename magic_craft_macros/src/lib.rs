use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, format_ident};
use syn::{
    parse_macro_input, DeriveInput, Data, Fields, Attribute, Meta, Expr, Lit, Type, GenericArgument, PathArguments,
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

fn parse_default_expr(attrs: &[Attribute]) -> Option<String> {
    for attr in attrs {
        if attr.path().is_ident("default_expr") {
            if let Ok(Meta::List(list)) = attr.meta.clone().try_into() as Result<Meta, _> {
                let tokens = list.tokens.to_string();
                let trimmed = tokens.trim();
                if trimmed.starts_with('"') && trimmed.ends_with('"') {
                    return Some(trimmed[1..trimmed.len()-1].to_string());
                }
            }
            if let Ok(lit) = attr.parse_args::<syn::LitStr>() {
                return Some(lit.value());
            }
        }
    }
    None
}

fn is_entities_field(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident("entities"))
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

fn is_scalar_expr_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "ScalarExpr";
        }
    }
    false
}

fn is_vec_expr_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "VecExpr";
        }
    }
    false
}

fn is_entity_expr_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "EntityExpr";
        }
    }
    false
}

fn is_vec_entity_def(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Vec" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        if let Type::Path(inner_path) = inner_ty {
                            if let Some(inner_seg) = inner_path.path.segments.last() {
                                return inner_seg.ident == "EntityDef";
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn is_option_scalar_expr(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return is_scalar_expr_type(inner_ty);
                    }
                }
            }
        }
    }
    false
}

fn is_option_vec_expr(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(args) = &segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return is_vec_expr_type(inner_ty);
                    }
                }
            }
        }
    }
    false
}

fn get_has_recalc_code(field_name: &syn::Ident, ty: &Type) -> Option<TokenStream2> {
    if is_scalar_expr_type(ty) || is_vec_expr_type(ty) {
        Some(quote! {
            if def.#field_name.uses_recalc() { return true; }
        })
    } else if is_option_scalar_expr(ty) || is_option_vec_expr(ty) {
        Some(quote! {
            if let Some(ref expr) = def.#field_name {
                if expr.uses_recalc() { return true; }
            }
        })
    } else {
        None
    }
}

struct UpdateParts {
    eval: TokenStream2,
    apply: TokenStream2,
    check: syn::Ident,
}

fn get_update_parts(field_name: &syn::Ident, ty: &Type) -> Option<UpdateParts> {
    let var = format_ident!("__recalc_{}", field_name);

    if is_scalar_expr_type(ty) || is_vec_expr_type(ty) {
        Some(UpdateParts {
            eval: quote! {
                let #var = if def.#field_name.uses_recalc() { Some(def.#field_name.eval(&__ctx)) } else { None };
            },
            apply: quote! {
                if let Some(v) = #var { comp.#field_name = v; }
            },
            check: var,
        })
    } else if is_option_scalar_expr(ty) || is_option_vec_expr(ty) {
        Some(UpdateParts {
            eval: quote! {
                let #var = match def.#field_name {
                    Some(ref expr) if expr.uses_recalc() => Some(expr.eval(&__ctx)),
                    _ => None,
                };
            },
            apply: quote! {
                if let Some(v) = #var { comp.#field_name = Some(v); }
            },
            check: var,
        })
    } else {
        None
    }
}

fn is_state_ref_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "StateRef";
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




#[proc_macro_attribute]
pub fn blueprint_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let provided_fields = if attr.is_empty() {
        None
    } else {
        let tokens = attr.to_string();
        let fields: Vec<String> = tokens
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if fields.is_empty() { None } else { Some(fields) }
    };

    let component_name = input.ident.clone();

    match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields) => {
                    generate_blueprint_component_named(&component_name, fields, provided_fields)
                }
                Fields::Unit => {
                    generate_blueprint_component_unit(&component_name, provided_fields)
                }
                Fields::Unnamed(_) => {
                    syn::Error::new(
                        input.ident.span(),
                        "blueprint_component does not support tuple structs"
                    ).to_compile_error().into()
                }
            }
        }
        _ => {
            syn::Error::new(
                input.ident.span(),
                "blueprint_component can only be applied to structs"
            ).to_compile_error().into()
        }
    }
}

fn generate_blueprint_component_unit(
    component_name: &syn::Ident,
    provided_fields: Option<Vec<String>>,
) -> TokenStream {
    let provided_fields_fn = if let Some(ref pf) = provided_fields {
        let pf_tokens: Vec<_> = pf.iter().map(|s| {
            let ident = format_ident!("{}", s);
            quote! { crate::blueprints::context::ProvidedFields::#ident }
        }).collect();

        let provided_expr = if pf_tokens.len() == 1 {
            quote! { #(#pf_tokens)* }
        } else {
            let first = &pf_tokens[0];
            let rest = &pf_tokens[1..];
            quote! { #first #(.union(#rest))* }
        };

        quote! {
            pub fn provided_fields() -> crate::blueprints::context::ProvidedFields {
                #provided_expr
            }
        }
    } else {
        quote! {}
    };

    let output = quote! {
        #[derive(Debug, Clone, serde::Deserialize, Default)]
        pub struct DefRaw;

        #[derive(Debug, Clone)]
        pub struct Def;

        impl DefRaw {
            pub fn resolve(&self, _stat_registry: &crate::stats::StatRegistry, _state_indices: Option<&std::collections::HashMap<String, usize>>) -> Def {
                Def
            }
        }

        #[cfg(test)]
        pub fn required_fields_and_nested(_raw: &DefRaw) -> (crate::blueprints::context::ProvidedFields, Option<(crate::blueprints::context::ProvidedFields, &[crate::blueprints::entity_def::EntityDefRaw])>) {
            (crate::blueprints::context::ProvidedFields::NONE, None)
        }

        #provided_fields_fn

        #[derive(bevy::prelude::Component)]
        pub struct #component_name;

        pub fn insert_component(commands: &mut bevy::prelude::EntityCommands, _def: &Def, _source: &crate::blueprints::core_components::SpawnSource, _stats: &crate::stats::ComputedStats) {
            commands.insert(#component_name);
        }

        pub fn update_component(_commands: &mut bevy::prelude::EntityCommands, _def: &Def, _source: &crate::blueprints::core_components::SpawnSource, _stats: &crate::stats::ComputedStats) {
        }

        pub fn has_recalc(_def: &Def) -> bool {
            false
        }

        pub fn remove_component(commands: &mut bevy::prelude::EntityCommands) {
            commands.remove::<#component_name>();
        }
    };

    output.into()
}

fn generate_blueprint_component_named(
    component_name: &syn::Ident,
    fields: &syn::FieldsNamed,
    provided_fields: Option<Vec<String>>,
) -> TokenStream {
    let mut raw_fields = Vec::new();
    let mut def_fields = Vec::new();
    let mut resolve_fields = Vec::new();
    let mut required_fields_code = Vec::new();
    let mut entities_field_name: Option<syn::Ident> = None;

    let mut component_fields = Vec::new();
    let mut insert_fields = Vec::new();
    let mut update_evals = Vec::new();
    let mut update_applies = Vec::new();
    let mut update_checks: Vec<syn::Ident> = Vec::new();
    let mut has_recalc_stmts = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        let raw_default = parse_raw_default(&field.attrs);
        let default_expr = parse_default_expr(&field.attrs);
        let is_entities = is_entities_field(&field.attrs) || is_vec_entity_def(field_ty);

        if is_entities {
            entities_field_name = Some(field_name.clone());
        }

        if let Some(parts) = get_update_parts(field_name, field_ty) {
            update_evals.push(parts.eval);
            update_applies.push(parts.apply);
            update_checks.push(parts.check);
        }

        if let Some(recalc_code) = get_has_recalc_code(field_name, field_ty) {
            has_recalc_stmts.push(recalc_code);
        }

        if is_scalar_expr_type(field_ty) {
            def_fields.push(quote! { pub #field_name: crate::blueprints::expr::ScalarExpr });

            if let Some(ref default_str) = default_expr {
                raw_fields.push(quote! {
                    #[serde(default)]
                    pub #field_name: Option<crate::blueprints::expr::ScalarExprRaw>
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.as_ref()
                        .map(|v| v.resolve(stat_registry))
                        .unwrap_or_else(|| crate::blueprints::expr::parse_and_resolve_scalar(#default_str, stat_registry))
                });

                required_fields_code.push(quote! {
                    fields = fields.union(
                        raw.#field_name.as_ref()
                            .map(|v| v.required_fields())
                            .unwrap_or_else(|| crate::blueprints::expr::parse_required_fields(#default_str))
                    );
                });
            } else if let Some(default) = raw_default {
                raw_fields.push(quote! {
                    #[serde(default)]
                    pub #field_name: Option<crate::blueprints::expr::ScalarExprRaw>
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.as_ref()
                        .map(|v| v.resolve(stat_registry))
                        .unwrap_or(crate::blueprints::expr::ScalarExpr::Literal(#default as f32))
                });

                required_fields_code.push(quote! {
                    fields = fields.union(
                        raw.#field_name.as_ref()
                            .map(|v| v.required_fields())
                            .unwrap_or(crate::blueprints::context::ProvidedFields::NONE)
                    );
                });
            } else {
                raw_fields.push(quote! {
                    pub #field_name: crate::blueprints::expr::ScalarExprRaw
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.resolve(stat_registry)
                });

                required_fields_code.push(quote! {
                    fields = fields.union(raw.#field_name.required_fields());
                });
            }

            component_fields.push(quote! { pub #field_name: f32 });
            insert_fields.push(quote! { #field_name: def.#field_name.eval(&__ctx) });

        } else if is_vec_expr_type(field_ty) {
            def_fields.push(quote! { pub #field_name: crate::blueprints::expr::VecExpr });

            if let Some(ref default_str) = default_expr {
                raw_fields.push(quote! {
                    #[serde(default)]
                    pub #field_name: Option<crate::blueprints::expr::VecExprRaw>
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.as_ref()
                        .map(|v| v.resolve(stat_registry))
                        .unwrap_or_else(|| crate::blueprints::expr::parse_and_resolve_vec(#default_str, stat_registry))
                });

                required_fields_code.push(quote! {
                    fields = fields.union(
                        raw.#field_name.as_ref()
                            .map(|v| v.required_fields())
                            .unwrap_or_else(|| crate::blueprints::expr::parse_required_fields(#default_str))
                    );
                });
            } else {
                raw_fields.push(quote! {
                    pub #field_name: crate::blueprints::expr::VecExprRaw
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.resolve(stat_registry)
                });

                required_fields_code.push(quote! {
                    fields = fields.union(raw.#field_name.required_fields());
                });
            }

            component_fields.push(quote! { pub #field_name: bevy::prelude::Vec2 });
            insert_fields.push(quote! { #field_name: def.#field_name.eval(&__ctx) });

        } else if is_entity_expr_type(field_ty) {
            def_fields.push(quote! { pub #field_name: crate::blueprints::expr::EntityExpr });

            if let Some(ref default_str) = default_expr {
                raw_fields.push(quote! {
                    #[serde(default)]
                    pub #field_name: Option<crate::blueprints::expr::EntityExprRaw>
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.as_ref()
                        .map(|v| v.resolve(stat_registry))
                        .unwrap_or_else(|| crate::blueprints::expr::parse_and_resolve_entity(#default_str, stat_registry))
                });

                required_fields_code.push(quote! {
                    fields = fields.union(
                        raw.#field_name.as_ref()
                            .map(|v| v.required_fields())
                            .unwrap_or_else(|| crate::blueprints::expr::parse_required_fields(#default_str))
                    );
                });
            } else {
                raw_fields.push(quote! {
                    pub #field_name: crate::blueprints::expr::EntityExprRaw
                });

                resolve_fields.push(quote! {
                    #field_name: self.#field_name.resolve(stat_registry)
                });

                required_fields_code.push(quote! {
                    fields = fields.union(raw.#field_name.required_fields());
                });
            }

            component_fields.push(quote! { pub #field_name: bevy::prelude::Entity });
            insert_fields.push(quote! { #field_name: def.#field_name.eval(&__ctx).unwrap() });

        } else if is_option_scalar_expr(field_ty) {
            def_fields.push(quote! { pub #field_name: Option<crate::blueprints::expr::ScalarExpr> });

            raw_fields.push(quote! {
                #[serde(default)]
                pub #field_name: Option<crate::blueprints::expr::ScalarExprRaw>
            });

            resolve_fields.push(quote! {
                #field_name: self.#field_name.as_ref().map(|v| v.resolve(stat_registry))
            });

            required_fields_code.push(quote! {
                if let Some(ref expr) = raw.#field_name {
                    fields = fields.union(expr.required_fields());
                }
            });

            component_fields.push(quote! { pub #field_name: Option<f32> });
            insert_fields.push(quote! { #field_name: def.#field_name.as_ref().map(|e| e.eval(&__ctx)) });

        } else if is_vec_entity_def(field_ty) {
            def_fields.push(quote! { pub #field_name: Vec<crate::blueprints::entity_def::EntityDef> });

            raw_fields.push(quote! {
                #[serde(default)]
                pub #field_name: Vec<crate::blueprints::entity_def::EntityDefRaw>
            });

            resolve_fields.push(quote! {
                #field_name: self.#field_name.iter().map(|e| e.resolve(stat_registry)).collect()
            });

            component_fields.push(quote! { pub #field_name: Vec<crate::blueprints::entity_def::EntityDef> });
            insert_fields.push(quote! { #field_name: def.#field_name.clone() });

        } else if is_state_ref_type(field_ty) {
            raw_fields.push(quote! {
                pub #field_name: String
            });

            def_fields.push(quote! { pub #field_name: usize });

            resolve_fields.push(quote! {
                #field_name: *state_indices.unwrap().get(&self.#field_name)
                    .unwrap_or_else(|| panic!("unknown state '{}'", self.#field_name))
            });

            component_fields.push(quote! { pub #field_name: usize });
            insert_fields.push(quote! { #field_name: def.#field_name });

        } else if is_bool_type(field_ty) {
            def_fields.push(quote! { pub #field_name: bool });

            raw_fields.push(quote! {
                #[serde(default)]
                pub #field_name: Option<bool>
            });

            let resolve_expr = if let Some(default) = raw_default {
                quote! { self.#field_name.unwrap_or(#default) }
            } else {
                quote! { self.#field_name.unwrap_or(false) }
            };

            resolve_fields.push(quote! { #field_name: #resolve_expr });

            component_fields.push(quote! { pub #field_name: bool });
            insert_fields.push(quote! { #field_name: def.#field_name });

        } else {
            def_fields.push(quote! { pub #field_name: #field_ty });

            if raw_default.is_some() {
                raw_fields.push(quote! {
                    #[serde(default)]
                    pub #field_name: #field_ty
                });
            } else {
                raw_fields.push(quote! {
                    pub #field_name: #field_ty
                });
            }

            resolve_fields.push(quote! {
                #field_name: self.#field_name.clone()
            });

            component_fields.push(quote! { pub #field_name: #field_ty });
            insert_fields.push(quote! { #field_name: def.#field_name.clone() });
        }
    }

    let nested_code = if let Some(entities_name) = &entities_field_name {
        if let Some(ref pf) = provided_fields {
            let pf_tokens: Vec<_> = pf.iter().map(|s| {
                let ident = format_ident!("{}", s);
                quote! { crate::blueprints::context::ProvidedFields::#ident }
            }).collect();

            let provided_expr = if pf_tokens.len() == 1 {
                quote! { #(#pf_tokens)* }
            } else {
                let first = &pf_tokens[0];
                let rest = &pf_tokens[1..];
                quote! { #first #(.union(#rest))* }
            };

            quote! {
                let nested = if raw.#entities_name.is_empty() {
                    None
                } else {
                    let provided = #provided_expr;
                    Some((provided, raw.#entities_name.as_slice()))
                };
            }
        } else {
            quote! {
                let nested = if raw.#entities_name.is_empty() {
                    None
                } else {
                    Some((crate::blueprints::context::ProvidedFields::NONE, raw.#entities_name.as_slice()))
                };
            }
        }
    } else {
        quote! { let nested = None; }
    };

    let provided_fields_fn = if let Some(ref pf) = provided_fields {
        let pf_tokens: Vec<_> = pf.iter().map(|s| {
            let ident = format_ident!("{}", s);
            quote! { crate::blueprints::context::ProvidedFields::#ident }
        }).collect();

        let provided_expr = if pf_tokens.len() == 1 {
            quote! { #(#pf_tokens)* }
        } else {
            let first = &pf_tokens[0];
            let rest = &pf_tokens[1..];
            quote! { #first #(.union(#rest))* }
        };

        quote! {
            pub fn provided_fields() -> crate::blueprints::context::ProvidedFields {
                #provided_expr
            }
        }
    } else {
        quote! {}
    };

    let update_component_fn = if update_checks.is_empty() {
        quote! {
            pub fn update_component(_commands: &mut bevy::prelude::EntityCommands, _def: &Def, _source: &crate::blueprints::core_components::SpawnSource, _stats: &crate::stats::ComputedStats) {}
        }
    } else {
        quote! {
            pub fn update_component(commands: &mut bevy::prelude::EntityCommands, def: &Def, source: &crate::blueprints::core_components::SpawnSource, stats: &crate::stats::ComputedStats) {
                let __ctx = crate::blueprints::expr::EvalCtx::from_source(source, stats);
                #(#update_evals)*
                if !(#(#update_checks.is_some())||*) { return; }
                commands.queue(move |mut entity: bevy::ecs::world::EntityWorldMut| {
                    if let Some(mut comp) = entity.get_mut::<#component_name>() {
                        #(#update_applies)*
                    }
                });
            }
        }
    };

    let output = quote! {
        #[derive(Debug, Clone, serde::Deserialize)]
        pub struct DefRaw {
            #(#raw_fields,)*
        }

        #[derive(Debug, Clone)]
        pub struct Def {
            #(#def_fields,)*
        }

        impl DefRaw {
            pub fn resolve(&self, stat_registry: &crate::stats::StatRegistry, state_indices: Option<&std::collections::HashMap<String, usize>>) -> Def {
                Def {
                    #(#resolve_fields,)*
                }
            }
        }

        #[cfg(test)]
        pub fn required_fields_and_nested(raw: &DefRaw) -> (crate::blueprints::context::ProvidedFields, Option<(crate::blueprints::context::ProvidedFields, &[crate::blueprints::entity_def::EntityDefRaw])>) {
            let mut fields = crate::blueprints::context::ProvidedFields::NONE;
            #(#required_fields_code)*
            #nested_code
            (fields, nested)
        }

        #provided_fields_fn

        #[derive(bevy::prelude::Component)]
        pub struct #component_name {
            #(#component_fields,)*
        }

        pub fn insert_component(commands: &mut bevy::prelude::EntityCommands, def: &Def, source: &crate::blueprints::core_components::SpawnSource, stats: &crate::stats::ComputedStats) {
            let __ctx = crate::blueprints::expr::EvalCtx::from_source(source, stats);
            commands.insert(#component_name {
                #(#insert_fields,)*
            });
        }

        #update_component_fn

        pub fn has_recalc(def: &Def) -> bool {
            #(#has_recalc_stmts)*
            false
        }

        pub fn remove_component(commands: &mut bevy::prelude::EntityCommands) {
            commands.remove::<#component_name>();
        }
    };

    output.into()
}
