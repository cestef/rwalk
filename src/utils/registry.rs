macro_rules! create_registry {
    // Base pattern that handles the common registry setup
    (@base
        $static_name:ident,
        $constructor_type:ty,
        $registry_var:ident,
        [$($implementor:ty),* $(,)?]
    ) => {
        static $registry_var: Lazy<HashMap<&'static str, $constructor_type>> = Lazy::new(|| {
            let mut registry = HashMap::new();

            $(
                registry.insert(<$implementor>::name(), <$implementor>::construct as $constructor_type);
                for &alias in <$implementor>::aliases() {
                    registry.insert(alias, <$implementor>::construct as $constructor_type);
                }
            )*

            registry
        });

        pub struct $static_name;

        impl $static_name {
            pub fn list() -> HashSet<&'static str> {
                $registry_var.keys().copied().collect()
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
            REGISTRY,
            [$($implementor),*]
        );

        impl $static_name {
            pub fn construct(name: &str, arg: Option<&str>) -> Result<Box<dyn Transform<$item_type>>> {
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
        type FilterConstructor = fn(&str, Option<&str>) -> Result<Box<dyn Filter<$item_type>>>;

        create_registry!(@base
            $static_name,
            FilterConstructor,
            REGISTRY,
            [$($implementor),*]
        );

        impl $static_name {
            pub fn construct(input: &str) -> Result<FilterExpr<Box<dyn Filter<$item_type>>>> {
                use crate::filters::expression::ExprParser;
                use crate::RwalkError;

                let mut parser = ExprParser::new(input);
                let raw_expr = parser.parse::<String>()?;

                let expr = raw_expr.try_map(|e| {
                    let (key, value) = e
                        .split_once(':')
                        .ok_or_else(|| crate::error!("Invalid filter: {}", e))?;
                    let (depth, key) = if key.starts_with('[') && key.ends_with(']') {
                        let filter = key
                            .strip_prefix('[')
                            .and_then(|s| s.strip_suffix(']'))
                            .ok_or_else(|| crate::error!("Invalid filter: {}", e))?;
                        let key = value;
                        (Some(filter), key)
                    } else {
                        (None, key)
                    };
                    match REGISTRY.get(key) {
                        Some(constructor) => constructor(value, depth),
                        None => Err(crate::error!("Unknown filter: {}", key)),
                    }
                })?;

                Ok(expr)
            }
        }
    };
}

pub(crate) use create_registry;
