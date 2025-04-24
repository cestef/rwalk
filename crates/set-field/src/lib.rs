// dynamic_fields_advanced/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(DynamicFields)]
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

    let field_names = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        
        quote! {
            #field_name_str
        }
    }).collect::<Vec<_>>();

    let get_match_arms = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        
        quote! {
            #field_name_str => Some(serde_json::to_value(&self.#field_name).unwrap()),
        }
    });

    let set_match_arms = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let field_type = &field.ty;
        
        quote! {
            #field_name_str => {
                self.#field_name = serde_json::from_value::<#field_type>(value.clone())
                    .map_err(|e| format!("Failed to deserialize value for field {}: {}", #field_name_str, e))?;
                Ok(())
            },
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn get(&self, field_name: &str) -> Option<serde_json::Value> {
                match field_name {
                    #(#get_match_arms)*
                    _ => None,
                }
            }

            pub fn set(&mut self, field_name: &str, value: serde_json::Value) -> Result<(), String> {
                match field_name {
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
            
            pub fn fields(&self) -> Vec<String> {
                vec![
                    #(#field_names.to_string()),*
                ]
            }
            
            pub fn as_nested_map(&self) -> std::collections::HashMap<String, serde_json::Value> {
                let mut result = std::collections::HashMap::new();
                
                // Process each top-level field
                for field_name in self.fields() {
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

                let mut current_value = self.get(parts[0])?;
                
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
                
                if parts.len() == 1 {
                    return self.set(parts[0], value);
                }
                
                let first_field = parts[0];
                let mut current_value = self.get(first_field)
                    .ok_or_else(|| format!("Field '{}' not found", first_field))?;
                
                let mut current_path = first_field.to_string();
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
                
                self.set(first_field, temp_obj)
            }
        }
    };

    TokenStream::from(expanded)
}