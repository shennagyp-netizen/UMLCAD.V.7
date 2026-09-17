use crate::OcctBackend;
use umlcad_v6_fillet_api::{BoxAllEdgesFillet, FilletBackend};
use umlcad_v6_geometry_api::{GeometryError, GeometryResult, ToleranceContext};

impl FilletBackend for OcctBackend {
    fn fillet_box_all_edges(
        &self,
        definition: BoxAllEdgesFillet,
        tolerance: ToleranceContext,
    ) -> Result<GeometryResult<Self::Shape>, GeometryError> {
        definition.realize_with(self, tolerance)
    }
}
