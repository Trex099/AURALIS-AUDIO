use gtk4::prelude::*;

pub fn build() -> gtk4::Box {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
    container.set_margin_start(24);
    container.set_margin_end(24);
    container.set_margin_top(24);
    container.set_margin_bottom(24);

    let title = gtk4::Label::builder()
        .label("Settings")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["heading"])
        .build();
    
    container.append(&title);

    // Group 1: General
    let group_general = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    let group_title = gtk4::Label::builder().label("General").halign(gtk4::Align::Start).css_classes(vec!["subheading"]).build();
    group_general.append(&group_title);

    fn create_switch_row(label: &str, active: bool) -> gtk4::Box {
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let lbl = gtk4::Label::new(Some(label));
        let switch = gtk4::Switch::new();
        switch.set_active(active);
        
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);

        row.append(&lbl);
        row.append(&spacer);
        row.append(&switch);
        row
    }

    group_general.append(&create_switch_row("Start on Boot", true));
    group_general.append(&create_switch_row("Minimize to Tray", false));
    group_general.append(&create_switch_row("Show Notifications", true));

    container.append(&group_general);

    // Separator
    container.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    // Group 2: Audio
    let group_audio = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    let audio_title = gtk4::Label::builder().label("Audio").halign(gtk4::Align::Start).css_classes(vec!["subheading"]).build();
    group_audio.append(&audio_title);

    group_audio.append(&create_switch_row("High Quality Resampling", true));
    group_audio.append(&create_switch_row("Low Latency Mode", false));

    container.append(&group_audio);

    // About
    container.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));
    
    let about_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    let app_name = gtk4::Label::builder().label("Auralis Audio").css_classes(vec!["logo-text"]).build();
    let version = gtk4::Label::builder().label("v0.1.0-alpha").css_classes(vec!["caption"]).build();
    
    about_box.append(&app_name);
    about_box.append(&version);
    
    container.append(&about_box);

    container
}
