//! IFC entity type enumeration.
//!
//! Covers the most commonly encountered entity types from the IFC4X3
//! schema.  Uncommon or unrecognised types fall into the [`IfcType::Unknown`]
//! variant so the parser can handle any valid file.

/// Discriminant for IFC entity types.
///
/// The named variants cover spatial structure, building elements,
/// geometry representations, profiles, properties, relations, units,
/// and georeferencing.  Everything else is captured by
/// [`IfcType::Unknown`].
///
/// # Examples
///
/// ```
/// use ifc_lite_core::IfcType;
///
/// let t = IfcType::from_name("IFCWALL");
/// assert_eq!(t, IfcType::IfcWall);
///
/// let u = IfcType::from_name("IFCSOMETHINGOBSCURE");
/// assert_eq!(u, IfcType::Unknown("IFCSOMETHINGOBSCURE".to_string()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IfcType {
    // ── Spatial structure ────────────────────────────────────────────
    IfcProject,
    IfcSite,
    IfcBuilding,
    IfcBuildingStorey,
    IfcSpace,

    // ── Building elements ───────────────────────────────────────────
    IfcWall,
    IfcWallStandardCase,
    IfcSlab,
    IfcBeam,
    IfcColumn,
    IfcRoof,
    IfcStair,
    IfcRamp,
    IfcDoor,
    IfcWindow,
    IfcRailing,
    IfcPlate,
    IfcMember,
    IfcFooting,
    IfcPile,
    IfcCovering,
    IfcCurtainWall,
    IfcChimney,
    IfcBuildingElementProxy,
    IfcBuildingElementPart,
    IfcOpeningElement,
    IfcFurnishingElement,
    IfcFurniture,

    // ── Geometry representations ────────────────────────────────────
    IfcExtrudedAreaSolid,
    IfcRevolvedAreaSolid,
    IfcBooleanResult,
    IfcBooleanClippingResult,
    IfcFacetedBrep,
    IfcTriangulatedFaceSet,
    IfcPolygonalFaceSet,
    IfcFaceBasedSurfaceModel,
    IfcShellBasedSurfaceModel,
    IfcMappedItem,
    IfcCartesianPoint,
    IfcCartesianPointList3D,
    IfcDirection,
    IfcAxis2Placement3D,
    IfcLocalPlacement,
    IfcPolyLoop,
    IfcProduct,

    // ── Profiles ────────────────────────────────────────────────────
    IfcRectangleProfileDef,
    IfcCircleProfileDef,
    IfcCircleHollowProfileDef,
    IfcIShapeProfileDef,
    IfcArbitraryClosedProfileDef,
    IfcCompositeProfileDef,

    // ── Properties ──────────────────────────────────────────────────
    IfcPropertySet,
    IfcPropertySingleValue,

    // ── Relations ───────────────────────────────────────────────────
    IfcRelContainedInSpatialStructure,
    IfcRelAggregates,

    // ── Units ───────────────────────────────────────────────────────
    IfcUnitAssignment,
    IfcSiUnit,
    IfcConversionBasedUnit,

    // ── Georeferencing ──────────────────────────────────────────────
    IfcMapConversion,
    IfcProjectedCRS,

    // ── Common auxiliary types ──────────────────────────────────────
    IfcOwnerHistory,
    IfcApplication,
    IfcPerson,
    IfcOrganization,
    IfcGeometricRepresentationContext,
    IfcDimensionalExponents,
    IfcMeasureWithUnit,

    // ── MEP / distribution ──────────────────────────────────────────
    IfcDistributionElement,
    IfcFlowSegment,
    IfcFlowFitting,
    IfcFlowTerminal,
    IfcDuctSegment,
    IfcPipeSegment,
    IfcCableSegment,

    // ── Catch-all ───────────────────────────────────────────────────
    /// Any IFC type name not covered by the named variants.
    Unknown(String),
}

