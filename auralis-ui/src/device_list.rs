use gtk4::prelude::*;
use std::sync::mpsc::Sender;
use auralis_core::{UiCommand, OrbKind};
use crate::state::SharedState;

pub fn build(state: SharedState, _cmd_tx: Sender<UiCommand>) -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    container.set_margin_start(24);
    container.set_margin_end(24);
    container.set_margin_top(24);
    container.set_margin_bottom(24);

    let title = gtk4::Label::builder()
        .label("Available Devices")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["heading"])
        .build();
    
    container.append(&title);

    let list_box = gtk4::ListBox::new();
    list_box.set_valign(gtk4::Align::Start);
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    list_box.add_css_class("boxed-list"); 
    list_box.add_css_class("device-list"); 

    // Initial update
    update_list(&list_box, &state);

    container.append(&list_box);
    container
}

pub fn update_list(list_box: &gtk4::ListBox, state: &SharedState) {
    // Clear existing children
    while let Some(child) = list_box.first_child() {
        list_box.remove(&child);
    }

    let state = state.borrow();
    let mut found_any = false;

    for orb in state.orbs.values() {
        // Filter: Only show physical Sinks
        let is_sink = matches!(orb.kind, OrbKind::PhysicalSink { .. });
        let is_monitor = orb.name.to_lowercase().contains("monitor");
        let is_dummy = orb.name.to_lowercase().contains("dummy");
        let is_app = matches!(orb.kind, OrbKind::ApplicationSource { .. });
        let is_in_zone = orb.position != (0.0, 0.0);

        if is_sink && !is_monitor && !is_dummy && !is_app && !is_in_zone {
            found_any = true;
            
            let row = gtk4::ListBoxRow::new();
            row.add_css_class("device-row");
            
            let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
            hbox.set_margin_top(8);
            hbox.set_margin_bottom(8);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);

            // Icon
            let icon_name = if orb.name.to_lowercase().contains("headphone") {
                "audio-headphones-symbolic"
            } else if orb.name.to_lowercase().contains("speaker") {
                "audio-speakers-symbolic"
            } else if orb.name.to_lowercase().contains("mic") {
                "audio-input-microphone-symbolic"
            } else {
                "audio-speakers-symbolic"
            };
            
            let icon = gtk4::Image::from_icon_name(icon_name);
            icon.set_pixel_size(20); 
            icon.set_opacity(0.8);
            
            hbox.append(&icon);

            // Name
            let name_lbl = gtk4::Label::builder()
                .label(&orb.name)
                .halign(gtk4::Align::Start)
                .hexpand(true)
                .ellipsize(gtk4::pango::EllipsizeMode::End)
                .build();
            hbox.append(&name_lbl);

            // Status
            let status_lbl = gtk4::Label::builder()
                .label("Active")
                .css_classes(vec!["caption"])
                .build();
            hbox.append(&status_lbl);

            row.set_child(Some(&hbox));

            // Drag Source Setup
            let drag_source = gtk4::DragSource::new();
            let orb_id = orb.id;
            
            drag_source.connect_prepare(move |_, _, _| {
                let content = gtk4::gdk::ContentProvider::for_value(&orb_id.to_string().to_value());
                Some(content)
            });

            drag_source.connect_drag_begin(|source, _| {
                let icon_theme = gtk4::IconTheme::default();
                let paintable = icon_theme.lookup_icon("audio-speakers-symbolic", &[], 32, 1, gtk4::TextDirection::Ltr, gtk4::IconLookupFlags::empty());
                source.set_icon(Some(&paintable), 16, 16);
            });

            row.add_controller(drag_source);

            list_box.append(&row);
        }
    }

    if !found_any {
        let row = gtk4::ListBoxRow::new();
        let lbl = gtk4::Label::new(Some("No devices found"));
        lbl.set_margin_top(12);
        lbl.set_margin_bottom(12);
        lbl.add_css_class("caption");
        row.set_child(Some(&lbl));
        list_box.append(&row);
    }
}
