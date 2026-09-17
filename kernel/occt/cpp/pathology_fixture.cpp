#include "bridge.hpp"

#include <BRepPrimAPI_MakeBox.hxx>
#include <TopAbs_ShapeEnum.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS_Builder.hxx>
#include <TopoDS_Shape.hxx>

#include <cmath>
#include <new>

struct umlcad_occt_shape {
    TopoDS_Shape value;
};

extern "C" int32_t umlcad_occt_make_non_manifold_box_shell(double width,
                                                              double depth,
                                                              double height,
                                                              umlcad_occt_shape** out_shape) {
    if (out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    if (!std::isfinite(width) || !std::isfinite(depth) || !std::isfinite(height) ||
        width <= 0.0 || depth <= 0.0 || height <= 0.0) {
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    }

    try {
        const TopoDS_Shape box = BRepPrimAPI_MakeBox(width, depth, height).Shape();
        if (box.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        TopoDS_Builder builder;
        TopoDS_Shell shell;
        builder.MakeShell(shell);

        TopoDS_Shape duplicate;
        uint32_t face_index = 0;
        for (TopExp_Explorer explorer(box, TopAbs_FACE); explorer.More(); explorer.Next()) {
            const TopoDS_Shape face = explorer.Current();
            builder.Add(shell, face);
            if (face_index++ == 0U) duplicate = face;
        }
        if (duplicate.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;

        // Deliberately add one face a second time. Its four boundary edges then
        // have three face uses, creating a deterministic non-manifold fixture.
        builder.Add(shell, duplicate);

        auto* result = new (std::nothrow) umlcad_occt_shape{shell};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}
