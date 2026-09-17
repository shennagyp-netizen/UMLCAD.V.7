#include "bridge.hpp"

#include <Bnd_Box.hxx>
#include <BRepBndLib.hxx>
#include <BRepBuilderAPI_Transform.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepGProp.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <BRepPrimAPI_MakeCylinder.hxx>
#include <BRepPrimAPI_MakeSphere.hxx>
#include <GProp_GProps.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedDataMapOfShapeListOfShape.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopTools_ListOfShape.hxx>
#include <TopoDS_Shape.hxx>
#include <gp_Ax1.hxx>
#include <gp_Dir.hxx>
#include <gp_Pnt.hxx>
#include <gp_Trsf.hxx>
#include <gp_Vec.hxx>

#include <cmath>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {

bool hasSolid(const TopoDS_Shape& shape) {
    for (TopExp_Explorer explorer(shape, TopAbs_SOLID); explorer.More(); explorer.Next()) return true;
    return false;
}

bool isFiniteValue(double value) { return std::isfinite(value); }

bool finiteAxis(double x, double y, double z) {
    return isFiniteValue(x) && isFiniteValue(y) && isFiniteValue(z) && (x != 0.0 || y != 0.0 || z != 0.0);
}

bool solidBoundaryIsEdgeManifold(const TopoDS_Shape& shape) {
    if (!hasSolid(shape)) return false;
    TopTools_IndexedMapOfShape edges;
    TopExp::MapShapes(shape, TopAbs_EDGE, edges);
    if (edges.IsEmpty()) return false;
    TopTools_IndexedDataMapOfShapeListOfShape edge_to_face_uses;
    for (TopExp_Explorer face_explorer(shape, TopAbs_FACE); face_explorer.More(); face_explorer.Next()) {
        const TopoDS_Shape& face = face_explorer.Current();
        for (TopExp_Explorer edge_explorer(face, TopAbs_EDGE); edge_explorer.More(); edge_explorer.Next()) {
            const TopoDS_Shape& edge = edge_explorer.Current();
            edge_to_face_uses.Add(edge, TopTools_ListOfShape());
            edge_to_face_uses.ChangeFromKey(edge).Append(face);
        }
    }
    for (int index = 1; index <= edges.Extent(); ++index) {
        const TopoDS_Shape& edge = edges(index);
        const int map_index = edge_to_face_uses.FindIndex(edge);
        if (map_index == 0 || edge_to_face_uses.FindFromIndex(map_index).Extent() != 2) return false;
    }
    return true;
}

bool exactBounds(const TopoDS_Shape& shape, double& min_x, double& min_y, double& min_z, double& max_x, double& max_y, double& max_z) {
    Bnd_Box bounds;
    bounds.SetGap(0.0);
    BRepBndLib::AddOptimal(shape, bounds, Standard_False, Standard_False);
    if (bounds.IsVoid()) return false;
    bounds.Get(min_x, min_y, min_z, max_x, max_y, max_z);
    return isFiniteValue(min_x) && isFiniteValue(min_y) && isFiniteValue(min_z)
        && isFiniteValue(max_x) && isFiniteValue(max_y) && isFiniteValue(max_z);
}

bool faceDescriptor(const TopoDS_Shape& face, double* values) {
    if (values == nullptr || face.IsNull() || face.ShapeType() != TopAbs_FACE) return false;
    GProp_GProps properties;
    BRepGProp::SurfaceProperties(face, properties);
    const double area = properties.Mass();
    if (!isFiniteValue(area) || area < 0.0) return false;
    double min_x, min_y, min_z, max_x, max_y, max_z;
    if (!exactBounds(face, min_x, min_y, min_z, max_x, max_y, max_z)) return false;
    uint32_t boundary_edge_count = 0;
    for (TopExp_Explorer edge_explorer(face, TopAbs_EDGE); edge_explorer.More(); edge_explorer.Next()) ++boundary_edge_count;
    if (boundary_edge_count == 0) return false;
    values[0] = area;
    values[1] = min_x; values[2] = min_y; values[3] = min_z;
    values[4] = max_x; values[5] = max_y; values[6] = max_z;
    values[7] = static_cast<double>(boundary_edge_count);
    return true;
}

