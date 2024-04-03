// vertex shader

//THIS IS FOR LOGARITHMIC Z FIGHTING THROBLES
   //gl_Position.z = log(gl_Position.w*C + 1)/log(far*C + 1);
    //gl_Position.z *= gl_Position.w;
const C:f32=0.01;
const FAR:f32=20000.0;

const PI:f32= 3.14159265358979323846;

const vx:vec3<f32>=vec3<f32>(1.0,0.0,0.0);
const vy:vec3<f32>=vec3<f32>(0.0,1.0,0.0);
const vz:vec3<f32>=vec3<f32>(0.0,0.0,1.0);


struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) material_index: i32,
    @location(3) id: i32,
};

struct Camera {
    mvp : mat4x4<f32>,
    n_matrix : mat4x4<f32>,
    forward_dir:vec3<f32>,
};
@binding(0) @group(0) var<uniform> camera : Camera;

struct CameraUniforms {
    light_position : vec4<f32>,
    eye_position : vec4<f32>,
    resolution : vec4<f32>
};
@binding(1) @group(0) var<uniform> camera_uniforms : CameraUniforms;

struct LightUniforms {
    color : vec4<f32>,
    specular_color : vec4<f32>,
    ambient_intensity: f32,
    diffuse_intensity :f32,
    specular_intensity: f32,
    specular_shininess: f32
};
@binding(2) @group(0)   var<uniform> light_uniformsArray: array<LightUniforms, 140>;

struct Mode {
    mode : i32,
    modeA : i32,
    modeB : i32,
    modeC : i32
};
@binding(3) @group(0) var<uniform> m : Mode;

struct Slice {
    x_max:f32,
    x_min:f32,
    y_max:f32,
    y_min:f32,
    z_max:f32,
    z_min:f32,
    z_minA:f32,
    z_minB:f32,
};
@binding(4) @group(0) var<uniform> slice : Slice;

struct SnapObject{
    t_snap_point_x: f32,
    t_snap_point_y: f32,
    t_snap_point_z: f32,
    is_active:i32,
}
@binding(5) @group(0) var<uniform> snap_object : SnapObject;

struct VertexMetaData {
     hull:array<i32>
};
@binding(6) @group(0) var<storage, read> vertex_meta_data : VertexMetaData;




struct Output {
    @builtin(position) position : vec4<f32>,
    @location(0) originalpos : vec4<f32>,
    @location(1) @interpolate(flat) oid : i32,
    @location(2) @interpolate(flat) mat_id: i32,

};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index : u32,in:VertexInput) -> Output {
    let hull_meta_data=vertex_meta_data.hull[vertex_index];
    var output: Output;
    output.originalpos= in.position;
    output.mat_id=hull_meta_data;
    //output.oid=in.id;
    output.oid=i32(vertex_index);
    output.position = camera.mvp  * in.position;
    return output;
}



@fragment
fn fs_main(in:Output) ->  @location(0)  vec4<f32>{
    if(
    in.originalpos.x>slice.x_max || in.originalpos.x<slice.x_min
    || in.originalpos.y>slice.y_max || in.originalpos.y<slice.y_min
    || in.originalpos.z>slice.z_max || in.originalpos.z<slice.z_min
    ) { discard;};
     if(in.mat_id==0){discard;}
     let out_test=vec4<f32>(in.originalpos.xyz,f32(in.oid));

     return out_test;
}
