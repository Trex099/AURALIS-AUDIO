use anyhow::Result;
use pipewire as pw;
use std::thread;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tracing::{info, error, warn};
use crate::graph::{Orb, OrbKind, OrbState, UiCommand, OrbEvent};
use uuid::Uuid;

/// Shared state for tracking Orbs and PipeWire nodes
#[derive(Clone)]
struct SharedState {
    orb_to_pw_id: Arc<Mutex<HashMap<Uuid, u32>>>,
    pw_id_to_orb: Arc<Mutex<HashMap<u32, Uuid>>>,
    orb_names: Arc<Mutex<HashMap<Uuid, String>>>,
    orb_kinds: Arc<Mutex<HashMap<Uuid, OrbKind>>>,
    combine_modules: Arc<Mutex<HashMap<Uuid, u32>>>, // Track combine-sink module IDs for cleanup (ClusterID -> ModuleID)
    active_cluster_members: Arc<Mutex<HashMap<String, String>>>, // Description -> NodeName mapping
    hidden_cluster_members: Arc<Mutex<HashMap<String, u32>>>, // Name -> PW_ID of ignored devices
    mock_modules: Arc<Mutex<Vec<u32>>>, // Track mock device module IDs
    saved_default_sink: Arc<Mutex<HashMap<Uuid, String>>>, // ClusterID -> Original Default Sink
}

impl SharedState {
    fn new() -> Self {
        Self {
            orb_to_pw_id: Arc::new(Mutex::new(HashMap::new())),
            pw_id_to_orb: Arc::new(Mutex::new(HashMap::new())),
            orb_names: Arc::new(Mutex::new(HashMap::new())),
            orb_kinds: Arc::new(Mutex::new(HashMap::new())),
            combine_modules: Arc::new(Mutex::new(HashMap::new())),
            active_cluster_members: Arc::new(Mutex::new(HashMap::new())),
            hidden_cluster_members: Arc::new(Mutex::new(HashMap::new())),
            mock_modules: Arc::new(Mutex::new(Vec::new())),
            saved_default_sink: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn register_orb(&self, orb_id: Uuid, pw_id: u32, name: String, kind: OrbKind) {
        self.orb_to_pw_id.lock().unwrap().insert(orb_id, pw_id);
        self.pw_id_to_orb.lock().unwrap().insert(pw_id, orb_id);
        self.orb_names.lock().unwrap().insert(orb_id, name);
        self.orb_kinds.lock().unwrap().insert(orb_id, kind);
    }
    
    fn cleanup_combine_sinks(&self) {
        let modules = self.combine_modules.lock().unwrap();
        info!("Cleaning up {} combine-sinks", modules.len());
        for module_id in modules.values() {
            let _ = std::process::Command::new("pactl")
                .args(&["unload-module", &module_id.to_string()])
                .output();
        }
        
        // Also cleanup mocks
        let mocks = self.mock_modules.lock().unwrap();
        if !mocks.is_empty() {
            info!("Cleaning up {} mock devices", mocks.len());
            for module_id in mocks.iter() {
                let _ = std::process::Command::new("pactl")
                    .args(&["unload-module", &module_id.to_string()])
                    .output();
            }
        }
    }
    
    fn is_cluster_member(&self, name: &str) -> bool {
        self.active_cluster_members.lock().unwrap().contains_key(name)
    }
    
    fn add_cluster_members(&self, members: Vec<(String, String)>) {
        let mut map = self.active_cluster_members.lock().unwrap();
        for (desc, node_name) in members {
            map.insert(desc, node_name);
        }
    }
    
    fn remove_cluster_members(&self, names: &Vec<String>) {
        let mut map = self.active_cluster_members.lock().unwrap();
        for name in names {
            map.remove(name);
        }
    }
}

pub struct PipeWireClient {
    _thread: thread::JoinHandle<()>,
    _cmd_thread: thread::JoinHandle<()>,
    _command_pool: threadpool::ThreadPool,
}


impl PipeWireClient {
    fn cleanup_stale_modules() {
        info!("🧹 [STARTUP] Checking for stale Auralis modules...");
        let output = std::process::Command::new("pactl")
            .args(&["list", "modules", "short"])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let mut count = 0;
            
            for line in stdout.lines() {
                // Line format: "536870932 module-combine-sink ..."
                if (line.contains("module-combine-sink") && (line.contains("sink_name=auralis_combined_") || line.contains("sink_name=auralis_cluster_"))) ||
                   (line.contains("module-null-sink") && line.contains("sink_name=Mock")) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if let Some(id_str) = parts.first() {
                        info!("Found stale module: {}", line);
                        let unload = std::process::Command::new("pactl")
                            .args(&["unload-module", id_str])
                            .output();
                            
                        match unload {
                            Ok(_) => {
                                info!("✓ Unloaded stale module {}", id_str);
                                count += 1;
                            }
                            Err(e) => error!("Failed to unload module {}: {}", id_str, e),
                        }
                    }
                }
            }
            if count > 0 {
                info!("✓ [STARTUP] Cleaned up {} stale modules", count);
            } else {
                info!("✓ [STARTUP] No stale modules found");
            }
        } else {
            error!("Failed to list modules for cleanup");
        }
    }

