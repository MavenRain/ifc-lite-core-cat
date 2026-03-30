//! Owned attribute values and decoded IFC entities.
//!
//! [`AttributeValue`] is the heap-owned counterpart of [`crate::token::Token`].
//! After tokenization the tokens are converted into [`AttributeValue`]s so they
//! can outlive the source string.  A [`DecodedEntity`] bundles an entity's id,
//! type, and its attribute vector.

use crate::entity_id::EntityId;
use crate::ifc_type::IfcType;
use crate::token::Token;

// ───────────────────────────────────────────────────────────────────
// AttributeValue
// ───────────────────────────────────────────────────────────────────

/// An owned, heap-allocated IFC attribute value.
///
/// Created by converting a borrowed [`Token`] via [`AttributeValue::from_token`].
#[derive(Debug, Clone)]
pub enum AttributeValue {
    /// Entity reference.
    EntityRef(EntityId),
    /// Owned string.
    String(String),
    /// Integer value.
    Integer(i64),
    /// Floating-point value.
    Float(f64),
    /// Enumeration value (without the surrounding dots).
    Enum(String),
    /// List of attribute values.
    List(Vec<AttributeValue>),
    /// Null (`$`) or missing.
    Null,
    /// Derived (`*`).
    Derived,
}

impl AttributeValue {
    /// Convert a borrowed [`Token`] into an owned [`AttributeValue`].
    #[must_use]
    pub fn from_token(token: &Token<'_>) -> Self {
        match token {
            Token::EntityRef(id) => Self::EntityRef(*id),
            Token::Str(s) => Self::String((*s).to_string()),
            Token::Integer(i) => Self::Integer(*i),
            Token::Float(f) => Self::Float(*f),
            Token::Enum(e) => Self::Enum((*e).to_string()),
            Token::List(items) => Self::List(items.iter().map(Self::from_token).collect()),
            Token::TypedValue(type_name, args) => {
                let values = std::iter::once(Self::String((*type_name).to_string()))
                    .chain(args.iter().map(Self::from_token))
                    .collect();
                Self::List(values)
            }
            Token::Null => Self::Null,
            Token::Derived => Self::Derived,
        }
    }

    // ── Accessors ───────────────────────────────────────────────────

    /// Try to extract an entity reference.
    #[must_use]
    pub fn as_entity_ref(&self) -> Option<EntityId> {
        match self {
            Self::EntityRef(id) => Some(*id),
            _ => None,
        }
    }

    /// Try to extract a string slice.
    #[must_use]
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract an enum name.
    #[must_use]
    pub fn as_enum(&self) -> Option<&str> {
        match self {
            Self::Enum(s) => Some(s),
            _ => None,
        }
    }

    /// Try to extract a floating-point value.
    ///
    /// Integers are promoted to `f64`.  Typed-value wrappers like
    /// `IFCNORMALISEDRATIOMEASURE(0.5)` (stored as
    /// `List([String(name), Float(v)])`) are also unwrapped.
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            Self::List(items) => items
                .first()
                .filter(|first| matches!(first, Self::String(_)))
                .and_then(|_| items.get(1))
                .and_then(|second| match second {
                    Self::Float(f) => Some(*f),
                    Self::Integer(i) => Some(*i as f64),
                    _ => None,
                }),
            _ => None,
        }
    }

    /// Try to extract an integer.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            Self::Float(f) => Some(*f as i64),
            _ => None,
        }
    }

    /// Try to extract a list slice.
    #[must_use]
    pub fn as_list(&self) -> Option<&[AttributeValue]> {
        match self {
            Self::List(items) => Some(items),
            _ => None,
        }
    }

    /// `true` when this value is null or derived.
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null | Self::Derived)
    }
}

// ───────────────────────────────────────────────────────────────────
// DecodedEntity
// ───────────────────────────────────────────────────────────────────

/// A fully-decoded IFC entity with its id, type, and attribute list.
///
/// Constructed by the decoder after tokenizing and converting a STEP
/// entity line.  All fields are private; use the accessor methods.
///
/// # Examples
///
/// ```
/// use ifc_lite_core_cat::{EntityId, IfcType, DecodedEntity, AttributeValue};
///
/// let entity = DecodedEntity::new(
///     EntityId::new(1),
///     IfcType::IfcWall,
///     vec![AttributeValue::String("Wall-001".into())],
/// );
/// assert_eq!(entity.id(), EntityId::new(1));
/// assert_eq!(entity.get_string(0), Some("Wall-001"));
/// ```
#[derive(Debug, Clone)]
pub struct DecodedEntity {
    id: EntityId,
    ifc_type: IfcType,
    attributes: Vec<AttributeValue>,
}

impl DecodedEntity {
    /// Construct a new decoded entity.
    #[must_use]
    pub fn new(id: EntityId, ifc_type: IfcType, attributes: Vec<AttributeValue>) -> Self {
        Self {
            id,
            ifc_type,
            attributes,
        }
    }

    /// Entity id.
    #[must_use]
    pub fn id(&self) -> EntityId {
        self.id
    }

    /// Entity type.
    #[must_use]
    pub fn ifc_type(&self) -> &IfcType {
        &self.ifc_type
    }

    /// Number of attributes.
    #[must_use]
    pub fn attribute_count(&self) -> usize {
        self.attributes.len()
    }

    /// Attribute at `index`, if present.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&AttributeValue> {
        self.attributes.get(index)
    }

    /// Entity reference at `index`.
    #[must_use]
    pub fn get_ref(&self, index: usize) -> Option<EntityId> {
        self.get(index).and_then(AttributeValue::as_entity_ref)
    }

    /// String at `index`.
    #[must_use]
    pub fn get_string(&self, index: usize) -> Option<&str> {
        self.get(index).and_then(AttributeValue::as_string)
    }

    /// Float at `index`.
    #[must_use]
    pub fn get_float(&self, index: usize) -> Option<f64> {
        self.get(index).and_then(AttributeValue::as_float)
    }

    /// List at `index`.
    #[must_use]
    pub fn get_list(&self, index: usize) -> Option<&[AttributeValue]> {
        self.get(index).and_then(AttributeValue::as_list)
    }
}
