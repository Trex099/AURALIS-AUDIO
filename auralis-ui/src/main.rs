use gtk4::prelude::*;
use auralis_core::PipeWireClient;
use std::cell::RefCell;
use std::rc::Rc;

pub mod state;
pub mod canvas;
pub mod device_list;
pub mod clusters_view;
pub mod settings_view;
pub mod window;

fn main() {
    tracing_subscriber::fmt::init();
    
    tracing::info!("🚀 [MAIN] Auralis Audio starting...");
    
    // 1. Create Core Channel (MPSC) - Core writes to this
    let (core_tx, core_rx) = std::sync::mpsc::channel();
    tracing::info!("📡 [MAIN] Created Core channel (MPSC)");
    
    // Create a channel for UI events (using async_channel)
    let (ui_tx, ui_rx) = async_channel::unbounded();
    tracing::info!("📡 [MAIN] Created UI channel (async-channel)");

    // Bridge thread: Core (MPSC) -> UI (async-channel)
    std::thread::spawn(move || {
        tracing::info!("🌉 [BRIDGE] Bridge thread started");
        while let Ok(event) = core_rx.recv() {
            // Forward event to UI MainContext
            // send_blocking returns Err if receiver is dropped (app closed)
            if let Err(_) = ui_tx.send_blocking(event) {
                tracing::info!("🌉 [BRIDGE] UI channel closed, stopping bridge");
                break;
            }
        }
        tracing::info!("🌉 [BRIDGE] Core channel closed, stopping bridge");
    });
    
    // Create channel for Commands
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    tracing::info!("📡 [MAIN] Created Command channel: UI → Core");
    
    // Init Core
    tracing::info!("⚙️ [MAIN] Initializing PipeWire Core with cmd_rx...");
    let _client = PipeWireClient::new(core_tx, cmd_rx).expect("Failed to initialize Auralis Core");
    tracing::info!("✓ [MAIN] PipeWire Core initialized");
    
    // We need to move ui_rx into the closure.
    // Since ui_rx is NOT Clone, we wrap it in Rc<RefCell<Option<...>>>
    let rx_holder = Rc::new(RefCell::new(Some(ui_rx)));
    
    // Clone cmd_tx for the signal handler  
    let cmd_tx_sig = cmd_tx.clone();
    
    // Use Arc<AtomicBool> for thread-safe shutdown signaling
    // (GTK objects can't be sent across threads)
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    let shutdown_signal = Arc::new(AtomicBool::new(false));
    let shutdown_signal_ctrlc = shutdown_signal.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        tracing::info!("🛑 [MAIN] Ctrl+C received, sending Shutdown command...");
        let _ = cmd_tx_sig.send(auralis_core::graph::UiCommand::Shutdown);
        
        // Signal shutdown (will be picked up by GTK main loop)
        shutdown_signal_ctrlc.store(true, Ordering::Relaxed);
    }).expect("Error setting Ctrl-C handler");

    let app = libadwaita::Application::builder()
        .application_id("org.example.AuralisAudio")
        .build();

    app.connect_activate(move |app| {
        // Load CSS
        let provider = gtk4::CssProvider::new();
        let style_path_1 = std::path::Path::new("src/style.css");
        let style_path_2 = std::path::Path::new("auralis-ui/src/style.css");
        
        if style_path_1.exists() {
            provider.load_from_path(style_path_1.to_str().unwrap());
        } else if style_path_2.exists() {
            provider.load_from_path(style_path_2.to_str().unwrap());
        } else {
            tracing::warn!("⚠️ [MAIN] Could not find style.css in src/ or auralis-ui/src/");
        }
        
        gtk4::style_context_add_provider_for_display(
            &gtk4::gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

       // Set up shutdown signal polling
        let shutdown_check = shutdown_signal.clone();
        let app_clone = app.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
            if shutdown_check.load(Ordering::Relaxed) {
                tracing::info!("🛑 [SHUTDOWN] Quitting GTK application...");
                app_clone.quit();
                glib::ControlFlow::Break
            } else {
                glib::ControlFlow::Continue
            }
        });

        if let Some(rx) = rx_holder.borrow_mut().take() {
            tracing::info!("🎨 [MAIN] Building UI window...");
            let cmd_tx_for_ui = cmd_tx.clone();
            window::build(app, rx, cmd_tx_for_ui);
            tracing::info!("✓ [MAIN] UI window built and activated");
        } else {
            tracing::warn!("Application activated again, but channel is already consumed");
        }
    });

    app.run();
}
