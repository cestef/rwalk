//! Facet-based helpers for dynamic field access
//!
//! This module provides an extension trait for `Opts` that wraps facet's
//! Peek and Poke APIs to provide a dyn-fields-compatible interface.

use facet::Facet;
use facet_reflect::Peek;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

use super::Opts;

/// Extension trait for Opts to provide dyn-fields-like API using facet
pub trait OptsFacetExt {
    /// Get a field value by path (dot notation supported)
    fn get_path(&self, path: &str) -> Option<JsonValue>;

    /// Set a field value by path
    fn set_path(&mut self, path: &str, value: JsonValue) -> Result<(), String>;

    /// Get all fields as a nested map (supports dot notation)
    fn as_nested_map(&self) -> HashMap<String, JsonValue>;

    /// List all field names
    fn fields() -> Vec<String>;
}

impl OptsFacetExt for Opts {
    fn get_path(&self, path: &str) -> Option<JsonValue> {
        use crate::cli::alias;

        // Resolve aliases
        let parts: Vec<&str> = path.split('.').collect();
        let first_part = alias::resolve_field_name(parts[0]);

        // Create a Peek to access the value
        let peek = Peek::new(self);

        // Convert to struct view
        let peek_struct = peek.into_struct().ok()?;

        // Get the field by name
        let mut current_peek = peek_struct.field_by_name(first_part).ok()?;

        // Navigate nested paths if any
        for part in parts.iter().skip(1) {
            let nested_struct = current_peek.into_struct().ok()?;
            current_peek = nested_struct.field_by_name(part).ok()?;
        }

        // Convert Peek to JSON string using facet-json
        let json_str = facet_json::peek_to_string(current_peek).ok()?;

        // Parse JSON string to serde_json::Value
        serde_json::from_str(&json_str).ok()
    }

    fn set_path(&mut self, path: &str, value: JsonValue) -> Result<(), String> {
        use crate::cli::alias;

        let parts: Vec<&str> = path.split('.').collect();
        if parts.is_empty() {
            return Err("Empty field path".to_string());
        }

        let first_part = alias::resolve_field_name(parts[0]);

        // For now, only support top-level field setting (no nested paths)
        if parts.len() > 1 {
            return Err("Nested field setting not yet supported".to_string());
        }

        // Serialize current state to JSON
        let mut json_val = serde_json::to_value(&*self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;

        // Update the field in JSON
        if let Some(obj) = json_val.as_object_mut() {
            obj.insert(first_part.to_string(), value);
        } else {
            return Err("Not a struct".to_string());
        }

        // Deserialize back to Opts
        *self = serde_json::from_value(json_val)
            .map_err(|e| format!("Failed to deserialize: {}", e))?;

        Ok(())
    }

    fn as_nested_map(&self) -> HashMap<String, JsonValue> {
        use crate::cli::alias;
        let mut result = HashMap::new();

        // Create a Peek view of the struct
        let peek = Peek::new(self);

        if let Ok(peek_struct) = peek.into_struct() {
            // Iterate over all fields using facet's Shape
            if let facet::Type::User(facet::UserType::Struct(struct_type)) = Opts::SHAPE.ty {
                for (i, field) in struct_type.fields.iter().enumerate() {
                    // Skip fields marked with skip
                    if alias::should_skip_field(field.name) {
                        continue;
                    }

                    // Get the field value using Peek
                    if let Ok(field_peek) = peek_struct.field(i) {
                        // Convert to JSON using facet-json
                        if let Ok(json_str) = facet_json::peek_to_string(field_peek)
                            && let Ok(json_val) = serde_json::from_str::<JsonValue>(&json_str) {
                                add_to_nested_map(&mut result, field.name, json_val);
                            }
                    }
                }
            }
        }

        result
    }

    fn fields() -> Vec<String> {
        crate::cli::alias::all_field_names()
    }
}

/// Recursively add fields to nested map with dot notation
///
/// For nested structures, creates both the direct field entry and
/// flattened entries with dot notation (e.g., "github.clientId")
fn add_to_nested_map(map: &mut HashMap<String, JsonValue>, key: &str, value: JsonValue) {
    map.insert(key.to_string(), value.clone());

    if let JsonValue::Object(obj) = &value {
        for (obj_key, obj_value) in obj.iter() {
            let nested_key = format!("{}.{}", key, obj_key);
            add_to_nested_map(map, &nested_key, obj_value.clone());
        }
    }
}
