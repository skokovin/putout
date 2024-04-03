// vertex shader

//THIS IS FOR LOGARITHMIC Z FIGHTING THROBLES
   //gl_Position.z = log(gl_Position.w*C + 1)/log(far*C + 1);
    //gl_Position.z *= gl_Position.w;
const C:f32=1.0;
const FAR:f32=2000000.0;

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
    //output.position.z =log(output.position.w*C + 1.0)/log(FAR*C + 1.0);
    //output.position.z =log(output.position.z*C + 1.0)/log(FAR*C + 1.0)*output.position.w;
    //output.position.z *= output.position.w;
    output.world_position = in.position;
    output.world_normal = in.normal;
    return output;
}



@fragment
fn fs_main(in:Output) ->  @location(0) vec4<f32> {
   let kd=0.5;
   let ks=0.1;
   let specular_factor:f32=64.0;
   let light_color =vec4<f32>(1.0,1.0,1.0,1.0);
    //let diffuze_color =vec4<f32>(0.0,0.0,1.0,1.0);
   let diffuze_color =vec4<f32>(in.color.xyz,1.0);

   let ambient_strength = 0.05;
   let ambient_color = light_color* ambient_strength;
   let view_dir:vec3<f32> = normalize(camera_uniforms.eye_position.xyz - in.world_position.xyz);



         let head_light =  vec4<f32>(camera_uniforms.eye_position.xyz,1.0);
         let light_dir_head_light:vec3<f32> = normalize(head_light.xyz - in.world_position.xyz);
         let half_dir_head_light:vec3<f32> = normalize(view_dir + light_dir_head_light);
         let diffuse_strength_head_light = max(dot(in.world_normal.xyz, half_dir_head_light), 0.0);
         let diffuse_color_head_light = diffuze_color * diffuse_strength_head_light;
         let specular_strength_head_light = pow(max(dot(in.world_normal.xyz, half_dir_head_light), 0.0), specular_factor);//8 is specular round
         let specular_color_head_light = specular_strength_head_light * light_color.xyz;
         //let head_light_contribution=kd*diffuse_color_head_light.xyz + ks*specular_color_head_light.xyz;
         let head_light_contribution=diffuse_color_head_light.xyz + ks*specular_color_head_light.xyz;


         let a1 = vec4<f32>(slice.x_min,slice.y_min,slice.z_min,1.0);
         let light_dir_a1:vec3<f32> = normalize(a1.xyz - in.world_position.xyz);
         let half_dir_a1:vec3<f32> = normalize(view_dir + light_dir_a1);
         let diffuse_strength_a1 = max(dot(in.world_normal.xyz, light_dir_a1), 0.0);
         let diffuse_color_a1 = diffuze_color * diffuse_strength_a1;
         let specular_strength_a1 = pow(max(dot(in.world_normal.xyz, half_dir_a1), 0.0), specular_factor);//8 is specular round
         let specular_color_a1 = specular_strength_a1 * light_color.xyz;
         let a1_contribution=kd*diffuse_color_a1.xyz + ks*specular_color_a1.xyz;

         let b1 = vec4<f32>(slice.x_max,slice.y_min,slice.z_min,1.0);
         let light_dir_b1:vec3<f32> = normalize(b1.xyz - in.world_position.xyz);
         let half_dir_b1:vec3<f32> = normalize(view_dir + light_dir_b1);
         let diffuse_strength_b1 = max(dot(in.world_normal.xyz, light_dir_b1), 0.0);
         let diffuse_color_b1 = diffuze_color * diffuse_strength_b1;
         let specular_strength_b1 = pow(max(dot(in.world_normal.xyz, half_dir_b1), 0.0), specular_factor);//8 is specular round
         let specular_color_b1 = specular_strength_b1 * light_color.xyz;
         let b1_contribution=kd*diffuse_color_b1.xyz + ks*specular_color_b1.xyz;

         let c1 = vec4<f32>(slice.x_max,slice.y_max,slice.z_min,1.0);
         let light_dir_c1:vec3<f32> = normalize(c1.xyz - in.world_position.xyz);
         let half_dir_c1:vec3<f32> = normalize(view_dir + light_dir_c1);
         let diffuse_strength_c1 = max(dot(in.world_normal.xyz, light_dir_c1), 0.0);
         let diffuse_color_c1 = diffuze_color * diffuse_strength_c1;
         let specular_strength_c1 = pow(max(dot(in.world_normal.xyz, half_dir_c1), 0.0), specular_factor);//8 is specular round
         let specular_color_c1 = specular_strength_c1 * light_color.xyz;
         let c1_contribution=kd*diffuse_color_c1.xyz + ks*specular_color_c1.xyz;


         let d1 = vec4<f32>(slice.x_min,slice.y_max,slice.z_min,1.0);
         let light_dir_d1:vec3<f32> = normalize(d1.xyz - in.world_position.xyz);
         let half_dir_d1:vec3<f32> = normalize(view_dir + light_dir_d1);
         let diffuse_strength_d1 = max(dot(in.world_normal.xyz, light_dir_d1), 0.0);
         let diffuse_color_d1 = diffuze_color * diffuse_strength_d1;
         let specular_strength_d1 = pow(max(dot(in.world_normal.xyz, half_dir_d1), 0.0), specular_factor);//8 is specular round
         let specular_color_d1 = specular_strength_d1 * light_color.xyz;
         let d1_contribution=kd*diffuse_color_d1.xyz + ks*specular_color_d1.xyz;

         let a2 = vec4<f32>(slice.x_min,slice.y_min,slice.z_max,1.0);
         let light_dir_a2:vec3<f32> = normalize(a2.xyz - in.world_position.xyz);
         let half_dir_a2:vec3<f32> = normalize(view_dir + light_dir_a2);
         let diffuse_strength_a2 = max(dot(in.world_normal.xyz, light_dir_a2), 0.0);
         let diffuse_color_a2 = diffuze_color * diffuse_strength_a2;
         let specular_strength_a2 = pow(max(dot(in.world_normal.xyz, half_dir_a2), 0.0), specular_factor);//8 is specular round
         let specular_color_a2 = specular_strength_a2 * light_color.xyz;
         let a2_contribution=kd*diffuse_color_a2.xyz + ks*specular_color_a2.xyz;

          let b2 = vec4<f32>(slice.x_max,slice.y_min,slice.z_max,1.0);
          let light_dir_b2:vec3<f32> = normalize(b2.xyz - in.world_position.xyz);
          let half_dir_b2:vec3<f32> = normalize(view_dir + light_dir_b2);
          let diffuse_strength_b2 = max(dot(in.world_normal.xyz, light_dir_b2), 0.0);
          let diffuse_color_b2 = diffuze_color * diffuse_strength_b2;
          let specular_strength_b2 = pow(max(dot(in.world_normal.xyz, half_dir_b2), 0.0), specular_factor);//8 is specular round
          let specular_color_b2 = specular_strength_b2 * light_color.xyz;
          let b2_contribution=kd*diffuse_color_b2.xyz + ks*specular_color_b2.xyz;


         let c2 = vec4<f32>(slice.x_max,slice.y_max,slice.z_max,1.0);
         let light_dir_c2:vec3<f32> = normalize(c2.xyz - in.world_position.xyz);
         let half_dir_c2:vec3<f32> = normalize(view_dir + light_dir_c2);
         let diffuse_strength_c2 = max(dot(in.world_normal.xyz, light_dir_c2), 0.0);
         let diffuse_color_c2 = diffuze_color * diffuse_strength_c2;
         let specular_strength_c2 = pow(max(dot(in.world_normal.xyz, half_dir_c2), 0.0), specular_factor);//8 is specular round
         let specular_color_c2 = specular_strength_c2 * light_color.xyz;
         let c2_contribution=kd*diffuse_color_c2.xyz + ks*specular_color_c2.xyz;

         let d2 = vec4<f32>(slice.x_min,slice.y_max,slice.z_max,1.0);
         let light_dir_d2:vec3<f32> = normalize(d2.xyz - in.world_position.xyz);
         let half_dir_d2:vec3<f32> = normalize(view_dir + light_dir_d2);
         let diffuse_strength_d2 = max(dot(in.world_normal.xyz, light_dir_d2), 0.0);
         let diffuse_color_d2 = diffuze_color * diffuse_strength_d2;
         let specular_strength_d2 = pow(max(dot(in.world_normal.xyz, half_dir_d2), 0.0), specular_factor);//8 is specular round
         let specular_color_d2 = specular_strength_d2 * light_color.xyz;
         let d2_contribution=kd*diffuse_color_d2.xyz + ks*specular_color_d2.xyz;

       let result = (
       head_light_contribution

       );


/*
       let result = (
       head_light_contribution+
       a1_contribution+
       b1_contribution+
       c1_contribution+
       d1_contribution+
       a2_contribution+
       b2_contribution+
       c2_contribution+
       d2_contribution
       );
*/

    return vec4<f32>(result,1.0);

}
