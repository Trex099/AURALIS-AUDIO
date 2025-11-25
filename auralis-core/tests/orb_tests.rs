// Test file for Orb data structures
// Tests basic creation and properties without touching PipeWire

use auralis_core::{Orb, OrbKind, OrbState};
use uuid::Uuid;

#[test]
fn test_create_physical_sink_orb() {
    // Test creating a physical sink orb
    let id = Uuid::new_v4();
    let orb = Orb {
        id,
        pw_id: 123,
        kind: OrbKind::PhysicalSink {
            description: "Test Speaker".to_string(),
        },
        name: "Test Device".to_string(),
        icon_name: "audio-card".to_string(),
        status: "Active".to_string(),
        state: OrbState::Floating,
        position: (0.0, 0.0),
        velocity: (0.0, 0.0),
    };

    // Verify fields
    assert_eq!(orb.pw_id, 123);
    assert_eq!(orb.name, "Test Device");
    assert_eq!(orb.icon_name, "audio-card");
    assert_eq!(orb.status, "Active");
    assert_eq!(orb.position, (0.0, 0.0));
    
    // Verify kind
    match orb.kind {
        OrbKind::PhysicalSink { description } => {
            assert_eq!(description, "Test Speaker");
        }
        _ => panic!("Wrong orb kind"),
    }
    
    // Verify state
    assert!(matches!(orb.state, OrbState::Floating));
}

#[test]
fn test_create_cluster_orb() {
    // Test creating a cluster orb
    let id = Uuid::new_v4();
    let devices = vec!["Device1".to_string(), "Device2".to_string()];
    
    let orb = Orb {
        id,
        pw_id: 456,
        kind: OrbKind::Cluster {
            devices: devices.clone(),
        },
        name: "Test Cluster".to_string(),
        icon_name: "view-grid-symbolic".to_string(),
        status: "Active".to_string(),
        state: OrbState::Floating,
        position: (100.0, 200.0),
        velocity: (0.0, 0.0),
    };

    assert_eq!(orb.pw_id, 456);
    assert_eq!(orb.name, "Test Cluster");
    assert_eq!(orb.position, (100.0, 200.0));
    
    // Verify cluster contains devices
    match orb.kind {
        OrbKind::Cluster { devices: devs } => {
            assert_eq!(devs.len(), 2);
            assert!(devs.contains(&"Device1".to_string()));
            assert!(devs.contains(&"Device2".to_string()));
        }
        _ => panic!("Wrong orb kind"),
    }
}

#[test]
fn test_create_application_source_orb() {
    // Test creating an application source orb
    let id = Uuid::new_v4();
    
    let orb = Orb {
        id,
        pw_id: 789,
        kind: OrbKind::ApplicationSource {
            app_name: "Firefox".to_string(),
        },
        name: "Firefox".to_string(),
        icon_name: "audio-x-generic".to_string(),
        status: "Playing".to_string(),
        state: OrbState::Floating,
        position: (50.0, 50.0),
        velocity: (1.0, 1.0),
    };

    assert_eq!(orb.pw_id, 789);
    assert_eq!(orb.velocity, (1.0, 1.0));
    
    match orb.kind {
        OrbKind::ApplicationSource { app_name } => {
            assert_eq!(app_name, "Firefox");
        }
        _ => panic!("Wrong orb kind"),
    }
}

#[test]
fn test_orb_state_transitions() {
    // Test that we can create orbs in different states
    let id = Uuid::new_v4();
    
    // Floating state
    let orb_floating = Orb {
        id,
        pw_id: 100,
        kind: OrbKind::PhysicalSink {
            description: "Test".to_string(),
        },
        name: "Test".to_string(),
        icon_name: "audio-card".to_string(),
        status: "Active".to_string(),
        state: OrbState::Floating,
        position: (0.0, 0.0),
        velocity: (0.0, 0.0),
    };
    
    assert!(matches!(orb_floating.state, OrbState::Floating));
    
    // Orbiting state
    let parent_id = Uuid::new_v4();
    let orb_orbiting = Orb {
        id,
        pw_id: 101,
        kind: OrbKind::PhysicalSink {
            description: "Test2".to_string(),
        },
        name: "Test2".to_string(),
        icon_name: "audio-card".to_string(),
        status: "Active".to_string(),
        state: OrbState::Orbiting { parent_id },
        position: (10.0, 10.0),
        velocity: (0.5, 0.5),
    };
    
    match orb_orbiting.state {
        OrbState::Orbiting { parent_id: pid } => {
            assert_eq!(pid, parent_id);
        }
        _ => panic!("Wrong state"),
    }
}

#[test]
fn test_orb_unique_ids() {
    // Test that UUIDs are unique
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    
    assert_ne!(id1, id2, "UUIDs should be unique");
}

#[test]
fn test_orb_position_and_velocity() {
    // Test position and velocity handling
    let id = Uuid::new_v4();
    
    let mut orb = Orb {
        id,
        pw_id: 200,
        kind: OrbKind::PhysicalSink {
            description: "Moving Device".to_string(),
        },
        name: "Mover".to_string(),
        icon_name: "audio-card".to_string(),
        status: "Active".to_string(),
        state: OrbState::Floating,
        position: (100.0, 100.0),
        velocity: (5.0, -3.0),
    };
    
    // Simulate physics update
    orb.position.0 += orb.velocity.0;
    orb.position.1 += orb.velocity.1;
    
    assert_eq!(orb.position, (105.0, 97.0));
}
