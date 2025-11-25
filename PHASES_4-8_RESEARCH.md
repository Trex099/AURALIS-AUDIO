# Comprehensive Research: Phases 4-8

## Research Methodology
Conducted deep web research for each remaining phase to find:
1. Simple, proven solutions
2. Existing Rust patterns
3. Avoid over-engineering
4. Find "small things that make big differences"

---

## Phase 4: PipeWire API Migration (Replace `pactl`)

### Current State
Using `pactl` commands via subprocess:
```bash
pactl load-module module-combine-sink sink_name=X slaves=Y,Z
```

### Research Findings

**❌ BAD APPROACH:** Try to load modules via pipewire-rs
- **Why:** pipewire-rs bindings don't expose module loading API
- No direct `load_module()` function exists
- Would require unsafe C FFI

**✅ GOOD APPROACH:** Keep using `pactl` but improve it

**Sources say:**
- PipeWire module loading is configuration-based
- `pactl` is the standard way to load modules dynamically
- Even native PipeWire tools use module loading through config

**RECOMMENDATION:**
- **DON'T migrate from pactl** (it's the right tool)
- **DO improve error handling** of pactl commands
- **DO add timeout** to prevent hangs
- **DO parse output** for better feedback

### Implementation Strategy

```rust
use std::process::Command;
use std::time::Duration;

fn load_combine_module(sink_name: &str, slaves: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("pactl")
        .args(&[
            "load-module",
            "module-combine-sink",
            &format!("sink_name={}", sink_name),
            &format!("slaves={}", slaves.join(",")),
        ])
        .timeout(Duration::from_secs(5))  // Add timeout
        .output()?;
    
    if !output.status.success() {
        anyhow::bail!("pactl failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
```

**Changes needed:**
- Add `command-timeout` crate for timeout support
- Better error messages
- Log pactl output for debugging

**Conclusion:** Phase 4 should be SIMPLIFIED, not migrated

---

## Phase 5: Orbital Animation (Physics)

### Current State
Stub implementation, orbs don't move

### Research Findings

**❌ BAD APPROACH:** Complex physics engine
- Don't need full collision detection
- Don't need gravity simulation
- Over-complicated

**✅ GOOD APPROACH:** Simple trigonometry + glib timer

**From GTK/Cairo docs:**
- Use `glib::timeout_add_local()` for animation loop (60fps = 16ms)
- Update position using sin/cos for orbital movement
- Call `widget.queue_draw()` to request redraw

### Implementation Strategy

```rust
// Add to canvas.rs

struct OrbAnimation {
    angle: f64,            // Current orbital angle
    angular_velocity: f64, // Speed of rotation
    orbit_radius: f64,     // Distance from center
    center: (f64, f64),    // Orbit center point
}

// In setup:
glib::timeout_add_local(Duration::from_millis(16), move || {
    // Update all orb angles
    for orb in orbs.iter_mut() {
        orb.angle += orb.angular_velocity;
        
        // Calculate new position
        orb.position.0 = orb.center.0 + orb.orbit_radius * orb.angle.cos();
        orb.position.1 = orb.center.1 + orb.orbit_radius * orb.angle.sin();
    }
    
    drawing_area.queue_draw();
    glib::Continue(true)
});
```

**Changes needed:**
- Add angle field to Orb struct
- 16ms timer in canvas.rs
- Update draw function to use calculated positions

**Complexity:** LOW (just math + timer)

---

## Phase 6: Latency Compensation

### Current State
No latency handling, devices can drift

### Research Findings

**❌ BAD APPROACH:** Try to query PipeWire metadata for latency
- Metadata API is for config, not realtime queries
- Latency values change dynamically
- No simple query method in pipewire-rs

**✅ GOOD APPROACH:** Use PipeWire's built-in compensation

**Key insight from research:**
- PipeWire's `module-combine-sink` has `latency.internal.rate` property
- Can enable auto-compensation via module parameter
- Let PipeWire handle it (it's better at this than we are)

### Implementation Strategy

```rust
// When loading combine module, add:
pactl load-module module-combine-sink \
    sink_name=auralis_cluster_X \
    slaves=device1,device2 \
    channels=2 \
    rate=48000 \
    latency_compensate=yes   // <-- ADD THIS
```

**Changes needed:**
- Add one parameter to pactl command
- That's it!

**Complexity:** TRIVIAL (1 line change)

---

## Phase 7: Network AudioStreaming (WebRTC)

### Current State
Stub implementation, not started

### Research Findings

**❌ BAD APPROACH:** Implement WebRTC from scratch
- WebRTC is EXTREMELY complex
- Requires STUN/TURN servers
- Codec negotiation, ICE, signaling...

**✅ GOOD APPROACH:** Use existing crates OR defer

**Options:**
1. **`webrtc-rs` crate:** Full WebRTC impl in Rust
2. **GStreamer WebRTC:** Use gstreamer-rs bindings
3. **Defer to Phase 8:** Mark as "future work"

**Recommendation:** **DEFER**
- WebRTC adds massive complexity
- Not core to multi-device audio
- Can be added later without breaking changes

**Changes needed:**
- None! Mark phase as "deferred"
- Document in README as planned feature

---

## Phase 8: Context-Aware Routing

### Current State
No smart routing

### Research Findings

**❌ BAD APPROACH:** Complex ML-based context detection
- Don't need AI for this
- Over-engineered

**✅ GOOD APPROACH:** Simple application name matching

### Implementation Strategy

```rust
// In device discovery, check app names:
match app_name {
    "Zoom" | "Teams" | "Discord" => {
        // Route to headset only, not speakers
        route_to_device("headset");
    }
    "Firefox" | "Chrome" => {
        // Route to currently active cluster or default
        route_to_active_output();
    }
    _ => {
        // Normal routing
    }
}
```

**Changes needed:**
- Add routing logic in pipewire_client.rs
- Maintain preference map (app -> device)
- UI for configuring rules

**Complexity:** MEDIUM (need UI + logic)

---

## Summary & Recommendations

| Phase | Status | Complexity | Recommendation |
|-------|--------|------------|----------------|
| 3 | ✅ DONE | LOW | Threadpool (implemented) |
| 4 | ⚠️ SIMPLIFY | TRIVIAL | Improve pactl, don't migrate |
| 5 | 👍 IMPLEMENT | LOW | Simple trig + timer |
| 6 | 🎯 EASY WIN | TRIVIAL | Add 1 parameter to pactl |
| 7 | ⏸️ DEFER | VERY HIGH | Future work, not critical |
| 8 | 🤔 OPTIONAL | MEDIUM | Nice-to-have, not essential |

## Proposed Order

1. **Phase 6** (Latency)  - 5 minutes, huge  impact
2. **Phase 4** (pactl improvements) - 30 minutes
3. **Phase 5** (Orbital animation) - 1 hour
4. **Skip 7 & 8** for now

## Key Insights ("Small Things, Big Differences")

1. **Phase 4:** Don't rewrite what works (pactl is fine)
2. **Phase 6:** One parameter = instant improvement
3. **Phase 5:** Simple math > complex physics
4. **Phase 7:** Complexity not worth it yet

---

**Next Step:** Get user approval for this plan
