mod io;

pub use io::*;

use std::error::Error;

/// Convenience definition for the boxed error type.
pub type BoxedError = Box<dyn Error + Send + Sync>;
