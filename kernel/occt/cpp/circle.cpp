#include "bridge.hpp"
#include <BRepBuilderAPI_MakeEdge.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Ax2.hxx>
#include <gp_Circ.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <cmath>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };

extern "C" int32_t umlcad_occt_circle_curve(double radius, umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (!std::isfinite(radius) || radius <= 0.0 || radius <= 1e-6) return UMLCAD_OCCT_INVALID_ARGUMENT;
    try {
        const gp_Ax2 axis(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0));
        const gp_Circ circle(axis, radius);
        const TopoDS_Edge edge = BRepBuilderAPI_MakeEdge(circle).Edge();
        if (edge.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        auto* result = new (std::nothrow) umlcad_occt_shape{edge};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
