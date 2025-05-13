use std::sync::Arc;
use std::time::Instant;
use imgui::{Condition, FontSource, MouseCursor};
use imgui_wgpu::{Renderer, RendererConfig};
use imgui_winit_support::WinitPlatform;
use pollster::block_on;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

struct ImGuiState {
	context: imgui::Context,
	platform: WinitPlatform,
	renderer: Renderer,
	clear_color: wgpu::Color,
	demo_open: bool,
	last_frame: Instant,
	last_cursor: Option<MouseCursor>
}

struct AppWindow {
	device: wgpu::Device,
	queue: wgpu::Queue,
	window: Arc<Window>,
	surface_desc: wgpu::SurfaceConfiguration,
	surface: wgpu::Surface<'static>,
	hidpi_factor: f64,
	imgui: Option<ImGuiState>
}

#[derive(Default)]
struct App {
	window: Option<AppWindow>
}

impl AppWindow {
	fn new(event_loop: &ActiveEventLoop) -> Self {
		let mut window = Self::setup_gpu(event_loop);
		window.setup_imgui();
		window
	}
	
	fn setup_gpu(event_loop: &ActiveEventLoop) -> Self {
		let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
			backends: wgpu::Backends::PRIMARY,
			..Default::default()
		});

		let window = {
			let size = LogicalSize::new(1280.0, 720.0);
			let attributes = Window::default_attributes()
				.with_inner_size(size)
				.with_title("Grayshift World View");

			Arc::new(event_loop.create_window(attributes).unwrap())
		};

		let size = window.inner_size();
		let hidpi_factor = window.scale_factor();
		let surface = instance.create_surface(window.clone()).unwrap();

		let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
			power_preference: wgpu::PowerPreference::HighPerformance,
			compatible_surface: Some(&surface),
			force_fallback_adapter: false
		}))
			.unwrap();

		let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).unwrap();

		let surface_desc = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			width: size.width,
			height: size.height,
			present_mode: wgpu::PresentMode::Fifo,
			desired_maximum_frame_latency: 2,
			alpha_mode: wgpu::CompositeAlphaMode::Auto,
			view_formats: vec![wgpu::TextureFormat::Bgra8Unorm]
		};

		surface.configure(&device, &surface_desc);

		let imgui = None;

		Self {
			device,
			queue,
			window,
			surface_desc,
			surface,
			hidpi_factor,
			imgui
		}
	}

	fn setup_imgui(&mut self) {
		let mut context = imgui::Context::create();
		let mut platform = WinitPlatform::new(&mut context);
		
		platform.attach_window(
			context.io_mut(),
			&self.window,
			imgui_winit_support::HiDpiMode::Default
		);
		context.set_ini_filename(None);
		
		let font_size = (13.0 * self.hidpi_factor) as f32;
		context.io_mut().font_global_scale = (1.0 / self.hidpi_factor) as f32;
		
		context.fonts().add_font(&[FontSource::DefaultFontData {
			config: Some(imgui::FontConfig {
				oversample_h: 1,
				pixel_snap_h: true,
				size_pixels: font_size,
				..Default::default()
			})
		}]);
		
		let clear_color = wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 };
		
		let renderer_config = RendererConfig {
			texture_format: self.surface_desc.format,
			..Default::default()
		};
		
		let renderer = Renderer::new(&mut context, &self.device, &self.queue, renderer_config);
		let last_frame = Instant::now();
		let last_cursor = None;
		let demo_open = true;
		
		self.imgui = Some(ImGuiState {
			context,
			platform,
			renderer,
			clear_color,
			demo_open,
			last_frame,
			last_cursor
		});
	}
}

fn window_redraw(window: &mut AppWindow) {
	let imgui = window.imgui.as_mut().unwrap();
	
	let delta_s = imgui.last_frame.elapsed();
	let now = Instant::now();
	imgui.context.io_mut().update_delta_time(now - imgui.last_frame);
	imgui.last_frame = now;
	
	let frame = match window.surface.get_current_texture() {
		Ok(frame) => frame,
		Err(e) => {
			eprintln!("dropped frame: {e:?}");
			return;
		}
	};
	
	imgui.platform
		.prepare_frame(imgui.context.io_mut(), &window.window)
		.expect("Failed to prepare frame");
	let ui = imgui.context.frame();
	
	{
		let ui_window = ui.window("Window");
		ui_window
			.size([300.0, 100.0], Condition::FirstUseEver)
			.build(|| {
				ui.text("Hello world!");
				ui.separator();
				let mouse_pos = ui.io().mouse_pos;
				ui.text(format!(
					"Mouse Position: ({:.1},{:.1})",
					mouse_pos[0], mouse_pos[1]
				));
			});
		
		let ui_window = ui.window("Window 2");
		ui_window
			.size([400.0, 200.0], Condition::FirstUseEver)
			.position([400.0, 200.0], Condition::FirstUseEver)
			.build(|| {
				ui.text(format!("Frame Time: {delta_s:?}"));
			});
		
		ui.show_demo_window(&mut imgui.demo_open);
	}
	
	let mut encoder: wgpu::CommandEncoder = window.device
		.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
	
	if imgui.last_cursor != ui.mouse_cursor() {
		imgui.last_cursor = ui.mouse_cursor();
		imgui.platform.prepare_render(ui, &window.window);
	}
	
	let view = frame.texture
		.create_view(&wgpu::TextureViewDescriptor::default());
	
	let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
		label: None,
		color_attachments: &[Some(wgpu::RenderPassColorAttachment {
			view: &view,
			resolve_target: None,
			ops: wgpu::Operations {
				load: wgpu::LoadOp::Clear(imgui.clear_color),
				store: wgpu::StoreOp::Store
			}
		})],
		depth_stencil_attachment: None,
		timestamp_writes: None,
		occlusion_query_set: None
	});
	
	imgui.renderer
		.render(
			imgui.context.render(),
			&window.queue,
			&window.device,
			&mut render_pass
		).expect("Rendering failed");
	
	drop(render_pass);
	
	window.queue.submit(Some(encoder.finish()));
	frame.present();
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		self.window = Some(AppWindow::new(event_loop));
	}

	fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
		let window = self.window.as_mut().unwrap();
		
		match &event {
			WindowEvent::Resized(size) => {
				window.surface_desc = wgpu::SurfaceConfiguration {
					usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
					format: wgpu::TextureFormat::Bgra8UnormSrgb,
					width: size.width,
					height: size.height,
					present_mode: wgpu::PresentMode::Fifo,
					desired_maximum_frame_latency: 2,
					alpha_mode: wgpu::CompositeAlphaMode::Auto,
					view_formats: vec![wgpu::TextureFormat::Bgra8Unorm]
				};
				
				window.surface.configure(&window.device, &window.surface_desc)
			},
			
			WindowEvent::CloseRequested => event_loop.exit(),
			WindowEvent::KeyboardInput { event, ..} => {
				if let Key::Named(NamedKey::Escape) = event.logical_key {
					if event.state.is_pressed() { event_loop.exit(); }	
				}
			},
			WindowEvent::RedrawRequested => window_redraw(window),
			_ => {}
		}

		let imgui = window.imgui.as_mut().unwrap();
		imgui.platform.handle_event::<()>(
			imgui.context.io_mut(),
			&window.window,
			&Event::WindowEvent { window_id, event }
		);
	}
}

pub fn world_view_main() {
	let event_loop = EventLoop::new().unwrap();
	event_loop.set_control_flow(ControlFlow::Poll);
	event_loop.run_app(&mut App::default()).unwrap();
}
