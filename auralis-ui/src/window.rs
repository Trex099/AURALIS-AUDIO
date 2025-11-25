use gtk4::prelude::*;
use libadwaita::Application;
use std::sync::mpsc::Sender;
use std::rc::Rc;
use std::cell::RefCell;
use auralis_core::{UiCommand, OrbEvent};

pub fn build(app: &Application, rx: async_channel::Receiver<OrbEvent>, cmd_tx: Sender<UiCommand>) {
    // Force Dark Mode
    let style_manager = libadwaita::StyleManager::default();
    style_manager.set_color_scheme(libadwaita::ColorScheme::ForceDark);

    // Shared State
    let state = Rc::new(RefCell::new(state::AppState::new()));

    // Main Content Box (Horizontal Split: Sidebar | Content)
    let main_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    main_box.add_css_class("main-window");

    // --- SIDEBAR ---
    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    sidebar.set_width_request(260);
    sidebar.add_css_class("sidebar");

    // Logo Area
    let logo_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    logo_box.set_margin_top(32);
    logo_box.set_margin_bottom(32);
    logo_box.set_margin_start(24);
    logo_box.set_margin_end(24);

    // Placeholder Logo
    let logo_icon = gtk4::Image::from_icon_name("audio-card-symbolic"); 
    logo_icon.set_pixel_size(28);
    
    let logo_text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let title_label = gtk4::Label::builder().label("AURALIS AUDIO").halign(gtk4::Align::Start).css_classes(vec!["logo-text"]).build();
    let subtitle_label = gtk4::Label::builder().label("Cluster Manager").halign(gtk4::Align::Start).css_classes(vec!["logo-subtext"]).build();

    logo_text_box.append(&title_label);
    logo_text_box.append(&subtitle_label);

    logo_box.append(&logo_icon);
    logo_box.append(&logo_text_box);
    sidebar.append(&logo_box);

    // Navigation
    let nav_list = gtk4::ListBox::new();
    nav_list.set_selection_mode(gtk4::SelectionMode::Single);
    nav_list.set_margin_start(12);
    nav_list.set_margin_end(12);
    nav_list.add_css_class("navigation");
    
    fn create_nav_row(icon: &str, text: &str, name: &str) -> gtk4::ListBoxRow {
        let row = gtk4::ListBoxRow::new();
        let box_ = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let img = gtk4::Image::from_icon_name(icon);
        let lbl = gtk4::Label::new(Some(text));
        
        box_.append(&img);
        box_.append(&lbl);
        row.set_child(Some(&box_));
        row.set_widget_name(name);
        row
    }

    let row_dev = create_nav_row("computer-symbolic", "Devices", "devices");
    let row_clus = create_nav_row("view-grid-symbolic", "Clusters", "clusters");
    let row_set = create_nav_row("emblem-system-symbolic", "Settings", "settings");
    
    nav_list.append(&row_dev);
    nav_list.append(&row_clus);
    nav_list.append(&row_set);
    
    sidebar.append(&nav_list);

    // Spacer
    let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&spacer);

    // Properties Section (Bottom)
    let props_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    props_box.add_css_class("properties-panel");
    
    let props_label = gtk4::Label::builder().label("Properties").halign(gtk4::Align::Start).css_classes(vec!["heading"]).build();
    
    let props_card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    props_card.add_css_class("properties-card");
    props_card.set_height_request(120);
    
    let props_icon = gtk4::Image::from_icon_name("touch-symbolic");
    props_icon.set_pixel_size(32);
    props_icon.set_opacity(0.3);
    
    let props_hint = gtk4::Label::new(Some("Select a device or cluster\nto see its properties."));
    props_hint.set_justify(gtk4::Justification::Center);
    props_hint.add_css_class("caption");
    props_hint.set_opacity(0.5);

    props_card.append(&props_icon);
    props_card.append(&props_hint);
    props_card.set_valign(gtk4::Align::Center);
    props_card.set_halign(gtk4::Align::Center);

    props_box.append(&props_label);
    props_box.append(&props_card);
    sidebar.append(&props_box);

    main_box.append(&sidebar);

    // --- MAIN CONTENT STACK ---
    let stack = gtk4::Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
    stack.set_hexpand(true);
    
    // PAGE 1: DEVICES (The main view)
    let devices_page = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    devices_page.add_css_class("main-content");

    // Header
    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    header_box.set_height_request(64);
    header_box.set_margin_start(24);
    header_box.set_margin_end(24);
    header_box.set_valign(gtk4::Align::Center);

    let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.add_css_class("btn-icon");
    let play_btn = gtk4::Button::from_icon_name("media-playback-start-symbolic");
    play_btn.add_css_class("btn-icon");

    let spacer_header = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer_header.set_hexpand(true);

    let create_cluster_btn = gtk4::Button::with_label("Create New Cluster");
    create_cluster_btn.add_css_class("btn-primary");
    create_cluster_btn.set_icon_name("list-add-symbolic");
    
    // Window Controls
    let window_controls = gtk4::WindowControls::new(gtk4::PackType::End);

    header_box.append(&refresh_btn);
    header_box.append(&play_btn);
    header_box.append(&spacer_header);
    header_box.append(&create_cluster_btn);
    header_box.append(&window_controls);

    // Wrap header in WindowHandle
    let header_handle = gtk4::WindowHandle::new();
    header_handle.set_child(Some(&header_box));

    devices_page.append(&header_handle);

    // Available Devices List
    let device_list_widget = device_list::build(state.clone(), cmd_tx.clone());
    
    // Callback for Canvas to update Device List
    let device_list_weak = device_list_widget.downgrade();
    let state_cb = state.clone();
    let on_drop = move || {
        if let Some(w) = device_list_weak.upgrade() {
            if let Some(box_widget) = w.downcast_ref::<gtk4::Box>() {
                if let Some(list_box) = box_widget.last_child().and_then(|w| w.downcast::<gtk4::ListBox>().ok()) {
                    device_list::update_list(&list_box, &state_cb);
                }
            }
        }
    };

    // Clustering Zone (Canvas)
    let zone_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    zone_box.set_vexpand(true);
    zone_box.set_margin_start(24);
    zone_box.set_margin_end(24);
    
    let canvas_widget = canvas::build(state.clone(), cmd_tx.clone(), on_drop);
    canvas_widget.add_css_class("clustering-zone");
    zone_box.append(&canvas_widget);
    
    devices_page.append(&zone_box);
    devices_page.append(&device_list_widget);
    
    stack.add_named(&devices_page, Some("devices"));

