#[macro_use]
extern crate serde;

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate searchable_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use searchable_derive::*;

pub trait Searchable {
    fn get_value(&self, field: &str) -> Option<&[u8]>;
}

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct Schema {
//     pub name: String,
//     pub fields: Vec<Field>,
// }

// #[derive(Clone, Debug, Serialize, Deserialize)]
// pub struct Field {
//     pub field_name: String,
//     pub field_type: String,
//     pub doc: Option<String>,
// }


