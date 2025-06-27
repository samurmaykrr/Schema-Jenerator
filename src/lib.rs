pub mod cli;
pub mod config;
pub mod error;
pub mod schema;
pub mod validation;

pub use error::AppError;
pub use schema::{generate_schema, SchemaOutputTier};

pub type Result<T> = std::result::Result<T, AppError>;