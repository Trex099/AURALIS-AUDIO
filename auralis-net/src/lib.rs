pub mod webrtc;
pub mod signaling;

pub fn init() {
    // Initialize GStreamer
    match gstreamer::init() {
        Ok(_) => tracing::info!("GStreamer initialized"),
        Err(e) => tracing::error!("Failed to initialize GStreamer: {}", e),
    }
}
