use winit::event::{WindowEvent, ElementState, ModifiersState, KeyboardInput};
pub use winit::event::{MouseButton, VirtualKeyCode};

#[derive(Debug, Clone, Copy)]
pub struct Event {
    pub msg: EventMsg,
    pub state: EventState,
}

impl Event {
    pub fn capture_mouse(&mut self) {
        if self.mouse_event() {
            self.msg = EventMsg::None;
        }
        self.state.mouse_x = -100.0;
        self.state.mouse_y = -100.0;
        self.state.left_press = false;
        self.state.right_press = false;
        self.state.middle_press = false;
        self.state.drag = false;
    }

    pub fn shortcut(&self, shortcut: &str) -> bool {
        if let EventMsg::KeyPress(keycode) = self.msg {
            let mut shortcut_mods = ModifiersState::empty();
            let mut consider_mods = true;
            
            let mut key_press = false;
            for s in shortcut.split('+') {
                match s {
                    "<ctrl>" => shortcut_mods.insert(ModifiersState::CTRL),
                    "<shift>" => shortcut_mods.insert(ModifiersState::SHIFT),
                    "<alt>" => shortcut_mods.insert(ModifiersState::ALT),
                    "<any>" => consider_mods = false,
                    key => key_press = format!("{:?}", keycode) == key,
                }
            }

            (self.state.modifiers == shortcut_mods || !consider_mods)
                && key_press
        } else {
            false
        }
    }

    pub fn mouse_event(&self) -> bool {
        match self.msg {
            EventMsg::MouseMove(_, _) => true,
            EventMsg::MousePress(_) => true,
            EventMsg::Click(_) => true,
            EventMsg::Scroll(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EventState {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub left_press: bool,
    pub right_press: bool,
    pub middle_press: bool,
    pub drag: bool,
    pub modifiers: ModifiersState,
}

impl EventState {
    pub fn new() -> EventState {
        EventState {
            mouse_x: 0.0,
            mouse_y: 0.0,
            left_press: false,
            right_press: false,
            middle_press: false,
            drag: false,
            modifiers: ModifiersState::empty(),
        }
    }

    pub fn handle_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => self.left_press = state == &ElementState::Pressed,
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                state,
                ..
            } => self.right_press = state == &ElementState::Pressed,
            WindowEvent::MouseInput {
                button: MouseButton::Middle,
                state,
                ..
            } => self.middle_press = state == &ElementState::Pressed,
            WindowEvent::CursorMoved {
                position,
                ..
            } => {
                self.mouse_x = position.x as f32;
                self.mouse_y = position.y as f32;
                self.drag = self.any_press();
            },
            WindowEvent::ModifiersChanged(modifiers) => {
                self.modifiers = *modifiers;
            },
            _ => ()
        }
        if !self.any_press() {
            self.drag = false;
        }
    }

    pub fn any_press(&self) -> bool {
        self.left_press || self.right_press || self.middle_press
    }
}

#[derive(Debug, Clone, Copy)]
pub enum EventMsg {
    None,
    MouseMove(f32, f32),
    MousePress(MouseButton),
    Scroll(f32),
    /// A Click is a fast mouse press without drag
    Click(MouseButton),
    KeyPress(VirtualKeyCode)
}

pub struct EventHandler {
    state: EventState,
}

impl EventHandler {
    pub fn new() -> EventHandler {
        EventHandler {
            state: EventState::new(),
        }
    }

    pub fn handle_event<T>(&mut self, event: &winit::event::Event<T>) -> Event {
        match event {
            winit::event::Event::WindowEvent {
                event,
                ..
            } => self.handle_windowevent(event),
            _ => Event {
                state: self.state,
                msg: EventMsg::None,
            },
        }
    }

    fn handle_windowevent(&mut self, event: &WindowEvent) -> Event {
        let orig_state = self.state;
        self.state.handle_event(event);
        let msg = match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                EventMsg::MousePress(*button)
            },
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } if !orig_state.drag => {
                EventMsg::Click(*button)
            },
            WindowEvent::CursorMoved { .. } => {
                let delta_x = self.state.mouse_x - orig_state.mouse_x;
                let delta_y = self.state.mouse_y - orig_state.mouse_y;
                EventMsg::MouseMove(delta_x, delta_y)
            },
            WindowEvent::MouseWheel {
                delta: winit::event::MouseScrollDelta::LineDelta(_, d),
                ..
            } => EventMsg::Scroll(*d),
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    virtual_keycode: Some(keycode),
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => EventMsg::KeyPress(*keycode),
            _ => EventMsg::None
        };


        Event {
            state: self.state,
            msg,
        }
    }
}
