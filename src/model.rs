//!
//! Various model types that are not part of the GraphQL schema
//!

use std::ops::Range;

use std::sync::Arc;
use thiserror::Error;

///
/// The set of errors this application can produce
///
#[derive(Error, Debug, Clone)]
pub enum AppError {
    #[error("database")]
    Db(Arc<sqlx::Error>),
}

/// Convert an SQL error into an AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::Db(Arc::new(err))
    }
}

///
/// How to filter a list of TODO items
///
#[derive(Debug)]
pub struct TodoFilter {
    /// Filter by a range (offset..size)
    pub range: Range<u32>,
}
