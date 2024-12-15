use egui::Context;
use egui_wgpu::Renderer;
use egui_wgpu::ScreenDescriptor;
use egui_winit::State;
use wgpu::CommandEncoder;
use winit::event::WindowEvent;
use winit::window::Window;

use super::Display;

pub struct EguiRenderer {
    pub context: Context,
    state: State,
    renderer: Renderer,
}

impl EguiRenderer {
    pub fn new(display: &Display, msaa_samples: u32) -> EguiRenderer {
        let egui_ctx = Context::default();
        let viewport_id = egui_ctx.viewport_id();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            viewport_id,
            display.window(),
            None,
            None,
            None,
        );
        let egui_renderer = Renderer::new(
            display.device(),
            display.format(),
            Some(display.depth_format()),
            msaa_samples,
            false,
        );

        EguiRenderer {
            context: egui_ctx,
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub fn draw(
        &mut self,
        display: &Display,
        encoder: &mut CommandEncoder,
        color_attachment: wgpu::RenderPassColorAttachment<'_>,
        depth_stencil_attachment: Option<wgpu::RenderPassDepthStencilAttachment<'_>>,
        run_ui: impl FnMut(&Context),
    ) {
        let raw_input = self.state.take_egui_input(display.window());
        let full_output = self.context.run(raw_input, run_ui);

        self.state
            .handle_platform_output(display.window(), full_output.platform_output);

        let tris = self
            .context
            .tessellate(full_output.shapes, full_output.pixels_per_point);
        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer
                .update_texture(display.device(), display.queue(), *id, &image_delta);
        }
        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: display.size_pixels().into(),
            pixels_per_point: display.window().scale_factor() as f32,
        };
        self.renderer.update_buffers(
            display.device(),
            display.queue(),
            encoder,
            &tris,
            &screen_descriptor,
        );
        let mut rpass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment,
                label: Some("egui main render pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            })
            .forget_lifetime();
        self.renderer.render(&mut rpass, &tris, &screen_descriptor);
        drop(rpass);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
    }
}
