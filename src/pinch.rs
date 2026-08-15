//! Touchpad pinch-to-zoom.
//!
//! winit has no touchpad-gesture backend on Linux: neither its Wayland nor its
//! X11 code binds anything gesture related, so `WindowEvent::PinchGesture` is
//! macOS-only and `egui::Event::Zoom` never reaches an eframe app here. A bare
//! two-finger pinch is simply invisible to the process.
//!
//! Wayland compositors *do* deliver pinch, through `zwp_pointer_gestures_v1`.
//! This module binds that protocol itself. The trick that makes it work
//! without patching winit is that pointer focus is tracked per *client*, not
//! per `wl_pointer`: a second `wl_seat`/`wl_pointer` created on winit's own
//! connection still receives focus for winit's surfaces, so the pinch gesture
//! object attached to it reports gestures over our window. A separate
//! connection would not work — it owns no surfaces and would never be focused.
//!
//! The protocol runs on its own event queue on a dedicated thread. The queue
//! is independent of winit's, and libwayland serialises reads across queues,
//! so the two coexist without winit knowing about it.
//!
//! X11 has no gesture protocol at all, so pinch stays unavailable there and
//! `PinchGestures` degrades to a no-op that always reports "no zoom".

/// Accumulated pinch input, drained once per frame by the app.
pub struct PinchGestures {
    #[cfg(all(unix, not(target_os = "macos")))]
    shared: Option<std::sync::Arc<imp::Shared>>,
}

impl PinchGestures {
    /// Start listening for pinch gestures on `display_handle`'s connection.
    ///
    /// Returns an inert listener when the platform, the compositor, or the
    /// windowing backend does not support the gesture protocol. Pinch is a
    /// convenience gesture, so every failure here is logged and swallowed
    /// rather than surfaced to the user.
    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn new(
        display_handle: Option<raw_window_handle::RawDisplayHandle>,
        ctx: &egui::Context,
    ) -> Self {
        let Some(raw_window_handle::RawDisplayHandle::Wayland(wayland)) = display_handle else {
            log::debug!("touchpad pinch: not a Wayland session, gesture support disabled");
            return Self { shared: None };
        };
        match imp::spawn(wayland.display.as_ptr(), ctx.clone()) {
            Ok(shared) => {
                log::info!("touchpad pinch: zwp_pointer_gestures_v1 bound");
                Self {
                    shared: Some(shared),
                }
            }
            Err(err) => {
                log::warn!("touchpad pinch: unavailable ({err})");
                Self { shared: None }
            }
        }
    }

    #[cfg(not(all(unix, not(target_os = "macos"))))]
    pub fn new(
        _display_handle: Option<raw_window_handle::RawDisplayHandle>,
        _ctx: &egui::Context,
    ) -> Self {
        Self {}
    }

    /// Take the multiplicative zoom accumulated since the last call.
    ///
    /// Returns `1.0` when no pinch happened, so callers can use it
    /// unconditionally.
    pub fn take_zoom_factor(&self) -> f32 {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            if let Some(shared) = &self.shared {
                return shared.take();
            }
        }
        1.0
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
mod imp {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    use wayland_client::backend::Backend;
    use wayland_client::globals::{registry_queue_init, GlobalList};
    use wayland_client::protocol::{wl_pointer, wl_registry, wl_seat};
    use wayland_client::{Connection, Dispatch, QueueHandle};
    use wayland_protocols::wp::pointer_gestures::zv1::client::{
        zwp_pointer_gesture_pinch_v1, zwp_pointer_gestures_v1,
    };

    /// Pinch factor shared between the Wayland thread and the UI thread.
    ///
    /// Stored as the bits of an `f32` in an atomic so the UI thread never
    /// blocks on the gesture thread mid-frame.
    pub struct Shared {
        factor_bits: AtomicU32,
    }

    impl Shared {
        fn new() -> Self {
            Self {
                factor_bits: AtomicU32::new(1.0f32.to_bits()),
            }
        }

        /// Fold another multiplicative step into the pending factor.
        fn accumulate(&self, step: f32) {
            if !step.is_finite() || step <= 0.0 {
                return;
            }
            let mut current = self.factor_bits.load(Ordering::Relaxed);
            loop {
                let next = (f32::from_bits(current) * step).to_bits();
                match self.factor_bits.compare_exchange_weak(
                    current,
                    next,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => return,
                    Err(observed) => current = observed,
                }
            }
        }

        /// Read and reset the pending factor.
        pub fn take(&self) -> f32 {
            let bits = self.factor_bits.swap(1.0f32.to_bits(), Ordering::Relaxed);
            let factor = f32::from_bits(bits);
            if factor.is_finite() && factor > 0.0 {
                factor
            } else {
                1.0
            }
        }
    }

    struct State {
        shared: Arc<Shared>,
        ctx: egui::Context,
        /// Kept so the seat's capabilities callback can attach a pinch object
        /// to the pointer it creates.
        gestures: zwp_pointer_gestures_v1::ZwpPointerGesturesV1,
        /// Absolute scale reported at the previous `update`, so consecutive
        /// updates can be turned into multiplicative steps. The protocol
        /// reports scale relative to the start of the gesture, not to the
        /// previous event.
        last_scale: f64,
        in_gesture: bool,
    }

