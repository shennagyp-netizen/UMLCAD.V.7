#include "bridge.hpp"

#include <BRepMesh_IncrementalMesh.hxx>
#include <BRep_Tool.hxx>
#include <TopAbs_Orientation.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopLoc_Location.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <Poly_Triangulation.hxx>

#include <cmath>
#include <cstdint>
#include <limits>
#include <utility>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {

bool finite_value(double value) { return std::isfinite(value); }

bool valid_deflection(double linear, double angular) {
    return finite_value(linear) && finite_value(angular) && linear > 0.0 && angular > 0.0;
}

bool checked_size(std::uint64_t value) {
    return value <= static_cast<std::uint64_t>(std::numeric_limits<std::uint32_t>::max());
}

int32_t collect_mesh(const umlcad_occt_shape* input,
                     double linear_deflection,
                     double angular_deflection_radians,
                     double* out_vertices_xyz,
                     uint32_t vertex_capacity,
                     uint32_t* out_vertex_count,
                     uint32_t* out_triangles_abc,
                     uint32_t triangle_capacity,
                     uint32_t* out_triangle_count) {
    if (input == nullptr || out_vertex_count == nullptr || out_triangle_count == nullptr) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }
    *out_vertex_count = 0;
    *out_triangle_count = 0;
    if (!valid_deflection(linear_deflection, angular_deflection_radians)) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }
    if (out_vertices_xyz == nullptr && vertex_capacity != 0) return UMLCAD_OCCT_INVALID_ARGUMENT;
    if (out_triangles_abc == nullptr && triangle_capacity != 0) return UMLCAD_OCCT_INVALID_ARGUMENT;

    try {
        if (input->value.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;

        BRepMesh_IncrementalMesh mesher(input->value,
                                         linear_deflection,
                                         Standard_False,
                                         angular_deflection_radians,
                                         Standard_False);
        if (!mesher.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        std::uint64_t vertex_count = 0;
        std::uint64_t triangle_count = 0;
        for (TopExp_Explorer face_explorer(input->value, TopAbs_FACE);
             face_explorer.More(); face_explorer.Next()) {
            const TopoDS_Face face = TopoDS::Face(face_explorer.Current());
            TopLoc_Location location;
            const Handle(Poly_Triangulation) triangulation = BRep_Tool::Triangulation(face, location);
            if (triangulation.IsNull()) continue;
            vertex_count += static_cast<std::uint64_t>(triangulation->NbNodes());
            triangle_count += static_cast<std::uint64_t>(triangulation->NbTriangles());
            if (!checked_size(vertex_count) || !checked_size(triangle_count)) {
                return UMLCAD_OCCT_INTERNAL_ERROR;
            }
        }

        *out_vertex_count = static_cast<uint32_t>(vertex_count);
        *out_triangle_count = static_cast<uint32_t>(triangle_count);
        if (vertex_capacity < *out_vertex_count || triangle_capacity < *out_triangle_count) {
            return UMLCAD_OCCT_INVALID_ARGUMENT;
        }
        if (*out_vertex_count == 0 || *out_triangle_count == 0) {
            return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        }

        uint32_t vertex_offset = 0;
        uint32_t triangle_offset = 0;
        for (TopExp_Explorer face_explorer(input->value, TopAbs_FACE);
             face_explorer.More(); face_explorer.Next()) {
            const TopoDS_Face face = TopoDS::Face(face_explorer.Current());
            TopLoc_Location location;
            const Handle(Poly_Triangulation) triangulation = BRep_Tool::Triangulation(face, location);
            if (triangulation.IsNull()) continue;

            const gp_Trsf transformation = location.Transformation();
            for (int node_index = 1; node_index <= triangulation->NbNodes(); ++node_index) {
                gp_Pnt point = triangulation->Node(node_index);
                point.Transform(transformation);
                if (!finite_value(point.X()) || !finite_value(point.Y()) || !finite_value(point.Z())) {
                    return UMLCAD_OCCT_INTERNAL_ERROR;
                }
                const std::size_t base = static_cast<std::size_t>(vertex_offset) * 3U;
                out_vertices_xyz[base + 0] = point.X();
                out_vertices_xyz[base + 1] = point.Y();
                out_vertices_xyz[base + 2] = point.Z();
                ++vertex_offset;
            }

            for (int triangle_index = 1; triangle_index <= triangulation->NbTriangles(); ++triangle_index) {
                int a = 0, b = 0, c = 0;
                triangulation->Triangle(triangle_index).Get(a, b, c);
                if (face.Orientation() == TopAbs_REVERSED) std::swap(b, c);
                if (a <= 0 || b <= 0 || c <= 0 ||
                    a > triangulation->NbNodes() || b > triangulation->NbNodes() || c > triangulation->NbNodes()) {
                    return UMLCAD_OCCT_INTERNAL_ERROR;
                }
                const uint32_t base_vertex = vertex_offset - static_cast<uint32_t>(triangulation->NbNodes());
                const std::size_t base = static_cast<std::size_t>(triangle_offset) * 3U;
                out_triangles_abc[base + 0] = base_vertex + static_cast<uint32_t>(a - 1);
                out_triangles_abc[base + 1] = base_vertex + static_cast<uint32_t>(b - 1);
                out_triangles_abc[base + 2] = base_vertex + static_cast<uint32_t>(c - 1);
                ++triangle_offset;
            }
        }
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}

}

extern "C" int32_t umlcad_occt_shape_tessellation(
    const umlcad_occt_shape* input,
    double linear_deflection,
    double angular_deflection_radians,
    double* out_vertices_xyz,
    uint32_t vertex_capacity,
    uint32_t* out_vertex_count,
    uint32_t* out_triangles_abc,
    uint32_t triangle_capacity,
    uint32_t* out_triangle_count) {
    return collect_mesh(input,
                        linear_deflection,
                        angular_deflection_radians,
                        out_vertices_xyz,
                        vertex_capacity,
                        out_vertex_count,
                        out_triangles_abc,
                        triangle_capacity,
                        out_triangle_count);
}
