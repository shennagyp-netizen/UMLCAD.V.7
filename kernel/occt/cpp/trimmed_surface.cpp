#include "bridge.hpp"

#include <BRepBuilderAPI_MakeEdge.hxx>
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakeWire.hxx>
#include <BRepLib.hxx>
#include <Geom2d_Line.hxx>
#include <Geom_BSplineSurface.hxx>
#include <TColStd_Array1OfInteger.hxx>
#include <TColStd_Array1OfReal.hxx>
#include <TColStd_Array2OfReal.hxx>
#include <TColgp_Array2OfPnt.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Wire.hxx>
#include <gp_Dir2d.hxx>
#include <gp_Pnt2d.hxx>
#include <TopoDS_Shape.hxx>

#include <cmath>
#include <cstdint>
#include <vector>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {

bool finite(double x) { return std::isfinite(x); }

struct KnotData {
    std::vector<double> values;
    std::vector<int> multiplicities;
};

KnotData distinctKnots(const double* knots, uint32_t count) {
    KnotData out;
    out.values.reserve(count);
    out.multiplicities.reserve(count);
    for (uint32_t i = 0; i < count; ++i) {
        if (i == 0 || knots[i] != knots[i - 1]) {
            out.values.push_back(knots[i]);
            out.multiplicities.push_back(1);
        } else {
            ++out.multiplicities.back();
        }
    }
    return out;
}

Handle(Geom_BSplineSurface) buildSurface(
    const double* poles_xyz, uint32_t count_u, uint32_t count_v,
    const double* weights, const double* knots_u, uint32_t knot_count_u,
    const double* knots_v, uint32_t knot_count_v,
    uint32_t degree_u, uint32_t degree_v) {
    TColgp_Array2OfPnt poles(1, static_cast<Standard_Integer>(count_u),
                             1, static_cast<Standard_Integer>(count_v));
    TColStd_Array2OfReal pole_weights(1, static_cast<Standard_Integer>(count_u),
                                      1, static_cast<Standard_Integer>(count_v));
    for (uint32_t u = 0; u < count_u; ++u) {
        for (uint32_t v = 0; v < count_v; ++v) {
            const uint64_t index = static_cast<uint64_t>(u) * count_v + v;
            poles.SetValue(static_cast<Standard_Integer>(u + 1), static_cast<Standard_Integer>(v + 1),
                           gp_Pnt(poles_xyz[3U * index], poles_xyz[3U * index + 1U], poles_xyz[3U * index + 2U]));
            pole_weights.SetValue(static_cast<Standard_Integer>(u + 1), static_cast<Standard_Integer>(v + 1), weights[index]);
        }
    }
    const KnotData u_data = distinctKnots(knots_u, knot_count_u);
    const KnotData v_data = distinctKnots(knots_v, knot_count_v);
    TColStd_Array1OfReal u_knots(1, static_cast<Standard_Integer>(u_data.values.size()));
    TColStd_Array1OfInteger u_mults(1, static_cast<Standard_Integer>(u_data.multiplicities.size()));
    TColStd_Array1OfReal v_knots(1, static_cast<Standard_Integer>(v_data.values.size()));
    TColStd_Array1OfInteger v_mults(1, static_cast<Standard_Integer>(v_data.multiplicities.size()));
    for (std::size_t i = 0; i < u_data.values.size(); ++i) {
        u_knots.SetValue(static_cast<Standard_Integer>(i + 1), u_data.values[i]);
        u_mults.SetValue(static_cast<Standard_Integer>(i + 1), u_data.multiplicities[i]);
    }
    for (std::size_t i = 0; i < v_data.values.size(); ++i) {
        v_knots.SetValue(static_cast<Standard_Integer>(i + 1), v_data.values[i]);
        v_mults.SetValue(static_cast<Standard_Integer>(i + 1), v_data.multiplicities[i]);
    }
    return new Geom_BSplineSurface(
        poles, pole_weights, u_knots, v_knots, u_mults, v_mults,
        static_cast<Standard_Integer>(degree_u), static_cast<Standard_Integer>(degree_v),
        Standard_False, Standard_False);
}

