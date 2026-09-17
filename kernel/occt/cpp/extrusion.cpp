#include "bridge.hpp"

#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepPrimAPI_MakePrism.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Pnt.hxx>
#include <gp_Vec.hxx>

#include <algorithm>
#include <cmath>
#include <new>
#include <vector>

struct umlcad_occt_shape {
    TopoDS_Shape value;
};

namespace {
struct Point2d {
    double x;
    double y;
};

long double orientation(const Point2d& a, const Point2d& b, const Point2d& c) {
    return static_cast<long double>(b.x - a.x) * static_cast<long double>(c.y - a.y)
        - static_cast<long double>(b.y - a.y) * static_cast<long double>(c.x - a.x);
}

bool onSegment(const Point2d& a, const Point2d& b, const Point2d& p) {
    constexpr long double epsilon = 1e-18L;
    return std::abs(orientation(a, b, p)) <= epsilon
        && p.x >= std::min(a.x, b.x) && p.x <= std::max(a.x, b.x)
        && p.y >= std::min(a.y, b.y) && p.y <= std::max(a.y, b.y);
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

bool validSimplePolygon(const std::vector<Point2d>& points) {
    const std::size_t count = points.size();
    if (count < 3) return false;

    for (std::size_t i = 0; i < count; ++i) {
        if (!std::isfinite(points[i].x) || !std::isfinite(points[i].y)) return false;
        const std::size_t next = (i + 1) % count;
        if (points[i].x == points[next].x && points[i].y == points[next].y) return false;
    }

    long double twice_area = 0.0L;
    for (std::size_t i = 0; i < count; ++i) {
        const std::size_t next = (i + 1) % count;
        twice_area += static_cast<long double>(points[i].x) * points[next].y
            - static_cast<long double>(points[next].x) * points[i].y;
    }
    if (std::abs(twice_area) <= 1e-18L) return false;

    for (std::size_t i = 0; i < count; ++i) {
        const std::size_t i_next = (i + 1) % count;
        for (std::size_t j = i + 1; j < count; ++j) {
            const std::size_t j_next = (j + 1) % count;
            if (i == j || i_next == j || j_next == i) continue;
            if (segmentsIntersect(points[i], points[i_next], points[j], points[j_next])) return false;
        }
    }
    return true;
}
}

extern "C" int32_t umlcad_occt_extrude_polygon(
    const double* points_xy,
    uint32_t point_count,
    double height,
    umlcad_occt_shape** out_shape) {
    if (points_xy == nullptr || out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (point_count < 3 || !std::isfinite(height) || height <= 0.0) return UMLCAD_OCCT_INVALID_ARGUMENT;

    try {
        std::vector<Point2d> points;
        points.reserve(point_count);
        for (uint32_t i = 0; i < point_count; ++i) {
            points.push_back(Point2d{points_xy[2 * i], points_xy[2 * i + 1]});
        }
        if (!validSimplePolygon(points)) return UMLCAD_OCCT_INVALID_ARGUMENT;

        BRepBuilderAPI_MakePolygon polygon_builder;
        for (const Point2d& point : points) {
            polygon_builder.Add(gp_Pnt(point.x, point.y, 0.0));
        }
        polygon_builder.Close();
        if (!polygon_builder.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        BRepBuilderAPI_MakeFace face_builder(polygon_builder.Wire());
        if (!face_builder.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        const TopoDS_Shape prism = BRepPrimAPI_MakePrism(face_builder.Face(), gp_Vec(0.0, 0.0, height)).Shape();
        if (prism.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        auto* result = new (std::nothrow) umlcad_occt_shape{prism};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
