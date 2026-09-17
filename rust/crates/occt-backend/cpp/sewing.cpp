#include "bridge.hpp"

#include <BRepBuilderAPI_Sewing.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Shape.hxx>

#include <cmath>
#include <new>

struct umlcad_occt_shape {
    TopoDS_Shape value;
};

namespace {
int32_t allocateResult(const TopoDS_Shape& shape, umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (shape.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;
    auto* result = new (std::nothrow) umlcad_occt_shape{shape};
    if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
    *out_shape = result;
    return UMLCAD_OCCT_OK;
}
}

extern "C" int32_t umlcad_occt_sew_box_faces(double width, double depth, double height,
                                                double tolerance,
                                                umlcad_occt_shape** out_shape,
                                                uint32_t* out_free_edges,
                                                uint32_t* out_multiple_edges,
                                                uint32_t* out_degenerated) {
    if (out_shape == nullptr || out_free_edges == nullptr || out_multiple_edges == nullptr ||
        out_degenerated == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    *out_free_edges = 0;
    *out_multiple_edges = 0;
    *out_degenerated = 0;
    if (!std::isfinite(width) || !std::isfinite(depth) || !std::isfinite(height) || width <= 0.0 ||
        depth <= 0.0 || height <= 0.0 || !std::isfinite(tolerance) || tolerance < 0.0) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }

    try {
        BRepBuilderAPI_Sewing sewing(tolerance, true, true, true, false);
        const TopoDS_Shape box = BRepPrimAPI_MakeBox(width, depth, height).Shape();
        if (box.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        uint32_t face_count = 0;
        for (TopExp_Explorer explorer(box, TopAbs_FACE); explorer.More(); explorer.Next()) {
            const TopoDS_Shape face = explorer.Current();
            if (face.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            sewing.Add(face);
            ++face_count;
        }
        if (face_count != 6) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        sewing.Perform();
        const TopoDS_Shape sewn = sewing.SewedShape();
        if (sewn.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        *out_free_edges = static_cast<uint32_t>(sewing.NbFreeEdges());
        *out_multiple_edges = static_cast<uint32_t>(sewing.NbMultipleEdges());
        *out_degenerated = static_cast<uint32_t>(sewing.NbDegeneratedShapes());

        const BRepCheck_Analyzer analyzer(sewn, true);
        if (!analyzer.IsValid()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        return allocateResult(sewn, out_shape);
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
