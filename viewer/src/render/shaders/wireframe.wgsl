// Vertex shader

struct Uniform {
    matrix: mat4x4<f32>,
    //view_matrix: mat3x3<f32>,
    wireframe_color: vec4<f32>,
    face_color_light: vec4<f32>,
    face_color_dark: vec4<f32>,
    light_dir: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> uniform: Uniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) barycentric: vec3<f32>,
    @location(2) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    //@builtin(front_facing) front_facing: bool,
    @location(0) bc: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(
    in: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.bc = in.barycentric;
    out.normal   = in.normal;
    out.position = uniform.matrix * vec4<f32>(in.position, 1.0);
    return out;
}


// Fragment shader

@fragment
fn fs_main(in: VertexOutput, @builtin(front_facing) front_facing: bool) -> @location(0) vec4<f32> {
    var d: vec3<f32> = fwidth(in.bc);
    var a3: vec3<f32> = smoothstep(vec3<f32>(0.0), d, in.bc);
    var mix_val: f32 = min(a3.x, min(a3.y, a3.z));

    var brightness: f32 = 1.0;

    if (front_facing) {
        brightness = dot(normalize(in.normal), normalize(uniform.light_dir));
    } else {
        brightness = dot(normalize(-in.normal), normalize(uniform.light_dir));
    }

    var color: vec4<f32> = mix(uniform.wireframe_color, mix(uniform.face_color_dark, uniform.face_color_light, brightness), mix_val);

    return color;
}

