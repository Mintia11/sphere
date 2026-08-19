use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};

#[derive(Default)]
pub struct EguiInputState {
    pointer_pos: egui::Pos2,
    modifiers: egui::Modifiers,
    events: Vec<egui::Event>,
}

impl EguiInputState {
    pub fn on_window_event(&mut self, event: &winit::event::WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.pointer_pos = egui::pos2(position.x as f32, position.y as f32);
                self.events
                    .push(egui::Event::PointerMoved(self.pointer_pos));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                let button = match *button {
                    MouseButton::Left => egui::PointerButton::Primary,
                    MouseButton::Middle => egui::PointerButton::Middle,
                    MouseButton::Right => egui::PointerButton::Secondary,
                    _ => return,
                };
                self.events.push(egui::Event::PointerButton {
                    pos: self.pointer_pos,
                    button,
                    pressed: *state == ElementState::Pressed,
                    modifiers: self.modifiers,
                });
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let d = match delta {
                    MouseScrollDelta::LineDelta(x, y) => egui::vec2(*x, *y) * 20.0,
                    MouseScrollDelta::PixelDelta(p) => egui::vec2(p.x as f32, p.y as f32),
                };
                self.events.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Point,
                    delta: d,
                    modifiers: self.modifiers,
                    phase: egui::TouchPhase::Move,
                });
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let Some(key) = translate_key(event.physical_key) {
                    self.events.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed: event.state == ElementState::Pressed,
                        repeat: false,
                        modifiers: self.modifiers,
                    });
                }
                if let Some(text) = &event.text {
                    self.events.push(egui::Event::Text(text.to_string()));
                }
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = translate_modifiers(mods);
            }
            _ => {}
        }
    }

    pub fn take_raw_input(&mut self, window: &winit::window::Window) -> egui::RawInput {
        let size = window.inner_size();
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(size.width as f32, size.height as f32) / window.scale_factor() as f32,
            )),
            events: std::mem::take(&mut self.events),
            ..Default::default()
        }
    }
}

fn translate_modifiers(mods: &winit::event::Modifiers) -> egui::Modifiers {
    let state: ModifiersState = mods.state();
    egui::Modifiers {
        alt: state.alt_key(),
        ctrl: state.control_key(),
        shift: state.shift_key(),
        #[cfg(target_os = "macos")]
        mac_cmd: state.super_key(),
        #[cfg(target_os = "macos")]
        command: state.super_key(),
        #[cfg(not(target_os = "macos"))]
        mac_cmd: false,
        #[cfg(not(target_os = "macos"))]
        command: state.control_key(),
    }
}

