#include "bridge.hpp"

#include <BRepBuilderAPI_MakeEdge.hxx>
#include <Geom_BSplineCurve.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColgp_Array1OfPnt.hxx>
#include <TopoDS_Edge.hxx>
#include <cmath>
#include <cstdint>
#include <new>
#include <vector>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {

bool finite(double value) { return std::isfinite(value); }

bool validInput(const double* poles_xyz, uint32_t pole_count,
                const double* weights, uint32_t weight_count,
                const double* knots, uint32_t knot_count, uint32_t degree) {
    if (poles_xyz == nullptr || weights == nullptr || knots == nullptr) return false;
    if (degree == 0 || pole_count < degree + 1) return false;
    if (weight_count != pole_count) return false;
    if (knot_count != pole_count + degree + 1) return false;
    for (uint32_t i = 0; i < pole_count * 3U; ++i) if (!finite(poles_xyz[i])) return false;
    for (uint32_t i = 0; i < weight_count; ++i) if (!finite(weights[i]) || weights[i] <= 0.0) return false;
    for (uint32_t i = 0; i < knot_count; ++i) {
        if (!finite(knots[i])) return false;
        if (i > 0 && knots[i] < knots[i - 1]) return false;
    }
    const double start = knots[degree];
    const double end = knots[pole_count];
    if (!(end > start)) return false;
    for (uint32_t i = 0; i <= degree; ++i) if (knots[i] != start) return false;
    for (uint32_t i = pole_count; i < knot_count; ++i) if (knots[i] != end) return false;
    return true;
}

}

extern "C" int32_t umlcad_occt_nurbs_curve3d(
    const double* poles_xyz, uint32_t pole_count,
    const double* weights, uint32_t weight_count,
    const double* knots, uint32_t knot_count, uint32_t degree,
    umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (!validInput(poles_xyz, pole_count, weights, weight_count, knots, knot_count, degree)) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }

    try {
        TColgp_Array1OfPnt poles(1, static_cast<Standard_Integer>(pole_count));
        TColStd_Array1OfReal pole_weights(1, static_cast<Standard_Integer>(pole_count));
        for (uint32_t i = 0; i < pole_count; ++i) {
            poles.SetValue(static_cast<Standard_Integer>(i + 1),
                           gp_Pnt(poles_xyz[3U * i], poles_xyz[3U * i + 1U], poles_xyz[3U * i + 2U]));
            pole_weights.SetValue(static_cast<Standard_Integer>(i + 1), weights[i]);
        }

        std::vector<double> distinct_knots;
        std::vector<int> multiplicities;
        distinct_knots.reserve(knot_count);
        multiplicities.reserve(knot_count);
        for (uint32_t i = 0; i < knot_count; ++i) {
            if (i == 0 || knots[i] != knots[i - 1]) {
                distinct_knots.push_back(knots[i]);
                multiplicities.push_back(1);
            } else {
                ++multiplicities.back();
            }
        }

        TColStd_Array1OfReal occt_knots(1, static_cast<Standard_Integer>(distinct_knots.size()));
        TColStd_Array1OfInteger occt_multiplicities(1, static_cast<Standard_Integer>(multiplicities.size()));
        for (std::size_t i = 0; i < distinct_knots.size(); ++i) {
            occt_knots.SetValue(static_cast<Standard_Integer>(i + 1), distinct_knots[i]);
            occt_multiplicities.SetValue(static_cast<Standard_Integer>(i + 1), multiplicities[i]);
        }

        const Handle(Geom_BSplineCurve) curve = new Geom_BSplineCurve(
            poles, pole_weights, occt_knots, occt_multiplicities,
            static_cast<Standard_Integer>(degree), Standard_False, Standard_True);
        if (curve.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        BRepBuilderAPI_MakeEdge maker(curve);
        if (!maker.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        const TopoDS_Edge edge = maker.Edge();
        if (edge.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        auto* result = new (std::nothrow) umlcad_occt_shape{edge};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