bool edgeDescriptor(const TopoDS_Shape& shape, const TopoDS_Shape& edge, double* values) {
    if (values == nullptr || edge.IsNull() || edge.ShapeType() != TopAbs_EDGE) return false;
    GProp_GProps properties;
    BRepGProp::LinearProperties(edge, properties);
    const double length = properties.Mass();
    if (!isFiniteValue(length) || length < 0.0) return false;
    double min_x, min_y, min_z, max_x, max_y, max_z;
    if (!exactBounds(edge, min_x, min_y, min_z, max_x, max_y, max_z)) return false;
    uint32_t vertex_use_count = 0;
    for (TopExp_Explorer vertex_explorer(edge, TopAbs_VERTEX); vertex_explorer.More(); vertex_explorer.Next()) ++vertex_use_count;
    if (vertex_use_count == 0) return false;
    TopTools_IndexedDataMapOfShapeListOfShape edge_to_faces;
    TopExp::MapShapesAndAncestors(shape, TopAbs_EDGE, TopAbs_FACE, edge_to_faces);
    const int index = edge_to_faces.FindIndex(edge);
    const uint32_t face_use_count = index == 0 ? 0U : static_cast<uint32_t>(edge_to_faces.FindFromIndex(index).Extent());
    if (face_use_count == 0) return false;
    values[0] = length;
    values[1] = min_x; values[2] = min_y; values[3] = min_z;
    values[4] = max_x; values[5] = max_y; values[6] = max_z;
    values[7] = static_cast<double>(face_use_count);
    values[8] = static_cast<double>(vertex_use_count);
    return true;
}

}

