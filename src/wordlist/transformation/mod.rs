pub mod case;

use crate::{error::RwalkError, Result};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Transformer<T> {
    transformers: Arc<Vec<Box<dyn Transform<T>>>>,
}

unsafe impl<T> Send for Transformer<T> {}
unsafe impl<T> Sync for Transformer<T> {}

pub trait Transform<T>: Debug {
    fn transform(&self, item: &mut T);
    fn name() -> &'static str
    where
        Self: Sized;
    fn aliases() -> &'static [&'static str]
    where
        Self: Sized,
    {
        &[]
    }
    fn construct(arg: Option<&str>) -> Result<Box<dyn Transform<T>>>
    where
        Self: Sized;
}

impl<T> Transformer<T> {
    pub fn new<I>(transformers: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn Transform<T>>>,
    {
        Self {
            transformers: Arc::new(transformers.into_iter().collect()),
        }
    }

    pub fn apply(&self, item: &mut T) {
        for transformer in self.transformers.iter() {
            transformer.transform(item);
        }
    }
}

macro_rules! create_transformer_registry {
    ($static_name:ident, $item_type:ty, [$($transformer:ty),*]) => {
        type TransformerConstructor = fn(Option<&str>) -> Result<Box<dyn Transform<$item_type>>>;

        static REGISTRY: Lazy<HashMap<&'static str, TransformerConstructor>> = Lazy::new(|| {
            let mut registry = HashMap::new();

            $(
                // Register main name
                registry.insert(<$transformer>::name(), <$transformer>::construct as TransformerConstructor);
                // Register aliases
                for &alias in <$transformer>::aliases() {
                    registry.insert(alias, <$transformer>::construct as TransformerConstructor);
                }
            )*

            registry
        });

        pub struct $static_name;

        impl $static_name {
            pub fn construct(name: &str, arg: Option<&str>) -> Result<Box<dyn Transform<$item_type>>> {
                match REGISTRY.get(name) {
                    Some(constructor) => constructor(arg),
                    None => Err(crate::error!("Unknown transformer: {}", name)),
                }
            }

            pub fn list() -> HashSet<&'static str> {
                REGISTRY.keys().copied().collect()
            }
        }
    };
}

pub(crate) use create_transformer_registry;

create_transformer_registry!(WordlistTransformerRegistry, String, [case::CaseTransformer]);
