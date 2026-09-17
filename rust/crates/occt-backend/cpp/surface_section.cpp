#include "bridge.hpp"

#include <BRepAlgoAPI_Section.hxx>
#include <BRepBndLib.hxx>
#include <Bnd_Box.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS_Shape.hxx>

#include <cmath>

struct umlcad_occt_shape {
    TopoDS_Shape value;
};

extern "C" int32_t umlcad_occt_surface_section_bbox(
    const umlcad_occt_shape* first,
    const umlcad_occt_shape* second,
    double tolerance,
    double* out_bounds,
    uint32_t* out_edge_count) {
    if (first == nullptr || second == nullptr || out_bounds == nullptr || out_edge_count == nullptr || !std::isfinite(tolerance) || tolerance < 0.0) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }
    (void)tolerance;
    try {
        if (first->value.IsNull() || second->value.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;
        BRepAlgoAPI_Section section(first->value, second->value);
        section.Build();
        if (!section.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        const TopoDS_Shape& result = section.Shape();
        if (result.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        Bnd_Box box;
        BRepBndLib::Add(result, box);
        if (box.IsVoid()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        Standard_Real xmin, ymin, zmin, xmax, ymax, zmax;
        box.Get(xmin, ymin, zmin, xmax, ymax, zmax);
        const double bounds[6] = {xmin, ymin, zmin, xmax, ymax, zmax};
        for (int i = 0; i < 6; ++i) {
            if (!std::isfinite(bounds[i])) return UMLCAD_OCCT_INTERNAL_ERROR;
            out_bounds[i] = bounds[i];
        }

        TopTools_IndexedMapOfShape edges;
        TopExp::MapShapes(result, TopAbs_EDGE, edges);
        *out_edge_count = static_cast<uint32_t>(edges.Extent());
        return *out_edge_count > 0 ? UMLCAD_OCCT_OK : UMLCAD_OCCT_CONSTRUCTION_FAILED;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
