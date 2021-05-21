use wgpu::{AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device, FilterMode, SamplerDescriptor, TextureDescriptor, TextureSampleType, TextureViewDescriptor, TextureViewDimension};
use imgui_wgpu::{Texture, TextureConfig};

/// Extension methods for `imgui_wgpu::Texture`
pub trait TextureExt {
    /// Create a new `imgui_wgpu::Texture` that uses `FilterMode::Nearest` for scaling.
    fn new_with_nearest_scaling(device: &Device, config: TextureConfig) -> Texture;
}

impl TextureExt for Texture {
    fn new_with_nearest_scaling(
        device: &Device,
        config: TextureConfig
    ) -> Texture {
        let wgpu_texture = device.create_texture(&TextureDescriptor {
            label: config.label,
            size: config.size,
            mip_level_count: config.mip_level_count,
            sample_count: config.sample_count,
            dimension: config.dimension,
            format: config.format.expect(
                "config.format must be provided for Texture::new_with_nearest_scaling"
            ),
            usage: config.usage,
        });

        // Extract the texture view.
        let view = wgpu_texture.create_view(&TextureViewDescriptor::default());

        // Create the texture sampler.
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("imgui-wgpu sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None,
        });

        // Unfortunately `texture_layout` is private on `imgui_wgpu::Renderer` so we
        // have to recreate it.
        let texture_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("imgui-wgpu pixels bind group layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
            ],
        });

        // Create the texture bind group from the layout.
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: config.label,
            layout: &texture_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Texture::from_raw_parts(wgpu_texture, view, bind_group, config.size)
    }
}
