// dynamic_fields_advanced/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, parse_str, Data, DeriveInput, Expr, Fields, Lit, Meta};


#[proc_macro_derive(DynamicFields, attributes(dyn_fields))]
pub fn dynamic_fields_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => &fields_named.named,
                _ => panic!("DynamicFields can only be derived for structs with named fields"),
            }
        },
        _ => panic!("DynamicFields can only be derived for structs"),
    };

    // Collect field information including transformers and aliases
    let mut field_infos = Vec::new();
    
    'field: for field in fields.iter() {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;
        
        // Initialize transformation functions and aliases to None
        let mut get_transformer: Option<String> = None;
        let mut set_transformer: Option<String> = None;
        let mut aliases: Vec<String> = Vec::new();
        
        // Parse transform attributes
        for attr in &field.attrs {
            if attr.path().is_ident("dyn_fields") {
                match attr.meta {
                    Meta::List(ref meta_list) => {
                        let res = meta_list.parse_nested_meta(|meta| -> Result<(), syn::Error> {
                            let path = meta.path.clone();
                            if path.is_ident("skip"){
                                // TODO: fix this hacky way to skip the field
                                return Err(syn::Error::new(proc_macro::Span::mixed_site().into(), "skipped"));
                            }
                            if path.is_ident("alias") {
                                let value = meta.value()?.parse::<Expr>()?;
                                if let Expr::Lit(expr_lit) = value {
                                    if let Lit::Str(lit_str) = expr_lit.lit {
                                        aliases.push(lit_str.value());
                                    }
                                }
                            } else {
                                let value = meta.value()?.parse::<Expr>()?;
                                if let Expr::Lit(expr_lit) = value {
                                    if path.is_ident("get") {
                                        if let Lit::Str(lit_str) = expr_lit.lit {
                                            get_transformer = Some(lit_str.value());
                                        }
                                    } else if path.is_ident("set") {
                                        if let Lit::Str(lit_str) = expr_lit.lit {
                                            set_transformer = Some(lit_str.value());
                                        }
                                    }
                                }
                            }
                            
                            Ok(())
                        });
                        if let Err(e) = res {
                            if e.to_string() != "skipped" {
                                panic!("Error parsing dyn_fields attribute: {}", e);
                            }
                            continue 'field; // Skip this field
                        }
                    },
                    _ => {
                        // Invalid attribute format, ignore
                    }
                }
            }
        }
        
        field_infos.push((field_name, field_name_str, field_type, get_transformer, set_transformer, aliases));
    }

    // Generate field names list including original names and aliases
    let field_names: Vec<_> = field_infos.iter()
        .map(|(_, field_name_str, _, _, _, _)| {
            quote! {
                #field_name_str
            }
        }).collect();

    // Generate alias mapping
    let alias_mapping_entries: Vec<_> = field_infos.iter()
        .flat_map(|(_, field_name_str, _, _, _, aliases)| {
            aliases.iter().map(move |alias| {
                let alias_str = alias.clone();
                quote! {
                    alias_map.insert(#alias_str.to_string(), #field_name_str.to_string());
                }
            })
        }).collect();

    // Generate get match arms with transformer application and alias support
    let get_match_arms = field_infos.iter().map(|(field_name, field_name_str, _, get_transformer, _, _)| {
        let get_transform = if let Some(transformer_path) = get_transformer {
            // Parse the path string into a syn Path
            let transformer_path = match parse_str::<syn::Path>(transformer_path) {
                Ok(path) => path,
                Err(_) => {
                    let err_msg = format!("Failed to parse transformer path: {}", transformer_path);
                    return quote! {
                        #field_name_str => {
                            compile_error!(#err_msg);
                            Some(serde_json::to_value(&self.#field_name).unwrap())
                        },
                    };
                }
            };
            
            quote! {
                let value = serde_json::to_value(&self.#field_name).unwrap();
                Some(#transformer_path(value))
            }
        } else {
            quote! {
                Some(serde_json::to_value(&self.#field_name).unwrap())
            }
        };
        
        quote! {
            #field_name_str => {
                #get_transform
            },
        }
    }).collect::<Vec<_>>();

    // Generate set match arms with transformer application
    let set_match_arms = field_infos.iter().map(|(field_name, field_name_str, field_type, _, set_transformer, _)| {
        let set_transform = if let Some(transformer_path) = set_transformer {
            // Parse the path string into a syn Path
            let transformer_path = match parse_str::<syn::Path>(transformer_path) {
                Ok(path) => path,
                Err(_) => {
                    let err_msg = format!("Failed to parse transformer path: {}", transformer_path);
                    return quote! {
                        #field_name_str => {
                            compile_error!(#err_msg);
                            Ok(())
                        },
                    };
                }
            };
            
            quote! {
                let transformed_value = #transformer_path(value.clone());
                self.#field_name = serde_json::from_value::<#field_type>(transformed_value)
                    .map_err(|e| format!("Failed to deserialize value for field {}: {}", #field_name_str, e))?;
            }
        } else {
            quote! {
                self.#field_name = serde_json::from_value::<#field_type>(value.clone())
                    .map_err(|e| format!("Failed to deserialize value for field {}: {}", #field_name_str, e))?;
            }
        };
        
        quote! {
            #field_name_str => {
                #set_transform
                Ok(())
            },
        }
    }).collect::<Vec<_>>();

    // Generate the implementation
    let expanded = quote! {
        impl #name {
            // Create and initialize alias mapping
            fn build_alias_map() -> std::collections::HashMap<String, String> {
                let mut alias_map = std::collections::HashMap::new();
                #(#alias_mapping_entries)*
                alias_map
            }
            
            // Resolve field name (check if it's an alias)
            fn resolve_field_name(&self, field_name: &str) -> String {
                lazy_static::lazy_static! {
                    static ref ALIAS_MAP: std::collections::HashMap<String, String> = #name::build_alias_map();
                }
                
                if let Some(original_name) = ALIAS_MAP.get(field_name) {
                    original_name.clone()
                } else {
                    field_name.to_string()
                }
            }

            pub fn get(&self, field_name: &str) -> Option<serde_json::Value> {
                let resolved_name = self.resolve_field_name(field_name);
                
                match resolved_name.as_str() {
                    #(#get_match_arms)*
                    _ => None,
                }
            }

            pub fn set(&mut self, field_name: &str, value: serde_json::Value) -> Result<(), String> {
                let resolved_name = self.resolve_field_name(field_name);
                
                match resolved_name.as_str() {
                    #(#set_match_arms)*
                    _ => Err(format!("Field '{}' not found in struct", field_name)),
                }
            }

            pub fn as_map(&self) -> std::collections::HashMap<String, serde_json::Value> {
                let mut map = std::collections::HashMap::new();
                
                #(
                    let name = #field_names;
                    if let Some(value) = self.get(name) {
                        map.insert(name.to_string(), value);
                    }
                )*
                
                map
            }
            
            pub fn fields() -> Vec<String> {
                vec![
                    #(#field_names.to_string()),*
                ]
            }
            
            pub fn as_nested_map(&self) -> std::collections::HashMap<String, serde_json::Value> {
                let mut result = std::collections::HashMap::new();
                
                // Process each top-level field
                for field_name in Self::fields() {
                    if let Some(value) = self.get(&field_name) {
                        Self::add_to_nested_map(&mut result, &field_name, value);
                    }
                }
                
                result
            }
            
            fn add_to_nested_map(
                map: &mut std::collections::HashMap<String, serde_json::Value>, 
                key: &str, 
                value: serde_json::Value
            ) {
                map.insert(key.to_string(), value.clone());
                
                if let serde_json::Value::Object(obj) = &value {
                    for (obj_key, obj_value) in obj.iter() {
                        let nested_key = format!("{}.{}", key, obj_key);
                        Self::add_to_nested_map(map, &nested_key, obj_value.clone());
                    }
                }
            }

            pub fn get_path(&self, path: &str) -> Option<serde_json::Value> {
                let parts: Vec<&str> = path.split('.').collect();
                if parts.is_empty() {
                    return None;
                }
                
                // Resolve the first part if it's an alias
                let first_part = self.resolve_field_name(parts[0]);
                let mut current_value = self.get(&first_part)?;
                
                for &part in &parts[1..] {
                    match &current_value {
                        serde_json::Value::Object(map) => {
                            if let Some(value) = map.get(part) {
                                current_value = value.clone();
                            } else {
                                return None;
                            }
                        },
                        _ => return None, // not an object, can't go further
                    }
                }
                
                Some(current_value)
            }

            pub fn set_path(&mut self, path: &str, value: serde_json::Value) -> Result<(), String> {
                let parts: Vec<&str> = path.split('.').collect();
                if parts.is_empty() {
                    return Err("Empty path provided".to_string());
                }
                
                // Resolve the first part if it's an alias
                let first_part = self.resolve_field_name(parts[0]);
                
                if parts.len() == 1 {
                    return self.set(&first_part, value);
                }
                
                let mut current_value = self.get(&first_part)
                    .ok_or_else(|| format!("Field '{}' not found", first_part))?;
                
                let mut current_path = first_part.to_string();
                let mut temp_obj = current_value.clone();
                
                for i in 1..parts.len() {
                    let part = parts[i];
                    current_path.push_str(&format!(".{}", part));
                    
                    if i == parts.len() - 1 {
                        if let serde_json::Value::Object(ref mut map) = temp_obj {
                            map.insert(part.to_string(), value.clone());
                        } else {
                            return Err(format!("Cannot set field '{}' because parent is not an object", current_path));
                        }
                    } else {
                        if let serde_json::Value::Object(ref mut map) = temp_obj {
                            if let Some(existing) = map.get_mut(part) {
                                if !existing.is_object() {
                                    *existing = serde_json::Value::Object(serde_json::Map::new());
                                }
                            } else {
                                map.insert(part.to_string(), serde_json::Value::Object(serde_json::Map::new()));
                            }
                            
                            if let Some(next_obj) = map.get_mut(part) {
                                temp_obj = next_obj.clone();
                            } else {
                                return Err(format!("Failed to navigate to '{}' in path", current_path));
                            }
                        } else {
                            return Err(format!("Cannot navigate to '{}' because parent is not an object", current_path));
                        }
                    }
                }
                
                self.set(&first_part, temp_obj)
            }
            
            pub fn aliases() -> std::collections::HashMap<String, String> {
                Self::build_alias_map()
            }
        }
    };

    TokenStream::from(expanded)
}