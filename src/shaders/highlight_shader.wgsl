// vertex shader

//THIS IS FOR LOGARITHMIC Z FIGHTING THROBLES
   //gl_Position.z = log(gl_Position.w*C + 1)/log(far*C + 1);
    //gl_Position.z *= gl_Position.w;
const C:f32=0.01;
const FAR:f32=20000.0;

const PI:f32= 3.14159265358979323846;
const PId6:f32= 3.14159265358979323846/6.0;

const vx:vec3<f32>=vec3<f32>(1.0,0.0,0.0);
const vy:vec3<f32>=vec3<f32>(0.0,1.0,0.0);
const vz:vec3<f32>=vec3<f32>(0.0,0.0,1.0);

const INTENSIVE_RADIUS:f32=100.0;

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

struct HighLightCabNodes {
     positions:array<vec4<f32>>
};
@binding(5) @group(0) var<storage, read> hlCabNodes : HighLightCabNodes;

struct Output {
    @builtin(position) position : vec4<f32>,
    @location(0) world_normal : vec4<f32>,
    @location(1) world_position : vec4<f32>,
    @location(2) @interpolate(flat)  mat_id: i32,
};

@vertex
fn vs_main(in:VertexInput) -> Output {

    var output: Output;
    output.mat_id=in.material_index;

    output.position = camera.mvp * in.position;
    output.world_position = in.position;
    output.world_normal = in.normal;
    return output;
}

@fragment
fn fs_main(in:Output) ->  @location(0) vec4<f32> {
  let material:LightUniforms=light_uniformsArray[1];
 var ret_color=vec4<f32>(0.0);
 var is_visited=false;
 let cab_sel_count:i32= i32(arrayLength(&hlCabNodes.positions));

    for(var i: i32 = 0; i < cab_sel_count; i++) {
         //let L_point=hlCabNodes.positions[i].xyz;
         let L_point=vec3<f32>(hlCabNodes.positions[i].xyz);
         let debug_point_dir_vec:vec3<f32>=L_point - in.world_position.xyz;
           let l:f32=length(debug_point_dir_vec);

         let eye_dir_vec:vec3<f32>=camera_uniforms.eye_position.xyz - L_point.xyz;
         //CHECK ANGLE > 90 when dot <0
         let cosA=dot(eye_dir_vec,debug_point_dir_vec);

         if(l<INTENSIVE_RADIUS && cosA<0.0){
                is_visited=true;
                         let kd:f32=material.diffuse_intensity;
                         let ks:f32=material.specular_intensity;
                         let specular_factor:f32=material.specular_shininess;
                         let diffuze_color =vec4<f32>(material.color.xyz,1.0);
                         let light_color =vec4<f32>(material.specular_color.xyz,1.0);

                          //let kDist:f32 =min(max(1.0-l/INTENSIVE_RADIUS,0.0),0.6);//1000
                         let kDist:f32 =min(1.0-l/INTENSIVE_RADIUS,0.4);
                         let kDistRev:f32=1.0-kDist;
                         let view_dir_vec:vec3<f32>=camera_uniforms.eye_position.xyz - in.world_position.xyz;
                         let view_dir:vec3<f32> = normalize(view_dir_vec);
                         let head_light =  vec4<f32>(camera_uniforms.eye_position.xyz,1.0);
                         let light_dir_head_vec:vec3<f32>=head_light.xyz - in.world_position.xyz;
                         let light_dir_head_light:vec3<f32> = normalize(light_dir_head_vec);
                         let half_dir_head_light:vec3<f32> = normalize(view_dir + light_dir_head_light);
                         let diffuse_strength_head_light:f32 = max(dot(in.world_normal.xyz, half_dir_head_light), 0.0);
                          let diffuse_color_head_light:vec4<f32> = diffuze_color * diffuse_strength_head_light;
                          let specular_strength_head_light:f32 = pow(max(dot(in.world_normal.xyz, half_dir_head_light), 0.0), specular_factor);//8 is specular round
                          let specular_color_head_light:vec4<f32> = specular_strength_head_light * light_color;
                          let head_light_contribution:vec4<f32>=kd*diffuse_color_head_light;// + ks*specular_color_head_light;
                          ret_color+= vec4<f32>(head_light_contribution.xyz,kDist);

            }

    }

     if(is_visited){
            return vec4<f32>(ret_color);
        }else{
            discard;
        }
        // ONLY FOR NAGA
   return vec4<f32>(ret_color);
}
