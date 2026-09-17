#include "bridge.hpp"

#include <IGESControl_Controller.hxx>
#include <IGESControl_Reader.hxx>
#include <IGESControl_Writer.hxx>
#include <STEPControl_Reader.hxx>
#include <STEPControl_Writer.hxx>
#include <STEPControl_StepModelType.hxx>
#include <TopoDS_Shape.hxx>

#include <cstdint>
#include <new>

struct umlcad_occt_shape { TopoDS_Shape value; };

namespace {

bool valid_path(const char* path) {
    return path != nullptr && path[0] != '\0';
}

int32_t export_shape(const umlcad_occt_shape* input, int32_t format, const char* path) {
    if (input == nullptr || !valid_path(path)) return UMLCAD_OCCT_INVALID_ARGUMENT;
    try {
        if (input->value.IsNull()) return UMLCAD_OCCT_NULL_SHAPE;
        if (format == 0) {
            STEPControl_Writer writer;
            const IFSelect_ReturnStatus transfer = writer.Transfer(input->value, STEPControl_AsIs);
            if (transfer != IFSelect_RetDone) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            return writer.Write(path) == IFSelect_RetDone ? UMLCAD_OCCT_OK : UMLCAD_OCCT_CONSTRUCTION_FAILED;
        }
        if (format == 1) {
            IGESControl_Controller::Init();
            IGESControl_Writer writer;
            if (!writer.AddShape(input->value)) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            return writer.Write(path) ? UMLCAD_OCCT_OK : UMLCAD_OCCT_CONSTRUCTION_FAILED;
        }
        return UMLCAD_OCCT_INVALID_ARGUMENT;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}

int32_t import_shape(int32_t format, const char* path, umlcad_occt_shape** out_shape) {
    if (!valid_path(path) || out_shape == nullptr) return UMLCAD_OCCT_INVALID_ARGUMENT;
    *out_shape = nullptr;
    try {
        TopoDS_Shape shape;
        if (format == 0) {
            STEPControl_Reader reader;
            if (reader.ReadFile(path) != IFSelect_RetDone) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            if (reader.TransferRoots() <= 0) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            shape = reader.OneShape();
        } else if (format == 1) {
            IGESControl_Controller::Init();
            IGESControl_Reader reader;
            if (reader.ReadFile(path) != IFSelect_RetDone) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            if (reader.TransferRoots() <= 0) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
            shape = reader.OneShape();
        } else {
            return UMLCAD_OCCT_INVALID_ARGUMENT;
        }

        if (shape.IsNull()) return UMLCAD_OCCT_CONSTRUCTION_FAILED;
        auto* result = new (std::nothrow) umlcad_occt_shape{shape};
        if (result == nullptr) return UMLCAD_OCCT_INTERNAL_ERROR;
        *out_shape = result;
        return UMLCAD_OCCT_OK;
    } catch (...) {
        return UMLCAD_OCCT_INTERNAL_ERROR;
    }
}

}

extern "C" int32_t umlcad_occt_shape_export_file(const umlcad_occt_shape* input, int32_t format, const char* path) {
    return export_shape(input, format, path);
}

extern "C" int32_t umlcad_occt_shape_import_file(int32_t format, const char* path, umlcad_occt_shape** out_shape) {
    return import_shape(format, path, out_shape);
}
