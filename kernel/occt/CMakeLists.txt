cmake_minimum_required(VERSION 3.16)
project(umlcad_occt_bridge LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)
find_package(OpenCASCADE REQUIRED)
add_library(umlcad_occt_bridge SHARED
    cpp/bridge.cpp
    cpp/cone.cpp
    cpp/boolean.cpp
    cpp/extrusion.cpp
    cpp/revolution.cpp
    cpp/loft.cpp
    cpp/fillet.cpp
    cpp/vertex.cpp
    cpp/torus.cpp
    cpp/circle.cpp
    cpp/line.cpp
    cpp/nurbs_curve.cpp
    cpp/nurbs_surface.cpp
    cpp/nurbs_surface_differential.cpp
    cpp/trimmed_surface.cpp
    cpp/curve_surface_operations.cpp
    cpp/surface_section.cpp
    cpp/sweep.cpp
    cpp/sewing.cpp
    cpp/topology_evidence.cpp
    cpp/pathology_fixture.cpp
    cpp/tessellation.cpp
    cpp/exchange.cpp
)
target_include_directories(umlcad_occt_bridge PRIVATE ${OpenCASCADE_INCLUDE_DIR})
target_link_directories(umlcad_occt_bridge PRIVATE ${OpenCASCADE_LIBRARY_DIR})
target_link_libraries(umlcad_occt_bridge PRIVATE TKSTEP TKIGES TKXSBase TKMesh TKBool TKOffset TKFillet TKPrim TKTopAlgo TKBRep TKGeomAlgo TKGeomBase TKG3d TKG2d TKMath TKernel)
set_target_properties(umlcad_occt_bridge PROPERTIES OUTPUT_NAME "umlcad_occt_bridge" LIBRARY_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/lib" RUNTIME_OUTPUT_DIRECTORY "${CMAKE_BINARY_DIR}/bin")
install(TARGETS umlcad_occt_bridge LIBRARY DESTINATION lib RUNTIME DESTINATION bin)
