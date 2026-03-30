//! Entity scanning: discover entities in STEP content.
//!
//! The scanner walks through raw bytes looking for `#id=TYPE(…);`
//! patterns without fully tokenizing every attribute.  This is the
//! first pass over an IFC file and feeds the decoder with byte
//! offsets.
//!
//! The public API exposes both a pure helper ([`scan_next`]) and a
//! [`comp_cat_rs::effect::stream::Stream`]-based interface
//! ([`scan_entities`]) for composable, lazy iteration.

use std::sync::Arc;

use comp_cat_rs::effect::io::Io;
use comp_cat_rs::effect::stream::Stream;

use crate::entity_id::EntityId;
use crate::error::Error;
use crate::ifc_type::IfcType;

// ═══════════════════════════════════════════════════════════════════
// ScannedEntity
// ═══════════════════════════════════════════════════════════════════

/// A lightweight descriptor produced by the first-pass scanner.
///
/// Contains just enough information to locate and decode the entity
/// later.  All fields are private.
#[derive(Debug, Clone)]
pub struct ScannedEntity {
    id: EntityId,
    ifc_type: IfcType,
    type_name: String,
    start: usize,
    end: usize,
}

impl ScannedEntity {
    /// Construct a new scanned entity descriptor.
    #[must_use]
    pub fn new(
        id: EntityId,
        ifc_type: IfcType,
        type_name: String,
        start: usize,
        end: usize,
    ) -> Self {
        Self {
            id,
            ifc_type,
            type_name,
            start,
            end,
        }
    }

    /// Entity identifier.
    #[must_use]
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Parsed IFC type.
    #[must_use]
    pub fn ifc_type(&self) -> &IfcType {
        &self.ifc_type
    }

    /// Raw type name as it appeared in the file.
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// Byte offset where the entity starts (at `#`).
    #[must_use]
    pub fn start(&self) -> usize {
        self.start
    }

    /// Byte offset one past the trailing `;`.
    #[must_use]
    pub fn end(&self) -> usize {
        self.end
    }
}

// ═══════════════════════════════════════════════════════════════════
// Pure scanning helpers
// ═══════════════════════════════════════════════════════════════════

/// Find the position of the entity-terminating `;`, respecting
/// single-quoted strings that may contain semicolons.
///
/// Returns the offset relative to `content[0]`.
fn find_entity_end(content: &[u8]) -> Option<usize> {
    #[derive(Clone, Copy)]
    enum S {
        Normal,
        InString,
        QuoteInString,
    }

    content
        .iter()
        .enumerate()
        .try_fold(S::Normal, |state, (i, &b)| match (state, b) {
            (S::Normal | S::QuoteInString, b';') => Err(i),
            (S::Normal | S::QuoteInString, b'\'') => Ok(S::InString),
            (S::Normal | S::QuoteInString, _) => Ok(S::Normal),
            (S::InString, b'\'') => Ok(S::QuoteInString),
            (S::InString, _) => Ok(S::InString),
        })
        .err()
}

/// Parse a `u32` from a slice of ASCII digit bytes.
fn parse_u32_from_digits(bytes: &[u8]) -> u32 {
    bytes.iter().fold(0u32, |acc, &b| {
        acc.wrapping_mul(10)
            .wrapping_add(u32::from(b.wrapping_sub(b'0')))
    })
}

/// Scan for the next entity starting at `position` in `bytes`.
///
/// Returns `Some((scanned_entity, next_position))` or `None` when
/// no more entities remain.
///
/// This is a **pure function**: it takes an immutable position and
/// produces the next state.
#[must_use]
pub fn scan_next(bytes: &[u8], position: usize) -> Option<(ScannedEntity, usize)> {
    let remaining = bytes.get(position..)?;
    let hash_offset = memchr::memchr(b'#', remaining)?;
    let line_start = position + hash_offset;

    // ── entity id ──────────────────────────────────────────────────
    let id_start = line_start + 1;
    let digit_count = bytes
        .get(id_start..)?
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .count();

    (digit_count > 0).then_some(())?;

    let id_end = id_start + digit_count;
    let id = parse_u32_from_digits(bytes.get(id_start..id_end)?);

    // ── skip whitespace → '=' ──────────────────────────────────────
    let after_id = bytes.get(id_end..)?;
    let ws1 = after_id
        .iter()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    let eq_pos = id_end + ws1;

    bytes.get(eq_pos).filter(|&&b| b == b'=')?;

    // ── skip whitespace → type name ────────────────────────────────
    let after_eq = bytes.get(eq_pos + 1..)?;
    let ws2 = after_eq
        .iter()
        .take_while(|b| b.is_ascii_whitespace())
        .count();
    let type_start = eq_pos + 1 + ws2;

    let type_slice = bytes.get(type_start..)?;
    let type_len = type_slice
        .iter()
        .take_while(|&&b| b != b'(' && !b.is_ascii_whitespace())
        .count();

    (type_len > 0).then_some(())?;

    let type_end = type_start + type_len;
    let type_name = std::str::from_utf8(bytes.get(type_start..type_end)?).ok()?;

    // ── find entity end (semicolon) ────────────────────────────────
    let entity_slice = bytes.get(line_start..)?;
    let semi_offset = find_entity_end(entity_slice)?;
    let line_end = line_start + semi_offset + 1;

    Some((
        ScannedEntity::new(
            EntityId::new(id),
            IfcType::from_name(type_name),
            type_name.to_string(),
            line_start,
            line_end,
        ),
        line_end,
    ))
}

