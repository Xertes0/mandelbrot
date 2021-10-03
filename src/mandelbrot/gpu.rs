extern crate wgpu;
extern crate bytemuck;
extern crate pollster;

use std::borrow::Cow;
use wgpu::util::DeviceExt;

use super::MandelbrotParameters;

use std::path::Path;

#[allow(dead_code)]
#[derive(Clone, Copy, Default)]
struct ShaderParameters{
	range_min: f32,
	range_max: f32,
	pos_x:     f32,
	pos_y:     f32,
    max_iter:  u32,
    width:     u32,
    height:    u32,
}

unsafe impl bytemuck::Zeroable for ShaderParameters {}
unsafe impl bytemuck::Pod for ShaderParameters {}

impl ShaderParameters {
    pub fn new(params: &MandelbrotParameters) -> Self {
        Self {
            range_min: params.range.0 as f32,
            range_max: params.range.1 as f32,
            pos_x:     params.pos.0 as f32,
            pos_y:     params.pos.1 as f32,
            max_iter:  params.max_iter,
            width:     params.width,
            height:    params.height,
        }
    }
}

const SHADER_PATH: &str = "./shaders/shader.wgsl";

pub struct GpuCompute {
    pixel_count: usize,
    device: wgpu::Device,
    queue:  wgpu::Queue,
    pixels_storage_buffer: wgpu::Buffer,
    params_storage_buffer: wgpu::Buffer,
    compute_pipeline: wgpu::ComputePipeline,
    bind_group:       wgpu::BindGroup,
}

impl GpuCompute {
    pub fn new(pixel_count: usize) -> Option<Self> {
        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
        let adapter = pollster::block_on(instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            )?;

        let (device, queue) = pollster::block_on(adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )).ok()?;

        let info = adapter.get_info();
        // Example does this for some reason
        if info.vendor == 0x10005 {
            return None;
        }

        let cs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(&Self::get_shader_source(pixel_count)?)),
        });

        let pixels_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pixel Storage Buffer"),
            contents: bytemuck::cast_slice(&vec![0u32;pixel_count]),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        });

        let params_storage_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Params Storage Buffer"),
            contents: bytemuck::bytes_of(&ShaderParameters::default()),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST,
        });

        let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &cs_module,
            entry_point: "main",
        });

        let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: pixels_storage_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: params_storage_buffer.as_entire_binding(),
                },
            ],
        });

        Some(Self{
            pixel_count,
            device,
            queue,
            pixels_storage_buffer,
            params_storage_buffer,
            compute_pipeline,
            bind_group,
        })
    }

    pub fn run(&mut self, params: &MandelbrotParameters) -> Option<Vec<u32>> {
        let params = ShaderParameters::new(params);
        self.queue.write_buffer(&self.params_storage_buffer,0,bytemuck::bytes_of(&params));
        pollster::block_on(self.execute_gpu_inner(self.pixel_count))
    }

    async fn execute_gpu_inner(&mut self, pixel_count: usize) -> Option<Vec<u32>> {
        let slice_size = pixel_count * std::mem::size_of::<u32>();
        let size = slice_size as wgpu::BufferAddress;

        let pixels_size = (std::mem::size_of::<u32>() * self.pixel_count) as wgpu::BufferAddress;
        let pixels_staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size:  pixels_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
            cpass.set_pipeline(&self.compute_pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.insert_debug_marker("Mandelbrot");
            cpass.dispatch((self.pixel_count/3) as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
        }

        encoder.copy_buffer_to_buffer(&self.pixels_storage_buffer, 0, &pixels_staging_buffer, 0, size);
        self.queue.submit(Some(encoder.finish()));

        let buffer_slice = pixels_staging_buffer.slice(..);
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

        self.device.poll(wgpu::Maintain::Wait);

        if let Ok(()) = buffer_future.await {
            let data = buffer_slice.get_mapped_range();
            let result = bytemuck::cast_slice(&data).to_vec();

            drop(data);
            pixels_staging_buffer.unmap();
            Some(result)
        } else {
            None
        }
    }

    fn get_shader_source(pixel_count: usize) -> Option<String> {
        let shader = std::fs::read_to_string(Path::new(SHADER_PATH)).ok()?;
        Some(shader
            .split(' ')
            .map(|x| {if x == "PIXEL_COUNT" {(pixel_count/3).to_string()} else {String::from(x)}})
            .collect::<Vec<String>>()
            .join(" ")
        )
    }
}
