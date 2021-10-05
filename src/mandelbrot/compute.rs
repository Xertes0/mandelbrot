use super::Mandelbrot;
#[cfg(feature = "gpu")]
use super::MandelbrotParameters;
#[cfg(feature = "gpu")]
use super::gpu::GpuCompute;

#[cfg(feature = "gpu")]
use std::sync::{Arc,Mutex};

pub trait Compute {
    fn compute(&mut self) -> Vec<u8>;
}

pub struct ComputeCPU {
    mandelbrot: Mandelbrot,
}

impl ComputeCPU {
    pub fn new(mandelbrot: Mandelbrot) -> Self {
        Self{
            mandelbrot,
        }
    }
}

impl Compute for ComputeCPU {
    fn compute(&mut self) -> Vec<u8> {
        self.mandelbrot.update();
        self.mandelbrot.pixels().to_vec()
    }
}

#[cfg(feature = "gpu")]
pub struct ComputeGPU {
    params: MandelbrotParameters,
    gpu_compute: Arc<Mutex<GpuCompute>>,
}

#[cfg(feature = "gpu")]
impl ComputeGPU {
    pub fn new(params: MandelbrotParameters, gpu_compute: Arc<Mutex<GpuCompute>>) -> Self {
        Self{
            params,
            gpu_compute,
        }
    }
}

#[cfg(feature = "gpu")]
impl Compute for ComputeGPU {
    fn compute(&mut self) -> Vec<u8> {
        self.gpu_compute.lock().unwrap().compute(&self.params).unwrap().iter().map(|x| *x as u8).collect::<Vec<u8>>()
    }
}
