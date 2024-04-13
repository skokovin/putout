// vertex shader

//THIS IS FOR LOGARITHMIC Z FIGHTING THROBLES
   //gl_Position.z = log(gl_Position.w*C + 1)/log(far*C + 1);
    //gl_Position.z *= gl_Position.w;

fn SMOOTH(r:f32,R:f32)->f32{
return (1.0-smoothstep(R - 1.0,R+1.0, r));
}
const red1= vec4<f32>(1.00,0.095,0.074,1.0);
const blue1= vec4<f32>(0.074,0.095,1.00,1.0);
const green1= vec4<f32>(0.074,1.0,0.095,1.0);
const white1= vec4<f32>(1.0,1.0,1.0,1.0);
const yellow1= vec4<f32>(1.0,1.0,0.0,1.0);

struct VertexInput {
    @location(0) position: vec4<f32>,
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
    p : vec4<f32>,
    p0 : vec4<f32>,
    p1 : vec4<f32>,
    p2 : vec4<f32>,
    mode : vec4<i32>,
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
    @location(0) snap_screen : vec2<f32>,
    @location(1) p0 : vec2<f32>,
    @location(2) p1 : vec2<f32>,
    @location(3) p2 : vec2<f32>,
    @location(4) @interpolate(flat) mode : vec4<i32>,
};

@vertex
fn vs_main(
@builtin(vertex_index) vertex_index : u32,
@builtin(instance_index) instance_index : u32,
in:VertexInput) -> Output {
    var output: Output;
    output.mode=snap_object.mode;
    let snap_point = vec4<f32>(snap_object.p);
    let mvp_snap_point= camera.mvp  * snap_point;
    let px_snap_point= mvp_snap_point.x/mvp_snap_point.w ;
    let py_snap_point= mvp_snap_point.y/mvp_snap_point.w ;
    output.snap_screen=vec2<f32>(px_snap_point,py_snap_point);

    let p0 = vec4<f32>(snap_object.p0);
    let mvp_p0= camera.mvp  * p0;
    let px_p0= mvp_p0.x/mvp_p0.w ;
    let py_p0= mvp_p0.y/mvp_p0.w ;
    output.p0=vec2<f32>(px_p0,py_p0);

    let p1 = vec4<f32>(snap_object.p1);
    let mvp_p1= camera.mvp  * p1;
    let px_p1= mvp_p1.x/mvp_p1.w ;
    let py_p1= mvp_p1.y/mvp_p1.w ;
    output.p1=vec2<f32>(px_p1,py_p1);

    let p2 = vec4<f32>(snap_object.p2);
    let mvp_p2= camera.mvp  * p2;
    let px_p2= mvp_p2.x/mvp_p2.w ;
    let py_p2= mvp_p2.y/mvp_p2.w ;
    output.p2=vec2<f32>(px_p2,py_p2);


    if(vertex_index==0){
      output.position =vec4<f32>(-1.0 ,-1.0,0.0,1.0);
    }
    if(vertex_index==1){
       output.position =vec4<f32>(1.0 ,1.0 ,0.0,1.0);
    }
    if(vertex_index==2){
       output.position =vec4<f32>(-1.0 ,1.0,0.0,1.0);
    }

    if(vertex_index==3){
            output.position =vec4<f32>(-1.0 ,-1.0,0.0,1.0);
        }
    if(vertex_index==4){
              output.position =vec4<f32>(1.0 ,-1.0 ,0.0,1.0);
        }
    if(vertex_index==5){
              output.position =vec4<f32>(1.0 ,1.0 ,0.0,1.0);
        }


    return output;
}

