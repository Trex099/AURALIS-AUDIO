// Test file for graph types and commands
// Tests UiCommand enum and other graph data structures

use auralis_core::{UiCommand, OrbEvent, Orb, OrbKind, OrbState};
use uuid::Uuid;

#[test]
fn test_ui_command_connect() {
    // Test Connect command creation
    let source = Uuid::new_v4();
    let target = Uuid::new_v4();
    
    let cmd = UiCommand::Connect { source, target };
    
    match cmd {
        UiCommand::Connect { source: s, target: t } => {
            assert_eq!(s, source);
            assert_eq!(t, target);
        }
        _ => panic!("Wrong command type"),
    }
}

#[test]
fn test_ui_command_disconnect() {
    // Test Disconnect command creation
    let source = Uuid::new_v4();
    let target = Uuid::new_v4();
    
    let cmd = UiCommand::Disconnect { source, target };
    
    match cmd {
        UiCommand::Disconnect { source: s, target: t } => {
            assert_eq!(s, source);
            assert_eq!(t, target);
        }
        _ => panic!("Wrong command type"),
    }
}

#[test]
fn test_ui_command_shutdown() {
    // Test Shutdown command
    let cmd = UiCommand::Shutdown;
    
    assert!(matches!(cmd, UiCommand::Shutdown));
}

#[test]
fn test_orb_event_add() {
    // Test OrbEvent::Add
    let id = Uuid::new_v4();
    let orb = Orb {
        id,
        pw_id: 100,
        kind: OrbKind::PhysicalSink {
            description: "Test".to_string(),
        },
        name: "TestOrb".to_string(),
        icon_name: "audio-card".to_string(),
        status: "Active".to_string(),
        state: OrbState::Floating,
        position: (0.0, 0.0),
        velocity: (0.0, 0.0),
    };
    
    let event = OrbEvent::Add(orb.clone());
    
    match event {
        OrbEvent::Add(o) => {
            assert_eq!(o.id, id);
            assert_eq!(o.name, "TestOrb");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_orb_event_remove() {
    // Test OrbEvent::Remove
    let id = Uuid::new_v4();
    let event = OrbEvent::Remove(id);
    
    match event {
        OrbEvent::Remove(removed_id) => {
            assert_eq!(removed_id, id);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_orb_kind_variants() {
    // Test all OrbKind variants can be created
    
    // PhysicalSink
    let sink = OrbKind::PhysicalSink {
        description: "Sink".to_string(),
    };
    assert!(matches!(sink, OrbKind::PhysicalSink { .. }));
    
    // Cluster
    let cluster = OrbKind::Cluster {
        devices: vec!["Dev1".to_string()],
    };
    assert!(matches!(cluster, OrbKind::Cluster { .. }));
    
    // ApplicationSource
    let app = OrbKind::ApplicationSource {
        app_name: "App".to_string(),
    };
    assert!(matches!(app, OrbKind::ApplicationSource { .. }));
}

#[test]
fn test_orb_state_variants() {
    // Test all OrbState variants
    
    let floating = OrbState::Floating;
    assert!(matches!(floating, OrbState::Floating));
    
    let orbiting = OrbState::Orbiting {
        parent_id: Uuid::new_v4(),
    };
    assert!(matches!(orbiting, OrbState::Orbiting { .. }));
}

#[test]
fn test_cluster_with_empty_devices() {
    // Test that cluster can be created with empty device list
    let cluster = OrbKind::Cluster {
        devices: vec![],
    };
    
    match cluster {
        OrbKind::Cluster { devices } => {
            assert_eq!(devices.len(), 0);
        }
        _ => panic!("Wrong kind"),
    }
}

#[test]
fn test_cluster_with_many_devices() {
    // Test cluster with multiple devices
    let devices: Vec<String> = (0..10)
        .map(|i| format!("Device{}", i))
        .collect();
    
    let cluster = OrbKind::Cluster {
        devices: devices.clone(),
    };
    
    match cluster {
        OrbKind::Cluster { devices: devs } => {
            assert_eq!(devs.len(), 10);
            assert_eq!(devs[0], "Device0");
            assert_eq!(devs[9], "Device9");
        }
        _ => panic!("Wrong kind"),
    }
}
