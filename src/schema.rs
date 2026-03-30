//! IFC schema metadata: geometry and profile categories.
//!
//! The [`IfcSchema`] struct provides lookup tables that classify IFC
//! entity types into geometry representation and profile categories.
//! Downstream code (geometry processors, renderers) uses these
//! categories to route entities to the correct handler.

use std::collections::HashMap;

use crate::ifc_type::IfcType;

/// Categories of geometry representation in an IFC model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeometryCategory {
    /// Swept solid: extrusion or revolution of a 2D profile.
    SweptSolid,
    /// CSG boolean: union, intersection, or subtraction.
    Boolean,
    /// Explicit mesh: tessellated or faceted boundary representation.
    ExplicitMesh,
    /// Instanced geometry via mapping.
    MappedItem,
    /// Surface model.
    Surface,
    /// Curve representation.
    Curve,
    /// Anything not covered by the other categories.
    Other,
}

/// Categories of profile definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProfileCategory {
    /// Standard parametric profile (rectangle, circle, I-shape, etc.).
    Parametric,
    /// Arbitrary closed curve profile.
    Arbitrary,
    /// Composite of multiple sub-profiles.
    Composite,
}

/// Lookup tables for IFC type classification.
///
/// Construct via [`IfcSchema::new`] (or `Default`) and query with
/// [`geometry_category`](IfcSchema::geometry_category) /
/// [`profile_category`](IfcSchema::profile_category).
#[derive(Clone)]
pub struct IfcSchema {
    geometry_types: HashMap<IfcType, GeometryCategory>,
    profile_types: HashMap<IfcType, ProfileCategory>,
}

impl IfcSchema {
    /// Build the schema with the standard IFC4 classification tables.
    #[must_use]
    pub fn new() -> Self {
        let geometry_types = [
            (IfcType::IfcExtrudedAreaSolid, GeometryCategory::SweptSolid),
            (IfcType::IfcRevolvedAreaSolid, GeometryCategory::SweptSolid),
            (IfcType::IfcBooleanResult, GeometryCategory::Boolean),
            (IfcType::IfcBooleanClippingResult, GeometryCategory::Boolean),
            (IfcType::IfcFacetedBrep, GeometryCategory::ExplicitMesh),
            (
                IfcType::IfcTriangulatedFaceSet,
                GeometryCategory::ExplicitMesh,
            ),
            (
                IfcType::IfcPolygonalFaceSet,
                GeometryCategory::ExplicitMesh,
            ),
            (
                IfcType::IfcFaceBasedSurfaceModel,
                GeometryCategory::Surface,
            ),
            (
                IfcType::IfcShellBasedSurfaceModel,
                GeometryCategory::Surface,
            ),
            (IfcType::IfcMappedItem, GeometryCategory::MappedItem),
        ]
        .into_iter()
        .collect();

        let profile_types = [
            (
                IfcType::IfcRectangleProfileDef,
                ProfileCategory::Parametric,
            ),
            (IfcType::IfcCircleProfileDef, ProfileCategory::Parametric),
            (
                IfcType::IfcCircleHollowProfileDef,
                ProfileCategory::Parametric,
            ),
            (IfcType::IfcIShapeProfileDef, ProfileCategory::Parametric),
            (
                IfcType::IfcArbitraryClosedProfileDef,
                ProfileCategory::Arbitrary,
            ),
            (IfcType::IfcCompositeProfileDef, ProfileCategory::Composite),
        ]
        .into_iter()
        .collect();

        Self {
            geometry_types,
            profile_types,
        }
    }

    /// Geometry category for the given type, if classified.
    #[must_use]
    pub fn geometry_category(&self, ifc_type: &IfcType) -> Option<GeometryCategory> {
        self.geometry_types.get(ifc_type).copied()
    }

    /// Profile category for the given type, if classified.
    #[must_use]
    pub fn profile_category(&self, ifc_type: &IfcType) -> Option<ProfileCategory> {
        self.profile_types.get(ifc_type).copied()
    }

    /// `true` if the type is a geometry representation.
    #[must_use]
    pub fn is_geometry_type(&self, ifc_type: &IfcType) -> bool {
        self.geometry_types.contains_key(ifc_type)
    }

    /// `true` if the type is a profile definition.
    #[must_use]
    pub fn is_profile_type(&self, ifc_type: &IfcType) -> bool {
        self.profile_types.contains_key(ifc_type)
    }
}

impl Default for IfcSchema {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_categories_populated() {
        let schema = IfcSchema::new();
        assert_eq!(
            schema.geometry_category(&IfcType::IfcExtrudedAreaSolid),
            Some(GeometryCategory::SweptSolid)
        );
        assert_eq!(
            schema.geometry_category(&IfcType::IfcTriangulatedFaceSet),
            Some(GeometryCategory::ExplicitMesh)
        );
        assert_eq!(schema.geometry_category(&IfcType::IfcWall), None);
    }

    #[test]
    fn profile_categories_populated() {
        let schema = IfcSchema::new();
        assert_eq!(
            schema.profile_category(&IfcType::IfcRectangleProfileDef),
            Some(ProfileCategory::Parametric)
        );
        assert_eq!(
            schema.profile_category(&IfcType::IfcArbitraryClosedProfileDef),
            Some(ProfileCategory::Arbitrary)
        );
    }
}
