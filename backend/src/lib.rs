/// Prontua Backend - Domain-centric Modular Monolith
pub mod domain;
pub mod features;
pub mod infrastructure;
pub mod modules;
pub mod shared;

pub use shared::{AppError, AppResult};
