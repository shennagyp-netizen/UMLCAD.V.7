#include "bridge.hpp"

#include <BRepAlgoAPI_Common.hxx>
#include <BRepAlgoAPI_Cut.hxx>
#include <BRepAlgoAPI_Fuse.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp.hxx>
#include <TopTools_IndexedMapOfShape.hxx>
#include <TopoDS_Shape.hxx>

#include <new>

struct umlcad_occt_shape {
    TopoDS_Shape value;
};

namespace {

bool hasSolid(const TopoDS_Shape& shape) {
    TopTools_IndexedMapOfShape solids;
    TopExp::MapShapes(shape, TopAbs_SOLID, solids);
    return !solids.IsEmpty();
}

int32_t runBoolean(const TopoDS_Shape& left, const TopoDS_Shape& right, TopoDS_Shape& result, int operation) {
    switch (operation) {
        case 0: {
            BRepAlgoAPI_Fuse algorithm(left, right);
            algorithm.Build();
            if (!algorithm.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            result = algorithm.Shape();
            break;
        }
        case 1: {
            BRepAlgoAPI_Common algorithm(left, right);
            algorithm.Build();
            if (!algorithm.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            result = algorithm.Shape();
            break;
        }
        case 2: {
            BRepAlgoAPI_Cut algorithm(left, right);
            algorithm.Build();
            if (!algorithm.IsDone()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            result = algorithm.Shape();
            break;
        }
        default:
            return UMLCAD_OCCT_INVALID_ARGUMENT;
    }

    // The current UMLCAD Boolean contract returns volumetric B-Rep solids.
    // Empty common/cut results cannot be represented as GeometryKind::Solid,
    // so they are rejected explicitly rather than returned as invalid geometry.
    if (result.IsNull() || !hasSolid(result)) {
        return UMLCAD_OCCT_CONSTRUCTION_FAILED;
    }
    return UMLCAD_OCCT_OK;
}

int32_t booleanOperation(const umlcad_occt_shape* left, const umlcad_occt_shape* right, umlcad_occt_shape** out_shape, int operation) {
    if (left == nullptr || right == nullptr || out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    try {
        if (left->value.IsNull() || right->value.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;
        TopoDS_Shape result_shape;
        const int32_t status = runBoolean(left->value, right->value, result_shape, operation);
        if (status != UMLCAD_OCCT_OK) return status;
        auto* result = new (std::nothrow) umlcad_occt_shape{result_shape};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
}

extern "C" int32_t umlcad_occt_fuse(const umlcad_occt_shape* left, const umlcad_occt_shape* right, umlcad_occt_shape** out_shape) {
    return booleanOperation(left, right, out_shape, 0);
}

extern "C" int32_t umlcad_occt_common(const umlcad_occt_shape* left, const umlcad_occt_shape* right, umlcad_occt_shape** out_shape) {
    return booleanOperation(left, right, out_shape, 1);
}

extern "C" int32_t umlcad_occt_cut(const umlcad_occt_shape* left, const umlcad_occt_shape* right, umlcad_occt_shape** out_shape) {
    return booleanOperation(left, right, out_shape, 2);
}
