mod iced_menu;

mod neo_btn;

use std::time::Duration;

use iced_menu::IcedMenu;
use iced_native::mouse::Interaction;
use neothesia_pipelines::background_animation::BgPipeline;

use winit::event::{MouseButton, WindowEvent};

use crate::{
    scene::{Scene, SceneType},
    target::Target,
    ui::{
        iced_conversion,
        iced_state::{self, Program},
    },
};

#[derive(Debug)]
pub enum Event {
    Play,
}

pub struct MenuScene {
    bg_pipeline: BgPipeline,
    iced_state: iced_state::State<IcedMenu>,
}

impl MenuScene {
    pub fn new(target: &mut Target) -> Self {
        let menu = IcedMenu::new(target);
        let iced_state = iced_state::State::new(
            menu,
            target.iced_manager.viewport.logical_size(),
            &mut target.iced_manager.renderer,
        );

        let mut scene = Self {
            bg_pipeline: BgPipeline::new(&target.gpu),
            iced_state,
        };

        scene.resize(target);
        scene
    }
}

impl Scene for MenuScene {
    fn scene_type(&self) -> SceneType {
        SceneType::MainMenu
    }

    fn update(&mut self, target: &mut Target, delta: Duration) {
        self.bg_pipeline.update_time(&mut target.gpu, delta);

        let outs = target.output_manager.get_outputs();
        self.iced_state
            .queue_message(iced_menu::Message::OutputsUpdated(outs));
    }

    fn render(&mut self, target: &mut Target, view: &wgpu::TextureView) {
        self.bg_pipeline
            .render(
                &mut target
                    .gpu
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: None,
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: None,
                    }),
            );

        target
            .iced_manager
            .renderer
            .with_primitives(|backend, primitive| {
                backend.present(
                    &target.gpu.device,
                    &mut target.gpu.staging_belt,
                    &mut target.gpu.encoder,
                    view,
                    primitive,
                    &target.iced_manager.viewport,
                    &target.iced_manager.debug.overlay(),
                )
            })
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        use winit::event::{ElementState, ModifiersState};

        let modifiers = ModifiersState::default();

        if let Some(event) = iced_conversion::window_event(
            event,
            target.iced_manager.viewport.scale_factor(),
            modifiers,
        ) {
            self.iced_state.queue_event(event.clone());

            if let iced_native::event::Event::Keyboard(event) = &event {
                if let Some(msg) = self.iced_state.program().keyboard_input(event) {
                    self.iced_state.queue_message(msg);
                }
            }
        }

        match &event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if self.iced_state.mouse_interaction() == Interaction::Idle {
                    target.window.drag_window();
                }
            }

            _ => {}
        }
    }

    fn main_events_cleared(&mut self, target: &mut Target) {
        if !self.iced_state.is_queue_empty() {
            self.iced_state.update(target);
        }
    }
}
