#include "bridge.hpp"

#include <BRepPrimAPI_MakeCone.hxx>
#include <TopoDS_Shape.hxx>

#include <cmath>
#include <new>

// Keep this definition identical to the private native shape in bridge.cpp.
// The C ABI only exposes the pointer opaquely; the complete type is required
// here solely to allocate the native owner.
struct umlcad_occt_shape {
    TopoDS_Shape value;
};

extern "C" int32_t umlcad_occt_cone(
    double base_radius,
    double top_radius,
    double height,
    umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }
    *out_shape = nullptr;

    if (!(base_radius > 0.0) || !(top_radius > 0.0) || !(height > 0.0)
        || !std::isfinite(base_radius) || !std::isfinite(top_radius) || !std::isfinite(height)) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }

    try {
        const TopoDS_Shape cone = BRepPrimAPI_MakeCone(base_radius, top_radius, height).Shape();
        if (cone.IsNull()) {
            return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        }

        auto* result = new (std::nothrow) umlcad_occt_shape{cone};
        if (result == nullptr) {
            return UMLCAD_OCCT_INTERNAL_ERROR;
        }

        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
