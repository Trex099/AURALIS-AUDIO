use uuid::Uuid;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum OrbKind {
    PhysicalSink { description: String },   // e.g. "Sony Headphones"
    ApplicationSource { app_name: String }, // e.g. "Firefox"
    BeamOutput { session_id: String },      // e.g. "Phone Beam"
    Cluster { devices: Vec<String> },       // Merged devices
}

#[derive(Debug, Clone, PartialEq)]
pub enum OrbState {
    Floating,
    Orbiting { parent_id: Uuid },
}

#[derive(Debug, Clone)]
pub struct Orb {
    pub id: Uuid,
    pub pw_id: u32,             // PipeWire Node ID
    pub kind: OrbKind,
    pub name: String,
    pub icon_name: String,
    pub status: String,
    pub state: OrbState,
    // Physics state (mirrored from UI)
    pub position: (f64, f64),
    pub velocity: (f64, f64),
}

#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: Uuid,
    pub master_sink_id: Uuid,
    pub satellites: Vec<Uuid>,
    pub latency_ms: u32,
}

#[derive(Debug, Default)]
pub struct AudioGraph {
    pub orbs: HashMap<Uuid, Orb>,
    pub clusters: HashMap<Uuid, Cluster>,
}

#[derive(Debug, Clone)]
pub enum UiCommand {
    Connect { source: Uuid, target: Uuid },
    Disconnect { source: Uuid, target: Uuid },
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum OrbEvent {
    Add(Orb),
    Remove(Uuid),
}
