use crate::{
    actions::{power_menu, set_tracing_filter, toggle_sticky_client, DragTerm},
    KeyHandler, MouseHandler,
};
use penrose::{
    builtin::{
        actions::{
            floating::{
                float_all, float_focused, sink_all, sink_focused, MouseDragHandler,
                MouseResizeHandler,
            },
            log_current_state, modify_with, send_layout_message, spawn,
        },
        layout::messages::{ExpandMain, IncMain, ShrinkMain},
    },
    core::bindings::MouseState,
    extensions::hooks::ToggleNamedScratchPad,
    map,
};
use std::collections::HashMap;
use tracing_subscriber::{reload::Handle, EnvFilter};

// Generate a raw key binding map in terms of parsable string key bindings rather than resolved key codes
pub fn raw_key_bindings<L, S>(
    toggle_scratch: ToggleNamedScratchPad,
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
        "M-space" => modify_with(|cs| cs.swap_focus_and_head()),
        "M-C-space" => modify_with(|cs| cs.rotate_focus_to_head()),
        "M-S-q" => modify_with(|cs| cs.kill_focused()),

        // Workspaces
        "M-Tab" => modify_with(|cs| cs.toggle_tag()),
        "M-bracketright" => modify_with(|cs| cs.next_screen()),
        "M-bracketleft" => modify_with(|cs| cs.previous_screen()),
        "M-S-bracketright" => modify_with(|cs| cs.drag_workspace_forward()),
        "M-S-bracketleft" => modify_with(|cs| cs.drag_workspace_backward()),
        "M-Left" => modify_with(|cs| cs.focus_previous_workspace()),
        "M-Right" => modify_with(|cs| cs.focus_next_workspace()),

        // Layouts
        "M-grave" => modify_with(|cs| cs.next_layout()),
        "M-S-grave" => modify_with(|cs| cs.previous_layout()),
        "M-S-Up" => send_layout_message(|| IncMain(1)),
        "M-S-Down" => send_layout_message(|| IncMain(-1)),
        "M-S-Right" => send_layout_message(|| ExpandMain),
        "M-S-Left" => send_layout_message(|| ShrinkMain),

        // Launchers
        "M-A-s" => spawn("flameshot gui"),
        "M-semicolon" => spawn("rofi-apps"),
        "M-Return" => spawn("st"),
        "M-slash" => Box::new(toggle_scratch),

        // Session management
        "M-A-l" => spawn("gnome-screensaver-command --lock"),
        "M-A-Escape" => power_menu(),

        "M-C-t" => toggle_sticky_client(),

        // Floating management
        "M-C-f" => float_focused(),
        "M-C-S-f" => float_all(),
        "M-C-s" => sink_focused(),
        "M-C-S-s" => sink_all(),

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

pub fn mouse_bindings() -> HashMap<MouseState, MouseHandler> {
    use penrose::core::bindings::{
        click_handler,
        ModifierKey::{Ctrl, Meta},
        MouseButton::{Left, Middle, Right},
    };

    map! {
        map_keys: |(button, modifiers)| MouseState { button, modifiers };

        (Left, vec![Meta]) => MouseDragHandler::boxed_default(),
        (Right, vec![Meta]) => MouseResizeHandler::boxed_default(),
        (Middle, vec![Meta]) => click_handler(sink_focused()),
        (Left, vec![Ctrl]) => DragTerm::boxed_default(),
    }
}