@fragment
fn fs_main(in:Output) ->  @location(0) vec4<f32> {
         let snap_mode:i32=in.mode.x;
         let dim_mode:i32=in.mode.y;
         let w=camera_uniforms.resolution.x;
         let h=camera_uniforms.resolution.y;
         let c:vec2<f32>=to_screen(in.snap_screen);
         let c_p0:vec2<f32>=to_screen(in.p0);
         let c_p1:vec2<f32>=to_screen(in.p1);
         let c_p2:vec2<f32>=to_screen(in.p2);

         var finalColor:vec4<f32>= vec4<f32>(0.0,0.0,0.0,0.0);

         let uv:vec2<f32> = in.position.xy;

        finalColor += ( circle(uv, c, 5.0, 3.0)+ circle(uv, c, 16.5, 1.5) ) * white1;

          switch dim_mode{
            case 1: {
                if (line_with_arrow(c_p0,c_p1,uv)){finalColor +=red1;}
            }
            default: {

            }
          };

          return finalColor;

}

fn to_screen(point: vec2<f32>)->vec2<f32>{
      let w=camera_uniforms.resolution.x;
      let h=camera_uniforms.resolution.y;
          let x_p0= ((point.x+1.0)*w)/2.0;
          let y_p0= ((point.y - 1.0)*h)/-2.0;
          let c_p0=vec2<f32>(x_p0, y_p0);
          return c_p0;
}

fn circle( uv:vec2<f32>, center: vec2<f32>,  radius:f32, width:f32)->f32 {
       let r:f32 = length(uv - center);
       return SMOOTH(r-width/2.0,radius)-SMOOTH(r+width/2.0,radius);
   }

fn line (p1:vec2<f32>,p2:vec2<f32>, uv:vec2<f32>, thin:f32)->bool {
    var output=false;
    let d_base=distance(p2,p1);
    let d_0=distance(p2,uv);
    let d_1=distance(p1,uv);
    let a = p1.y-p2.y;
    let b = p2.x-p1.x;
    let d= abs(a*uv.x+b*uv.y+p1.x*p2.y-p2.x*p1.y) / sqrt(a*a+b*b);
    if(d_0<d_base && d_1<d_base){
     if(abs(a*uv.x+b*uv.y+p1.x*p2.y-p2.x*p1.y) / sqrt(a*a+b*b)<thin){
     output=true;
     }
    }
    return output;
}


fn arrow(p1:vec2<f32>,p2:vec2<f32>,uv:vec2<f32>)->bool {

        let vs:vec2<f32>=normalize(p2-p1);
        let v_cross:vec2<f32>=vec2<f32>(vs.y,vs.x*-1);
        let c:vec2<f32>=p1-vs*50.0;

        let t0:vec2<f32>=p1;
        let t1:vec2<f32>=c+v_cross*15.0;
        let t2:vec2<f32>=c-v_cross*15.0;

        let Area:f32 = 0.5 *(-t1.y*t2.x + t0.y*(-t1.x + t2.x) + t0.x*(t1.y - t2.y) + t1.x*t2.y);
        let s:f32 = 1/(2*Area)*(t0.y*t2.x - t0.x*t2.y + (t2.y - t0.y)*uv.x + (t0.x - t2.x)*uv.y);
        let t:f32 = 1/(2*Area)*(t0.x*t1.y - t0.y*t1.x + (t0.y - t1.y)*uv.x + (t1.x - t0.x)*uv.y);
        if(s>=0 && t>=0 && 1-s-t >=0){
        return true;
        }else{
        return false;
        }
}

fn line_with_arrow(p1:vec2<f32>,p2:vec2<f32>,uv:vec2<f32>)->bool {
    let dist=distance(p1,p2);
    let w=camera_uniforms.resolution.x;
    let h=camera_uniforms.resolution.y;

    var output=false;
    if(dist>5  && p1.x>0 && p1.x<w  && p1.y>0 && p1.y<h && p2.x>0 && p2.x<w  && p2.y>0 && p2.y<h){
           if(arrow(p1,p2,uv)){
                    output=true;
                }else if(arrow(p2,p1,uv)) {
                  output=true;
                }else if(line(p1,p2,uv,1.0)) {
                   output=true;
                 }
    }

    return output;

}