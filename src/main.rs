mod compose;
mod keyboard;
mod ui;

use compose::ComposeIndex;
use keyboard::XkbManager;
use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_registry,
    delegate_seat, delegate_shm,
    output::{OutputHandler, OutputState},
    reexports::{
        calloop::{EventLoop, LoopSignal},
        calloop_wayland_source::WaylandSource,
        client::{
            globals::registry_queue_init,
            protocol::{wl_keyboard, wl_output, wl_seat, wl_surface},
            Connection, QueueHandle,
        },
    },
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Modifiers},
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
const WINDOW_HEIGHT: u32 = 400;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Starting kbdviz character reference tool...");

    // Initialize XKB and build compose index
    let xkb_manager = XkbManager::new()?;
    let compose_index = Arc::new(ComposeIndex::build(&xkb_manager)?);
    eprintln!("Loaded {} base characters with variants", compose_index.count());

    let (mut app, mut event_loop) = App::new(compose_index)?;

    eprintln!("Layer surface created, starting event loop...");
    event_loop.run(None, &mut app, |_| {})?;

    Ok(())
}

struct App {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    compositor_state: CompositorState,
    layer_shell: LayerShell,
    shm: Shm,
    loop_signal: LoopSignal,

    layer_surface: Option<LayerSurface>,
    configured: bool,

    ui: Option<CharRefUI>,
    compose_index: Arc<ComposeIndex>,
}

impl App {
    fn new(compose_index: Arc<ComposeIndex>) -> Result<(Self, EventLoop<'static, Self>), Box<dyn std::error::Error>> {
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
            compositor_state,
            layer_shell,
            shm,
            loop_signal,
            layer_surface: Some(layer_surface),
            configured: false,
            ui: None,
            compose_index,
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

    fn configure(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &LayerSurface, configure: LayerSurfaceConfigure, _: u32) {
        if !self.configured {
            eprintln!("Layer surface configured: {}x{}", configure.new_size.0, configure.new_size.1);

            if let Some(ref layer_surface) = self.layer_surface {
                self.ui = Some(CharRefUI::new(
                    layer_surface.wl_surface(),
                    configure.new_size.0,
                    configure.new_size.1,
                    &self.shm,
                    self.compose_index.clone(),
                ));
            }

            self.configured = true;
            self.render();
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
    }
    fn remove_capability(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat, _: Capability) {}
    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for App {
    fn enter(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32, _: &[u32], _: &[xkbcommon::xkb::Keysym]) {}
    fn leave(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: &wl_surface::WlSurface, _: u32) {}

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
delegate_layer!(App);
delegate_shm!(App);
delegate_registry!(App);
