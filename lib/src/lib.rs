pub mod auth;
#[cfg(any(feature = "diesel", feature = "sqlx"))]
pub mod db;
pub mod pidgey;
#[cfg(all(not(feature = "pokedex-v2"), feature = "serde"))]
mod pokedex_v1;
#[cfg(feature = "pokedex-v2")]
mod pokedex_v2;
pub mod pokedex {
    #[cfg(all(not(feature = "pokedex-v2"), feature = "serde"))]
    pub use crate::pokedex_v1::*;
    #[cfg(feature = "pokedex-v2")]
    pub use crate::pokedex_v2::*;
}
#[cfg(feature = "settings")]
pub mod settings;
pub mod types;
