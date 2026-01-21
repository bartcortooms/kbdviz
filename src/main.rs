mod compose;
mod keyboard;
mod ui;

use compose::ComposeIndex;
use keyboard::XkbKeymap;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
    delegate_registry, delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{EventLoop, LoopSignal},
        calloop_wayland_source::WaylandSource,
        client::{
            globals::registry_queue_init,
            protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_surface},
            Connection, QueueHandle,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Keymap, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        wlr_layer::{
            Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
            LayerSurfaceConfigure,
        },
        WaylandSurface,
    },
    shm::{Shm, ShmHandler},
};
use std::sync::Arc;
use ui::CharRefUI;

const WINDOW_WIDTH: u32 = 280;
const WINDOW_HEIGHT: u32 = 420;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting kbdviz character reference tool...");

    let (mut app, mut event_loop) = App::new()?;

    eprintln!("Layer surface created, starting event loop...");
    eprintln!("Waiting for keymap from compositor...");
    event_loop.run(None, &mut app, |_| {})?;

    Ok(())
}

struct App {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    _compositor_state: CompositorState,
    _layer_shell: LayerShell,
    shm: Shm,
    loop_signal: LoopSignal,

    layer_surface: Option<LayerSurface>,
    configured: bool,

    ui: Option<CharRefUI>,
    compose_index: Option<Arc<ComposeIndex>>,  // None until we receive keymap from compositor
}

impl App {
    fn new() -> Result<(Self, EventLoop<'static, Self>), Box<dyn std::error::Error>> {
        let conn = Connection::connect_to_env()?;
        let (globals, event_queue) = registry_queue_init::<Self>(&conn)?;
        let qh: QueueHandle<Self> = event_queue.handle();

        let registry_state = RegistryState::new(&globals);
        let seat_state = SeatState::new(&globals, &qh);
        let output_state = OutputState::new(&globals, &qh);
        let compositor_state = CompositorState::bind(&globals, &qh)?;
        let layer_shell = LayerShell::bind(&globals, &qh)?;
        let shm = Shm::bind(&globals, &qh)?;

        // Create layer surface
        let surface = compositor_state.create_surface(&qh);
        let layer_surface = layer_shell.create_layer_surface(
            &qh,
            surface,
            Layer::Overlay,
            Some("kbdviz"),
            None,
        );

        layer_surface.set_anchor(Anchor::empty());  // Centered
        layer_surface.set_exclusive_zone(0);
        layer_surface.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        layer_surface.set_size(WINDOW_WIDTH, WINDOW_HEIGHT);
        layer_surface.commit();

        eprintln!("Layer surface created. Press ESC or click to close.");

        let event_loop: EventLoop<Self> = EventLoop::try_new()?;
        let loop_signal = event_loop.get_signal();

        WaylandSource::new(conn, event_queue).insert(event_loop.handle())?;

        let app = Self {
            registry_state,
            seat_state,
            output_state,
            _compositor_state: compositor_state,
            _layer_shell: layer_shell,
            shm,
            loop_signal,
            layer_surface: Some(layer_surface),
            configured: false,
            ui: None,
            compose_index: None,  // Will be populated when we receive keymap
        };

        Ok((app, event_loop))
    }

    fn exit(&self) {
        self.loop_signal.stop();
    }

    fn render(&mut self) {
        if !self.configured {
            return;
        }

        if let (Some(ref layer_surface), Some(ref mut ui)) = (&self.layer_surface, &mut self.ui) {
            ui.render();
            layer_surface.wl_surface().commit();
        }
    }

    /// Try to create the UI - requires both surface configured and keymap received
    fn try_create_ui(&mut self) {
        if !self.configured || self.compose_index.is_none() || self.ui.is_some() {
            return;
        }

        if let Some(ref layer_surface) = self.layer_surface {
            let surface = layer_surface.wl_surface();
            // Get the configured size from the layer surface
            self.ui = Some(CharRefUI::new(
                surface,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
                &self.shm,
                self.compose_index.clone().unwrap(),
            ));
            self.render();
        }
    }
}

