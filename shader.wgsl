@group(0) @binding(0) var<storage, read_write> input_buffer : array<i32>;
@group(0) @binding(1) var<storage, read_write> output_buffer : array<i32>;

@compute @workgroup_size(1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;
	output_buffer[index] = input_buffer[index] + 1;
}