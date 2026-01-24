use std::sync::Arc;

use anyhow::{Result, anyhow};
use vulkano::{
    device::Device,
    shader::{ShaderModule, ShaderModuleCreateInfo},
};

#[derive(Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

pub fn compile_shader(
    device: Arc<Device>,
    ty: ShaderType,
    source: &str,
) -> Result<Arc<ShaderModule>> {
    let compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    options.add_macro_definition("EP", Some("main"));
    let binary_result = compiler
        .compile_into_spirv(
            source,
            match ty {
                ShaderType::Vertex => shaderc::ShaderKind::Vertex,
                ShaderType::Fragment => shaderc::ShaderKind::Fragment,
            },
            "shader.glsl",
            "main",
            Some(&options),
        )
        .unwrap();
    unsafe {
        ShaderModule::new(
            device,
            ShaderModuleCreateInfo::new(binary_result.as_binary()),
        )
    }
    .map_err(|e| anyhow!("failed to compile {:?} shader: {:?}", ty, e))
}