extern "C" int32_t umlcad_occt_shape_curve_length(const umlcad_occt_shape* input, double* out_length) {
    if (input == nullptr || out_length == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_length = 0.0;
    try {
        if (input->value.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;
        if (input->value.ShapeType() != TopAbs_EDGE) return UMLCAD_OCCT_INVALID_ARGUMENT;
        GProp_GProps properties;
        BRepGProp::LinearProperties(input->value, properties);
        const double length = properties.Mass();
        if (!isFiniteValue(length) || length <= 0.0) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        *out_length = length;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}

extern "C" int32_t umlcad_occt_box(double width,double depth,double height,umlcad_occt_shape** out_shape){
    if(out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;if(!(width>0.0)||!(depth>0.0)||!(height>0.0))return UMLCAD_OCCT_INVALID_ARGUMENT;try{const TopoDS_Shape box=BRepPrimAPI_MakeBox(width,depth,height).Shape();if(box.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;auto* result=new(std::nothrow) umlcad_occt_shape{box};if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_cylinder(double radius,double height,umlcad_occt_shape** out_shape){if(out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;if(!(radius>0.0)||!(height>0.0))return UMLCAD_OCCT_INVALID_ARGUMENT;try{const TopoDS_Shape cylinder=BRepPrimAPI_MakeCylinder(radius,height).Shape();if(cylinder.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;auto* result=new(std::nothrow) umlcad_occt_shape{cylinder};if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_sphere(double radius,umlcad_occt_shape** out_shape){if(out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;if(!(radius>0.0))return UMLCAD_OCCT_INVALID_ARGUMENT;try{const TopoDS_Shape sphere=BRepPrimAPI_MakeSphere(radius).Shape();if(sphere.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;auto* result=new(std::nothrow) umlcad_occt_shape{sphere};if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_clone(const umlcad_occt_shape* input,umlcad_occt_shape** out_shape){if(input==nullptr||out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;auto* result=new(std::nothrow) umlcad_occt_shape{input->value};if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_translate(const umlcad_occt_shape* input,double dx,double dy,double dz,umlcad_occt_shape** out_shape){if(input==nullptr||out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;gp_Trsf transformation;transformation.SetTranslation(gp_Vec(dx,dy,dz));BRepBuilderAPI_Transform transformer(input->value,transformation,true);if(!transformer.IsDone())return UMLCAD_OCCT_TRANSFORM_FAILED;const TopoDS_Shape translated=transformer.Shape();if(translated.IsNull())return UMLCAD_OCCT_TRANSFORM_FAILED;auto* result=new(std::nothrow) umlcad_occt_shape{translated};if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_rotate(const umlcad_occt_shape* input,double axis_x,double axis_y,double axis_z,double angle_radians,umlcad_occt_shape** out_shape){if(input==nullptr||out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;if(!finiteAxis(axis_x,axis_y,axis_z)||!isFiniteValue(angle_radians))return UMLCAD_OCCT_INVALID_ARGUMENT;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;const gp_Ax1 axis(gp_Pnt(0.0,0.0,0.0),gp_Dir(axis_x,axis_y,axis_z));gp_Trsf transformation;transformation.SetRotation(axis,angle_radians);BRepBuilderAPI_Transform transformer(input->value,transformation,true);if(!transformer.IsDone())return UMLCAD_OCCT_TRANSFORM_FAILED;const TopoDS_Shape rotated=transformer.Shape();if(rotated.IsNull())return UMLCAD_OCCT_TRANSFORM_FAILED;auto* result=new(std::nothrow) umlcad_occt_shape{rotated};if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_bounding_box(const umlcad_occt_shape* input,double* out_bounds){if(input==nullptr||out_bounds==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;for(int index=0;index<6;++index)out_bounds[index]=0.0;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;if(!exactBounds(input->value,out_bounds[0],out_bounds[1],out_bounds[2],out_bounds[3],out_bounds[4],out_bounds[5]))return UMLCAD_OCCT_TRANSFORM_FAILED;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_topology_counts(const umlcad_occt_shape* input,uint32_t* out_counts){if(input==nullptr||out_counts==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;for(int index=0;index<5;++index)out_counts[index]=0;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;const TopAbs_ShapeEnum kinds[]={TopAbs_SOLID,TopAbs_SHELL,TopAbs_FACE,TopAbs_EDGE,TopAbs_VERTEX};for(int index=0;index<5;++index){TopTools_IndexedMapOfShape unique_shapes;TopExp::MapShapes(input->value,kinds[index],unique_shapes);out_counts[index]=static_cast<uint32_t>(unique_shapes.Extent());}return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_face_descriptor_count(const umlcad_occt_shape* input,uint32_t* out_count){if(input==nullptr||out_count==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_count=0;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;uint32_t count=0;for(TopExp_Explorer explorer(input->value,TopAbs_FACE);explorer.More();explorer.Next())++count;*out_count=count;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_face_descriptors(const umlcad_occt_shape* input,double* out_values,uint32_t capacity){if(input==nullptr||out_values==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;uint32_t index=0;for(TopExp_Explorer explorer(input->value,TopAbs_FACE);explorer.More();explorer.Next()){if(index>=capacity)return UMLCAD_OCCT_INVALID_ARGUMENT;if(!faceDescriptor(explorer.Current(),out_values+(static_cast<size_t>(index)*8U)))return UMLCAD_OCCT_INTERNAL_ERROR;++index;}return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_edge_descriptor_count(const umlcad_occt_shape* input,uint32_t* out_count){if(input==nullptr||out_count==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_count=0;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;TopTools_IndexedMapOfShape edges;TopExp::MapShapes(input->value,TopAbs_EDGE,edges);*out_count=static_cast<uint32_t>(edges.Extent());return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_edge_descriptors(const umlcad_occt_shape* input,double* out_values,uint32_t capacity){if(input==nullptr||out_values==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;TopTools_IndexedMapOfShape edges;TopExp::MapShapes(input->value,TopAbs_EDGE,edges);if(static_cast<uint32_t>(edges.Extent())>capacity)return UMLCAD_OCCT_INVALID_ARGUMENT;for(int index=1;index<=edges.Extent();++index){if(!edgeDescriptor(input->value,edges(index),out_values+(static_cast<size_t>(index-1)*9U)))return UMLCAD_OCCT_INTERNAL_ERROR;}return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" int32_t umlcad_occt_shape_validate(const umlcad_occt_shape* input,int32_t* valid,int32_t* manifold){if(input==nullptr||valid==nullptr||manifold==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*valid=0;*manifold=0;try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;const BRepCheck_Analyzer analyzer(input->value,true);*valid=analyzer.IsValid()?1:0;*manifold=solidBoundaryIsEdgeManifold(input->value)?1:0;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}}
extern "C" void umlcad_occt_shape_delete(umlcad_occt_shape* shape){delete shape;}
