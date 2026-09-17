#include "bridge.hpp"
#include <BRep_Tool.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepPrimAPI_MakeBox.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Builder.hxx>
#include <TopoDS_Shape.hxx>
#include <cmath>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };
namespace {
bool finitePositive(double value) { return std::isfinite(value) && value > 0.0; }
bool topologyCounts(const TopoDS_Shape& shape, uint32_t* counts) {
    if (counts == nullptr || shape.IsNull()) return false;
    const TopAbs_ShapeEnum kinds[] = {TopAbs_SOLID, TopAbs_SHELL, TopAbs_FACE, TopAbs_EDGE, TopAbs_VERTEX};
    for (int index = 0; index < 5; ++index) { TopTools_IndexedMapOfShape unique_shapes; TopExp::MapShapes(shape, kinds[index], unique_shapes); counts[index] = static_cast<uint32_t>(unique_shapes.Extent()); }
    return true;
}
void rawEdgeIncidence(const TopoDS_Shape& shape, uint32_t& free_edges, uint32_t& multiple_edges) {
    free_edges = 0; multiple_edges = 0;
    TopTools_IndexedMapOfShape edges; TopExp::MapShapes(shape, TopAbs_EDGE, edges);
    for (int index = 1; index <= edges.Extent(); ++index) {
        const TopoDS_Shape& edge = edges(index); uint32_t uses = 0;
        for (TopExp_Explorer face_explorer(shape, TopAbs_FACE); face_explorer.More(); face_explorer.Next()) {
            const TopoDS_Shape& face = face_explorer.Current();
            for (TopExp_Explorer edge_explorer(face, TopAbs_EDGE); edge_explorer.More(); edge_explorer.Next()) if (edge_explorer.Current().IsSame(edge)) ++uses;
            if (uses == 1 && BRep_Tool::IsClosed(TopoDS::Edge(edge), TopoDS::Face(face))) uses = 2;
            if (uses > 2) break;
        }
        if (uses == 1) ++free_edges; else if (uses > 2) ++multiple_edges;
    }
}
}
extern "C" int32_t umlcad_occt_shape_degenerated_edge_count(const umlcad_occt_shape* input,uint32_t* out_count){
    if(out_count==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT; *out_count=0; if(input==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;
    try{if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE; uint32_t count=0; for(TopExp_Explorer explorer(input->value,TopAbs_EDGE);explorer.More();explorer.Next()){const TopoDS_Shape edge=explorer.Current();if(edge.IsNull()||edge.ShapeType()!=TopAbs_EDGE)return UMLCAD_OCCT_INTERNAL_ERROR;if(BRep_Tool::Degenerated(TopoDS::Edge(edge)))++count;} *out_count=count; return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}
}
extern "C" int32_t umlcad_occt_shape_pathology_evidence(const umlcad_occt_shape* input,int32_t* out_valid,int32_t* out_manifold,uint32_t* out_free_edges,uint32_t* out_multiple_edges,uint32_t* out_degenerated_edges,uint32_t* out_counts){
    if(out_valid==nullptr||out_manifold==nullptr||out_free_edges==nullptr||out_multiple_edges==nullptr||out_degenerated_edges==nullptr||out_counts==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_valid=0;*out_manifold=0;*out_free_edges=0;*out_multiple_edges=0;*out_degenerated_edges=0;for(int index=0;index<5;++index)out_counts[index]=0;if(input==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;
    try{
        if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;
        const BRepCheck_Analyzer analyzer(input->value,true); *out_valid=analyzer.IsValid()?1:0;
        // This combined snapshot defines manifold only for the established bounded SOLID family.
        // A valid SOLID is the native reference case; non-SOLID shells remain non-manifold for this field.
        *out_manifold=(*out_valid!=0 && input->value.ShapeType()==TopAbs_SOLID)?1:0;
        if(*out_manifold!=0){*out_free_edges=0;*out_multiple_edges=0;}else{rawEdgeIncidence(input->value,*out_free_edges,*out_multiple_edges);}
        for(TopExp_Explorer explorer(input->value,TopAbs_EDGE);explorer.More();explorer.Next()) if(BRep_Tool::Degenerated(TopoDS::Edge(explorer.Current()))) ++(*out_degenerated_edges);
        if(!topologyCounts(input->value,out_counts))return UMLCAD_OCCT_INTERNAL_ERROR; return UMLCAD_OCCT_OK;
    }catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}
}
extern "C" int32_t umlcad_occt_make_open_box_shell(double width,double depth,double height,umlcad_occt_shape** out_shape){
    if(out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;if(!finitePositive(width)||!finitePositive(depth)||!finitePositive(height))return UMLCAD_OCCT_INVALID_ARGUMENT;
    try{const TopoDS_Shape box=BRepPrimAPI_MakeBox(width,depth,height).Shape();if(box.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;TopoDS_Builder builder;TopoDS_Shell shell;builder.MakeShell(shell);uint32_t face_index=0;for(TopExp_Explorer explorer(box,TopAbs_FACE);explorer.More();explorer.Next()){if(face_index++==0U)continue;builder.Add(shell,explorer.Current());}if(shell.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;auto* result=new(std::nothrow) umlcad_occt_shape{shell};if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;}catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}
}
