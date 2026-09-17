#include "bridge.hpp"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt.hxx>
#include <cmath>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };

extern "C" int32_t umlcad_occt_line_curve(
    double x1,double y1,double z1,double x2,double y2,double z2,
    umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    const double values[] = {x1,y1,z1,x2,y2,z2};
    for (double value : values) if (!std::isfinite(value)) return UMLCAD_OCCT_INVALID_ARGUMENT;
    if (x1 == x2 && y1 == y2 && z1 == z2) return UMLCAD_OCCT_INVALID_ARGUMENT;
    try {
        const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(gp_Pnt(x1,y1,z1), gp_Pnt(x2,y2,z2)).Edge();
        if (edge.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        auto* result = new (std::nothrow) umlcad_occt_shape{edge};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