    /// Bind the gesture protocol on `display` and dispatch it on a dedicated
    /// thread. `display` must be the `wl_display` the windowing backend is
    /// already using, and must outlive the process's UI.
    pub fn spawn(
        display: *mut std::ffi::c_void,
        ctx: egui::Context,
    ) -> Result<Arc<Shared>, String> {
        // SAFETY: `display` comes from the window backend's live display
        // handle. `from_foreign_display` borrows it without taking ownership,
        // so the backend keeps its usual control over the connection.
        let backend = unsafe { Backend::from_foreign_display(display.cast()) };
        let conn = Connection::from_backend(backend);

        let (globals, mut queue): (GlobalList, _) =
            registry_queue_init::<State>(&conn).map_err(|e| format!("registry init: {e}"))?;
        let qh = queue.handle();

        // Version 1 is enough: `pinch.update` carries `scale` from the start.
        let gestures: zwp_pointer_gestures_v1::ZwpPointerGesturesV1 = globals
            .bind(&qh, 1..=3, ())
            .map_err(|e| format!("zwp_pointer_gestures_v1 not offered: {e}"))?;
        let seat: wl_seat::WlSeat = globals
            .bind(&qh, 1..=9, ())
            .map_err(|e| format!("wl_seat: {e}"))?;

        let shared = Arc::new(Shared::new());
        let mut state = State {
            shared: Arc::clone(&shared),
            ctx,
            gestures,
            last_scale: 1.0,
            in_gesture: false,
        };

        // The pinch object is created from the seat's capabilities callback,
        // which arrives on this roundtrip.
        queue
            .roundtrip(&mut state)
            .map_err(|e| format!("seat roundtrip: {e}"))?;

        std::thread::Builder::new()
            .name("mkdv-pinch".to_owned())
            .spawn(move || {
                // Holding the seat keeps the pointer (and with it the pinch
                // object) alive for as long as we dispatch.
                let _keep_alive = seat;
                loop {
                    if let Err(err) = queue.blocking_dispatch(&mut state) {
                        log::debug!("touchpad pinch: dispatch stopped ({err})");
                        return;
                    }
                }
            })
            .map_err(|e| format!("thread spawn: {e}"))?;

        Ok(shared)
    }

    impl Dispatch<wl_seat::WlSeat, ()> for State {
        fn event(
            state: &mut Self,
            seat: &wl_seat::WlSeat,
            event: wl_seat::Event,
            _: &(),
            _: &Connection,
            qh: &QueueHandle<Self>,
        ) {
            let wl_seat::Event::Capabilities {
                capabilities: wayland_client::WEnum::Value(caps),
            } = event
            else {
                return;
            };
            if !caps.contains(wl_seat::Capability::Pointer) {
                return;
            }
            // Creating our own pointer is what gives the gesture object a
            // focus to follow; see the module docs.
            let pointer = seat.get_pointer(qh, ());
            state.gestures.get_pinch_gesture(&pointer, qh, ());
        }
    }

    impl Dispatch<zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1, ()> for State {
        fn event(
            state: &mut Self,
            _: &zwp_pointer_gesture_pinch_v1::ZwpPointerGesturePinchV1,
            event: zwp_pointer_gesture_pinch_v1::Event,
            _: &(),
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
            use zwp_pointer_gesture_pinch_v1::Event;
            match event {
                Event::Begin { .. } => {
                    state.last_scale = 1.0;
                    state.in_gesture = true;
                }
                Event::Update { scale, .. } => {
                    if !state.in_gesture || !scale.is_finite() || scale <= 0.0 {
                        return;
                    }
                    let step = scale / state.last_scale;
                    state.last_scale = scale;
                    state.shared.accumulate(step as f32);
                    // The UI thread is idle between gestures, so it has to be
                    // told that there is new input to consume.
                    state.ctx.request_repaint();
                }
                Event::End { .. } => {
                    state.in_gesture = false;
                    state.last_scale = 1.0;
                }
                _ => {}
            }
        }
    }

    // The remaining objects are bound only so the compositor keeps sending us
    // gesture events; their own events carry nothing we act on.
    macro_rules! ignore_events {
        ($($iface:ty),* $(,)?) => {
            $(impl Dispatch<$iface, ()> for State {
                fn event(
                    _: &mut Self,
                    _: &$iface,
                    _: <$iface as wayland_client::Proxy>::Event,
                    _: &(),
                    _: &Connection,
                    _: &QueueHandle<Self>,
                ) {
                }
            })*
        };
    }

    ignore_events!(
        wl_pointer::WlPointer,
        zwp_pointer_gestures_v1::ZwpPointerGesturesV1,
    );

    impl Dispatch<wl_registry::WlRegistry, wayland_client::globals::GlobalListContents> for State {
        fn event(
            _: &mut Self,
            _: &wl_registry::WlRegistry,
            _: wl_registry::Event,
            _: &wayland_client::globals::GlobalListContents,
            _: &Connection,
            _: &QueueHandle<Self>,
        ) {
        }
    }
}