impl IfcType {
    /// Look up an [`IfcType`] from its uppercase STEP name.
    ///
    /// IFC type names in STEP files are always uppercase, so this
    /// function performs a direct match without case folding.
    #[must_use]
    pub fn from_name(s: &str) -> Self {
        match s {
            "IFCPROJECT" => Self::IfcProject,
            "IFCSITE" => Self::IfcSite,
            "IFCBUILDING" => Self::IfcBuilding,
            "IFCBUILDINGSTOREY" => Self::IfcBuildingStorey,
            "IFCSPACE" => Self::IfcSpace,

            "IFCWALL" => Self::IfcWall,
            "IFCWALLSTANDARDCASE" => Self::IfcWallStandardCase,
            "IFCSLAB" => Self::IfcSlab,
            "IFCBEAM" => Self::IfcBeam,
            "IFCCOLUMN" => Self::IfcColumn,
            "IFCROOF" => Self::IfcRoof,
            "IFCSTAIR" => Self::IfcStair,
            "IFCRAMP" => Self::IfcRamp,
            "IFCDOOR" => Self::IfcDoor,
            "IFCWINDOW" => Self::IfcWindow,
            "IFCRAILING" => Self::IfcRailing,
            "IFCPLATE" => Self::IfcPlate,
            "IFCMEMBER" => Self::IfcMember,
            "IFCFOOTING" => Self::IfcFooting,
            "IFCPILE" => Self::IfcPile,
            "IFCCOVERING" => Self::IfcCovering,
            "IFCCURTAINWALL" => Self::IfcCurtainWall,
            "IFCCHIMNEY" => Self::IfcChimney,
            "IFCBUILDINGELEMENTPROXY" => Self::IfcBuildingElementProxy,
            "IFCBUILDINGELEMENTPART" => Self::IfcBuildingElementPart,
            "IFCOPENINGELEMENT" => Self::IfcOpeningElement,
            "IFCFURNISHINGELEMENT" => Self::IfcFurnishingElement,
            "IFCFURNITURE" => Self::IfcFurniture,

            "IFCEXTRUDEDAREASOLID" => Self::IfcExtrudedAreaSolid,
            "IFCREVOLVEDAREASOLID" => Self::IfcRevolvedAreaSolid,
            "IFCBOOLEANRESULT" => Self::IfcBooleanResult,
            "IFCBOOLEANCLIPPINGRESULT" => Self::IfcBooleanClippingResult,
            "IFCFACETEDBREP" => Self::IfcFacetedBrep,
            "IFCTRIANGULATEDFACESET" => Self::IfcTriangulatedFaceSet,
            "IFCPOLYGONALFACESET" => Self::IfcPolygonalFaceSet,
            "IFCFACEBASEDSURFACEMODEL" => Self::IfcFaceBasedSurfaceModel,
            "IFCSHELLBASEDSURFACEMODEL" => Self::IfcShellBasedSurfaceModel,
            "IFCMAPPEDITEM" => Self::IfcMappedItem,
            "IFCCARTESIANPOINT" => Self::IfcCartesianPoint,
            "IFCCARTESIANPOINTLIST3D" => Self::IfcCartesianPointList3D,
            "IFCDIRECTION" => Self::IfcDirection,
            "IFCAXIS2PLACEMENT3D" => Self::IfcAxis2Placement3D,
            "IFCLOCALPLACEMENT" => Self::IfcLocalPlacement,
            "IFCPOLYLOOP" => Self::IfcPolyLoop,
            "IFCPRODUCT" => Self::IfcProduct,

            "IFCRECTANGLEPROFILEDEF" => Self::IfcRectangleProfileDef,
            "IFCCIRCLEPROFILEDEF" => Self::IfcCircleProfileDef,
            "IFCCIRCLEHOLLOWPROFILEDEF" => Self::IfcCircleHollowProfileDef,
            "IFCISHAPEPROFILEDEF" => Self::IfcIShapeProfileDef,
            "IFCARBITRARYCLOSEDPROFILEDEF" => Self::IfcArbitraryClosedProfileDef,
            "IFCCOMPOSITEPROFILEDEF" => Self::IfcCompositeProfileDef,

            "IFCPROPERTYSET" => Self::IfcPropertySet,
            "IFCPROPERTYSINGLEVALUE" => Self::IfcPropertySingleValue,

            "IFCRELCONTAINEDINSPATIALSTRUCTURE" => Self::IfcRelContainedInSpatialStructure,
            "IFCRELAGGREGATES" => Self::IfcRelAggregates,

            "IFCUNITASSIGNMENT" => Self::IfcUnitAssignment,
            "IFCSIUNIT" => Self::IfcSiUnit,
            "IFCCONVERSIONBASEDUNIT" => Self::IfcConversionBasedUnit,

            "IFCMAPCONVERSION" => Self::IfcMapConversion,
            "IFCPROJECTEDCRS" => Self::IfcProjectedCRS,

            "IFCOWNERHISTORY" => Self::IfcOwnerHistory,
            "IFCAPPLICATION" => Self::IfcApplication,
            "IFCPERSON" => Self::IfcPerson,
            "IFCORGANIZATION" => Self::IfcOrganization,
            "IFCGEOMETRICREPRESENTATIONCONTEXT" => Self::IfcGeometricRepresentationContext,
            "IFCDIMENSIONALEXPONENTS" => Self::IfcDimensionalExponents,
            "IFCMEASUREWITHUNIT" => Self::IfcMeasureWithUnit,

            "IFCDISTRIBUTIONELEMENT" => Self::IfcDistributionElement,
            "IFCFLOWSEGMENT" => Self::IfcFlowSegment,
            "IFCFLOWFITTING" => Self::IfcFlowFitting,
            "IFCFLOWTERMINAL" => Self::IfcFlowTerminal,
            "IFCDUCTSEGMENT" => Self::IfcDuctSegment,
            "IFCPIPESEGMENT" => Self::IfcPipeSegment,
            "IFCCABLESEGMENT" => Self::IfcCableSegment,

            other => Self::Unknown(other.to_string()),
        }
    }

