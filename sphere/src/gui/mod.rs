use crate::gui::input::EguiInputState;

mod input;

#[derive(Default)]
pub struct EguiContext {
    input: EguiInputState,
}

impl EguiContext {
    pub fn on_window_event(&mut self, event: &winit::event::WindowEvent) {
        self.input.on_window_event(event);
    }
}