use crate::{canvas, device_list, clusters_view, settings_view, state};

// ... (inside build function)

    // PAGE 2: CLUSTERS
    let clusters_page = clusters_view::build(state.clone(), cmd_tx.clone());
    stack.add_named(&clusters_page, Some("clusters"));

    // PAGE 3: SETTINGS
    let settings_page = settings_view::build();
    stack.add_named(&settings_page, Some("settings"));

    main_box.append(&stack);
    
    // ... (navigation connection remains same)

    // --- EVENT LOOP ---
    let state_evt = state.clone();
    let device_list_weak = device_list_widget.downgrade(); 
    let clusters_view_weak = clusters_page.downgrade(); // To update clusters
    let canvas_weak = canvas_widget.downgrade(); 

    glib::MainContext::default().spawn_local(async move {
        while let Ok(event) = rx.recv().await {
            let mut state = state_evt.borrow_mut();
            match event {
                OrbEvent::Add(orb) => {
                    state.orbs.insert(orb.id, orb);
                }
                OrbEvent::Remove(id) => {
                    state.orbs.remove(&id);
                }
            }
            drop(state); // Release lock

            // Update Device List
            if let Some(w) = device_list_weak.upgrade() {
                // The device_list::build returns a Box containing the ListBox
                // Structure: Box -> [Label, ListBox]
                if let Some(box_widget) = w.downcast_ref::<gtk4::Box>() {
                    if let Some(list_box) = box_widget.last_child().and_then(|w| w.downcast::<gtk4::ListBox>().ok()) {
                        device_list::update_list(&list_box, &state_evt);
                    }
                }
            }

            // Update Clusters View
            if let Some(w) = clusters_view_weak.upgrade() {
                if let Some(box_widget) = w.downcast_ref::<gtk4::Box>() {
                    // We need to find the FlowBox inside the container
                    // The structure is Box -> [Label, FlowBox]
                    // So we get the last child
                    if let Some(flow_box) = box_widget.last_child().and_then(|w| w.downcast::<gtk4::FlowBox>().ok()) {
                        clusters_view::update_list(&flow_box, &state_evt, &cmd_tx);
                    }
                }
            }
            
            // Update Canvas
            if let Some(w) = canvas_weak.upgrade() {
                w.queue_draw();
            }
        }
    });

    let window = libadwaita::ApplicationWindow::builder()
        .application(app)
        .title("PipeWire Cluster Manager")
        .content(&main_box)
        .default_width(1280)
        .default_height(800)
        .build();

    window.present();
}