fn translate_key(physical_key: PhysicalKey) -> Option<egui::Key> {
    let PhysicalKey::Code(code) = physical_key else {
        return None; // unidentified / non-standard key
    };

    Some(match code {
        // letters
        KeyCode::KeyA => egui::Key::A,
        KeyCode::KeyB => egui::Key::B,
        KeyCode::KeyC => egui::Key::C,
        KeyCode::KeyD => egui::Key::D,
        KeyCode::KeyE => egui::Key::E,
        KeyCode::KeyF => egui::Key::F,
        KeyCode::KeyG => egui::Key::G,
        KeyCode::KeyH => egui::Key::H,
        KeyCode::KeyI => egui::Key::I,
        KeyCode::KeyJ => egui::Key::J,
        KeyCode::KeyK => egui::Key::K,
        KeyCode::KeyL => egui::Key::L,
        KeyCode::KeyM => egui::Key::M,
        KeyCode::KeyN => egui::Key::N,
        KeyCode::KeyO => egui::Key::O,
        KeyCode::KeyP => egui::Key::P,
        KeyCode::KeyQ => egui::Key::Q,
        KeyCode::KeyR => egui::Key::R,
        KeyCode::KeyS => egui::Key::S,
        KeyCode::KeyT => egui::Key::T,
        KeyCode::KeyU => egui::Key::U,
        KeyCode::KeyV => egui::Key::V,
        KeyCode::KeyW => egui::Key::W,
        KeyCode::KeyX => egui::Key::X,
        KeyCode::KeyY => egui::Key::Y,
        KeyCode::KeyZ => egui::Key::Z,

        // digits (top row)
        KeyCode::Digit0 => egui::Key::Num0,
        KeyCode::Digit1 => egui::Key::Num1,
        KeyCode::Digit2 => egui::Key::Num2,
        KeyCode::Digit3 => egui::Key::Num3,
        KeyCode::Digit4 => egui::Key::Num4,
        KeyCode::Digit5 => egui::Key::Num5,
        KeyCode::Digit6 => egui::Key::Num6,
        KeyCode::Digit7 => egui::Key::Num7,
        KeyCode::Digit8 => egui::Key::Num8,
        KeyCode::Digit9 => egui::Key::Num9,

        // numpad digits — egui has no separate numpad variants, map to same Num*
        KeyCode::Numpad0 => egui::Key::Num0,
        KeyCode::Numpad1 => egui::Key::Num1,
        KeyCode::Numpad2 => egui::Key::Num2,
        KeyCode::Numpad3 => egui::Key::Num3,
        KeyCode::Numpad4 => egui::Key::Num4,
        KeyCode::Numpad5 => egui::Key::Num5,
        KeyCode::Numpad6 => egui::Key::Num6,
        KeyCode::Numpad7 => egui::Key::Num7,
        KeyCode::Numpad8 => egui::Key::Num8,
        KeyCode::Numpad9 => egui::Key::Num9,
        KeyCode::NumpadEnter => egui::Key::Enter,
        KeyCode::NumpadAdd => egui::Key::Plus,
        KeyCode::NumpadSubtract => egui::Key::Minus,
        KeyCode::NumpadDecimal => egui::Key::Period,

        // function keys
        KeyCode::F1 => egui::Key::F1,
        KeyCode::F2 => egui::Key::F2,
        KeyCode::F3 => egui::Key::F3,
        KeyCode::F4 => egui::Key::F4,
        KeyCode::F5 => egui::Key::F5,
        KeyCode::F6 => egui::Key::F6,
        KeyCode::F7 => egui::Key::F7,
        KeyCode::F8 => egui::Key::F8,
        KeyCode::F9 => egui::Key::F9,
        KeyCode::F10 => egui::Key::F10,
        KeyCode::F11 => egui::Key::F11,
        KeyCode::F12 => egui::Key::F12,
        KeyCode::F13 => egui::Key::F13,
        KeyCode::F14 => egui::Key::F14,
        KeyCode::F15 => egui::Key::F15,
        KeyCode::F16 => egui::Key::F16,
        KeyCode::F17 => egui::Key::F17,
        KeyCode::F18 => egui::Key::F18,
        KeyCode::F19 => egui::Key::F19,
        KeyCode::F20 => egui::Key::F20,

        // navigation / editing
        KeyCode::ArrowDown => egui::Key::ArrowDown,
        KeyCode::ArrowLeft => egui::Key::ArrowLeft,
        KeyCode::ArrowRight => egui::Key::ArrowRight,
        KeyCode::ArrowUp => egui::Key::ArrowUp,
        KeyCode::Escape => egui::Key::Escape,
        KeyCode::Tab => egui::Key::Tab,
        KeyCode::Backspace => egui::Key::Backspace,
        KeyCode::Enter => egui::Key::Enter,
        KeyCode::Space => egui::Key::Space,
        KeyCode::Insert => egui::Key::Insert,
        KeyCode::Delete => egui::Key::Delete,
        KeyCode::Home => egui::Key::Home,
        KeyCode::End => egui::Key::End,
        KeyCode::PageUp => egui::Key::PageUp,
        KeyCode::PageDown => egui::Key::PageDown,

        // punctuation
        KeyCode::Minus => egui::Key::Minus,
        KeyCode::Equal => egui::Key::Equals,
        KeyCode::BracketLeft => egui::Key::OpenBracket,
        KeyCode::BracketRight => egui::Key::CloseBracket,
        KeyCode::Backslash => egui::Key::Backslash,
        KeyCode::Semicolon => egui::Key::Semicolon,
        KeyCode::Quote => egui::Key::Quote,
        KeyCode::Comma => egui::Key::Comma,
        KeyCode::Period => egui::Key::Period,
        KeyCode::Slash => egui::Key::Slash,
        KeyCode::Backquote => egui::Key::Backtick,

        _ => return None, // shift/ctrl/alt/super handled via ModifiersChanged, not Key events
    })
}
