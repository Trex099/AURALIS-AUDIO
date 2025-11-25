use gtk4::prelude::*;
use gtk4::{DrawingArea, DropTarget, GestureClick};
use std::sync::mpsc::Sender;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
use auralis_core::{Orb, OrbKind, UiCommand};
use crate::state::SharedState;
use uuid::Uuid;
use cairo;

// Animation state for orbital movement
#[derive(Debug, Clone)]
struct AnimationState {
    angle: f64,              // Current angle in radians
    angular_velocity: f64,   // Rotation speed
    orbit_radius: f64,       // Distance from center
}

impl Default for AnimationState {
    fn default() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            angle: rng.gen::<f64>() * 2.0 * std::f64::consts::PI,
            angular_velocity: 0.01 + rng.gen::<f64>() * 0.02,
            orbit_radius: 60.0 + rng.gen::<f64>() * 80.0,
        }
    }
}

pub fn build(state: SharedState, cmd_tx: Sender<UiCommand>, on_drop: impl Fn() + 'static) -> DrawingArea {
    let drawing_area = DrawingArea::builder()
        .hexpand(true)
        .vexpand(true)
        .build();

    // Animation state storage
    let animations = Rc::new(RefCell::new(HashMap::<Uuid, AnimationState>::new()));
    
    // Animation Loop - runs at ~60fps
    let state_tick = state.clone();
    let anims_tick = animations.clone();
    drawing_area.add_tick_callback(move |da, _clock| {
        // Update all animation angles
        let mut anims = anims_tick.borrow_mut();
        let state_borrow = state_tick.borrow();
        
        // Add animations for new orbs, remove for deleted ones
        for (id, orb) in &state_borrow.orbs {
            if !anims.contains_key(id) && matches!(orb.kind, OrbKind::PhysicalSink { .. } | OrbKind::ApplicationSource { .. }) {
                anims.insert(*id, AnimationState::default());
            }
        }
        
        // Remove animation states for orbs that no longer exist
        anims.retain(|id, _| state_borrow.orbs.contains_key(id));
        
        // Update angles
        for anim in anims.values_mut() {
            anim.angle += anim.angular_velocity;
            // Wrap angle to prevent overflow
            if anim.angle > 2.0 * std::f64::consts::PI {
                anim.angle -= 2.0 * std::f64::consts::PI;
            }
        }
        
        da.queue_draw();
        gtk4::glib::ControlFlow::Continue
    });

    let state_draw = state.clone();
    let anims_draw = animations.clone();
    drawing_area.set_draw_func(move |_, cr, w, h| {
        let state = state_draw.borrow();
        let anims = anims_draw.borrow();
        
        // Background is handled by CSS (dashed border)
        // We just draw the clusters here
        
        let mut has_clusters = false;

        for orb in state.orbs.values() {
            match &orb.kind {
                OrbKind::Cluster { devices } => {
                    has_clusters = true;
                    draw_cluster(cr, orb, devices);
                },
                _ => {
                    // Draw floating orbs with animation
                    if let Some(anim) = anims.get(&orb.id) {
                        has_clusters = true;
                        draw_floating_orb(cr, orb, anim, w as f64, h as f64);
                    }
                }
            }
        }

        if !has_clusters {
            // Draw "Drop here" text if empty? 
            // Or handled by overlay widget?
            // Let's draw it manually for simplicity
            cr.set_source_rgb(1.0, 1.0, 1.0);
            cr.select_font_face("Space Grotesk", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
            cr.set_font_size(24.0);
            let text = "Clustering Zone";
            let extents = cr.text_extents(text).unwrap();
            cr.move_to((w as f64 - extents.width()) / 2.0, (h as f64) / 2.0 - 20.0);
            cr.show_text(text).unwrap();
            
            cr.set_source_rgba(0.6, 0.7, 0.8, 0.5);
            cr.set_font_size(14.0);
            let subtext = "Drag and drop device icons here to form or modify a cluster.";
            let sub_extents = cr.text_extents(subtext).unwrap();
            cr.move_to((w as f64 - sub_extents.width()) / 2.0, (h as f64) / 2.0 + 10.0);
            cr.show_text(subtext).unwrap();
        }
    });

    // Drop Target
    let target = DropTarget::new(gtk4::glib::Type::STRING, gtk4::gdk::DragAction::COPY);
    let state_drop = state.clone();
    let cmd_tx_drop = cmd_tx.clone();
    let da_drop = drawing_area.clone();
    
    target.connect_drop(move |_, value, x, y| {
        let id_str = value.get::<String>().unwrap();
        if let Ok(dropped_id) = Uuid::parse_str(&id_str) {
            println!("Dropped ID: {}", dropped_id);
            
            // Logic:
            // 1. If dropped on empty space -> Create new cluster (if single device) or move it?
            //    Actually, dropping a single device into the zone should probably just "park" it there ready to be clustered?
            //    Or does it immediately create a cluster?
            //    The mockup shows "Living Room Audio" with 2 devices.
            //    If I drop 1 device, maybe it stays as a "Potential Cluster"?
            //    For now, let's say dropping a device creates a "Cluster of 1" or just moves it visually.
            //    BUT, our Core logic requires 2 devices to make a cluster.
            //    So maybe we need a "Staging Area" in the UI state?
            
            //    SIMPLIFICATION: If dropped on an EXISTING cluster -> Add to it.
            //    If dropped on EMPTY space -> Wait for a second drop?
            //    Let's implement: Drop 1 device -> It becomes a "Floating Orb" in the zone (removed from list).
            //    Drop 2nd device on it -> Merge.
            
            //    We need to update the OrbState to "Floating" (in zone) vs "Listed" (in list).
            //    Currently OrbState has Floating/Orbiting.
            //    We can use that.
            
            //    Let's send a command to Core? No, Core doesn't know about "List vs Zone".
            //    We handle this in UI state.
            
            //    TODO: We need to update the Orb's position to where it was dropped.
            {
                let mut state_ref = state_drop.borrow_mut();
                if let Some(orb) = state_ref.orbs.get_mut(&dropped_id) {
                    orb.position = (x, y);
                    println!("Moved {} to zone at ({}, {})", orb.name, x, y);
                }
            }
            
            
            // Auto-Clustering Logic  
            // If there are ANY other devices/clusters in the zone, automatically cluster with them
            let mut target_id = None;
            {
                let state_ref = state_drop.borrow();
                
                println!("Auto-clustering: Checking for existing devices in zone...");
                
                // First, check if there is an existing cluster (clusters might be at 0,0 initially)
                for other in state_ref.orbs.values() {
                    if other.id == dropped_id { continue; }
                    
                    println!("  Found orb: {} (kind: {:?}, pos: {:?})", other.name, other.kind, other.position);
                    
                    // Prefer clustering with existing clusters (ignore position for clusters)
                    if matches!(other.kind, OrbKind::Cluster { .. }) {
                        println!("  → Found cluster! Will connect to it.");
                        target_id = Some(other.id);
                        break;
                    }
                }
                
                // If no cluster found, cluster with the first floating orb in the zone
                if target_id.is_none() {
                    println!("  No cluster found, looking for floating orbs...");
                    for other in state_ref.orbs.values() {
                        if other.id == dropped_id { continue; }
                        if other.position == (0.0, 0.0) { continue; } // Skip devices not in zone
                        
                        println!("  → Found floating orb: {}. Will connect to it.", other.name);
                        target_id = Some(other.id);
                        break;
                    }
                }
                
                if target_id.is_none() {
                    println!("  No clustering target found. Device will stay as floating orb.");
                }
            }
            
            if let Some(tid) = target_id {
                println!("Auto-clustering: {} -> {}", dropped_id, tid);
                let _ = cmd_tx_drop.send(UiCommand::Connect { source: dropped_id, target: tid });
            }
            
            // Trigger redraw
            da_drop.queue_draw();
            
            // Notify list to update
            on_drop();
            
            return true;
        }
        false
    });
    
    drawing_area.add_controller(target);

    // Click Controller for "Separate" button
    let click = GestureClick::new();
    let state_click = state.clone();
    let cmd_tx_click = cmd_tx.clone();
    
    click.connect_pressed(move |_, _, x, y| {
        let state = state_click.borrow();
        for orb in state.orbs.values() {
            if !matches!(orb.kind, OrbKind::Cluster { .. }) { continue; }
            
            // Check button bounds (relative to orb pos)
            // Button is at bottom of card
            let ox = orb.position.0;
            let oy = orb.position.1;
            let w = 300.0;
            let h = 150.0;
            
            let btn_x = ox + w - 100.0 - 20.0;
            let btn_y = oy + h - 40.0;
            let btn_w = 100.0;
            let btn_h = 30.0;
            
            if x >= btn_x && x <= btn_x + btn_w && y >= btn_y && y <= btn_y + btn_h {
                println!("Separate clicked for {}", orb.name);
                let _ = cmd_tx_click.send(UiCommand::Disconnect { source: orb.id, target: orb.id });
                break;
            }
        }
    });
    drawing_area.add_controller(click);

    // Drag Controller for moving floating orbs
    let drag = gtk4::GestureDrag::new();
    let state_drag = state.clone();
    let _cmd_tx_drag = cmd_tx.clone();
    let _da_drag = drawing_area.clone();
    
    // Track which orb is being dragged
    let dragged_orb_id = std::rc::Rc::new(std::cell::RefCell::new(None::<Uuid>));
    let start_pos = std::rc::Rc::new(std::cell::RefCell::new((0.0, 0.0)));
    
    let dragged_id_begin = dragged_orb_id.clone();
    let start_pos_begin = start_pos.clone();
    
    drag.connect_drag_begin(move |_, x, y| {
        let state = state_drag.borrow();
        for orb in state.orbs.values() {
            // Check hit test
            // Floating Orb: Circle radius ~32
            // Cluster: Rect 300x150
            
            let hit = match orb.kind {
                OrbKind::Cluster { .. } => {
                     x >= orb.position.0 && x <= orb.position.0 + 300.0 &&
                     y >= orb.position.1 && y <= orb.position.1 + 150.0
                },
                _ => {
                    let dx = x - (orb.position.0 + 32.0);
                    let dy = y - (orb.position.1 + 32.0);
                    (dx*dx + dy*dy).sqrt() < 40.0 // Slightly larger hit area
                }
            };
            
            if hit {
                println!("Drag begin on {}", orb.name);
                *dragged_id_begin.borrow_mut() = Some(orb.id);
                *start_pos_begin.borrow_mut() = orb.position;
                break;
            }
        }
    });
    
    let state_update = state.clone();
    let dragged_id_update = dragged_orb_id.clone();
    let start_pos_update = start_pos.clone();
    let da_update = drawing_area.clone();
    
    drag.connect_drag_update(move |_, offset_x, offset_y| {
        if let Some(id) = *dragged_id_update.borrow() {
            let mut state = state_update.borrow_mut();
            if let Some(orb) = state.orbs.get_mut(&id) {
                let (sx, sy) = *start_pos_update.borrow();
                orb.position = (sx + offset_x, sy + offset_y);
                da_update.queue_draw();
            }
        }
    });
    
    let state_end = state.clone();
    let dragged_id_end = dragged_orb_id.clone();
    let cmd_tx_end = cmd_tx.clone();
    
    drag.connect_drag_end(move |_, _offset_x, _offset_y| {
        let dragged_id = dragged_id_end.borrow().clone();
        
        if let Some(id) = dragged_id {
            println!("Drag end for {}", id);
            
            // Check for collision/clustering
            let mut target_id = None;
            {
                let state = state_end.borrow();
                if let Some(dragged_orb) = state.orbs.get(&id) {
                    for other in state.orbs.values() {
                        if other.id == id { continue; }
                        
                        // Simple distance check for now
                        // If dragging ONTO a cluster, check cluster bounds
                        // If dragging ONTO a floating orb, check distance
                        
                        let hit = match other.kind {
                            OrbKind::Cluster { .. } => {
                                // Center of dragged orb
                                let cx = dragged_orb.position.0 + 32.0;
                                let cy = dragged_orb.position.1 + 32.0;
                                
                                cx >= other.position.0 && cx <= other.position.0 + 300.0 &&
                                cy >= other.position.1 && cy <= other.position.1 + 150.0
                            },
                            _ => {
                                let dx = dragged_orb.position.0 - other.position.0;
                                let dy = dragged_orb.position.1 - other.position.1;
                                (dx*dx + dy*dy).sqrt() < 80.0 // Overlap threshold
                            }
                        };
                        
                        if hit {
                            target_id = Some(other.id);
                            break;
                        }
                    }
                }
            }
            
            if let Some(tid) = target_id {
                println!("Triggering Connect (Drag): {} -> {}", id, tid);
                let _ = cmd_tx_end.send(UiCommand::Connect { source: id, target: tid });
            }
            
            *dragged_id_end.borrow_mut() = None;
        }
    });
    
    drawing_area.add_controller(drag);

    drawing_area}

fn draw_cluster(cr: &cairo::Context, orb: &Orb, _devices: &Vec<String>) {
    let x = orb.position.0;
    let y = orb.position.1;
    let w = 300.0;
    let h = 150.0;
    
    // Draw Card Background
    cr.set_source_rgba(0.17, 0.42, 0.93, 0.2); // Primary/20
    // Rounded Rect
    let r = 12.0;
    cr.new_sub_path();
    cr.arc(x + r, y + r, r, std::f64::consts::PI, 3.0 * std::f64::consts::PI / 2.0);
    cr.arc(x + w - r, y + r, r, 3.0 * std::f64::consts::PI / 2.0, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::PI / 2.0);
    cr.arc(x + r, y + h - r, r, std::f64::consts::PI / 2.0, std::f64::consts::PI);
    cr.close_path();
    cr.fill_preserve().unwrap();
    
    // Border
    cr.set_source_rgba(0.17, 0.42, 0.93, 0.5);
    cr.set_line_width(1.0);
    cr.stroke().unwrap();
    
    // Title
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.select_font_face("Space Grotesk", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(16.0);
    cr.move_to(x + 20.0, y + 30.0);
    cr.show_text(&orb.name).unwrap();
    
    // "Separate" Button
    let btn_x = x + w - 100.0 - 20.0;
    let btn_y = y + h - 40.0;
    let btn_w = 100.0;
    let btn_h = 30.0;
    
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.1);
    // Rounded btn
    let br = 6.0;
    cr.new_sub_path();
    cr.arc(btn_x + br, btn_y + br, br, std::f64::consts::PI, 3.0 * std::f64::consts::PI / 2.0);
    cr.arc(btn_x + btn_w - br, btn_y + br, br, 3.0 * std::f64::consts::PI / 2.0, 0.0);
    cr.arc(btn_x + btn_w - br, btn_y + btn_h - br, br, 0.0, std::f64::consts::PI / 2.0);
    cr.arc(btn_x + br, btn_y + btn_h - br, br, std::f64::consts::PI / 2.0, std::f64::consts::PI);
    cr.close_path();
    cr.fill().unwrap();
    
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.set_font_size(12.0);
    cr.move_to(btn_x + 24.0, btn_y + 20.0);
    cr.show_text("Separate").unwrap();
}

fn draw_floating_orb(cr: &cairo::Context, orb: &Orb, anim: &AnimationState, canvas_width: f64, canvas_height: f64) {
    // Calculate orbital position
    let center_x = canvas_width / 2.0;
    let center_y = canvas_height / 2.0;
    
    let x = center_x + anim.orbit_radius * anim.angle.cos() - 32.0; // Offset for orb size
    let y = center_y + anim.orbit_radius * anim.angle.sin() - 32.0;
    
    let size = 64.0;
    
    // Radial Gradient Background
    let pattern = cairo::RadialGradient::new(x + size/2.0, y + size/2.0, 0.0, x + size/2.0, y + size/2.0, size/2.0);
    
    // Color based on type
    match &orb.kind {
        OrbKind::PhysicalSink { .. } => {
            pattern.add_color_stop_rgba(0.0, 0.17, 0.42, 0.93, 0.8); // Blue center
            pattern.add_color_stop_rgba(1.0, 0.12, 0.16, 0.23, 0.9); // Dark edge
        }
        OrbKind::ApplicationSource { .. } => {
            pattern.add_color_stop_rgba(0.0, 0.93, 0.42, 0.17, 0.8); // Orange center
            pattern.add_color_stop_rgba(1.0, 0.23, 0.12, 0.12, 0.9); // Dark edge
        }
        _ => {
            pattern.add_color_stop_rgba(0.0, 0.5, 0.5, 0.5, 0.8);
            pattern.add_color_stop_rgba(1.0, 0.2, 0.2, 0.2, 0.9);
        }
    }
    
    cr.set_source(&pattern).unwrap();
    cr.arc(x + size/2.0, y + size/2.0, size/2.0, 0.0, 2.0 * std::f64::consts::PI);
    cr.fill().unwrap();
    
    // Glow / Border
    cr.set_source_rgba(0.4, 0.6, 1.0, 0.6);
    cr.set_line_width(3.0);
    cr.arc(x + size/2.0, y + size/2.0, size/2.0, 0.0, 2.0 * std::f64::consts::PI);
    cr.stroke().unwrap();
    
    // Inner Icon / Symbol
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.9);
    let cx = x + size/2.0;
    let cy = y + size/2.0;
    
    cr.set_line_width(2.0);
    cr.move_to(cx - 8.0, cy - 8.0);
    cr.line_to(cx - 8.0, cy + 8.0);
    cr.line_to(cx + 4.0, cy + 14.0);
    cr.line_to(cx + 4.0, cy - 14.0);
    cr.close_path();
    cr.fill().unwrap();
    
    // Sound waves
    cr.new_sub_path();
    cr.arc(cx + 4.0, cy, 6.0, -0.5, 0.5);
    cr.stroke().unwrap();
    cr.new_sub_path();
    cr.arc(cx + 4.0, cy, 10.0, -0.6, 0.6);
    cr.stroke().unwrap();

    // Label below with shadow
    cr.select_font_face("Space Grotesk", cairo::FontSlant::Normal, cairo::FontWeight::Bold);
    cr.set_font_size(12.0);
    let text = &orb.name;
    let extents = cr.text_extents(text).unwrap();
    let text_x = x + size/2.0 - extents.width()/2.0;
    let text_y = y + size + 20.0;

    // Text Shadow
    cr.set_source_rgba(0.0, 0.0, 0.0, 0.5);
    cr.move_to(text_x + 1.0, text_y + 1.0);
    cr.show_text(text).unwrap();

    // Text
    cr.set_source_rgb(1.0, 1.0, 1.0);
    cr.move_to(text_x, text_y);
    cr.show_text(text).unwrap();
}