    fn spawn_mock_devices(state: &SharedState) {
        let mocks: Vec<(&str, &str)> = vec![
            // ("Mock1", "Living_Room"),
            // ("Mock2", "Kitchen"),
            // ("Mock3", "Patio"),
            // ("Mock4", "Bedroom"),
            // ("Mock5", "Office"),
        ];

        info!("🛠️ [MOCK] Spawning {} mock devices...", mocks.len());

        for (name, desc) in mocks {
            let output = std::process::Command::new("pactl")
                .args(&[
                    "load-module",
                    "module-null-sink",
                    &format!("sink_name={}", name),
                    &format!("sink_properties=device.description={}", desc),
                ])
                .output();
            
            match output {
                Ok(out) => {
                    let id_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if let Ok(id) = id_str.parse::<u32>() {
                        state.mock_modules.lock().unwrap().push(id);
                        info!("  ✓ Created mock: {} (ID: {})", desc, id);
                    } else {
                        warn!("  ⚠ Failed to parse ID for mock {}", desc);
                    }
                }
                Err(e) => error!("  ❌ Failed to create mock {}: {}", desc, e),
            }
        }
    }

    pub fn new(sender: Sender<OrbEvent>, receiver: Receiver<UiCommand>) -> Result<Self> {
        // Cleanup before anything else
        Self::cleanup_stale_modules();

        pw::init();

        // Shared state
        let state = SharedState::new();
        
        // Spawn mocks
        Self::spawn_mock_devices(&state);
        
        let state_discovery = state.clone();
        let state_commands = state.clone();
        
        // Clone sender for command thread
        let sender_commands = sender.clone();

        let thread = thread::spawn(move || {
            let mainloop = match pw::main_loop::MainLoop::new(None) {
                Ok(ml) => ml,
                Err(e) => {
                    error!("Failed to create MainLoop: {}", e);
                    return;
                }
            };
            
            let context = match pw::context::Context::new(&mainloop) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to create Context: {}", e);
                    return;
                }
            };
            
            let core = match context.connect(None) {
                Ok(c) => c,
                Err(e) => {
                    error!("Failed to connect to Core: {}", e);
                    return;
                }
            };
            
            let registry = match core.get_registry() {
                Ok(r) => r,
                Err(e) => {
                    error!("Failed to get Registry: {}", e);
                    return;
                }
            };

            let state_remove = state_discovery.clone();
            let sender_remove = sender.clone();
            let _listener = registry
                .add_listener_local()
                .global(move |global| {
                    if let Some(props) = global.props {
                        // Filter for Audio Sinks and Sink Inputs (Streams)
                        let is_sink = props.get("media.class").map(|s| s == "Audio/Sink").unwrap_or(false);
                        let is_stream = props.get("media.class").map(|s| s == "Stream/Output/Audio").unwrap_or(false);
                        
                        if is_sink || is_stream {
                            let name = props.get("node.name").unwrap_or("Unknown");
                            let description = props.get("node.description").unwrap_or(name);
                            let app_name = props.get("application.name").unwrap_or("");
                            
                            // Check if this device is part of an active cluster
                            if is_sink && state_discovery.is_cluster_member(description) {
                                info!("Parking hidden cluster member: {} [ID: {}]", description, global.id);
                                state_discovery.hidden_cluster_members.lock().unwrap().insert(description.to_string(), global.id);
                                return;
                            }
                            
                            // Filter out Mutter (System Sounds/Compositor) and Dummy devices
                            if app_name == "Mutter" || name.contains("Mutter") || name.to_lowercase().contains("dummy") {
                                return;
                            }

                            info!("Found Orb: {} ({}) [ID: {}]", description, props.get("media.class").unwrap_or("?"), global.id);

                            let kind = if is_sink {
                                if name.starts_with("auralis_combined_") || name.starts_with("auralis_cluster_") {
                                    return; 
                                }
                                OrbKind::PhysicalSink { description: description.to_string() }
                            } else {
                                OrbKind::ApplicationSource { app_name: app_name.to_string() }
                            };

                            let id = Uuid::new_v4();
                            
                            // Register in shared state
                            state_discovery.register_orb(id, global.id, name.to_string(), kind.clone());

                            let orb = Orb {
                                id,
                                pw_id: global.id,
                                kind,
                                name: if !app_name.is_empty() { app_name.to_string() } else { description.to_string() },
                                icon_name: if is_sink { "audio-card".to_string() } else { "audio-x-generic".to_string() },
                                status: "Active".to_string(), // Default to Active for now
                                state: OrbState::Floating,
                                position: (0.0, 0.0),
                                velocity: (0.0, 0.0),
                            };

                            let _ = sender.send(OrbEvent::Add(orb));
                        }
                    }
                })
                .register();
                
            let _remove_listener = registry
                .add_listener_local()
                .global_remove(move |id| {
                    info!("Global removed: {}", id);
                    // Find Orb ID by PipeWire ID
                    let orb_id = {
                        let map = state_remove.pw_id_to_orb.lock().unwrap();
                        map.get(&id).cloned()
                    };

                    if let Some(uuid) = orb_id {
                        info!("✓ Found Orb for PW_ID {}: {}", id, uuid);
                        
                        // Remove from hidden members if present
                        state_remove.hidden_cluster_members.lock().unwrap().retain(|_, &mut v| v != id);

                        // Check if this was a cluster member
                        // We must use OrbKind to get the friendly description, as that's what we track in active_cluster_members
                        let kind_opt = state_remove.orb_kinds.lock().unwrap().get(&uuid).cloned();
                        
                        if let Some(OrbKind::PhysicalSink { description: name }) = kind_opt {
                            if state_remove.is_cluster_member(&name) {
                                info!("💔 Cluster member disconnected: {}", name);
                                
                                // Find the cluster this device belonged to
                                let cluster_info = {
                                    let kinds = state_remove.orb_kinds.lock().unwrap();
                                    kinds.iter().find_map(|(cid, kind)| {
                                        if let OrbKind::Cluster { devices } = kind {
                                            if devices.contains(&name) {
                                                return Some((*cid, devices.clone()));
                                            }
                                        }
                                        None
                                    })
                                };

                                if let Some((cluster_id, devices)) = cluster_info {
                                    info!("💥 Dissolving cluster {} due to member loss", cluster_id);
                                    
                                    // 1. Unload combine-sinks (Clean up system state)
                                    state_remove.cleanup_combine_sinks();
                                    
                                    // 2. Remove Cluster Orb from UI
                                    let _ = sender_remove.send(OrbEvent::Remove(cluster_id));
                                    
                                    // 3. Remove Cluster Orb from State
                                    {
                                        let mut kinds = state_remove.orb_kinds.lock().unwrap();
                                        kinds.remove(&cluster_id);
                                        state_remove.orb_names.lock().unwrap().remove(&cluster_id);
                                    }

                                    // 4. Update active members list
                                    state_remove.remove_cluster_members(&devices);
                                    
                                    // 5. Restore OTHER devices
                                    for dev_name in devices {
                                        if dev_name == name { continue; } // Don't restore the dying device
                                        
                                        info!("♻️ Restoring survivor: {}", dev_name);
                                        
                                        // Check if we have a "parked" new instance of this device
                                        let parked_id = state_remove.hidden_cluster_members.lock().unwrap().get(&dev_name).cloned();
                                        
                                        if let Some(new_pw_id) = parked_id {
                                            info!("✓ Found parked survivor {} with new ID {}", dev_name, new_pw_id);
                                            // Create NEW Orb for this survivor
                                            let new_uuid = Uuid::new_v4();
                                            
                                            // Register in shared state
                                            state_remove.register_orb(new_uuid, new_pw_id, dev_name.clone(), OrbKind::PhysicalSink { description: dev_name.clone() });
                                            
                                            let orb = Orb {
                                                id: new_uuid,
                                                pw_id: new_pw_id,
                                                kind: OrbKind::PhysicalSink { description: dev_name.clone() },
                                                name: dev_name.clone(),
                                                icon_name: "audio-card".to_string(),
                                                status: "Active".to_string(),
                                                state: OrbState::Floating,
                                                position: (0.0, 0.0),
                                                velocity: (0.0, 0.0),
                                            };
                                            let _ = sender_remove.send(OrbEvent::Add(orb));
                                            
                                            // Remove from hidden members as it's now active
                                            state_remove.hidden_cluster_members.lock().unwrap().remove(&dev_name);
                                            
                                        } else {
                                            // Fallback: Try to restore old one (if it wasn't removed)
                                            // Find UUID/Details of the survivor
                                            // CRITICAL FIX: Search orb_kinds for PhysicalSink, NOT just orb_names
                                            // This prevents matching the combine-sink output stream which might share the same name
                                            let survivor_uuid = {
                                                let kinds = state_remove.orb_kinds.lock().unwrap();
                                                kinds.iter().find_map(|(u, k)| {
                                                    if let OrbKind::PhysicalSink { description } = k {
                                                        if description == &dev_name {
                                                            return Some(*u);
                                                        }
                                                    }
                                                    None
                                                })
                                            };
                                            
                                            if let Some(suid) = survivor_uuid {
                                                // Only restore if it still exists in the maps (wasn't removed)
                                                let (spw_id, skind) = {
                                                    let pw_map = state_remove.orb_to_pw_id.lock().unwrap();
                                                    let kind_map = state_remove.orb_kinds.lock().unwrap();
                                                    (pw_map.get(&suid).cloned(), kind_map.get(&suid).cloned())
                                                };
                                                
                                                if let (Some(spw_id), Some(skind)) = (spw_id, skind) {
                                                    info!("✓ Restoring existing survivor {} (ID: {})", dev_name, spw_id);
                                                    let orb = Orb {
                                                        id: suid,
                                                        pw_id: spw_id,
                                                        kind: skind,
                                                        name: dev_name.clone(),
                                                        icon_name: "audio-card".to_string(),
                                                        status: "Active".to_string(),
                                                        state: OrbState::Floating,
                                                        position: (0.0, 0.0),
                                                        velocity: (0.0, 0.0),
                                                    };
                                                    let _ = sender_remove.send(OrbEvent::Add(orb));
                                                } else {
                                                    info!("⚠ Survivor {} not found in state (likely removed)", dev_name);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Remove from shared state
                        {
                            state_remove.orb_to_pw_id.lock().unwrap().remove(&uuid);
                            state_remove.pw_id_to_orb.lock().unwrap().remove(&id);
                            state_remove.orb_names.lock().unwrap().remove(&uuid);
                            state_remove.orb_kinds.lock().unwrap().remove(&uuid);
                        }
                        
                        // Notify UI
                        let _ = sender_remove.send(OrbEvent::Remove(uuid));
                    } else {
                        // info!("Ignored removal of unknown global: {}", id);
                    }
                })
                .register();

            info!("Starting PipeWire main loop");
            mainloop.run();
        });
        
        // Create thread pool for command handlers (max 10 concurrent)
        let pool = threadpool::ThreadPool::new(10);
        let pool_for_thread = pool.clone();
        
        // Command handling thread
        let state_for_thread = state_commands.clone();
        let cmd_thread = thread::spawn(move || {
            info!("🟢 [CORE-THREAD] Command receiver thread STARTED and waiting for commands");
            info!("🔍 [CORE-THREAD] Thread ID: {:?}", thread::current().id());
            
            let mut cmd_count = 0;
            loop {
                // Add periodic health check
                if cmd_count % 10 == 0 && cmd_count > 0 {
                    info!("💓 [CORE-HEALTH] Receiver thread alive, processed {} commands so far", cmd_count);
                }
                
                match receiver.recv() {
                    Ok(cmd) => {
                        cmd_count += 1;
                        info!("📨 [CORE-RECV] Command #{} received: {:?}", cmd_count, cmd);
                        
                        // Execute handler in thread pool (bounded to 10 workers)
                        let state_clone = state_for_thread.clone();
                        let sender_clone = sender_commands.clone();
                        
                        pool_for_thread.execute(move || {
                            match cmd {
                                UiCommand::Connect { source, target } => {
                                    info!("🔗 [CORE-EXEC] Executing Connect: {} -> {}", source, target);
                                    Self::handle_connect(&state_clone, &sender_clone, source, target);
                                    info!("✓ [CORE-DONE] Connect command completed");
                                }
                                UiCommand::Disconnect { source, target } => {
                                    info!("🔴 [CORE-RECV] Disconnect command received: {} -> {}", source, target);
                                    info!("🔧 [CORE-EXEC] Executing Disconnect handler");
                                    Self::handle_disconnect(&state_clone, &sender_clone, source, target);
                                    info!("✓ [CORE-DONE] Disconnect command completed");
                                }
                                UiCommand::Shutdown => {
                                    info!("🛑 [CORE-RECV] Shutdown command received");
                                    state_clone.cleanup_combine_sinks();
                                    info!("✓ [CORE-DONE] Cleanup complete, exiting thread");
                                }
                            }
                        });
                    }
                    Err(e) => {
                        error!("💀 [CORE-ERROR] Command receiver channel closed: {}", e);
                        error!("💀 [CORE-EXIT] Receiver thread terminating after {} commands", cmd_count);
                        break;
                    }
                }
            }
        });

        Ok(Self {
            _thread: thread,
            _cmd_thread: cmd_thread,
            _command_pool: pool,
        })
    }

    fn handle_connect(state: &SharedState, sender: &Sender<OrbEvent>, source: Uuid, target: Uuid) {
        let src_kind;
        let tgt_kind;
        let src_name;
        let tgt_name;
        
        {
            let names = state.orb_names.lock().unwrap();
            let kinds = state.orb_kinds.lock().unwrap();
            
            src_kind = kinds.get(&source).cloned();
            tgt_kind = kinds.get(&target).cloned();
            
            src_name = names.get(&source).map(|s| s.to_string()).unwrap_or_else(|| "Unknown".to_string());
            tgt_name = names.get(&target).map(|s| s.to_string()).unwrap_or_else(|| "Unknown".to_string());
        } // Locks dropped here!

        match (src_kind, tgt_kind) {
            // Case 1: Sink + Sink = New Cluster
            (Some(OrbKind::PhysicalSink { description: desc1 }), Some(OrbKind::PhysicalSink { description: desc2 })) => {
                info!("Creating cluster for {} + {}", src_name, tgt_name);
                Self::create_cluster(state, sender, vec![desc1, desc2]);
            }
            
            // Case 2: Sink + Cluster = Add to Cluster
            (Some(OrbKind::PhysicalSink { description }), Some(OrbKind::Cluster { devices })) => {
                info!("Adding {} to cluster {:?}", description, devices);
                
                // 1. Unload old cluster
                Self::unload_cluster(state, sender, target);
                
                // 2. Create new cluster
                let mut new_devices = devices.clone();
                new_devices.push(description);
                Self::create_cluster(state, sender, new_devices);
            }
            
            // Case 3: Cluster + Sink = Add to Cluster
            (Some(OrbKind::Cluster { devices }), Some(OrbKind::PhysicalSink { description })) => {
                info!("Adding {} to cluster {:?}", description, devices);
                
                // 1. Unload old cluster
                Self::unload_cluster(state, sender, source);
                
                // 2. Create new cluster
                let mut new_devices = devices.clone();
                new_devices.push(description);
                Self::create_cluster(state, sender, new_devices);
            }
            
            // Case 4: Cluster + Cluster = Merge Clusters
            (Some(OrbKind::Cluster { devices: d1 }), Some(OrbKind::Cluster { devices: d2 })) => {
                info!("Merging clusters {:?} + {:?}", d1, d2);
                
                // 1. Unload both
                Self::unload_cluster(state, sender, source);
                Self::unload_cluster(state, sender, target);
                
                // 2. Create super-cluster
                let mut new_devices = d1.clone();
                new_devices.extend(d2);
                Self::create_cluster(state, sender, new_devices);
            }

            // Case 5: Source -> Sink = Link
            (Some(OrbKind::ApplicationSource { .. }), Some(OrbKind::PhysicalSink { .. })) |
            (Some(OrbKind::ApplicationSource { .. }), Some(OrbKind::Cluster { .. })) => {
                info!("Linking source {} -> sink {}", src_name, tgt_name);
                Self::link_source_to_sink(state, source, target);
            }
            
            _ => {
                warn!("Invalid connection type");
            }
        }
    }
    
    fn unload_cluster(state: &SharedState, sender: &Sender<OrbEvent>, cluster_id: Uuid) {
        // 1. Determine Target Sink for Streams
        let target_sink = {
            let mut saved = state.saved_default_sink.lock().unwrap();
            saved.remove(&cluster_id)
        };
        
        // If no saved sink, try to find the first device in the cluster
        let fallback_sink = if target_sink.is_none() {
            let kinds = state.orb_kinds.lock().unwrap();
            if let Some(OrbKind::Cluster { devices }) = kinds.get(&cluster_id) {
                devices.first().cloned()
            } else {
                None
            }
        } else {
            None
        };

        let restore_to = target_sink.or(fallback_sink);

        // 2. Move Streams & Restore Default Sink
        if let Some(sink_name) = restore_to {
            info!("Restoring streams to: {}", sink_name);
            
            // Move streams
            // We need the ID of the cluster module to find its sink ID, but we can also just move ALL streams 
            // that are currently on the cluster sink.
            // Since we don't have the cluster sink name easily here (we constructed it dynamically),
            // we'll use a broad approach: Move ALL sink-inputs to the target.
            // This is safer than leaving them to fallback.
            let _ = std::process::Command::new("bash")
                .arg("-c")
                .arg(format!(
                    "pactl list sink-inputs short | cut -f1 | xargs -I{{}} pactl move-sink-input {{}} {} 2>/dev/null || true",
                    sink_name
                ))
                .output();
                
            // Restore default sink
            let _ = std::process::Command::new("pactl")
                .args(&["set-default-sink", &sink_name])
                .output();
        }

        // 3. Unload Module
        let module_id = {
            let mut modules = state.combine_modules.lock().unwrap();
            modules.remove(&cluster_id)
        };
        
        if let Some(mid) = module_id {
            let _ = std::process::Command::new("pactl")
                .args(&["unload-module", &mid.to_string()])
                .output();
            info!("✓ Unloaded cluster module {}", mid);
        }
        
        // Remove from UI
        let _ = sender.send(OrbEvent::Remove(cluster_id));
        
        // Remove from State
        state.orb_kinds.lock().unwrap().remove(&cluster_id);
        state.orb_names.lock().unwrap().remove(&cluster_id);
        state.orb_to_pw_id.lock().unwrap().remove(&cluster_id);
        
        // Note: We do NOT restore devices here, because we are immediately creating a new cluster
        // that will "consume" them.
    }

    fn handle_disconnect(state: &SharedState, sender: &Sender<OrbEvent>, source: Uuid, _target: Uuid) {
        // For Cluster orbs, unload ALL combine-sink modules containing their devices
        let kind = {
            let kinds = state.orb_kinds.lock().unwrap();
            kinds.get(&source).cloned()
        };
        
        if let Some(OrbKind::Cluster { devices }) = kind {
            info!("Separating cluster: {:?}", devices);
            
            // 1. Determine Target for Restoration (Saved Default or First Device)
            let target_sink = {
                let mut saved = state.saved_default_sink.lock().unwrap();
                saved.remove(&source)
            };
            
            let restore_to = target_sink.or_else(|| devices.first().cloned());
            
            if let Some(sink_name) = restore_to {
                info!("Restoring streams to: {}", sink_name);
                let _ = std::process::Command::new("bash")
                    .arg("-c")
                    .arg(format!(
                        "pactl list sink-inputs short | cut -f1 | xargs -I{{}} pactl move-sink-input {{}} {} 2>/dev/null || true",
                        sink_name
                    ))
                    .output();
                    
                let _ = std::process::Command::new("pactl")
                    .args(&["set-default-sink", &sink_name])
                    .output();
            }

            // Get module ID to unload
            let module_id = {
                let mut modules = state.combine_modules.lock().unwrap();
                modules.remove(&source)
            };
            
            if let Some(mid) = module_id {
                let result = std::process::Command::new("pactl")
                    .args(&["unload-module", &mid.to_string()])
                    .output();
                
                match result {
                    Ok(out) if out.status.success() => {
                        info!("✓ Unloaded combine-sink module {}", mid);
                    }
                    Ok(out) => {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        warn!("Failed to unload module {}: {}", mid, stderr);
                    }
                    Err(e) => {
                        error!("Failed to execute pactl: {}", e);
                    }
                }
            } else {
                warn!("No module found for cluster {}", source);
            }
            
            // Clear tracked modules (already removed above)
            // state.combine_modules.lock().unwrap().clear();
            
            // Remove cluster from state
            state.orb_kinds.lock().unwrap().remove(&source);
            state.orb_names.lock().unwrap().remove(&source);
            state.orb_to_pw_id.lock().unwrap().remove(&source);
            
            // 1. Remove Cluster from UI
            let _ = sender.send(OrbEvent::Remove(source));

            // 2. Restore Original Devices
            info!("Restoring original devices from cluster: {:?}", devices);
            
            // Remove from active cluster members so they can be rediscovered/shown
            state.remove_cluster_members(&devices);
            
            // Since we kept them in state, we can iterate and find them.
            let kinds = state.orb_kinds.lock().unwrap();
            let names = state.orb_names.lock().unwrap();
            let pw_ids = state.orb_to_pw_id.lock().unwrap();
            
            let mut restored_count = 0;
            
            for (uuid, kind) in kinds.iter() {
                if let OrbKind::PhysicalSink { description } = kind {
                    if devices.contains(description) {
                        // Found one of the original devices!
                        if let Some(name) = names.get(uuid) {
                            if let Some(pw_id) = pw_ids.get(uuid) {
                                // Determine display name based on kind, similar to discovery logic
                                let display_name = match &kind {
                                    OrbKind::PhysicalSink { description } => description.clone(),
                                    OrbKind::ApplicationSource { app_name } => app_name.clone(),
                                    _ => name.clone(),
                                };

                                let orb = Orb {
                                    id: *uuid,
                                    pw_id: *pw_id,
                                    kind: kind.clone(),
                                    name: display_name.clone(), // Use friendly name
                                    icon_name: "audio-card".to_string(), // Default, UI might override
                                    status: "Active".to_string(),
                                    state: OrbState::Floating,
                                    position: (0.0, 0.0),
                                    velocity: (0.0, 0.0),
                                };
                                info!("Restoring device to UI: {}", display_name);
                                let _ = sender.send(OrbEvent::Add(orb));
                                restored_count += 1;
                            }
                        }
                    }
                }
            }
            
            info!("✓ Devices separated - restored {} devices to UI", restored_count);
        } else {
            info!("Disconnect requested for non-cluster Orb: {:?}", kind);
        }
    }


    fn create_cluster(state: &SharedState, sender: &Sender<OrbEvent>, devices: Vec<String>) {
        // 1. Resolve Device Names to PipeWire Node Names
        let mut node_names = Vec::new();
        let mut member_pairs = Vec::new(); // (Description, NodeName)
        
        {
            let kinds = state.orb_kinds.lock().unwrap();
            let names = state.orb_names.lock().unwrap();
            let active_members = state.active_cluster_members.lock().unwrap();
            
            for desc in &devices {
                // Try to find in current Orbs (Floating)
                let uuid = kinds.iter().find_map(|(u, k)| {
                    if let OrbKind::PhysicalSink { description } = k {
                        if description == desc {
                            return Some(*u);
                        }
                    }
                    None
                });
                
                if let Some(u) = uuid {
                    if let Some(name) = names.get(&u) {
                        node_names.push(name.clone());
                        member_pairs.push((desc.clone(), name.clone()));
                    }
                } else {
                    // Try to find in existing cluster members
                    if let Some(node_name) = active_members.get(desc) {
                        node_names.push(node_name.clone());
                        member_pairs.push((desc.clone(), node_name.clone()));
                    } else {
                        warn!("Could not find Node Name for device: {}", desc);
                    }
                }
            }
        }
        
        if node_names.is_empty() {
            error!("No valid devices found for cluster");
            return;
        }

        info!("Creating cluster with {} devices: {:?}", node_names.len(), devices);

        // 2. Create combine-sink using pactl
        // Generate a deterministic name based on sorted device names to avoid duplicates?
        // Or just random? Random is safer for now to avoid collisions with old modules.
        let combine_name = format!("auralis_cluster_{}", Uuid::new_v4().simple());
        let slaves = node_names.join(",");
        
        let output = std::process::Command::new("pactl")
            .args(&[
                "load-module",
                "module-combine-sink",
                &format!("sink_name={}", combine_name),
                &format!("slaves={}", slaves),
                "latency_compensate=yes",  // Enable automatic latency compensation
                "rate=48000",               // Standard sample rate
                "channels=2",               // Stereo
            ])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    let module_id_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if let Ok(module_id) = module_id_str.parse::<u32>() {
                        info!("✓ Cluster created (module {})", module_id);
                        
                        // 3. Create Cluster Orb
                        let cluster_id = Uuid::new_v4();
                        
                        // Store module ID mapping
                        state.combine_modules.lock().unwrap().insert(cluster_id, module_id);
                        
                        // Track active members
                        state.add_cluster_members(member_pairs);
                        
                            // 4. Set as Default Sink
                        // Save current default first
                        let current_default = std::process::Command::new("pactl")
                            .args(&["get-default-sink"])
                            .output()
                            .ok()
                            .and_then(|out| String::from_utf8(out.stdout).ok())
                            .map(|s| s.trim().to_string());

                        if let Some(def) = current_default {
                            info!("Saved default sink: {}", def);
                            state.saved_default_sink.lock().unwrap().insert(cluster_id, def);
                        }

                        let _ = std::process::Command::new("pactl")
                            .args(&["set-default-sink", &combine_name])
                            .output();
                        info!("✓ Set cluster as default sink");

                        // 5. Move active streams
                        std::thread::sleep(std::time::Duration::from_millis(200));
                        let _ = std::process::Command::new("bash")
                            .arg("-c")
                            .arg(format!(
                                "pactl list sink-inputs short | cut -f1 | xargs -I{{}} pactl move-sink-input {{}} {} 2>/dev/null || true",
                                combine_name
                            ))
                            .output();
                        
                        // 6. Register Cluster Orb
                        let cluster_orb = Orb {
                            id: cluster_id,
                            pw_id: 999, // Placeholder
                            kind: OrbKind::Cluster { devices: devices.clone() },
                            name: format!("Cluster ({})", devices.len()), // Simple name
                            icon_name: "audio-card".to_string(),
                            status: "Active".to_string(),
                            state: OrbState::Floating,
                            position: (0.0, 0.0),
                            velocity: (0.0, 0.0),
                        };
                        
                        state.register_orb(
                            cluster_id, 
                            999, 
                            combine_name,
                            cluster_orb.kind.clone()
                        );
                        
                        let _ = sender.send(OrbEvent::Add(cluster_orb));
                        
                        // 7. Remove original devices from UI
                        // 7. Remove original devices from UI
                        // We need to find the UUIDs of the devices we just clustered to remove them from the UI
                        {
                            let kinds = state.orb_kinds.lock().unwrap();
                            for desc in &devices {
                                // Find UUID for this description (Floating only)
                                let uuid = kinds.iter().find_map(|(u, k)| {
                                    if let OrbKind::PhysicalSink { description } = k {
                                        if description == desc {
                                            return Some(*u);
                                        }
                                    }
                                    None
                                });
                                
                                if let Some(u) = uuid {
                                    let _ = sender.send(OrbEvent::Remove(u));
                                    // We do NOT remove from state here. We keep them in state so we can restore them later.
                                    // They are effectively "hidden" from the UI but tracked by the backend.
                                    // The `active_cluster_members` set prevents them from being re-added by discovery.
                                }
                            }
                        }
                        
                    } else {
                        error!("Failed to parse module ID");
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    error!("Failed to create combine-sink: {}", stderr);
                }
            }
            Err(e) => {
                error!("Failed to execute pactl: {}", e);
            }
        }
    }


    fn link_source_to_sink(state: &SharedState, source: Uuid, sink: Uuid) {
        let names = state.orb_names.lock().unwrap();
        let src_name = names.get(&source).map(|s| s.as_str()).unwrap_or("source");
        let sink_name = names.get(&sink).map(|s| s.as_str()).unwrap_or("sink");

        // Use pw-cli to link
        let output = std::process::Command::new("pw-link")
            .args(&[src_name, sink_name])
            .output();

        match output {
            Ok(out) => {
                if out.status.success() {
                    info!("Linked {} -> {}", src_name, sink_name);
                } else {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    error!("Failed to link: {}", stderr);
                }
            }
            Err(e) => {
                error!("Failed to execute pw-link: {}", e);
            }
        }
    }
}
