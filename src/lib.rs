use winit::
{
	window::Window,
	window::WindowBuilder,
	event_loop::EventLoop,
	event::{ Event, WindowEvent },
};

pub fn run()
{
	env_logger::init();

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("frost")
		.build(&event_loop)
		.unwrap();

	pollster::block_on(main_loop(event_loop, window));
}

async fn main_loop(event_loop: EventLoop<()>, window: Window)
{
	let size = window.inner_size();
	let instance = wgpu::Instance::default();
	let surface = unsafe { instance.create_surface(&window) }.unwrap();
	let adapter = instance
		.request_adapter(&wgpu::RequestAdapterOptions
		{
			power_preference: wgpu::PowerPreference::default(),
			force_fallback_adapter: false,
			compatible_surface: Some(&surface),
		})
		.await
		.expect("Failed to find an appropriate adapter");


	let (device, queue) = adapter
		.request_device(
			&wgpu::DeviceDescriptor
			{
				label: None,
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::default(),
			},
			None,
		)
		.await
		.expect("Failed to create device");


	let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

	let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor
	{
		label: None,
		bind_group_layouts: &[],
		push_constant_ranges: &[],
	});

	let swapchain_capabilities = surface.get_capabilities(&adapter);
	let swapchain_format = swapchain_capabilities.formats[0];

	let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor
	{
		label: None,
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState
		{
			module: &shader,
			entry_point: "vs_main",
			buffers: &[],
		},
		fragment: Some(wgpu::FragmentState
		{
			module: &shader,
			entry_point: "fs_main",
			targets: &[Some(swapchain_format.into())],
		}),
		primitive: wgpu::PrimitiveState
		{
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			..Default::default()
		},
		depth_stencil: Some(wgpu::DepthStencilState
		{
			format: wgpu::TextureFormat::Depth32Float,
			depth_write_enabled: true,
			depth_compare: wgpu::CompareFunction::Less,
			stencil: wgpu::StencilState::default(),
			bias: wgpu::DepthBiasState::default(),
		}),
		multisample: wgpu::MultisampleState::default(),
		multiview: None,
	});

	let mut config = wgpu::SurfaceConfiguration
	{
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: swapchain_format,
		width: size.width,
		height: size.height,
		present_mode: wgpu::PresentMode::Fifo,
		alpha_mode: swapchain_capabilities.alpha_modes[0],
		view_formats: vec![],
	};

	surface.configure(&device, &config);

	let mut depth_view = create_depth_texture(&config, &device);

	event_loop.run(move | event, _, control_flow |
	{
		let _ = (&instance, &adapter, &shader, &pipeline_layout, &config, &depth_view);

		control_flow.set_wait();

		match event
		{
			Event::WindowEvent { event, window_id } if window_id == window.id() => match event
			{
				WindowEvent::CloseRequested =>
				{
					control_flow.set_exit();
				},
				WindowEvent::KeyboardInput { input, .. } =>
				{
					if let Some(keycode) = input.virtual_keycode
					{
						if keycode == winit::event::VirtualKeyCode::Escape
						{
							control_flow.set_exit();
						}
					}
				},
				WindowEvent::Resized(size) =>
				{
					config.width = size.width;
					config.height = size.height;
					surface.configure(&device, &config);
					depth_view = create_depth_texture(&config, &device);
					window.request_redraw();
				}
				_ => {}
			},
			Event::MainEventsCleared =>
			{
				window.request_redraw();
			},
			Event::RedrawRequested(_) =>
			{
				let frame = surface
					.get_current_texture()
					.expect("Failed to acquire next swap chain texture");

				let view = frame
					.texture
					.create_view(&wgpu::TextureViewDescriptor::default());

				let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

				// render pass
				{
					let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor
					{
						label: None,
						color_attachments: &[Some(wgpu::RenderPassColorAttachment
						{
							view: &view,
							resolve_target: None,
							ops: wgpu::Operations
							{
								load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
								store: true,
							},
						})],
						depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment
						{
							view: &depth_view,
							depth_ops: Some(wgpu::Operations
							{
								load: wgpu::LoadOp::Clear(1.0),
								store: true,
							}),
							stencil_ops: None,
						}),
					});

					rpass.set_pipeline(&render_pipeline);
					rpass.draw(0..3, 0..1);
				}

				queue.submit(Some(encoder.finish()));
				frame.present();
			},
            _ => {}
        }
	});
}

fn create_depth_texture(config: &wgpu::SurfaceConfiguration, device: &wgpu::Device,) -> wgpu::TextureView
{
	let depth_texture = device.create_texture(&wgpu::TextureDescriptor
	{
		size: wgpu::Extent3d
		{
			width: config.width,
			height: config.height,
			depth_or_array_layers: 1,
		},
		mip_level_count: 1,
		sample_count: 1,
		dimension: wgpu::TextureDimension::D2,
		format: wgpu::TextureFormat::Depth32Float,
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		label: None,
		view_formats: &[],
	});

	depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}