// ═══════════════════════════════════════════════════════════════════
// comp-cat-rs Stream API
// ═══════════════════════════════════════════════════════════════════

/// Produce a lazy [`Stream`] of [`ScannedEntity`] values from owned
/// content.
///
/// The content `String` is moved into the stream state; each step
/// advances the scan position and yields the next entity.  Call
/// `.collect()` or `.fold(…)` to materialise results inside an
/// [`Io`].
///
/// # Examples
///
/// ```
/// use ifc_lite_core_cat::scan::scan_entities;
///
/// let content = "#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);".to_string();
/// let stream = scan_entities(content);
/// let all = stream.collect().run().unwrap();
/// assert_eq!(all.len(), 1);
/// ```
#[must_use]
pub fn scan_entities(content: String) -> Stream<Error, ScannedEntity> {
    Stream::unfold(
        (content, 0usize),
        Arc::new(|(content, pos): (String, usize)| {
            Io::suspend(move || {
                Ok(scan_next(content.as_bytes(), pos)
                    .map(|(entity, next_pos)| (entity, (content, next_pos))))
            })
        }),
    )
}

/// Build a [`HashMap`](std::collections::HashMap) index mapping
/// [`EntityId`] → `(start, end)` byte offsets for O(1) lookup.
///
/// This performs a single pass over the content.
#[must_use]
pub fn build_entity_index(content: &str) -> std::collections::HashMap<EntityId, (usize, usize)> {
    let bytes = content.as_bytes();
    std::iter::successors(
        scan_next(bytes, 0),
        #[allow(clippy::needless_borrows_for_generic_args)]
        |&(ref _entity, pos)| scan_next(bytes, pos),
    )
    .map(|(entity, _)| (entity.id(), (entity.start(), entity.end())))
    .collect()
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_single_entity() {
        let content = "#1=IFCPROJECT('guid',$,$,$,$,$,$,$,$);";
        let (entity, next_pos) = scan_next(content.as_bytes(), 0).expect("scan_next");
        assert_eq!(entity.id(), EntityId::new(1));
        assert_eq!(*entity.ifc_type(), IfcType::IfcProject);
        assert_eq!(next_pos, content.len());
    }

    #[test]
    fn scan_multiple_entities() {
        let content = "\
#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);
#2=IFCWALL('g2',$,$,$,$,$,$,$);
#3=IFCDOOR('g3',$,$,$,$,$,$,$);
";
        let bytes = content.as_bytes();
        let (e1, p1) = scan_next(bytes, 0).expect("first");
        let (e2, p2) = scan_next(bytes, p1).expect("second");
        let (e3, p3) = scan_next(bytes, p2).expect("third");
        assert!(scan_next(bytes, p3).is_none());

        assert_eq!(e1.id(), EntityId::new(1));
        assert_eq!(e2.id(), EntityId::new(2));
        assert_eq!(e3.id(), EntityId::new(3));
    }

    #[test]
    fn scan_respects_quoted_semicolons() {
        let content = "#1=IFCWALL('has;semi',$,$,$,$,$,$,$);";
        let (entity, _) = scan_next(content.as_bytes(), 0).expect("scan");
        assert_eq!(entity.id(), EntityId::new(1));
    }

    #[test]
    fn stream_collect() {
        let content = "\
#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);
#2=IFCWALL('g2',$,$,$,$,$,$,$);
"
        .to_string();
        let all = scan_entities(content).collect().run().expect("collect");
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn build_index_maps_ids_to_offsets() {
        let content = "\
#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);
#5=IFCWALL('g2',$,$,$,$,$,$,$);
";
        let index = build_entity_index(content);
        assert_eq!(index.len(), 2);
        assert!(index.contains_key(&EntityId::new(1)));
        assert!(index.contains_key(&EntityId::new(5)));
    }
}
