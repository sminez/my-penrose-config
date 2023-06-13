use crate::actions::{power_menu, set_tracing_filter, toggle_sticky_client};
use crate::KeyHandler;
use penrose::{
    builtin::{
        actions::{
            floating::{float_focused, reposition, resize, sink_all, sink_focused},
            log_current_state, modify_with, send_layout_message, spawn,
        },
        layout::messages::{ExpandMain, IncMain, ShrinkMain},
    },
    extensions::hooks::ToggleNamedScratchPad,
    map,
};
use std::collections::HashMap;
use tracing_subscriber::{reload::Handle, EnvFilter};

// Delta for moving / resizing floating windows
const DELTA: i32 = 10;

// Generate a raw key binding map in terms of parsable string key bindings rather than resolved key codes
pub fn raw_key_bindings<L, S>(
    toggle_scratch: ToggleNamedScratchPad,
    toggle_scratch_py: ToggleNamedScratchPad,
    handle: Handle<L, S>,
) -> HashMap<String, KeyHandler>
where
    L: From<EnvFilter> + 'static,
    S: 'static,
{
    let mut raw_bindings = map! {
        map_keys: |k: &str| k.to_owned();

        // Windows
        "M-j" => modify_with(|cs| cs.focus_down()),
        "M-k" => modify_with(|cs| cs.focus_up()),
        "M-S-j" => modify_with(|cs| cs.swap_down()),
        "M-S-k" => modify_with(|cs| cs.swap_up()),
        "M-S-q" => modify_with(|cs| cs.kill_focused()),

        // Workspaces
        "M-Tab" => modify_with(|cs| cs.toggle_tag()),
        "M-bracketright" => modify_with(|cs| cs.next_screen()),
        "M-bracketleft" => modify_with(|cs| cs.previous_screen()),
        "M-S-bracketright" => modify_with(|cs| cs.drag_workspace_forward()),
        "M-S-bracketleft" => modify_with(|cs| cs.drag_workspace_backward()),

        // Layouts
        "M-grave" => modify_with(|cs| cs.next_layout()),
        "M-S-grave" => modify_with(|cs| cs.previous_layout()),
        "M-Up" => send_layout_message(|| IncMain(1)),
        "M-Down" => send_layout_message(|| IncMain(-1)),
        "M-Right" => send_layout_message(|| ExpandMain),
        "M-Left" => send_layout_message(|| ShrinkMain),

        // Launchers
        "M-A-s" => spawn("screenshot"),
        "M-semicolon" => spawn("rofi-apps"),
        "M-Return" => spawn("st"),
        "M-A-w" => spawn("floating-webcam"),
        "M-slash" => Box::new(toggle_scratch),
        "M-p" => Box::new(toggle_scratch_py),

        // Session management
        "M-A-l" => spawn("xflock4"),
        "M-A-Escape" => power_menu(),

        "M-C-t" => toggle_sticky_client(),

        // Floating management
        "M-C-f" => float_focused(),
        "M-C-s" => sink_focused(),
        "M-C-S-s" => sink_all(),
        // Floating resize
        "M-C-Right" => resize(DELTA, 0),
        "M-C-Left" => resize(-DELTA, 0),
        "M-C-Up" => resize(0, -DELTA),
        "M-C-Down" => resize(0, DELTA),
        // Floating position
        "M-C-l" => reposition(DELTA, 0),
        "M-C-h" => reposition(-DELTA, 0),
        "M-C-k" => reposition(0, -DELTA),
        "M-C-j" => reposition(0, DELTA),

        // Debugging
        "M-A-t" => set_tracing_filter(handle),
        "M-A-d" => log_current_state(),
    };

    for tag in &["1", "2", "3", "4", "5", "6", "7", "8", "9"] {
        raw_bindings.extend([
            (
                format!("M-{tag}"),
                modify_with(move |client_set| client_set.pull_tag_to_screen(tag)),
            ),
            (
                format!("M-S-{tag}"),
                modify_with(move |client_set| client_set.move_focused_to_tag(tag)),
            ),
        ]);
    }

    raw_bindings
}
