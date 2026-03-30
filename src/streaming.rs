//! High-level streaming parser built on [`comp_cat_rs::effect::stream::Stream`].
//!
//! [`parse_stream`] wraps the low-level scanner to emit [`ParseEvent`]
//! values.  Callers compose the stream with `map`, `fold`, or
//! `collect` and defer `run` to the boundary.

use std::rc::Rc;

use comp_cat_rs::effect::io::Io;
use comp_cat_rs::effect::stream::Stream;

use crate::error::Error;
use crate::ifc_type::IfcType;
use crate::scan::{scan_next, ScannedEntity};

// ═══════════════════════════════════════════════════════════════════
// ParseEvent
// ═══════════════════════════════════════════════════════════════════

/// Events emitted during a streaming parse.
#[derive(Debug, Clone)]
pub enum ParseEvent {
    /// The parse has started.
    Started {
        /// Total size of the content in bytes.
        file_size: usize,
    },

    /// An entity was scanned.
    EntityScanned {
        /// The scanned entity descriptor.
        entity: ScannedEntity,
    },

    /// The parse has finished.
    Completed {
        /// Number of entities found.
        entity_count: usize,
    },
}

// ═══════════════════════════════════════════════════════════════════
// StreamConfig
// ═══════════════════════════════════════════════════════════════════

/// Configuration for the streaming parser.
///
/// All fields are private; construct via [`Default`] and modify
/// through the builder methods.
#[derive(Debug, Clone)]
pub struct StreamConfig {
    skip_types: Vec<IfcType>,
    only_types: Option<Vec<IfcType>>,
}

impl StreamConfig {
    /// Set entity types to skip during scanning.
    #[must_use]
    pub fn with_skip_types(self, types: Vec<IfcType>) -> Self {
        Self {
            skip_types: types,
            ..self
        }
    }

    /// Restrict scanning to only these entity types.
    #[must_use]
    pub fn with_only_types(self, types: Vec<IfcType>) -> Self {
        Self {
            only_types: Some(types),
            ..self
        }
    }

    fn should_skip(&self, ifc_type: &IfcType) -> bool {
        self.skip_types.contains(ifc_type)
            || self
                .only_types
                .as_ref()
                .is_some_and(|only| !only.contains(ifc_type))
    }
}

impl Default for StreamConfig {
    fn default() -> Self {
        Self {
            skip_types: vec![
                IfcType::IfcOwnerHistory,
                IfcType::IfcPerson,
                IfcType::IfcOrganization,
                IfcType::IfcApplication,
            ],
            only_types: None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Stream constructor
// ═══════════════════════════════════════════════════════════════════

/// Internal state threaded through the stream unfold.
struct ParseState {
    content: String,
    config: StreamConfig,
    position: usize,
    started: bool,
    completed: bool,
    entity_count: usize,
}

/// Produce a lazy [`Stream`] of [`ParseEvent`]s from owned content.
///
/// The stream first emits [`ParseEvent::Started`], then one
/// [`ParseEvent::EntityScanned`] per entity (filtered by `config`),
/// and finally [`ParseEvent::Completed`].
///
/// # Examples
///
/// ```
/// use ifc_lite_core_cat::streaming::{parse_stream, StreamConfig, ParseEvent};
///
/// let content = "#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);".to_string();
/// let events = parse_stream(content, StreamConfig::default())
///     .collect()
///     .run()
///     .unwrap();
/// // Started, EntityScanned, Completed
/// assert_eq!(events.len(), 3);
/// ```
#[must_use]
pub fn parse_stream(content: String, config: StreamConfig) -> Stream<Error, ParseEvent> {
    let file_size = content.len();
    Stream::unfold(
        ParseState {
            content,
            config,
            position: 0,
            started: false,
            completed: false,
            entity_count: 0,
        },
        Rc::new(move |state: ParseState| {
            Io::suspend(move || {
                if state.completed {
                    Ok(None)
                } else if state.started {
                    advance_until_accepted(state)
                } else {
                    Ok(Some((
                        ParseEvent::Started { file_size },
                        ParseState {
                            started: true,
                            ..state
                        },
                    )))
                }
            })
        }),
    )
}

/// Advance past skipped types until an accepted entity or
/// end-of-content is reached.
fn advance_until_accepted(
    state: ParseState,
) -> std::result::Result<Option<(ParseEvent, ParseState)>, Error> {
    match scan_next(state.content.as_bytes(), state.position) {
        None => Ok(Some((
            ParseEvent::Completed {
                entity_count: state.entity_count,
            },
            ParseState {
                completed: true,
                ..state
            },
        ))),
        Some((entity, next_pos)) => {
            if state.config.should_skip(entity.ifc_type()) {
                advance_until_accepted(ParseState {
                    position: next_pos,
                    ..state
                })
            } else {
                Ok(Some((
                    ParseEvent::EntityScanned { entity },
                    ParseState {
                        position: next_pos,
                        entity_count: state.entity_count + 1,
                        ..state
                    },
                )))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_emits_started_entities_completed() {
        let content = "\
#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);
#2=IFCWALL('g2',$,$,$,$,$,$,$);
#3=IFCDOOR('g3',$,$,$,$,$,$,$);
"
        .to_string();

        let events = parse_stream(content, StreamConfig::default())
            .collect()
            .run()
            .expect("collect");

        assert!(matches!(&events[0], ParseEvent::Started { .. }));
        assert!(matches!(events.last(), Some(ParseEvent::Completed { entity_count: 3 })));
    }

    #[test]
    fn stream_skips_configured_types() {
        let content = "\
#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);
#2=IFCOWNERHISTORY('g2',$,$,$,$,$,$,$);
#3=IFCWALL('g3',$,$,$,$,$,$,$);
"
        .to_string();

        let config = StreamConfig::default(); // skips OwnerHistory
        let entity_events: Vec<_> = parse_stream(content, config)
            .collect()
            .run()
            .expect("collect")
            .into_iter()
            .filter(|e| matches!(e, ParseEvent::EntityScanned { .. }))
            .collect();

        assert_eq!(entity_events.len(), 2);
    }

    #[test]
    fn stream_only_types_filter() {
        let content = "\
#1=IFCPROJECT('g',$,$,$,$,$,$,$,$);
#2=IFCWALL('g2',$,$,$,$,$,$,$);
#3=IFCDOOR('g3',$,$,$,$,$,$,$);
"
        .to_string();

        let config = StreamConfig {
            skip_types: vec![],
            only_types: Some(vec![IfcType::IfcWall]),
        };

        let entity_events: Vec<_> = parse_stream(content, config)
            .collect()
            .run()
            .expect("collect")
            .into_iter()
            .filter(|e| matches!(e, ParseEvent::EntityScanned { .. }))
            .collect();

        assert_eq!(entity_events.len(), 1);
    }
}
