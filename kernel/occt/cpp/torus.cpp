#include "bridge.hpp"
#include <BRepPrimAPI_MakeTorus.hxx>
#include <TopoDS_Shape.hxx>
#include <cmath>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };

extern "C" int32_t umlcad_occt_torus(
    double major_radius,
    double minor_radius,
    umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (!std::isfinite(major_radius) || !std::isfinite(minor_radius)
        || major_radius <= 0.0 || minor_radius <= 0.0
        || major_radius <= minor_radius) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }
    try {
        const TopoDS_Shape torus = BRepPrimAPI_MakeTorus(major_radius, minor_radius).Shape();
        if (torus.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        auto* result = new (std::nothrow) umlcad_occt_shape{torus};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
