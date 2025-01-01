#[macro_use(concat_string)]
extern crate concat_string;

pub mod auth;
pub mod pidgey;

#[cfg(feature = "serde")]
pub mod pokedex;
pub mod types;

#[cfg(feature = "diesel")]
pub mod db;
