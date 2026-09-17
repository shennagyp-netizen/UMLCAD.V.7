#include "bridge.hpp"
#include <BRep_Tool.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Shape.hxx>
#include <cmath>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {
bool finite(double value) { return std::isfinite(value); }

uint32_t incidentEdgeCount(const TopoDS_Shape& shape, const TopoDS_Shape& vertex) {
    TopTools_IndexedMapOfShape edges;
    TopExp::MapShapes(shape, TopAbs_EDGE, edges);
    uint32_t count = 0;
    for (int i = 1; i <= edges.Extent(); ++i) {
        const TopoDS_Shape& edge = edges(i);
        bool touches = false;
        for (TopExp_Explorer explorer(edge, TopAbs_VERTEX); explorer.More(); explorer.Next()) {
            if (explorer.Current().IsSame(vertex)) { touches = true; break; }
        }
        if (touches) ++count;
    }
    return count;
}

uint32_t incidentFaceCount(const TopoDS_Shape& shape, const TopoDS_Shape& vertex) {
    TopTools_IndexedMapOfShape faces;
    TopExp::MapShapes(shape, TopAbs_FACE, faces);
    uint32_t count = 0;
    for (int i = 1; i <= faces.Extent(); ++i) {
        const TopoDS_Shape& face = faces(i);
        bool touches = false;
        for (TopExp_Explorer explorer(face, TopAbs_VERTEX); explorer.More(); explorer.Next()) {
            if (explorer.Current().IsSame(vertex)) { touches = true; break; }
        }
        if (touches) ++count;
    }
    return count;
}

bool descriptor(const TopoDS_Shape& shape, const TopoDS_Shape& vertex, double* values) {
    if (values == nullptr || vertex.IsNull() || vertex.ShapeType() != TopAbs_VERTEX) return false;
    const gp_Pnt point = BRep_Tool::Pnt(TopoDS::Vertex(vertex));
    const double x = point.X(), y = point.Y(), z = point.Z();
    if (!finite(x) || !finite(y) || !finite(z)) return false;

    const uint32_t edge_use_count = incidentEdgeCount(shape, vertex);
    const uint32_t face_use_count = incidentFaceCount(shape, vertex);
    if (edge_use_count == 0 || face_use_count == 0) return false;

    values[0] = x; values[1] = y; values[2] = z;
    values[3] = static_cast<double>(edge_use_count);
    values[4] = static_cast<double>(face_use_count);
    return true;
}
}

extern "C" int32_t umlcad_occt_shape_vertex_descriptor_count(const umlcad_occt_shape* input, uint32_t* out_count) {
    if (input == nullptr || out_count == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_count = 0;
    try {
        if (input->value.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;
        TopTools_IndexedMapOfShape vertices;
        TopExp::MapShapes(input->value, TopAbs_VERTEX, vertices);
        *out_count = static_cast<uint32_t>(vertices.Extent());
        return UMLCAD_OCCT_OK;
    } catch (...) { return UMLCAD_OCCT_INTERNAL_ERROR; }
}

extern "C" int32_t umlcad_occt_shape_vertex_descriptors(const umlcad_occt_shape* input, double* out_values, uint32_t capacity) {
    if (input == nullptr || out_values == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    try {
        if (input->value.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;
        TopTools_IndexedMapOfShape vertices;
        TopExp::MapShapes(input->value, TopAbs_VERTEX, vertices);
        if (capacity < static_cast<uint32_t>(vertices.Extent())) return UMLCAD_OCCT_INVALID_ARGUMENT;
        for (int i = 1; i <= vertices.Extent(); ++i) {
            if (!descriptor(input->value, vertices(i), out_values + (static_cast<size_t>(i - 1) * 5U))) return UMLCAD_OCCT_INTERNAL_ERROR;
        }
        return UMLCAD_OCCT_OK;
    } catch (...) { return UMLCAD_OCCT_INTERNAL_ERROR; }
}
