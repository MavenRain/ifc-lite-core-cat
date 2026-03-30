//! Newtype wrapper for IFC entity identifiers.
//!
//! Every entity in a STEP file is assigned a numeric id (e.g. `#123`).
//! [`EntityId`] prevents accidental confusion with other `u32` values.

/// A strongly-typed IFC entity identifier.
///
/// In the STEP physical file format each entity is labelled with a
/// unique positive integer preceded by `#`.  [`EntityId`] wraps that
/// integer so it cannot be silently mixed with array indices, counts,
/// or other unrelated numbers.
///
/// # Examples
///
/// ```
/// use ifc_lite_core_cat::EntityId;
///
/// let id = EntityId::new(42);
/// assert_eq!(id.value(), 42);
/// assert_eq!(format!("{id}"), "#42");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EntityId(u32);

impl EntityId {
    /// Wrap a raw entity number.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Unwrap to the underlying `u32`.
    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl From<u32> for EntityId {
    fn from(raw: u32) -> Self {
        Self(raw)
    }
}

impl From<EntityId> for u32 {
    fn from(id: EntityId) -> Self {
        id.0
    }
}