bool validSurfaceInput(const double* poles_xyz, uint32_t count_u, uint32_t count_v,
                       const double* weights, const double* knots_u, uint32_t knot_count_u,
                       const double* knots_v, uint32_t knot_count_v,
                       uint32_t degree_u, uint32_t degree_v, double tolerance) {
    if (!poles_xyz || !weights || !knots_u || !knots_v || degree_u == 0 || degree_v == 0) return false;
    if (count_u < degree_u + 1U || count_v < degree_v + 1U) return false;
    if (knot_count_u != count_u + degree_u + 1U || knot_count_v != count_v + degree_v + 1U) return false;
    if (!finite(tolerance) || tolerance < 0.0) return false;
    const uint64_t count = static_cast<uint64_t>(count_u) * count_v;
    for (uint64_t i = 0; i < count * 3U; ++i) if (!finite(poles_xyz[i])) return false;
    for (uint64_t i = 0; i < count; ++i) if (!finite(weights[i]) || weights[i] <= 0.0) return false;
    for (uint32_t i = 0; i < knot_count_u; ++i) {
        if (!finite(knots_u[i]) || (i > 0 && knots_u[i] < knots_u[i - 1])) return false;
    }
    for (uint32_t i = 0; i < knot_count_v; ++i) {
        if (!finite(knots_v[i]) || (i > 0 && knots_v[i] < knots_v[i - 1])) return false;
    }
    const double u0 = knots_u[degree_u];
    const double u1 = knots_u[count_u];
    const double v0 = knots_v[degree_v];
    const double v1 = knots_v[count_v];
    if (!(u1 > u0) || !(v1 > v0)) return false;
    for (uint32_t i = 0; i <= degree_u; ++i) if (knots_u[i] != u0) return false;
    for (uint32_t i = count_u; i < knot_count_u; ++i) if (knots_u[i] != u1) return false;
    for (uint32_t i = 0; i <= degree_v; ++i) if (knots_v[i] != v0) return false;
    for (uint32_t i = count_v; i < knot_count_v; ++i) if (knots_v[i] != v1) return false;
    return true;
}

TopoDS_Wire buildWire(const double* uv, uint32_t point_count, const Handle(Geom_Surface)& surface) {
    BRepBuilderAPI_MakeWire wire_builder;
    for (uint32_t i = 0; i < point_count; ++i) {
        const uint32_t j = (i + 1U) % point_count;
        const gp_Pnt2d p0(uv[2U * i], uv[2U * i + 1U]);
        const gp_Pnt2d p1(uv[2U * j], uv[2U * j + 1U]);
        const double du = p1.X() - p0.X();
        const double dv = p1.Y() - p0.Y();
        const double length = std::hypot(du, dv);
        if (!(length > 0.0) || !finite(length)) return TopoDS_Wire();
        const Handle(Geom2d_Line) line = new Geom2d_Line(p0, gp_Dir2d(du, dv));
        BRepBuilderAPI_MakeEdge edge_builder(line, surface, 0.0, length);
        if (!edge_builder.IsDone()) return TopoDS_Wire();
        wire_builder.Add(edge_builder.Edge());
        if (!wire_builder.IsDone()) return TopoDS_Wire();
    }
    TopoDS_Wire wire = wire_builder.Wire();
    if (wire.IsNull()) return TopoDS_Wire();
    if (!BRepLib::BuildCurves3d(wire)) return TopoDS_Wire();
    return wire;
}

}

extern "C" int32_t umlcad_occt_trimmed_nurbs_surface3d(
    const double* poles_xyz, uint32_t count_u, uint32_t count_v,
    const double* weights, const double* knots_u, uint32_t knot_count_u,
    const double* knots_v, uint32_t knot_count_v,
    uint32_t degree_u, uint32_t degree_v,
    const double* loop_uv, const uint32_t* loop_counts, uint32_t loop_count,
    double face_tolerance, umlcad_occt_shape** out_shape) {
    if (!out_shape) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (!validSurfaceInput(poles_xyz, count_u, count_v, weights, knots_u, knot_count_u,
                           knots_v, knot_count_v, degree_u, degree_v, face_tolerance) ||
        !loop_uv || !loop_counts || loop_count == 0U) return UMLCAD_OCCT_INVALID_ARGUMENT;
    try {
        const Handle(Geom_BSplineSurface) surface = buildSurface(
            poles_xyz, count_u, count_v, weights, knots_u, knot_count_u,
            knots_v, knot_count_v, degree_u, degree_v);
        if (surface.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        uint64_t offset = 0;
        std::vector<TopoDS_Wire> wires;
        wires.reserve(loop_count);
        for (uint32_t i = 0; i < loop_count; ++i) {
            const uint32_t count = loop_counts[i];
            if (count < 3U) return UMLCAD_OCCT_INVALID_ARGUMENT;
            TopoDS_Wire wire = buildWire(loop_uv + offset * 2U, count, surface);
            if (wire.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            wires.push_back(wire);
            offset += count;
        }

        BRepBuilderAPI_MakeFace face_builder(surface, wires.front(), true);
        if (!face_builder.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        for (uint32_t i = 1; i < loop_count; ++i) {
            face_builder.Add(wires[i]);
        }
        if (!face_builder.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        const TopoDS_Face face = face_builder.Face();
        if (face.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        umlcad_occt_shape* result = new umlcad_occt_shape{face};
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
