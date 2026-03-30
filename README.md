# ifc-lite-core

High-performance IFC/STEP parser for building data, built on
[comp-cat-rs](https://github.com/MavenRain/comp-cat-rs).

## Overview

`ifc-lite-core` provides the core parsing functionality for
IFC (Industry Foundation Classes) files used in Building Information
Modeling (BIM).  All effectful operations are expressed through the
`comp-cat-rs` effect system (`Io`, `Stream`), keeping the core
parsing logic pure and composable.

**Capabilities:**

- **STEP Tokenization** -- zero-copy parsing via `nom`
- **Entity Scanning** -- SIMD-accelerated entity discovery via `memchr`,
  exposed as a `comp_cat_rs::effect::stream::Stream`
- **Lazy Decoding** -- on-demand attribute parsing wrapped in
  `comp_cat_rs::effect::io::Io`
- **Streaming Parser** -- event-based parsing for large files with
  type filtering

## Quick Start

```rust
use ifc_lite_core::{parse_entity, EntityId, IfcType};

let input = "#123=IFCWALL('guid',$,$,$,'Wall-001',$,$,$);";
let (id, ifc_type, attrs) = parse_entity(input).unwrap();
assert_eq!(id, EntityId::new(123));
assert_eq!(ifc_type, IfcType::IfcWall);
```

## Scanning with comp-cat-rs Stream

```rust
use ifc_lite_core::scan::scan_entities;

let content = std::fs::read_to_string("model.ifc").unwrap();
let entities = scan_entities(content)
    .collect()
    .run()
    .unwrap();
println!("Found {} entities", entities.len());
```

## Decoding entities with comp-cat-rs Io

```rust
use ifc_lite_core::decode::{decode_entity, decode_by_id};
use ifc_lite_core::scan::build_entity_index;

let content = std::fs::read_to_string("model.ifc").unwrap();
let index = build_entity_index(&content);

// Decode a specific entity by id -- returns Io<Error, DecodedEntity>
let wall = decode_by_id(&content, &index, 42.into())
    .run()
    .unwrap();
println!("Entity: {} ({})", wall.id(), wall.ifc_type());
```

## Streaming parse events

```rust
use ifc_lite_core::streaming::{parse_stream, StreamConfig, ParseEvent};

let content = std::fs::read_to_string("model.ifc").unwrap();
let events = parse_stream(content, StreamConfig::default())
    .collect()
    .run()
    .unwrap();

for event in &events {
    match event {
        ParseEvent::EntityScanned { entity } => {
            println!("#{}: {}", entity.id(), entity.ifc_type());
        }
        ParseEvent::Completed { entity_count } => {
            println!("Done: {entity_count} entities");
        }
        _ => {}
    }
}
```

## Architecture

| Module      | Purpose                                                |
|-------------|--------------------------------------------------------|
| `parse`     | nom-based STEP tokenizer; pure `&str -> Result` API   |
| `scan`      | Entity scanning; returns `Stream<Error, ScannedEntity>`|
| `decode`    | Entity decoding; returns `Io<Error, DecodedEntity>`    |
| `streaming` | High-level parse events; returns `Stream<Error, ParseEvent>` |
| `schema`    | Geometry and profile category lookups                  |
| `token`     | Zero-copy `Token<'a>` sum type                        |
| `attribute` | Owned `AttributeValue` and `DecodedEntity`             |
| `ifc_type`  | `IfcType` enum covering IFC4X3 entity types            |
| `entity_id` | `EntityId` newtype                                     |
| `error`     | Hand-rolled `Error` enum                               |

## Design Principles

- **Functional**: no `mut`, no loops, combinators everywhere
- **Type-driven**: newtypes for domain primitives (`EntityId`, `IfcType`)
- **Effect-aware**: side effects wrapped in `comp-cat-rs` `Io` and `Stream`
- **Delay `run`**: stay inside effects via combinators; call `run` only at the boundary
- **Zero-copy tokenization**: `Token<'a>` borrows from input; owned `AttributeValue` for storage

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.
