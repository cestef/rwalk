use eyre::eyre;
use num_traits::PrimInt;
use std::{fmt::Display, str::FromStr};

#[derive(Clone, Copy)]
pub struct IntRange<T>
where
    T: PrimInt,
{
    pub start: T,
    pub end: T,
}

impl<T> IntRange<T>
where
    T: PrimInt,
{
    pub fn new(start: T, end: T) -> Self {
        Self { start, end }
    }
    pub fn contains(&self, value: T) -> bool {
        value >= self.start && value <= self.end
    }
}

impl<T> std::fmt::Debug for IntRange<T>
where
    T: PrimInt + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.start, self.end)
    }
}

impl<T> FromStr for IntRange<T>
where
    T: PrimInt + FromStr,
{
    type Err = eyre::Report;
    // Range can be parsed from a string in the format "start-end" or "start"
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('-').collect();

        // If there is only one part, we assume it is the start and the end is the same
        if parts.len() == 1 {
            let start = parts[0].parse().map_err(|_| eyre!("Invalid start value"))?;
            return Ok(IntRange::new(start, start));
        }

        if parts.len() != 2 {
            return Err(eyre!("Invalid range format"));
        }

        let start = parts[0].parse().map_err(|_| eyre!("Invalid start value"))?;
        let end = parts[1].parse().map_err(|_| eyre!("Invalid end value"))?;

        Ok(IntRange::new(start, end))
    }
}
