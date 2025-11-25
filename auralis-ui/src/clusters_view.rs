use gtk4::prelude::*;
use std::sync::mpsc::Sender;
use auralis_core::{UiCommand, OrbKind};
use crate::state::SharedState;

pub fn build(state: SharedState, cmd_tx: Sender<UiCommand>) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    container.set_margin_start(24);
    container.set_margin_end(24);
    container.set_margin_top(24);
    container.set_margin_bottom(24);

    let title = gtk4::Label::builder()
        .label("Active Clusters")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["heading"])
        .build();
    
    container.append(&title);

    let flow_box = gtk4::FlowBox::new();
    flow_box.set_valign(gtk4::Align::Start);
    flow_box.set_selection_mode(gtk4::SelectionMode::None);
    flow_box.set_min_children_per_line(1);
    flow_box.set_max_children_per_line(3);
    flow_box.set_column_spacing(12);
    flow_box.set_row_spacing(12);

    // Initial update
    update_list(&flow_box, &state, &cmd_tx);

    container.append(&flow_box);
    container
}

pub fn update_list(flow_box: &gtk4::FlowBox, state: &SharedState, cmd_tx: &Sender<UiCommand>) {
    // Clear existing children
    while let Some(child) = flow_box.first_child() {
        flow_box.remove(&child);
    }

    let state = state.borrow();
    let mut found_any = false;

    for orb in state.orbs.values() {
        if let OrbKind::Cluster { devices } = &orb.kind {
            found_any = true;
            let card = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
            card.add_css_class("device-card"); // Reuse card styling
            card.set_width_request(200);

            // Icon
            let icon_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
            icon_box.add_css_class("device-icon-container");
            icon_box.set_halign(gtk4::Align::Center);
            
            let icon = gtk4::Image::from_icon_name("view-grid-symbolic");
            icon.set_pixel_size(24);
            icon.set_halign(gtk4::Align::Center);
            icon.set_valign(gtk4::Align::Center);
            icon_box.set_halign(gtk4::Align::Center);
            icon_box.set_valign(gtk4::Align::Center);
            icon_box.append(&icon);
            
            card.append(&icon_box);

            // Name
            let name_lbl = gtk4::Label::builder()
                .label(&orb.name)
                .css_classes(vec!["device-name"])
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            card.append(&name_lbl);

            // Device Count
            let count_lbl = gtk4::Label::builder()
                .label(&format!("{} Devices", devices.len()))
                .css_classes(vec!["device-status"])
                .build();
            card.append(&count_lbl);

            // Separate Button
            let separate_btn = gtk4::Button::with_label("Separate");
            separate_btn.add_css_class("btn-destructive"); // Need to define this or use standard
            let cmd_tx_clone = cmd_tx.clone();
            let orb_id = orb.id;
            
            separate_btn.connect_clicked(move |_| {
                // Send Disconnect command (Source -> Target, but for cluster separation we just need source)
                // The core handles Disconnect(cluster_id, _) as separation
                let _ = cmd_tx_clone.send(UiCommand::Disconnect { 
                    source: orb_id, 
                    target: orb_id // Target ignored for separation
                });
            });
            
            card.append(&separate_btn);

            flow_box.insert(&card, -1);
        }
    }

    if !found_any {
        let empty_lbl = gtk4::Label::new(Some("No active clusters. Drag devices together to create one."));
        empty_lbl.add_css_class("caption");
        flow_box.insert(&empty_lbl, -1);
    }
}
