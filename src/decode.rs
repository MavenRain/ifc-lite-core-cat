//! Entity decoding via [`comp_cat_rs::effect::io::Io`].
//!
//! Given a byte range in the source content, [`decode_entity`]
//! tokenises the STEP line, converts tokens to owned
//! [`AttributeValue`]s, and yields
//! a [`DecodedEntity`] inside an
//! [`Io`].
//!
//! Because decoding is a pure function over a byte slice, the `Io`
//! suspension is a thin wrapper that defers execution without
//! performing real I/O.  This lets callers compose decoding with
//! other effects via `flat_map` and delay `run` until the boundary.

use std::collections::HashMap;

use comp_cat_rs::effect::io::Io;

use crate::attribute::{AttributeValue, DecodedEntity};
use crate::entity_id::EntityId;
use crate::error::Error;
use crate::parse::parse_entity;

/// Decode one entity from a byte range in `content`.
///
/// The returned [`Io`] captures the parse as a suspended computation
/// so it composes with other effects.  Call `.run()` only at the
/// outermost boundary.
///
/// # Examples
///
/// ```
/// use ifc_lite_core::{decode::decode_entity, EntityId, IfcType};
///
/// let line = "#5=IFCWALL('guid',$,$,$,'W1',$,$,$);";
/// let entity = decode_entity(line, 0, line.len()).run().unwrap();
/// assert_eq!(entity.id(), EntityId::new(5));
/// assert_eq!(entity.get_string(4), Some("W1"));
/// ```
#[must_use]
pub fn decode_entity(content: &str, start: usize, end: usize) -> Io<Error, DecodedEntity> {
    let slice = content[start..end].to_string();
    Io::suspend(move || {
        let (id, ifc_type, tokens) = parse_entity(&slice)?;
        let attributes = tokens.iter().map(AttributeValue::from_token).collect();
        Ok(DecodedEntity::new(id, ifc_type, attributes))
    })
}

/// Decode an entity by its [`EntityId`] using a prebuilt index.
///
/// The `index` maps entity ids to `(start, end)` byte offsets in
/// `content`.
///
/// # Errors
///
/// Returns [`Error::InvalidEntityRef`] if the id is not in the index.
#[must_use]
pub fn decode_by_id<S: std::hash::BuildHasher>(
    content: &str,
    index: &HashMap<EntityId, (usize, usize), S>,
    entity_id: EntityId,
) -> Io<Error, DecodedEntity> {
    index.get(&entity_id).map_or_else(
        || Io::suspend(move || Err(Error::InvalidEntityRef(entity_id.value()))),
        |&(start, end)| decode_entity(content, start, end),
    )
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ifc_type::IfcType;
    use crate::scan::build_entity_index;

    #[test]
    fn decode_entity_from_range() {
        let content = "#2=IFCWALL('guid',$,$,$,'Wall-001',$,$,$);";
        let entity = decode_entity(content, 0, content.len())
            .run()
            .expect("decode");
        assert_eq!(entity.id(), EntityId::new(2));
        assert_eq!(*entity.ifc_type(), IfcType::IfcWall);
        assert_eq!(entity.get_string(4), Some("Wall-001"));
    }

    #[test]
    fn decode_by_id_success() {
        let content = "\
#1=IFCPROJECT('guid',$,$,$,$,$,$,$,$);
#5=IFCWALL('guid2',$,$,$,'Wall-001',$,$,$);
";
        let index = build_entity_index(content);
        let entity = decode_by_id(content, &index, EntityId::new(5))
            .run()
            .expect("decode");
        assert_eq!(entity.id(), EntityId::new(5));
        assert_eq!(entity.get_string(4), Some("Wall-001"));
    }

    #[test]
    fn decode_by_id_missing() {
        let content = "#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);";
        let index = build_entity_index(content);
        let err = decode_by_id(content, &index, EntityId::new(99))
            .run()
            .expect_err("should fail");
        assert!(matches!(err, Error::InvalidEntityRef(99)));
    }
}
