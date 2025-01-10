use crate::Result;
use std::{fmt::Debug, sync::Arc};

pub mod expression;
pub mod length;
pub mod registry;
pub mod status;

#[derive(Debug, Clone)]
pub struct Filtrerer<T> {
    filters: Arc<Vec<Box<dyn Filter<T>>>>,
}

unsafe impl<T> Send for Filtrerer<T> where Box<dyn Filter<T>>: Send {}
unsafe impl<T> Sync for Filtrerer<T> where Box<dyn Filter<T>>: Sync {}

pub trait Filter<T>: Debug + Send + Sync {
    fn filter(&self, item: &T) -> bool;
    fn name() -> &'static str
    where
        Self: Sized;
    fn aliases() -> &'static [&'static str]
    where
        Self: Sized,
    {
        &[]
    }
    fn needs_body(&self) -> bool {
        false
    }
    fn construct(arg: &str) -> Result<Box<dyn Filter<T>>>
    where
        Self: Sized;
}

impl<T> Filtrerer<T> {
    pub fn new<I>(filters: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn Filter<T>>>,
    {
        Self {
            filters: Arc::new(filters.into_iter().collect()),
        }
    }

    pub fn all(&self, item: &T) -> bool {
        self.filters.iter().all(|f| f.filter(item))
    }

    pub fn any(&self, item: &T) -> bool {
        self.filters.iter().any(|f| f.filter(item))
    }

    pub fn needs_body(&self) -> bool {
        self.filters.iter().any(|f| f.needs_body())
    }
}
