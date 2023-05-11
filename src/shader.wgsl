struct VInput
{
	@location(0) position: vec3<f32>,
	@location(1) normal: vec3<f32>,
	@location(2) color: vec3<f32>,
};

struct VOutput
{
	@builtin(position) position: vec4<f32>,
	@location(0) normal: vec4<f32>,
	@location(1) color: vec4<f32>,
};

@vertex
fn vs_main(in: VInput) -> VOutput
{
	var out: VOutput;

	out.position = vec4<f32>(in.position, 1.0);

	out.position.x = (out.position.x / out.position.z) / 256.0 + 0.5;
	out.position.y = (out.position.y / out.position.z) / 256.0 + 0.5;
	out.position.w = 1.0 / out.position.z;
	out.position.z = 1.0;

	out.normal = vec4<f32>(in.normal, 1.0);
	out.color = vec4<f32>(in.color, 1.0);

	return out;
}

@fragment
fn fs_main(in: VOutput) -> @location(0) vec4<f32>
{
	return in.color;
}
