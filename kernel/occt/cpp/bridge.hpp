#pragma once
#include <cstdint>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct umlcad_occt_shape umlcad_occt_shape;
typedef enum umlcad_occt_status { UMLCAD_OCCT_OK=0, UMLCAD_OCCT_INVALID_ARGUMENT=1, UMLCAD_OCCT_NULL_SHAPE=2, UMLCAD_OCCT_CONSTRUCTION_FAILED=3, UMLCAD_OCCT_TRANSFORM_FAILED=4, UMLCAD_OCCT_INTERNAL_ERROR=5 } umlcad_occt_status;
int32_t umlcad_occt_box(double width,double depth,double height,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_cylinder(double radius,double height,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_sphere(double radius,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_cone(double base_radius,double top_radius,double height,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_torus(double major_radius,double minor_radius,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_circle_curve(double radius,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_line_curve(double x1,double y1,double z1,double x2,double y2,double z2,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_nurbs_curve3d(const double* poles_xyz,uint32_t pole_count,const double* weights,uint32_t weight_count,const double* knots,uint32_t knot_count,uint32_t degree,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_nurbs_surface3d(const double* poles_xyz,uint32_t count_u,uint32_t count_v,const double* weights,const double* knots_u,uint32_t knot_count_u,const double* knots_v,uint32_t knot_count_v,uint32_t degree_u,uint32_t degree_v,double face_tolerance,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_nurbs_surface3d_differential(const umlcad_occt_shape* shape,double u,double v,double* out_values);
int32_t umlcad_occt_trimmed_nurbs_surface3d(const double* poles_xyz,uint32_t count_u,uint32_t count_v,const double* weights,const double* knots_u,uint32_t knot_count_u,const double* knots_v,uint32_t knot_count_v,uint32_t degree_u,uint32_t degree_v,const double* loop_uv,const uint32_t* loop_counts,uint32_t loop_count,double face_tolerance,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_shape_curve_length(const umlcad_occt_shape* input,double* out_length);
int32_t umlcad_occt_extrude_polygon(const double* points_xy,uint32_t point_count,double height,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_revolve_polygon(const double* profile_rz,uint32_t point_count,double angle_radians,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_loft_polygons(const double* lower_xy,uint32_t lower_count,double lower_z,const double* upper_xy,uint32_t upper_count,double upper_z,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_sweep_linear_circular(double start_x,double start_y,double start_z,double end_x,double end_y,double end_z,double radius,double normal_x,double normal_y,double normal_z,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_fillet_all_edges(const umlcad_occt_shape* input,double radius,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_chamfer_all_edges(const umlcad_occt_shape* input,double distance,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_fuse(const umlcad_occt_shape* left,const umlcad_occt_shape* right,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_common(const umlcad_occt_shape* left,const umlcad_occt_shape* right,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_cut(const umlcad_occt_shape* left,const umlcad_occt_shape* right,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_surface_section_bbox(const umlcad_occt_shape* first,const umlcad_occt_shape* second,double tolerance,double* out_bounds,uint32_t* out_edge_count);
int32_t umlcad_occt_sew_box_faces(double width,double depth,double height,double tolerance,umlcad_occt_shape** out_shape,uint32_t* out_free_edges,uint32_t* out_multiple_edges,uint32_t* out_degenerated);
int32_t umlcad_occt_shape_clone(const umlcad_occt_shape* input,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_shape_translate(const umlcad_occt_shape* input,double dx,double dy,double dz,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_shape_rotate(const umlcad_occt_shape* input,double axis_x,double axis_y,double axis_z,double angle_radians,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_shape_bounding_box(const umlcad_occt_shape* input,double* out_bounds);
int32_t umlcad_occt_shape_topology_counts(const umlcad_occt_shape* input,uint32_t* out_counts);
int32_t umlcad_occt_shape_face_descriptor_count(const umlcad_occt_shape* input,uint32_t* out_count);
int32_t umlcad_occt_shape_face_descriptors(const umlcad_occt_shape* input,double* out_values,uint32_t capacity);
int32_t umlcad_occt_shape_edge_descriptor_count(const umlcad_occt_shape* input,uint32_t* out_count);
int32_t umlcad_occt_shape_edge_descriptors(const umlcad_occt_shape* input,double* out_values,uint32_t capacity);
int32_t umlcad_occt_shape_vertex_descriptor_count(const umlcad_occt_shape* input,uint32_t* out_count);
int32_t umlcad_occt_shape_vertex_descriptors(const umlcad_occt_shape* input,double* out_values,uint32_t capacity);
int32_t umlcad_occt_shape_validate(const umlcad_occt_shape* input,int32_t* valid,int32_t* manifold);
int32_t umlcad_occt_shape_narrow_manifold(const umlcad_occt_shape* input,int32_t* out_manifold);
int32_t umlcad_occt_shape_degenerated_edge_count(const umlcad_occt_shape* input,uint32_t* out_count);
int32_t umlcad_occt_shape_pathology_evidence(const umlcad_occt_shape* input,int32_t* out_valid,int32_t* out_manifold,uint32_t* out_free_edges,uint32_t* out_multiple_edges,uint32_t* out_degenerated_edges,uint32_t* out_counts);
int32_t umlcad_occt_make_open_box_shell(double width,double depth,double height,umlcad_occt_shape** out_shape);
int32_t umlcad_occt_shape_tessellation(const umlcad_occt_shape* input,double linear_deflection,double angular_deflection_radians,double* out_vertices_xyz,uint32_t vertex_capacity,uint32_t* out_vertex_count,uint32_t* out_triangles_abc,uint32_t triangle_capacity,uint32_t* out_triangle_count);
int32_t umlcad_occt_shape_export_file(const umlcad_occt_shape* input,int32_t format,const char* path);
int32_t umlcad_occt_shape_import_file(int32_t format,const char* path,umlcad_occt_shape** out_shape);
void umlcad_occt_shape_delete(umlcad_occt_shape* shape);
#ifdef __cplusplus
}
#endif
