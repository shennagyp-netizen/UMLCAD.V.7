#include "bridge.hpp"

#include <BRep_Tool.hxx>
#include <Geom_Surface.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

#include <cmath>
#include <cstdint>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {
bool is_finite_value(double value) { return std::isfinite(value); }
}

extern "C" int32_t umlcad_occt_nurbs_surface3d_differential(
    const umlcad_occt_shape* shape, double u, double v, double* out_values) {
    if (shape == nullptr || out_values == nullptr || !is_finite_value(u) || !is_finite_value(v)) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }
    try {
        if (shape->value.IsNull() || shape->value.ShapeType() != TopAbs_FACE) {
            return UMLCAD_OCCT_INVALID_ARGUMENT;
        }
        const TopoDS_Face face = TopoDS::Face(shape->value);
        TopLoc_Location location;
        const Handle(Geom_Surface) surface = BRep_Tool::Surface(face, location);
        if (surface.IsNull()) return UMLCAD_OCCT_INVALID_ARGUMENT;

        double u1 = 0.0, u2 = 0.0, v1 = 0.0, v2 = 0.0;
        surface->Bounds(u1, u2, v1, v2);
        if (!is_finite_value(u1) || !is_finite_value(u2) || !is_finite_value(v1) ||
            !is_finite_value(v2) || u < u1 || u > u2 || v < v1 || v > v2) {
            return UMLCAD_OCCT_INVALID_ARGUMENT;
        }

        gp_Pnt point;
        gp_Vec du, dv, duu, dvv, duv;
        surface->D2(u, v, point, du, dv, duu, dvv, duv);
        point.Transform(location.Transformation());
        du.Transform(location.Transformation());
        dv.Transform(location.Transformation());
        duu.Transform(location.Transformation());
        duv.Transform(location.Transformation());
        dvv.Transform(location.Transformation());

        const double values[18] = {
            point.X(), point.Y(), point.Z(),
            du.X(), du.Y(), du.Z(),
            dv.X(), dv.Y(), dv.Z(),
            duu.X(), duu.Y(), duu.Z(),
            duv.X(), duv.Y(), duv.Z(),
            dvv.X(), dvv.Y(), dvv.Z()
        };
        for (double value : values) {
            if (!is_finite_value(value)) return UMLCAD_OCCT_INTERNAL_ERROR;
        }
        for (int i = 0; i < 18; ++i) out_values[i] = values[i];
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
