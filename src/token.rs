//! Zero-copy STEP tokens.
//!
//! A [`Token`] is the result of lexing a single attribute value inside
//! a STEP entity line.  Tokens borrow from the source string so no
//! allocations are needed during tokenization.

use crate::entity_id::EntityId;

/// A zero-copy token parsed from a STEP entity line.
///
/// Lifetime `'a` borrows from the input source text.
///
/// # Variants
///
/// ```text
/// #123          → EntityRef(EntityId(123))
/// 'hello'       → Str("hello")
/// 42 / -7       → Integer(42) / Integer(-7)
/// 3.14 / 0.     → Float(3.14) / Float(0.0)
/// .TRUE.        → Enum("TRUE")
/// (1, 2, 3)     → List([Integer(1), Integer(2), Integer(3)])
/// IFCBOOLEAN(…) → TypedValue("IFCBOOLEAN", […])
/// $             → Null
/// *             → Derived
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Token<'a> {
    /// Entity reference: `#123`
    EntityRef(EntityId),
    /// String literal: `'text'`
    Str(&'a str),
    /// Integer literal: `42`, `-7`
    Integer(i64),
    /// Floating-point literal: `3.14`, `0.`, `1.5E-10`
    Float(f64),
    /// Enumeration: `.TRUE.`, `.ELEMENT.`
    Enum(&'a str),
    /// Parenthesised list of tokens.
    List(Vec<Token<'a>>),
    /// Typed value wrapper: `IFCPARAMETERVALUE(0.)`
    TypedValue(&'a str, Vec<Token<'a>>),
    /// The null marker `$`.
    Null,
    /// The derived marker `*`.
    Derived,
}
