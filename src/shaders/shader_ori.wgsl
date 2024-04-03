// vertex shader

//THIS IS FOR LOGARITHMIC Z FIGHTING THROBLES
   //gl_Position.z = log(gl_Position.w*C + 1)/log(far*C + 1);
    //gl_Position.z *= gl_Position.w;
const C:f32=0.05;
const FAR:f32=100000.0;
//const FAR:f32=4000.0;
const PI:f32= 3.14159265358979323846;

const vx:vec3<f32>=vec3<f32>(1.0,0.0,0.0);
const vy:vec3<f32>=vec3<f32>(0.0,1.0,0.0);
const vz:vec3<f32>=vec3<f32>(0.0,0.0,1.0);



struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) normal: vec4<f32>,
    @location(2) material_index: i32,
};


struct Output {
    @builtin(position) position : vec4<f32>,
    @location(0) originalpos : vec4<f32>,
    @location(1) normal : vec4<f32>,
    @location(2) material_index: i32,
    @location(3) color: vec4<f32>,
    @location(4) snap_position: vec4<f32>,
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



@vertex
fn vs_main(in:VertexInput) -> Output {

    let sp=vec4<f32>(snap_object.t_snap_point_x,snap_object.t_snap_point_y,snap_object.t_snap_point_z,0.0);

    let light_uniforms=light_uniformsArray[in.material_index];
    var output: Output;
    output.originalpos= in.position;

    output.normal = vec4<f32>((camera.mvp * (in.normal)).xyz,1.0);
    output.color=light_uniforms.color;
    output.position = camera.mvp  * in.position;
    output.snap_position = camera.mvp  * sp;

    output.material_index=in.material_index;
    //output.position.z =1.0 - 1.0/output.position.z;

    output.position.z =1.0 - 1.0/output.position.z*output.position.w;
    //output.position.z =log(output.position.w*C + 1.0)/log(FAR*C + 1.0);
    //output.position.z *= 2.0*log(output.position.w*C + 1.0)/log(FAR*C + 1.0) - 1.0;

    output.position.z *= output.position.w;
    return output;
}



@fragment
fn fs_main(in:Output) ->  @location(0) vec4<f32> {

    let light_uniforms=light_uniformsArray[in.material_index];
    let N:vec3<f32> = normalize(in.normal.xyz);
    //let L:vec3<f32> = normalize(in.originalpos.xyz-camera_uniforms.light_position.xyz);
    let L:vec3<f32> = -normalize(camera.forward_dir);


    let V:vec3<f32> = normalize(in.normal.xyz);
    let H:vec3<f32> = normalize(L);

    let diffuse:f32 = light_uniforms.diffuse_intensity * max(dot(-N, L), 0.0);
    let specular: f32 = light_uniforms.specular_intensity * pow(max(dot(-N, H),0.0), light_uniforms.specular_shininess);
    let ambient:f32 = light_uniforms.ambient_intensity;
    let final_color:vec3<f32> =  in.color.xyz *(ambient + diffuse) + light_uniforms.specular_color.xyz * specular;

        if(
            in.originalpos.x>slice.x_max || in.originalpos.x<slice.x_min
            || in.originalpos.y>slice.y_max || in.originalpos.y<slice.y_min
            || in.originalpos.z>slice.z_max || in.originalpos.z<slice.z_min
         ) { discard;};



/*                if(snap_object.is_active==1){
                         let d=distance(vec2<f32>(in.position.x/in.position.w,in.position.y/in.position.w),
                         vec2<f32>(in.snap_position.x/in.snap_position.w,in.snap_position.y/in.snap_position.w));
                         if(d<5.0){
                         //discard;
                          return vec4<f32>(vec3<f32>(1.0,0.0,0.0), 1.0);
                         }else{
                             return vec4<f32>(final_color, 1.0);
                         }
                  }
                  else{
                       return vec4<f32>(final_color, 1.0);
                  }*/

        if(snap_object.is_active==1){
                let d=distance(vec3<f32>(snap_object.t_snap_point_x,snap_object.t_snap_point_y,snap_object.t_snap_point_z),
                vec3<f32>(in.originalpos.x,in.originalpos.y,in.originalpos.z));

                if(d<100.0 && d>50.0){
                //discard;
                 return vec4<f32>(vec3<f32>(1.0,0.0,0.0), 1.0);
                }else if(d<30.0 ){
                                //discard;
                                 return vec4<f32>(vec3<f32>(0.0,1.0,0.0), 1.0);
                                }else{
                                    return vec4<f32>(final_color, 1.0);
                                }


         }
         else{
              return vec4<f32>(final_color, 1.0);
         }

}


