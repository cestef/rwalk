macro_rules! create_registry {
    // Base pattern that handles the common registry setup
    (@base
        $static_name:ident,
        $constructor_type:ty,
        [$($implementor:ty),* $(,)?]
    ) => {
        // Aliases registry $registry_var_ALIASES
        static ALIASES: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
            let mut registry = HashMap::new();

            $(
                for &alias in <$implementor>::aliases() {
                    registry.insert(alias, <$implementor>::name());
                }
            )*

            registry
        });
        static REGISTRY: Lazy<HashMap<&'static str, $constructor_type>> = Lazy::new(|| {
            let mut registry = HashMap::new();

            $(
                registry.insert(<$implementor>::name(), <$implementor>::construct as $constructor_type);
            )*

            registry
        });

        pub struct $static_name;

        impl $static_name {
            pub fn list() -> Vec<(&'static str, Vec<&'static str>)> {
                let mut list = Vec::new();

                $(
                    let mut aliases = Vec::new();
                    for &alias in <$implementor>::aliases() {
                        aliases.push(alias);
                    }
                    list.push((<$implementor>::name(), aliases));
                )*

                list
            }
        }
    };

    // Transformer registry pattern
    (
        transform,
        $static_name:ident,
        $item_type:ty,
        [$($implementor:ty),* $(,)?]
    ) => {
        type TransformerConstructor = fn(Option<&str>) -> Result<Box<dyn Transform<$item_type>>>;

        create_registry!(@base
            $static_name,
            TransformerConstructor,
            [$($implementor),*]
        );

        impl $static_name {
            pub fn construct(name: &str, arg: Option<&str>) -> Result<Box<dyn Transform<$item_type>>> {
                let name = ALIASES.get(name).copied().unwrap_or(name);
                match REGISTRY.get(name) {
                    Some(constructor) => constructor(arg),
                    None => Err(crate::error!("Unknown transformer: {}", name)),
                }
            }
        }
    };

    // Filter registry pattern
    (
        filter,
        $static_name:ident,
        $item_type:ty,
        [$($implementor:ty),* $(,)?]
    ) => {
        type FilterConstructor = fn(&str, Option<HashSet<cowstr::CowStr>>) -> Result<Box<dyn Filter<$item_type>>>;

        create_registry!(@base
            $static_name,
            FilterConstructor,
            [$($implementor),*]
        );

        impl $static_name {
            pub fn construct(input: &str) -> Result<FilterExpr<Box<dyn Filter<$item_type>>>> {
                use crate::filters::expression::ExprParser;
                use crate::RwalkError;
                use std::collections::HashSet;

                let mut parser = ExprParser::new(input);
                let raw_expr = parser.parse::<String>()?;

                let expr = raw_expr.try_map(|e| {
                    let (key, value) = e
                        .split_once(':')
                        .ok_or_else(|| crate::error!("Invalid filter: {}", e))?;
                    let (filter, key) = if key.starts_with('[') {
                        // [filter]key
                        let (filter, key) = key.split_once(']').ok_or_else(|| crate::error!("Invalid filter: {}", e))?;
                        let filter = &filter[1..];

                        let filter = filter.split(',').map(cowstr::CowStr::from).collect::<HashSet<_>>();
                        (Some(filter), key)
                    } else {
                        (None, key)
                    };

                    let key = ALIASES.get(key).copied().unwrap_or(key);
                    match REGISTRY.get(key) {
                        Some(constructor) => constructor(value, filter),
                        None => Err(crate::error!("Unknown filter: {}", key)),
                    }
                })?;

                Ok(expr)
            }
        }
    };

    // Command registry pattern
    (
        command,
        $static_name:ident,
        $ctx_type:ty,
        [$($implementor:ty),* $(,)?]
    ) => {
        type CommandConstructor = fn() -> Box<dyn Command<$ctx_type>>;

        create_registry!(@base
            $static_name,
            CommandConstructor,
            [$($implementor),*]
        );

        impl $static_name {
            pub fn construct(name: &str) -> Result<Box<dyn Command<$ctx_type>>> {
                let name = ALIASES.get(name).copied().unwrap_or(name);
                match REGISTRY.get(name) {
                    Some(constructor) => Ok(constructor()),
                    None => Err(crate::error!("Unknown command: {}", name)),
                }
            }
        }
    };
}

pub(crate) use create_registry;