impl CompositorHandler for App {
    fn scale_factor_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: i32) {}
    fn frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: u32) {}
    fn transform_changed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: wl_output::Transform) {}
    fn surface_enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
    fn surface_leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_surface::WlSurface, _: &wl_output::WlOutput) {}
}

impl OutputHandler for App {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_output::WlOutput) {}
}

impl LayerShellHandler for App {
    fn closed(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &LayerSurface) {
        self.exit();
    }

    fn configure(&mut self, _: &Connection, _qh: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        if !self.configured {
            eprintln!("Layer surface configured: {}x{}", configure.new_size.0, configure.new_size.1);
            self.configured = true;
            self.try_create_ui();
        }
    }
}

impl SeatHandler for App {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
    fn new_capability(&mut self, _: &Connection, qh: &QueueHandle<Self>, seat: wl_seat::WlSeat, capability: Capability) {
        if capability == Capability::Keyboard {
            let _ = self.seat_state.get_keyboard(qh, &seat, None);
        }
        if capability == Capability::Pointer {
            let _ = self.seat_state.get_pointer(qh, &seat);
        }
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for App {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[xkbcommon::xkb::Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}

    fn update_keymap(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, keymap: Keymap<'_>) {
        eprintln!("Received keymap from compositor");

        // Get the keymap as a string and create our XkbKeymap
        let keymap_string = keymap.as_string();
        match XkbKeymap::from_string(&keymap_string) {
            Ok(xkb_keymap) => {
                // Build the compose index from the actual keymap
                match ComposeIndex::build(&xkb_keymap) {
                    Ok(index) => {
                        eprintln!("Loaded {} base characters with variants", index.count());
                        self.compose_index = Some(Arc::new(index));
                        self.try_create_ui();
                    }
                    Err(e) => eprintln!("Failed to build compose index: {}", e),
                }
            }
            Err(e) => eprintln!("Failed to parse keymap: {}", e),
        }
    }

    fn press_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        eprintln!("Key pressed: code={}, keysym={:?}", event.raw_code, event.keysym);

        // ESC to exit (keycode 1 on this system!)
        if event.raw_code == 1 {  // ESC
            eprintln!("ESC pressed, closing...");
            self.exit();
            return;
        }

        // Handle text input
        if let Some(ref mut ui) = self.ui {
            ui.handle_key_press(event.raw_code, event.keysym);
            self.render();
        }
    }

    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: KeyEvent) {}
    fn update_modifiers(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _: Modifiers, _: u32) {}
}

impl PointerHandler for App {
    fn pointer_frame(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_pointer::WlPointer, events: &[PointerEvent]) {
        for event in events {
            match event.kind {
                PointerEventKind::Press { button, .. } => {
                    // Left mouse button = 272 (BTN_LEFT)
                    if button == 272 {
                        if let Some(ref mut ui) = self.ui {
                            if let Some(character) = ui.handle_click(event.position.0, event.position.1) {
                                // Copy to clipboard using wl-copy
                                if let Err(e) = std::process::Command::new("wl-copy")
                                    .arg(&character)
                                    .spawn()
                                {
                                    eprintln!("Failed to copy to clipboard: {}", e);
                                } else {
                                    eprintln!("Copied '{}' to clipboard", character);
                                }
                                self.render();
                            }
                        }
                    }
                }
                PointerEventKind::Motion { .. } => {
                    // Handle hover state
                    if let Some(ref mut ui) = self.ui {
                        if ui.handle_mouse_move(event.position.0, event.position.1) {
                            // Hover state changed, re-render
                            self.render();
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl ProvidesRegistryState for App {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState];
}

impl ShmHandler for App {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

delegate_compositor!(App);
delegate_output!(App);
delegate_seat!(App);
delegate_keyboard!(App);
delegate_pointer!(App);
delegate_layer!(App);
delegate_shm!(App);
delegate_registry!(App);
