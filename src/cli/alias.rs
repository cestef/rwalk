//! Field alias resolution for facet reflection
//!
//! This module provides a custom alias layer on top of facet to support
//! user-friendly field names like "wordlist" instead of "wordlists".

use std::collections::HashMap;
use once_cell::sync::Lazy;
use facet::Facet;

use super::Opts;

/// Mapping from alias names to actual field names
static ALIAS_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("wordlist", "wordlists");
    map
});

/// Resolve a field name, converting aliases to actual field names
///
/// # Examples
///
/// ```
/// let actual = resolve_field_name("wordlist");
/// assert_eq!(actual, "wordlists");
///
/// let actual = resolve_field_name("threads");
/// assert_eq!(actual, "threads");
/// ```
pub fn resolve_field_name(name: &str) -> &str {
    ALIAS_MAP.get(name).copied().unwrap_or(name)
}

/// Check if a field should be skipped from dynamic access
///
/// Fields marked with #[facet(skip)] should not appear in field listings
/// or be accessible via dynamic field access.
pub fn should_skip_field(name: &str) -> bool {
    matches!(name, "help" | "help_long" | "generate_markdown" | "interactive")
}

/// Get all valid field names including both actual fields and aliases
///
/// This function introspects the Opts struct using facet's Shape system
/// and returns all field names that should be accessible, excluding
/// skipped fields but including aliases.
pub fn all_field_names() -> Vec<String> {
    let mut names = Vec::new();

    // Get actual field names from Facet's Shape
    if let facet::Type::User(facet::UserType::Struct(struct_type)) = Opts::SHAPE.ty {
        for field in struct_type.fields {
            // Skip fields marked with facet(skip)
            if should_skip_field(field.name) {
                continue;
            }
            names.push(field.name.to_string());
        }
    }

    // Add aliases
    for (alias, _) in ALIAS_MAP.iter() {
        names.push(alias.to_string());
    }

    names.sort();
    names.dedup();
    names
}
