/// Shared module exports
pub mod errors;
pub mod kernel;

pub use errors::{AppError, AppResult, ErrorResponse};
pub use kernel::{result::ResultExt, traits};
