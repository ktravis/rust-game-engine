// #[repr(C)]
// #[derive(Default, Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
// struct PickingData {
//     transform: Mat4,
//     id: u64,
// }
//
// impl VertexLayout for PickingData {
//     fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Instance,
//             attributes: &Self::ATTRIBUTES,
//         }
//     }
// }
//
// impl PickingData {
//     const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
//         0 => Float32x4,
//         1 => Float32x4,
//         2 => Float32x4,
//         3 => Float32x4,
//         4 => Uint32x2,
//     ];
// }
// impl InstanceData for PickingData {}
//
// let picking_texture = ctx.render_state.create_offscreen_framebuffer(
//     &ctx.display,
//     Point::new(960, 720),
//     wgpu::TextureFormat::Rgba8Uint,
// );
// self.render_pipelines.picking = ctx
//     .render_state
//     .pipeline_builder()
//     .with_label("Picking Render Pipeline")
//     .with_key(self.render_pipelines.picking)
//     // .with_extra_bindings(PickingPipelineBindings::types())
//     .with_color_target_states(vec![Some(wgpu::ColorTargetState {
//         format: wgpu::TextureFormat::Rgba8Uint,
//         blend: None,
//         write_mask: wgpu::ColorWrites::ALL,
//     })])
//     .build(
//         ctx.display.device(),
//         &ctx.display
//             .device()
//             .create_shader_module(wgpu::ShaderModuleDescriptor {
//                 label: Some("picking"),
//                 source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
//                     &self.asset_manager.shader_sources.picking,
//                 )),
//             }),
//     );

// ctx.render_state
//     .render_pass(
//         &ctx.display,
//         "Picking",
//         &self.picking_texture.clear(wgpu::Color::TRANSPARENT),
//         &ViewProjectionUniforms::for_camera(&self.camera),
//         |r| {
//             r.draw_instance(&InstanceRenderData {
//                 texture: None,
//                 mesh: self.cube_mesh,
//                 instance: PickingData {
//                     id: 100,
//                     transform: Default::default(),
//                 },
//                 pipeline: Some(self.render_pipelines.picking),
//             });
//             let mut id = 200;
//             for (mesh, _mat) in &self.model_meshes {
//                 r.draw_instance(&InstanceRenderData {
//                     mesh: *mesh,
//                     instance: PickingData {
//                         id,
//                         transform: Transform3D {
//                             position: vec3(0.0, 2.0, 0.0),
//                             ..Default::default()
//                         }
//                         .as_mat4(),
//                     },
//                     texture: None,
//                     pipeline: Some(self.render_pipelines.picking),
//                 });
//                 id += 100;
//             }
//             for i in 0..self.lights.count {
//                 let light = &self.lights.items[i as usize];
//                 r.draw_instance(&InstanceRenderData {
//                     texture: None,
//                     mesh: self.cube_mesh,
//                     instance: PickingData {
//                         id: id + i as u64,
//                         transform: light.instance().transform,
//                     },
//                     pipeline: Some(self.render_pipelines.picking),
//                 });
//             }
//         },
//     )
//     .submit();
//
// let p = {
//     let picking_texture = ctx.render_state.get_texture(self.picking_texture.color);
//     let block_size = picking_texture.format().block_copy_size(None).unwrap();
//     let bytes_per_row = picking_texture.size_pixels().x * block_size;
//     let buf_view = ctx.display.read_texture_data(&picking_texture);
//     let mouse_x = ctx.input.mouse_position.x.floor() as u32;
//     let mouse_y = ctx.input.mouse_position.y.floor() as u32;
//     let i = (block_size * mouse_x + mouse_y * bytes_per_row) as usize;
//     let mut b = [0u8, 0, 0, 0];
//     b.copy_from_slice(&buf_view[i..i + block_size as usize]);
//     u32::from_le_bytes(b)
// };
// println!("{:?}", p);
