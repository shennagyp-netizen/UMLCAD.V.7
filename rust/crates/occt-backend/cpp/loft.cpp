#include "bridge.hpp"
#include <BRepBuilderAPI_MakeFace.hxx>
#include <BRepBuilderAPI_MakePolygon.hxx>
#include <BRepBuilderAPI_MakeSolid.hxx>
#include <BRepBuilderAPI_Sewing.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Face.hxx>
#include <TopoDS_Shape.hxx>
#include <TopoDS_Shell.hxx>
#include <TopoDS_Solid.hxx>
#include <TopoDS.hxx>
#include <gp_Pnt.hxx>
#include <algorithm>
#include <cmath>
#include <new>
#include <vector>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {
bool validSection(const double* xy,uint32_t count,double z){
    if(!xy||count<3||!std::isfinite(z)) return false;
    for(uint32_t i=0;i<count;++i){if(!std::isfinite(xy[2*i])||!std::isfinite(xy[2*i+1])) return false;}
    for(uint32_t i=0;i<count;++i){uint32_t j=(i+1)%count;if(xy[2*i]==xy[2*j]&&xy[2*i+1]==xy[2*j+1]) return false;}
    long double area=0.0L;for(uint32_t i=0;i<count;++i){uint32_t j=(i+1)%count;area+=static_cast<long double>(xy[2*i])*xy[2*j+1]-static_cast<long double>(xy[2*j])*xy[2*i+1];}
    return std::abs(area)>1e-18L;
}
TopoDS_Face makePlanarFace(const std::vector<gp_Pnt>& points,bool& done){
    BRepBuilderAPI_MakePolygon builder;for(const gp_Pnt& point:points)builder.Add(point);builder.Close();
    if(!builder.IsDone()){done=false;return TopoDS_Face();}
    BRepBuilderAPI_MakeFace face_builder(builder.Wire());done=face_builder.IsDone();return done?face_builder.Face():TopoDS_Face();
}
}

extern "C" int32_t umlcad_occt_loft_polygons(const double* lower_xy,uint32_t lower_count,double lower_z,const double* upper_xy,uint32_t upper_count,double upper_z,umlcad_occt_shape** out_shape){
    if(out_shape==nullptr||lower_xy==nullptr||upper_xy==nullptr)return UMLCAD_OCCT_INVALID_ARGUMENT;*out_shape=nullptr;
    if(lower_count!=upper_count||lower_count<3||lower_z==upper_z||!validSection(lower_xy,lower_count,lower_z)||!validSection(upper_xy,upper_count,upper_z))return UMLCAD_OCCT_INVALID_ARGUMENT;
    try{
        std::vector<gp_Pnt> lower,upper;lower.reserve(lower_count);upper.reserve(upper_count);
        for(uint32_t i=0;i<lower_count;++i) lower.emplace_back(lower_xy[2*i],lower_xy[2*i+1],lower_z);
        for(uint32_t i=0;i<upper_count;++i) upper.emplace_back(upper_xy[2*i],upper_xy[2*i+1],upper_z);

        BRepBuilderAPI_Sewing sewing;
        sewing.SetTolerance(1e-12);
        bool done=false;
        TopoDS_Face lower_face=makePlanarFace(lower,done);if(!done)return UMLCAD_OCCT_CONSTRUCTION_FAILED;sewing.Add(lower_face);
        std::vector<gp_Pnt> reversed_upper=upper;std::reverse(reversed_upper.begin(),reversed_upper.end());
        TopoDS_Face upper_face=makePlanarFace(reversed_upper,done);if(!done)return UMLCAD_OCCT_CONSTRUCTION_FAILED;sewing.Add(upper_face);
        for(uint32_t i=0;i<lower_count;++i){uint32_t j=(i+1)%lower_count;std::vector<gp_Pnt> side{lower[i],lower[j],upper[j],upper[i]};TopoDS_Face side_face=makePlanarFace(side,done);if(!done)return UMLCAD_OCCT_CONSTRUCTION_FAILED;sewing.Add(side_face);}
        sewing.Perform();
        TopoDS_Shape sewed=sewing.SewedShape();
        if(sewed.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        TopoDS_Shell shell;
        for(TopExp_Explorer explorer(sewed,TopAbs_SHELL);explorer.More();explorer.Next()){shell=TopoDS::Shell(explorer.Current());break;}
        if(shell.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        BRepBuilderAPI_MakeSolid solid_builder(shell);if(!solid_builder.IsDone())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        TopoDS_Solid solid=solid_builder.Solid();if(solid.IsNull())return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        auto* result=new(std::nothrow) umlcad_occt_shape{solid};if(!result)return UMLCAD_OCCT_INTERNAL_ERROR;*out_shape=result;return UMLCAD_OCCT_OK;
    }catch(...){return UMLCAD_OCCT_INTERNAL_ERROR;}
}
