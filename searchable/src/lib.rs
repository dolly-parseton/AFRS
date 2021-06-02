extern crate serde;

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate searchable_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use searchable_derive::*;

pub trait Searchable {
    fn get_value(&self, field: &str) -> Option<Vec<u8>>;
}

impl<T: IntoIterator<Item = impl Searchable>> Searchable for T {
    fn get_value(&self, field: &str) -> Option<Vec<u8>> {
        let mut v = Vec::new();
        for i in self {
            v.append(i.get_value(field));
        }
        Some(v)
    }
}

impl Searchable for str {
    fn get_value(&self, _field: &str) -> Option<Vec<u8>> {
        Some(self.as_bytes().to_owned())
    }
}

impl Searchable for String {
    fn get_value(&self, _field: &str) -> Option<Vec<u8>> {
        Some(self.as_bytes().to_owned())
    }
}

impl Searchable for u8 {
    fn get_value(&self, _field: &str) -> Option<Vec<u8>> {
        Some(vec![*self])
    }
}

impl Searchable for u16 {
    fn get_value(&self, _field: &str) -> Option<Vec<u8>> {
        Some(vec![(self >> 8) as u8, *self as u8])
    }
}

impl Searchable for u32 {
    fn get_value(&self, _field: &str) -> Option<Vec<u8>> {
        Some(vec![
            (self >> 24) as u8,
            (self >> 16) as u8,
            (self >> 8) as u8,
            *self as u8,
        ])
    }
}

impl Searchable for u64 {
    fn get_value(&self, _field: &str) -> Option<Vec<u8>> {
        Some(vec![
            (self >> 56) as u8,
            (self >> 48) as u8,
            (self >> 40) as u8,
            (self >> 32) as u8,
            (self >> 24) as u8,
            (self >> 16) as u8,
            (self >> 8) as u8,
            *self as u8,
        ])
    }
}

impl Searchable for u128 {
    fn get_value(&self, _field: &str) -> Option<Vec<u8>> {
        Some(vec![
            (self >> 120) as u8,
            (self >> 112) as u8,
            (self >> 104) as u8,
            (self >> 96) as u8,
            (self >> 88) as u8,
            (self >> 80) as u8,
            (self >> 72) as u8,
            (self >> 64) as u8,
            (self >> 56) as u8,
            (self >> 48) as u8,
            (self >> 40) as u8,
            (self >> 32) as u8,
            (self >> 24) as u8,
            (self >> 16) as u8,
            (self >> 8) as u8,
            *self as u8,
        ])
    }
}
