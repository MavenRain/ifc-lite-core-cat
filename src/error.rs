//! Hand-rolled error type for IFC parsing operations.
//!
//! All fallible operations in this crate return [`Result<T, Error>`].
//! Error variants are converted via [`From`] impls so `?` propagation works
//! throughout the codebase.

/// Errors that can occur during IFC parsing.
#[derive(Debug)]
pub enum Error {
    /// A syntactic error encountered during STEP tokenization or entity parsing.
    Parse {
        /// Byte offset in the source where the error was detected.
        position: usize,
        /// Human-readable description of what went wrong.
        message: String,
    },

    /// An entity reference (`#id`) pointed to a non-existent entity.
    InvalidEntityRef(u32),

    /// A type name string did not map to any known [`crate::ifc_type::IfcType`] variant.
    InvalidIfcType(String),

    /// The tokenizer found a token it did not expect at the given position.
    UnexpectedToken {
        /// Byte offset in the source.
        position: usize,
        /// What the parser expected to see.
        expected: String,
        /// What the parser actually encountered.
        got: String,
    },

    /// An I/O error propagated from [`std::io`].
    Io(std::io::Error),

    /// A UTF-8 decoding error propagated from [`std::str`].
    Utf8(std::str::Utf8Error),
}

/// Convenience alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

// ---------------------------------------------------------------------------
// Display
// ---------------------------------------------------------------------------

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { position, message } => {
                write!(f, "parse error at position {position}: {message}")
            }
            Self::InvalidEntityRef(id) => write!(f, "invalid entity reference: #{id}"),
            Self::InvalidIfcType(name) => write!(f, "invalid IFC type: {name}"),
            Self::UnexpectedToken {
                position,
                expected,
                got,
            } => write!(
                f,
                "unexpected token at position {position}: expected {expected}, got {got}"
            ),
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Utf8(e) => write!(f, "UTF-8 error: {e}"),
        }
    }
}

// ---------------------------------------------------------------------------
// std::error::Error
// ---------------------------------------------------------------------------

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            Self::Utf8(e) => Some(e),
            Self::Parse { .. }
            | Self::InvalidEntityRef(_)
            | Self::InvalidIfcType(_)
            | Self::UnexpectedToken { .. } => None,
        }
    }
}

// ---------------------------------------------------------------------------
// From conversions for ? ergonomics
// ---------------------------------------------------------------------------

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Self::Utf8(e)
    }
}

// ---------------------------------------------------------------------------
// Constructors
// ---------------------------------------------------------------------------

impl Error {
    /// Create a parse error at the given byte position.
    #[must_use]
    pub fn parse(position: usize, message: impl Into<String>) -> Self {
        Self::Parse {
            position,
            message: message.into(),
        }
    }

    /// Create an unexpected-token error.
    #[must_use]
    pub fn unexpected(
        position: usize,
        expected: impl Into<String>,
        got: impl Into<String>,
    ) -> Self {
        Self::UnexpectedToken {
            position,
            expected: expected.into(),
            got: got.into(),
        }
    }
}
