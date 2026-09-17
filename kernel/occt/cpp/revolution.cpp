#include "bridge.hpp"

#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepPrimAPI_MakeRevol.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Ax1.hxx>
#include <gp_Pnt.hxx>

#include <algorithm>
#include <cmath>
#include <new>
#include <vector>

struct umlcad_occt_shape {
    TopoDS_Shape value;
};

namespace {
struct Point2d { double r; double z; };

long double orientation(const Point2d& a, const Point2d& b, const Point2d& c) {
    return static_cast<long double>(b.r - a.r) * static_cast<long double>(c.z - a.z)
        - static_cast<long double>(b.z - a.z) * static_cast<long double>(c.r - a.r);
}

bool onSegment(const Point2d& a, const Point2d& b, const Point2d& p) {
    constexpr long double epsilon = 1e-18L;
    return std::abs(orientation(a, b, p)) <= epsilon
        && p.r >= std::min(a.r, b.r) && p.r <= std::max(a.r, b.r)
        && p.z >= std::min(a.z, b.z) && p.z <= std::max(a.z, b.z);
}

bool segmentsIntersect(const Point2d& a, const Point2d& b, const Point2d& c, const Point2d& d) {
    const long double ab_c = orientation(a, b, c);
    const long double ab_d = orientation(a, b, d);
    const long double cd_a = orientation(c, d, a);
    const long double cd_b = orientation(c, d, b);
    const bool proper = ((ab_c > 0.0L && ab_d < 0.0L) || (ab_c < 0.0L && ab_d > 0.0L))
        && ((cd_a > 0.0L && cd_b < 0.0L) || (cd_a < 0.0L && cd_b > 0.0L));
    if (proper) return true;
    return onSegment(a, b, c) || onSegment(a, b, d) || onSegment(c, d, a) || onSegment(c, d, b);
}

bool validProfile(const std::vector<Point2d>& points) {
    if (points.size() < 3) return false;
    for (std::size_t i = 0; i < points.size(); ++i) {
        if (!std::isfinite(points[i].r) || !std::isfinite(points[i].z) || points[i].r <= 0.0) return false;
        const std::size_t next = (i + 1) % points.size();
        if (points[i].r == points[next].r && points[i].z == points[next].z) return false;
    }
    long double twice_area = 0.0L;
    for (std::size_t i = 0; i < points.size(); ++i) {
        const std::size_t next = (i + 1) % points.size();
        twice_area += static_cast<long double>(points[i].r) * points[next].z
            - static_cast<long double>(points[next].r) * points[i].z;
    }
    if (std::abs(twice_area) <= 1e-18L) return false;
    for (std::size_t i = 0; i < points.size(); ++i) {
        const std::size_t i_next = (i + 1) % points.size();
        for (std::size_t j = i + 1; j < points.size(); ++j) {
            const std::size_t j_next = (j + 1) % points.size();
            if (i == j || i_next == j || j_next == i) continue;
            if (segmentsIntersect(points[i], points[i_next], points[j], points[j_next])) return false;
        }
    }
    return true;
}
}

extern "C" int32_t umlcad_occt_revolve_polygon(
    const double* profile_rz,
    uint32_t point_count,
    double angle_radians,
    umlcad_occt_shape** out_shape) {
    if (profile_rz == nullptr || out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    constexpr double two_pi = 6.283185307179586476925286766559;
    if (point_count < 3 || !std::isfinite(angle_radians) || std::abs(angle_radians) <= 1e-12
        || std::abs(angle_radians) > two_pi + 1e-12) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }
    try {
        std::vector<Point2d> profile;
        profile.reserve(point_count);
        for (uint32_t i = 0; i < point_count; ++i) {
            profile.push_back(Point2d{profile_rz[2 * i], profile_rz[2 * i + 1]});
        }
        if (!validProfile(profile)) return UMLCAD_OCCT_INVALID_ARGUMENT;

        BRepBuilderAPI_MakePolygon polygon_builder;
        for (const Point2d& point : profile) {
            polygon_builder.Add(gp_Pnt(point.r, 0.0, point.z));
        }
        polygon_builder.Close();
        if (!polygon_builder.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        BRepBuilderAPI_MakeFace face_builder(polygon_builder.Wire());
        if (!face_builder.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        const gp_Ax1 axis(gp_Pnt(0.0, 0.0, 0.0), gp_Dir(0.0, 0.0, 1.0));
        const TopoDS_Shape revolution = BRepPrimAPI_MakeRevol(face_builder.Face(), axis, angle_radians).Shape();
        if (revolution.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        auto* result = new (std::nothrow) umlcad_occt_shape{revolution};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
