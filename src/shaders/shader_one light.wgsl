// vertex shader

//THIS IS FOR LOGARITHMIC Z FIGHTING THROBLES
   //gl_Position.z = log(gl_Position.w*C + 1)/log(far*C + 1);
    //gl_Position.z *= gl_Position.w;
const C:f32=0.05;
const FAR:f32=100000.0;

const PI:f32= 3.14159265358979323846;

const vx:vec3<f32>=vec3<f32>(1.0,0.0,0.0);
const vy:vec3<f32>=vec3<f32>(0.0,1.0,0.0);
const vz:vec3<f32>=vec3<f32>(0.0,0.0,1.0);

struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) material_index: i32,
};

struct Camera {
    mvp : mat4x4<f32>,
    n_matrix : mat4x4<f32>,
    forward_dir:vec3<f32>,
};
@binding(0) @group(0) var<uniform> camera : Camera;

struct CameraUniforms {
    light_position : vec4<f32>,
    eye_position : vec4<f32>
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
@binding(2) @group(0)   var<uniform> light_uniformsArray: array<LightUniforms, 5>;

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
    is_active:i32,
    t_snap_point_x: f32,
    t_snap_point_y: f32,
    t_snap_point_z: f32,
}
@binding(5) @group(0) var<uniform> snap_object : SnapObject;




struct Output {
    @builtin(position) position : vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) world_normal : vec4<f32>,
    @location(2) world_position : vec4<f32>,
};

@vertex
fn vs_main(in:VertexInput) -> Output {
    let light_uniforms=light_uniformsArray[in.material_index];
    var output: Output;
    output.color=light_uniforms.color;
    output.position = camera.mvp  * in.position;
    output.position.z =log(output.position.w*C + 1.0)/log(FAR*C + 1.0);
    output.position.z *= output.position.w;
    output.world_position = in.position;
    output.world_normal = in.normal;
    return output;
}



@fragment
fn fs_main(in:Output) ->  @location(0) vec4<f32> {

   let light_color =vec4<f32>(1.0,1.0,1.0,1.0);
   let ambient_strength = 0.1;
   let ambient_color = light_color* ambient_strength;

   let light_position = vec4<f32>(slice.x_max,slice.y_max,slice.z_max,1.0);
   let light_dir:vec3<f32> = normalize(light_position.xyz - in.world_position.xyz);
   let view_dir:vec3<f32> = normalize(camera_uniforms.eye_position.xyz - in.world_position.xyz);
   let half_dir:vec3<f32> = normalize(view_dir + light_dir);

   let diffuse_strength = max(dot(in.world_normal.xyz, light_dir), 0.0);
   let diffuse_color = light_color * diffuse_strength;

   let d=dot(in.world_normal.xyz, half_dir);
   let specular_strength = pow(max(d, 0.0), 8.0);//8 is specular round
   let specular_color = specular_strength * in.color.xyz;

   let result = (ambient_color.xyz + diffuse_color.xyz + specular_color.xyz) * in.color.xyz;

    return vec4<f32>(result,1.0);

}
