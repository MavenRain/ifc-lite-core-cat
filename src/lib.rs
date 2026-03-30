//! # IFC-Lite Core Parser
//!
//! High-performance STEP/IFC parser for building data, built on
//! [`comp_cat_rs`].
//!
//! ## Overview
//!
//! This crate provides the core parsing functionality for
//! IFC (Industry Foundation Classes) files:
//!
//! - **STEP Tokenization** -- zero-copy parsing via [nom](https://docs.rs/nom)
//! - **Entity Scanning** -- SIMD-accelerated entity discovery via
//!   [memchr](https://docs.rs/memchr), exposed as a
//!   [`comp_cat_rs::effect::stream::Stream`]
//! - **Lazy Decoding** -- on-demand attribute parsing wrapped in
//!   [`comp_cat_rs::effect::io::Io`]
//! - **Streaming Parser** -- event-based parsing for large files
//!
//! ## Quick Start
//!
//! ```rust
//! use ifc_lite_core::{parse_entity, EntityId, IfcType};
//!
//! let input = "#123=IFCWALL('guid',$,$,$,'Wall-001',$,$,$);";
//! let (id, ifc_type, attrs) = parse_entity(input).unwrap();
//! assert_eq!(id, EntityId::new(123));
//! assert_eq!(ifc_type, IfcType::IfcWall);
//! ```
//!
//! ## Streaming with comp-cat-rs
//!
//! ```rust
//! use ifc_lite_core::scan::scan_entities;
//!
//! let content = "#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);".to_string();
//! let entities = scan_entities(content).collect().run().unwrap();
//! assert_eq!(entities.len(), 1);
//! ```
//!
//! ## Decoding entities
//!
//! ```rust
//! use ifc_lite_core::decode::decode_entity;
//!
//! let line = "#5=IFCWALL('g',$,$,$,'W1',$,$,$);";
//! let entity = decode_entity(line, 0, line.len()).run().unwrap();
//! assert_eq!(entity.get_string(4), Some("W1"));
//! ```

pub mod attribute;
pub mod decode;
pub mod entity_id;
pub mod error;
pub mod ifc_type;
pub mod parse;
pub mod scan;
pub mod schema;
pub mod streaming;
pub mod token;

// ── Convenience re-exports ──────────────────────────────────────────

pub use attribute::{AttributeValue, DecodedEntity};
pub use entity_id::EntityId;
pub use error::{Error, Result};
pub use ifc_type::IfcType;
pub use parse::parse_entity;
pub use token::Token;