    /// Return the STEP-file name for this type.
    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Self::IfcProject => "IFCPROJECT",
            Self::IfcSite => "IFCSITE",
            Self::IfcBuilding => "IFCBUILDING",
            Self::IfcBuildingStorey => "IFCBUILDINGSTOREY",
            Self::IfcSpace => "IFCSPACE",
            Self::IfcWall => "IFCWALL",
            Self::IfcWallStandardCase => "IFCWALLSTANDARDCASE",
            Self::IfcSlab => "IFCSLAB",
            Self::IfcBeam => "IFCBEAM",
            Self::IfcColumn => "IFCCOLUMN",
            Self::IfcRoof => "IFCROOF",
            Self::IfcStair => "IFCSTAIR",
            Self::IfcRamp => "IFCRAMP",
            Self::IfcDoor => "IFCDOOR",
            Self::IfcWindow => "IFCWINDOW",
            Self::IfcRailing => "IFCRAILING",
            Self::IfcPlate => "IFCPLATE",
            Self::IfcMember => "IFCMEMBER",
            Self::IfcFooting => "IFCFOOTING",
            Self::IfcPile => "IFCPILE",
            Self::IfcCovering => "IFCCOVERING",
            Self::IfcCurtainWall => "IFCCURTAINWALL",
            Self::IfcChimney => "IFCCHIMNEY",
            Self::IfcBuildingElementProxy => "IFCBUILDINGELEMENTPROXY",
            Self::IfcBuildingElementPart => "IFCBUILDINGELEMENTPART",
            Self::IfcOpeningElement => "IFCOPENINGELEMENT",
            Self::IfcFurnishingElement => "IFCFURNISHINGELEMENT",
            Self::IfcFurniture => "IFCFURNITURE",
            Self::IfcExtrudedAreaSolid => "IFCEXTRUDEDAREASOLID",
            Self::IfcRevolvedAreaSolid => "IFCREVOLVEDAREASOLID",
            Self::IfcBooleanResult => "IFCBOOLEANRESULT",
            Self::IfcBooleanClippingResult => "IFCBOOLEANCLIPPINGRESULT",
            Self::IfcFacetedBrep => "IFCFACETEDBREP",
            Self::IfcTriangulatedFaceSet => "IFCTRIANGULATEDFACESET",
            Self::IfcPolygonalFaceSet => "IFCPOLYGONALFACESET",
            Self::IfcFaceBasedSurfaceModel => "IFCFACEBASEDSURFACEMODEL",
            Self::IfcShellBasedSurfaceModel => "IFCSHELLBASEDSURFACEMODEL",
            Self::IfcMappedItem => "IFCMAPPEDITEM",
            Self::IfcCartesianPoint => "IFCCARTESIANPOINT",
            Self::IfcCartesianPointList3D => "IFCCARTESIANPOINTLIST3D",
            Self::IfcDirection => "IFCDIRECTION",
            Self::IfcAxis2Placement3D => "IFCAXIS2PLACEMENT3D",
            Self::IfcLocalPlacement => "IFCLOCALPLACEMENT",
            Self::IfcPolyLoop => "IFCPOLYLOOP",
            Self::IfcProduct => "IFCPRODUCT",
            Self::IfcRectangleProfileDef => "IFCRECTANGLEPROFILEDEF",
            Self::IfcCircleProfileDef => "IFCCIRCLEPROFILEDEF",
            Self::IfcCircleHollowProfileDef => "IFCCIRCLEHOLLOWPROFILEDEF",
            Self::IfcIShapeProfileDef => "IFCISHAPEPROFILEDEF",
            Self::IfcArbitraryClosedProfileDef => "IFCARBITRARYCLOSEDPROFILEDEF",
            Self::IfcCompositeProfileDef => "IFCCOMPOSITEPROFILEDEF",
            Self::IfcPropertySet => "IFCPROPERTYSET",
            Self::IfcPropertySingleValue => "IFCPROPERTYSINGLEVALUE",
            Self::IfcRelContainedInSpatialStructure => "IFCRELCONTAINEDINSPATIALSTRUCTURE",
            Self::IfcRelAggregates => "IFCRELAGGREGATES",
            Self::IfcUnitAssignment => "IFCUNITASSIGNMENT",
            Self::IfcSiUnit => "IFCSIUNIT",
            Self::IfcConversionBasedUnit => "IFCCONVERSIONBASEDUNIT",
            Self::IfcMapConversion => "IFCMAPCONVERSION",
            Self::IfcProjectedCRS => "IFCPROJECTEDCRS",
            Self::IfcOwnerHistory => "IFCOWNERHISTORY",
            Self::IfcApplication => "IFCAPPLICATION",
            Self::IfcPerson => "IFCPERSON",
            Self::IfcOrganization => "IFCORGANIZATION",
            Self::IfcGeometricRepresentationContext => "IFCGEOMETRICREPRESENTATIONCONTEXT",
            Self::IfcDimensionalExponents => "IFCDIMENSIONALEXPONENTS",
            Self::IfcMeasureWithUnit => "IFCMEASUREWITHUNIT",
            Self::IfcDistributionElement => "IFCDISTRIBUTIONELEMENT",
            Self::IfcFlowSegment => "IFCFLOWSEGMENT",
            Self::IfcFlowFitting => "IFCFLOWFITTING",
            Self::IfcFlowTerminal => "IFCFLOWTERMINAL",
            Self::IfcDuctSegment => "IFCDUCTSEGMENT",
            Self::IfcPipeSegment => "IFCPIPESEGMENT",
            Self::IfcCableSegment => "IFCCABLESEGMENT",
            Self::Unknown(s) => s.as_str(),
        }
    }

    /// `true` when this type typically carries geometry in an IFC model.
    #[must_use]
    pub fn has_geometry(&self) -> bool {
        matches!(
            self,
            Self::IfcWall
                | Self::IfcWallStandardCase
                | Self::IfcSlab
                | Self::IfcBeam
                | Self::IfcColumn
                | Self::IfcRoof
                | Self::IfcStair
                | Self::IfcRamp
                | Self::IfcDoor
                | Self::IfcWindow
                | Self::IfcRailing
                | Self::IfcPlate
                | Self::IfcMember
                | Self::IfcFooting
                | Self::IfcPile
                | Self::IfcCovering
                | Self::IfcCurtainWall
                | Self::IfcChimney
                | Self::IfcBuildingElementProxy
                | Self::IfcBuildingElementPart
                | Self::IfcOpeningElement
                | Self::IfcFurnishingElement
                | Self::IfcFurniture
                | Self::IfcProduct
                | Self::IfcDistributionElement
                | Self::IfcFlowSegment
                | Self::IfcFlowFitting
                | Self::IfcFlowTerminal
                | Self::IfcSpace
                | Self::IfcSite
        )
    }
}

impl std::fmt::Display for IfcType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
