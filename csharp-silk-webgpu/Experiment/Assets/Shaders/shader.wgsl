@vertex
fn vs_main(@builtin(vertex_index) vid: u32) -> @builtin(position) vec4f
{
	if (vid == 0u)
	{
		return vec4f(-0.5, -0.5, 0.0, 1.0); 
	}
	else if (vid == 1u)
	{
		return vec4f(0.5, -0.5, 0.0, 1.0); 
	}

	return vec4f(0.0, 0.5, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4f 
{
	return vec4f(1.0, 0.0, 0.0, 1.0);
}