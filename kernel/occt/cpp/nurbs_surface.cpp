#include "bridge.hpp"

#include <BRepBuilderAPI_MakeFace.hxx>
#include <Geom_BSplineSurface.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TopoDS_Face.hxx>

#include <cmath>
#include <cstdint>
#include <new>
#include <vector>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {

bool finite(double value) { return std::isfinite(value); }

struct KnotData {
    std::vector<double> values;
    std::vector<int> multiplicities;
};

KnotData distinctKnots(const double* knots, uint32_t count) {
    KnotData result;
    result.values.reserve(count);
    result.multiplicities.reserve(count);
    for (uint32_t i = 0; i < count; ++i) {
        if (i == 0 || knots[i] != knots[i - 1]) {
            result.values.push_back(knots[i]);
            result.multiplicities.push_back(1);
        } else {
            ++result.multiplicities.back();
        }
    }
    return result;
}

bool validInput(const double* poles_xyz, uint32_t count_u, uint32_t count_v,
                const double* weights, const double* knots_u, uint32_t knot_count_u,
                const double* knots_v, uint32_t knot_count_v,
                uint32_t degree_u, uint32_t degree_v, double face_tolerance) {
    if (poles_xyz == nullptr || weights == nullptr || knots_u == nullptr || knots_v == nullptr) return false;
    if (degree_u == 0 || degree_v == 0) return false;
    if (count_u < degree_u + 1U || count_v < degree_v + 1U) return false;
    if (knot_count_u != count_u + degree_u + 1U || knot_count_v != count_v + degree_v + 1U) return false;
    if (!finite(face_tolerance) || face_tolerance < 0.0) return false;

    const uint64_t point_count = static_cast<uint64_t>(count_u) * static_cast<uint64_t>(count_v);
    for (uint64_t i = 0; i < point_count * 3U; ++i) if (!finite(poles_xyz[i])) return false;
    for (uint64_t i = 0; i < point_count; ++i) if (!finite(weights[i]) || weights[i] <= 0.0) return false;

    for (uint32_t i = 0; i < knot_count_u; ++i) {
        if (!finite(knots_u[i])) return false;
        if (i > 0 && knots_u[i] < knots_u[i - 1]) return false;
    }
    for (uint32_t i = 0; i < knot_count_v; ++i) {
        if (!finite(knots_v[i])) return false;
        if (i > 0 && knots_v[i] < knots_v[i - 1]) return false;
    }

    const double u_start = knots_u[degree_u];
    const double u_end = knots_u[count_u];
    const double v_start = knots_v[degree_v];
    const double v_end = knots_v[count_v];
    if (!(u_end > u_start) || !(v_end > v_start)) return false;

    for (uint32_t i = 0; i <= degree_u; ++i) if (knots_u[i] != u_start) return false;
    for (uint32_t i = count_u; i < knot_count_u; ++i) if (knots_u[i] != u_end) return false;
    for (uint32_t i = 0; i <= degree_v; ++i) if (knots_v[i] != v_start) return false;
    for (uint32_t i = count_v; i < knot_count_v; ++i) if (knots_v[i] != v_end) return false;
    return true;
}

}

extern "C" int32_t umlcad_occt_nurbs_surface3d(
    const double* poles_xyz, uint32_t count_u, uint32_t count_v,
    const double* weights, const double* knots_u, uint32_t knot_count_u,
    const double* knots_v, uint32_t knot_count_v,
    uint32_t degree_u, uint32_t degree_v, double face_tolerance,
    umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (!validInput(poles_xyz, count_u, count_v, weights, knots_u, knot_count_u,
                    knots_v, knot_count_v, degree_u, degree_v, face_tolerance)) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }

    try {
        TColgp_Array2OfPnt poles(1, static_cast<Standard_Integer>(count_u),
                                 1, static_cast<Standard_Integer>(count_v));
        TColStd_Array2OfReal pole_weights(1, static_cast<Standard_Integer>(count_u),
                                          1, static_cast<Standard_Integer>(count_v));

        for (uint32_t u = 0; u < count_u; ++u) {
            for (uint32_t v = 0; v < count_v; ++v) {
                const uint64_t index = static_cast<uint64_t>(u) * count_v + v;
                poles.SetValue(static_cast<Standard_Integer>(u + 1),
                               static_cast<Standard_Integer>(v + 1),
                               gp_Pnt(poles_xyz[3U * index],
                                      poles_xyz[3U * index + 1U],
                                      poles_xyz[3U * index + 2U]));
                pole_weights.SetValue(static_cast<Standard_Integer>(u + 1),
                                      static_cast<Standard_Integer>(v + 1),
                                      weights[index]);
            }
        }

        const KnotData u_data = distinctKnots(knots_u, knot_count_u);
        const KnotData v_data = distinctKnots(knots_v, knot_count_v);
        TColStd_Array1OfReal occt_knots_u(1, static_cast<Standard_Integer>(u_data.values.size()));
        TColStd_Array1OfInteger occt_mults_u(1, static_cast<Standard_Integer>(u_data.multiplicities.size()));
        TColStd_Array1OfReal occt_knots_v(1, static_cast<Standard_Integer>(v_data.values.size()));
        TColStd_Array1OfInteger occt_mults_v(1, static_cast<Standard_Integer>(v_data.multiplicities.size()));

        for (std::size_t i = 0; i < u_data.values.size(); ++i) {
            occt_knots_u.SetValue(static_cast<Standard_Integer>(i + 1), u_data.values[i]);
            occt_mults_u.SetValue(static_cast<Standard_Integer>(i + 1), u_data.multiplicities[i]);
        }
        for (std::size_t i = 0; i < v_data.values.size(); ++i) {
            occt_knots_v.SetValue(static_cast<Standard_Integer>(i + 1), v_data.values[i]);
            occt_mults_v.SetValue(static_cast<Standard_Integer>(i + 1), v_data.multiplicities[i]);
        }

        const Handle(Geom_BSplineSurface) surface = new Geom_BSplineSurface(
            poles, pole_weights, occt_knots_u, occt_knots_v,
            occt_mults_u, occt_mults_v,
            static_cast<Standard_Integer>(degree_u),
            static_cast<Standard_Integer>(degree_v),
            Standard_False, Standard_False);
        if (surface.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        const Standard_Real u_min = knots_u[degree_u];
        const Standard_Real u_max = knots_u[count_u];
        const Standard_Real v_min = knots_v[degree_v];
        const Standard_Real v_max = knots_v[count_v];
        BRepBuilderAPI_MakeFace maker(surface, u_min, u_max, v_min, v_max, face_tolerance);
        if (!maker.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        const TopoDS_Face face = maker.Face();
        if (face.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        auto* result = new (std::nothrow) umlcad_occt_shape{face};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
