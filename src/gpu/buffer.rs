use std::mem;
use wgpu::{Buffer, BufferAddress, BufferDescriptor, BufferUsages};

pub struct BatchedRingBuffer<'a, T> {
	buffers: Vec<Buffer>,

	staging_buffers: [Vec<T>; 2],
	current_staging_buffer: usize,

	gpu_device: &'a wgpu::Device,
}

impl<'a, T> BatchedRingBuffer<'a, T> {
	pub fn new(gpu_device: &'a wgpu::Device, num_buffers: usize, buffer_length: usize) -> Self {

		let mut buffers = Vec::with_capacity(num_buffers);
		for i in 0..num_buffers {
			buffers[i] = gpu_device.create_buffer(&BufferDescriptor {
				label: Some(format!("Batched Ring Buffer {}", i).as_str()),
				size: (buffer_length * size_of::<T>()) as BufferAddress,
				usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
				mapped_at_creation: true,
			});
		}

		let staging_buffers = [
			Vec::with_capacity(buffer_length),
			Vec::with_capacity(buffer_length)
		];
		let current_staging_buffer = 0;

		Self { buffers, staging_buffers, current_staging_buffer, gpu_device }
	}

	pub fn push(&mut self, item: T) {
		let staging_buffer = &mut self.staging_buffers[self.current_staging_buffer];
		staging_buffer.push(item);

		if staging_buffer.len() == staging_buffer.capacity() {

		}
	}
}