/// Domain layer - core business entities and rules
pub mod entities;
pub mod value_objects;

pub use entities::User;
pub use value_objects::{Email, Password};
