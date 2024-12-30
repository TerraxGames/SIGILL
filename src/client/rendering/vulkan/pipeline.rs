//! # Graphics Pipeline
//! An interface with the graphics pipeline.

use ash::vk;

pub struct PipelineBuilder<'a> {
	input_assembly: vk::PipelineInputAssemblyStateCreateInfo<'a>,
	viewport: vk::PipelineViewportStateCreateInfo<'a>,
	rasterization: vk::PipelineRasterizationStateCreateInfo<'a>,
	depth_stencil: vk::PipelineDepthStencilStateCreateInfo<'a>,
	multisample: vk::PipelineMultisampleStateCreateInfo<'a>,
	color_blend: vk::PipelineColorBlendStateCreateInfo<'a>,
	dynamic_state: vk::PipelineDynamicStateCreateInfo<'a>,
	layout: vk::PipelineLayout,
}

pub struct GraphicsPipeline {}
