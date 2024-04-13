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
     ids:array<i32>
};
@binding(6) @group(0) var<storage, read> vertex_meta_data0 : VertexMetaData;
@binding(7) @group(0) var<storage, read> vertex_meta_data1 : VertexMetaData;
@binding(8) @group(0) var<storage, read> vertex_meta_data2 : VertexMetaData;
@binding(9) @group(0) var<storage, read> vertex_meta_data3 : VertexMetaData;
@binding(10) @group(0) var<storage, read> vertex_meta_data4 : VertexMetaData;
@binding(11) @group(0) var<storage, read> vertex_meta_data5 : VertexMetaData;
@binding(12) @group(0) var<storage, read> vertex_meta_data6 : VertexMetaData;
@binding(13) @group(0) var<storage, read> vertex_meta_data7 : VertexMetaData;




struct Output {
    @builtin(position) position : vec4<f32>,
    @location(0) world_normal : vec4<f32>,
    @location(1) world_position : vec4<f32>,
    @location(2) @interpolate(flat)  mat_id: i32,
    @location(3) originalpos : vec4<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index : u32,in:VertexInput) -> Output {
    let raw_id=in.material_index;
    let pack_id:i32=raw_id%100;
    let mat_id:i32=(raw_id-pack_id)/100;
    var hull_meta_data=37;
    if(pack_id==0){
        hull_meta_data=vertex_meta_data0.ids[vertex_index];
    }
    if(pack_id==1){
        hull_meta_data=vertex_meta_data1.ids[vertex_index];
    }
    if(pack_id==2){
      hull_meta_data=vertex_meta_data2.ids[vertex_index];

    }
    if(pack_id==3){
       hull_meta_data=vertex_meta_data3.ids[vertex_index];
    }
    if(pack_id==4){
       hull_meta_data=vertex_meta_data4.ids[vertex_index];
    }
    if(pack_id==5){
        hull_meta_data=vertex_meta_data5.ids[vertex_index];
   }
   if(pack_id==6){
     hull_meta_data=vertex_meta_data6.ids[vertex_index];
   }
   if(pack_id==7){
       hull_meta_data=vertex_meta_data7.ids[vertex_index];
   }


    var output: Output;
    output.originalpos= in.position;
    output.mat_id=hull_meta_data;

    output.position = camera.mvp  * in.position;
    output.world_position = in.position;
    output.world_normal = in.normal;
    return output;
}

@fragment
fn fs_main(in:Output) ->  @location(0) vec4<f32> {
    if(
    in.originalpos.x>slice.x_max || in.originalpos.x<slice.x_min
    || in.originalpos.y>slice.y_max || in.originalpos.y<slice.y_min
    || in.originalpos.z>slice.z_max || in.originalpos.z<slice.z_min
    ) { discard;};


   if(in.mat_id!=0){
      let material:LightUniforms=light_uniformsArray[in.mat_id];
      let kd:f32=material.diffuse_intensity;
      let ks:f32=material.specular_intensity;
      let specular_factor:f32=material.specular_shininess;
      let diffuze_color =vec4<f32>(material.color);
      let light_color =vec4<f32>(material.specular_color);
      let view_dir:vec3<f32> = normalize(camera_uniforms.eye_position.xyz - in.world_position.xyz);
            let head_light =  vec4<f32>(camera_uniforms.eye_position.xyz,1.0);
            let light_dir_head_vec:vec3<f32>=head_light.xyz - in.world_position.xyz;
            let light_dir_head_light:vec3<f32> = normalize(light_dir_head_vec);
            let half_dir_head_light:vec3<f32> = normalize(view_dir + light_dir_head_light);
            let diffuse_strength_head_light:f32= max(dot(in.world_normal.xyz, half_dir_head_light), 0.0);
            let diffuse_color_head_light:vec4<f32> = diffuze_color * diffuse_strength_head_light;
            let specular_strength_head_light:f32 = pow(max(dot(in.world_normal.xyz, half_dir_head_light), 0.0), specular_factor);//8 is specular round
            let specular_color_head_light:vec4<f32> =light_color*specular_strength_head_light ;
            let head_light_contribution:vec4<f32>=diffuse_color_head_light*kd + specular_color_head_light*ks;

           if(diffuze_color.a==1.0){
               return vec4<f32>(head_light_contribution.xyz,1.0);
           }else{
               return vec4<f32>(head_light_contribution);
           }
   }
   else{discard;}



    //FOR WASM
   return vec4<f32>(1.0,1.0,1.0,0.0);
}

/*
@fragment
fn fs_main(in:Output) ->  @location(0) vec4<f32> {
    if(
    in.originalpos.x>slice.x_max || in.originalpos.x<slice.x_min
    || in.originalpos.y>slice.y_max || in.originalpos.y<slice.y_min
    || in.originalpos.z>slice.z_max || in.originalpos.z<slice.z_min
    ) { discard;};


   if(in.mat_id!=0){
      let material:LightUniforms=light_uniformsArray[in.mat_id];
      let kd:f32=material.diffuse_intensity;
      let ks:f32=material.specular_intensity;
      let specular_factor:f32=material.specular_shininess;
      let diffuze_color =vec4<f32>(material.color);
      let light_color =vec4<f32>(material.specular_color);
      let view_dir:vec3<f32> = normalize(camera_uniforms.eye_position.xyz - in.world_position.xyz);
            let head_light =  vec4<f32>(camera_uniforms.eye_position.xyz,1.0);
            let light_dir_head_vec:vec3<f32>=head_light.xyz - in.world_position.xyz;
            let light_dir_head_light:vec3<f32> = normalize(light_dir_head_vec);
            let half_dir_head_light:vec3<f32> = normalize(view_dir + light_dir_head_light);
            let diffuse_strength_head_light:f32= max(dot(in.world_normal.xyz, half_dir_head_light), 0.0);
            let diffuse_color_head_light:vec4<f32> = diffuze_color * diffuse_strength_head_light;
            let specular_strength_head_light:f32 = pow(max(dot(in.world_normal.xyz, half_dir_head_light), 0.0), specular_factor);//8 is specular round
            let specular_color_head_light:vec4<f32> =light_color*specular_strength_head_light ;
            let head_light_contribution:vec4<f32>=diffuse_color_head_light*kd + specular_color_head_light*ks;

           if(diffuze_color.a==1.0){
               return vec4<f32>(head_light_contribution.xyz,1.0);
           }else{
               return vec4<f32>(head_light_contribution);
           }
   }
   else{discard;}



    //FOR WASM
   return vec4<f32>(1.0,1.0,1.0,0.0);
}
*/
