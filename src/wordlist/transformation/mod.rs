mod case;
mod encode;
mod prefix;
mod remove;
mod replace;
mod suffix;

create_registry!(
    transform,
    WordlistTransformerRegistry,
    String,
    [
        case::CaseTransformer,
        prefix::PrefixTransformer,
        suffix::SuffixTransformer,
        replace::ReplaceTransformer,
        remove::RemoveTransformer,
        encode::EncodeTransformer
    ]
);

use crate::utils::registry::create_registry;
use crate::{error::RwalkError, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;
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
