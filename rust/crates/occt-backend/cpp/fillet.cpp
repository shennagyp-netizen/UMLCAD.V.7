#include "bridge.hpp"
#include <BRepFilletAPI_MakeFillet.hxx>
#include <BRepFilletAPI_MakeChamfer.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Edge.hxx>
#include <TopoDS_Shape.hxx>
#include <cmath>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };

extern "C" int32_t umlcad_occt_fillet_all_edges(const umlcad_occt_shape* input,double radius,umlcad_occt_shape** out_shape){
    if(input==nullptr||out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape=nullptr;
    if(!std::isfinite(radius)||radius<=0.0)return UMLCAD_OCCT_INVALID_ARGUMENT;
    try{
        if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;
        bool has_solid=false;
        for(TopExp_Explorer explorer(input->value,TopAbs_SOLID);explorer.More();explorer.Next()){has_solid=true;break;}
        if(!has_solid)return UMLCAD_OCCT_INVALID_ARGUMENT;
        BRepFilletAPI_MakeFillet fillet(input->value);
        bool has_edge=false;
        for(TopExp_Explorer explorer(input->value,TopAbs_EDGE);explorer.More();explorer.Next()){
            has_edge=true;
            fillet.Add(radius,TopoDS::Edge(explorer.Current()));
        }
        if(!has_edge)return UMLCAD_OCCT_INVALID_ARGUMENT;
        fillet.Build();
        if(!fillet.IsDone())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        TopoDS_Shape shape=fillet.Shape();
        if(shape.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        auto* result=new(std::nothrow) umlcad_occt_shape{shape};
        if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape=result;
        return UMLCAD_OCCT_OK;
    }catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}
}

extern "C" int32_t umlcad_occt_chamfer_all_edges(const umlcad_occt_shape* input,double distance,umlcad_occt_shape** out_shape){
    if(input==nullptr||out_shape==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape=nullptr;
    if(!std::isfinite(distance)||distance<=0.0)return UMLCAD_OCCT_INVALID_ARGUMENT;
    try{
        if(input->value.IsNull())return UMLCAD_OCCT_NULL_SHAPE;
        bool has_solid=false;
        for(TopExp_Explorer explorer(input->value,TopAbs_SOLID);explorer.More();explorer.Next()){has_solid=true;break;}
        if(!has_solid)return UMLCAD_OCCT_INVALID_ARGUMENT;
        BRepFilletAPI_MakeChamfer chamfer(input->value);
        bool has_edge=false;
        for(TopExp_Explorer explorer(input->value,TopAbs_EDGE);explorer.More();explorer.Next()){
            has_edge=true;
            chamfer.Add(distance,TopoDS::Edge(explorer.Current()));
        }
        if(!has_edge)return UMLCAD_OCCT_INVALID_ARGUMENT;
        chamfer.Build();
        if(!chamfer.IsDone())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        TopoDS_Shape shape=chamfer.Shape();
        if(shape.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        auto* result=new(std::nothrow) umlcad_occt_shape{shape};
        if(result==nullptr)return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape=result;
        return UMLCAD_OCCT_OK;
    }catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}
}
