use std::{ffi::CString, sync::Arc};

use ash::vk;

use crate::{Device, error::Error};

pub struct Shader {
    name: CString,
    stage: vk::ShaderStageFlags,
    module: vk::ShaderModule,

    device: Arc<Device>,
}

impl Device {
    pub fn create_shader(
        self: &Arc<Self>,
        name: impl Into<Vec<u8>>,
        stage: vk::ShaderStageFlags,
        code: &[u8],
    ) -> Result<Shader, Error> {
        assert!(code.len().is_multiple_of(4));
        let code = code
            .chunks_exact(4)
            .map(|c| u32::from_ne_bytes(c.try_into().unwrap()))
            .collect::<Vec<_>>();
        let shader_module_info = vk::ShaderModuleCreateInfo::default().code(&code);

        let module = unsafe {
            self.handle()
                .create_shader_module(&shader_module_info, None)?
        };

        Ok(Shader {
            name: CString::new(name).unwrap(),
            stage,
            module,

            device: self.clone(),
        })
    }
}

impl Shader {
    pub fn pipeline_shader_stage<'a>(&'a self) -> vk::PipelineShaderStageCreateInfo<'a> {
        vk::PipelineShaderStageCreateInfo::default()
            .module(self.module)
            .name(&self.name)
            .stage(self.stage)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.device
                .handle()
                .destroy_shader_module(self.module, None);
        }
    }
}
