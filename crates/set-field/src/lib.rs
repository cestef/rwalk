// dynamic_fields_advanced/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data, Fields};

#[proc_macro_derive(DynamicFields)]
pub fn dynamic_fields_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    // Extract field information
    let fields = match &input.data {
        Data::Struct(data_struct) => {
            match &data_struct.fields {
                Fields::Named(fields_named) => &fields_named.named,
                _ => panic!("DynamicFields can only be derived for structs with named fields"),
            }
        },
        _ => panic!("DynamicFields can only be derived for structs"),
    };

    // Extract field names for later use in as_map
    let field_names = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        
        quote! {
            #field_name_str
        }
    }).collect::<Vec<_>>();

    // Create match arms for get method (simple fields only)
    let get_match_arms = fields.iter().map(|field| {
        let field_name = &field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        
        quote! {
            #field_name_str => Some(serde_json::to_value(&self.#field_name).unwrap()),
        }
    });

    // Create match arms for set method (simple fields only)
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

    // Generate the implementation
    let expanded = quote! {
        impl #name {
            // Simple get for direct field access
            pub fn get(&self, field_name: &str) -> Option<serde_json::Value> {
                match field_name {
                    #(#get_match_arms)*
                    _ => None,
                }
            }

            // Simple set for direct field access
            pub fn set(&mut self, field_name: &str, value: serde_json::Value) -> Result<(), String> {
                match field_name {
                    #(#set_match_arms)*
                    _ => Err(format!("Field '{}' not found in struct", field_name)),
                }
            }

            // Return all fields as a map of field name to value
            pub fn as_map(&self) -> std::collections::HashMap<String, serde_json::Value> {
                let mut map = std::collections::HashMap::new();
                
                // Add each field to the map
                #(
                    let name = #field_names;
                    if let Some(value) = self.get(name) {
                        map.insert(name.to_string(), value);
                    }
                )*
                
                map
            }
            
            // List all available fields in the struct
            pub fn fields(&self) -> Vec<String> {
                vec![
                    #(#field_names.to_string()),*
                ]
            }
            
            // Return all fields as a nested map that also expands nested objects
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
            
            // Helper method to recursively process nested fields
            fn add_to_nested_map(
                map: &mut std::collections::HashMap<String, serde_json::Value>, 
                key: &str, 
                value: serde_json::Value
            ) {
                // Add the value itself first
                map.insert(key.to_string(), value.clone());
                
                // If it's an object, also add flattened keys
                if let serde_json::Value::Object(obj) = &value {
                    // Create a copy of the object to iterate through
                    for (obj_key, obj_value) in obj.iter() {
                        let nested_key = format!("{}.{}", key, obj_key);
                        Self::add_to_nested_map(map, &nested_key, obj_value.clone());
                    }
                }
            }

            // Advanced get with dot notation for nested fields
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
                        _ => return None, // Not an object, can't navigate further
                    }
                }
                
                Some(current_value)
            }

            // Advanced set with dot notation for nested fields
            pub fn set_path(&mut self, path: &str, value: serde_json::Value) -> Result<(), String> {
                let parts: Vec<&str> = path.split('.').collect();
                if parts.is_empty() {
                    return Err("Empty path provided".to_string());
                }
                
                // If it's a direct field, use the simple set method
                if parts.len() == 1 {
                    return self.set(parts[0], value);
                }
                
                // For nested fields, we need to get the object, modify it, and set it back
                let first_field = parts[0];
                let mut current_value = self.get(first_field)
                    .ok_or_else(|| format!("Field '{}' not found", first_field))?;
                
                // Navigate to the nested field and update it
                let mut current_path = first_field.to_string();
                let mut temp_obj = current_value.clone();
                
                for i in 1..parts.len() {
                    let part = parts[i];
                    current_path.push_str(&format!(".{}", part));
                    
                    if i == parts.len() - 1 {
                        // We've reached the final part, set the value here
                        if let serde_json::Value::Object(ref mut map) = temp_obj {
                            map.insert(part.to_string(), value.clone());
                        } else {
                            return Err(format!("Cannot set field '{}' because parent is not an object", current_path));
                        }
                    } else {
                        // We need to navigate deeper
                        if let serde_json::Value::Object(ref mut map) = temp_obj {
                            if let Some(existing) = map.get_mut(part) {
                                if !existing.is_object() {
                                    // Replace with empty object if not already an object
                                    *existing = serde_json::Value::Object(serde_json::Map::new());
                                }
                            } else {
                                // Create new empty object if key doesn't exist
                                map.insert(part.to_string(), serde_json::Value::Object(serde_json::Map::new()));
                            }
                            
                            // Get a reference to continue navigation
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
                
                // Set the modified value back to the struct
                self.set(first_field, temp_obj)
            }
        }
    };

    // Return the generated code
    TokenStream::from(expanded)
}