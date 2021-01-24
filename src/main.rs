#![deny(clippy::all)]
/// My personal penrose config (build from the head of develop)

#[macro_use]
extern crate penrose;

#[macro_use]
extern crate penrose_sminez;

use penrose::{
    contrib::{actions::focus_or_spawn, extensions::Scratchpad, hooks::AutoSetMonitorsViaXrandr},
    core::{
        bindings::MouseEvent,
        config::Config,
        data_types::RelativePosition,
        helpers::{index_selectors, spawn},
        hooks::Hooks,
        layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
        ring::Selector,
    },
    draw::{dwm_bar, Color, TextStyle},
    logging_error_handler,
    xcb::{new_xcb_backed_window_manager, XcbDraw},
    Backward, Forward, Less, More, Result,
};

use penrose_sminez::{
    actions::{k_open, power_menu, redetect_monitors, set_log_level},
    hooks::CenterFloat,
    Conn, Wm, BLACK, BLUE, BROWSER, FLOAT_CLASS, FOLLOW_FOCUS_CONF, GREY, HEIGHT, MON_1, MON_2,
    PROFONT, QT_CONSOLE, TERMINAL, WHITE,
};

use std::{convert::TryFrom, env};

fn main() -> Result<()> {
    set_log_level();

    let floating_classes = vec!["rofi", "dmenu", "dunst", "pinentry-gtk-2", FLOAT_CLASS];
    let config = Config::default()
        .builder()
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .floating_classes(floating_classes)
        .layouts(vec![
            layout!("[side]", side_stack),
            layout!("[botm]", bottom_stack),
            layout!("[mono]", FOLLOW_FOCUS_CONF, monocle),
        ])
        .build()
        .unwrap();

    let sp = Scratchpad::new(TERMINAL, 0.8, 0.8);
    let hooks: Hooks<Conn> = vec![
        sp.get_hook(),
        AutoSetMonitorsViaXrandr::new(MON_1, MON_2, RelativePosition::Right),
        CenterFloat::new(FLOAT_CLASS, 0.9),
        Box::new(dwm_bar(
            XcbDraw::new()?,
            HEIGHT,
            &TextStyle {
                font: PROFONT.to_string(),
                point_size: 11,
                fg: Color::try_from(WHITE)?,
                bg: Some(Color::try_from(BLACK)?),
                padding: (2.0, 2.0),
            },
            Color::try_from(BLUE)?, // highlight
            Color::try_from(GREY)?, // empty_ws
            config.workspaces().clone(),
        )?),
    ];

    let key_bindings = gen_keybindings! {
        // Program launch
        "M-semicolon" => run_external!("rofi-apps");
        "M-Return" => run_external!(TERMINAL);
        "M-C-b" => focus_or_spawn(BROWSER, BROWSER);
        "M-C-p" => focus_or_spawn(QT_CONSOLE, QT_CONSOLE);

        // actions
        "M-A-s" => run_external!("screenshot");
        "M-A-l" => run_external!("lock-screen");
        "M-A-m" => redetect_monitors();
        "M-A-Escape" => power_menu();
        "M-slash" => sp.toggle();
        "M-S-slash" => k_open(BROWSER, FLOAT_CLASS);

        // client management
        "M-j" => run_internal!(cycle_client, Forward);
        "M-k" => run_internal!(cycle_client, Backward);
        "M-S-j" => run_internal!(drag_client, Forward);
        "M-S-k" => run_internal!(drag_client, Backward);
        "M-C-bracketleft" => run_internal!(client_to_screen, &Selector::Index(0));
        "M-C-bracketright" => run_internal!(client_to_screen, &Selector::Index(1));
        "M-S-f" => run_internal!(toggle_client_fullscreen, &Selector::Focused);
        "M-S-q" => run_internal!(kill_client);

        // workspace management
        "M-Tab" => run_internal!(toggle_workspace);
        "M-A-period" => run_internal!(cycle_workspace, Forward);
        "M-A-comma" => run_internal!(cycle_workspace, Backward);
        "M-bracketright" => run_internal!(cycle_screen, Forward);
        "M-bracketleft" => run_internal!(cycle_screen, Backward);
        "M-S-bracketright" => run_internal!(drag_workspace, Forward);
        "M-S-bracketleft" => run_internal!(drag_workspace, Backward);

        // Layout & window management
        "M-grave" => run_internal!(cycle_layout, Forward);
        "M-S-grave" => run_internal!(cycle_layout, Backward);
        "M-A-Up" => run_internal!(update_max_main, More);
        "M-A-Down" => run_internal!(update_max_main, Less);
        "M-A-Right" => run_internal!(update_main_ratio, More);
        "M-A-Left" => run_internal!(update_main_ratio, Less);
        "M-A-C-Escape" => run_internal!(exit);

        map: { "1", "2", "3", "4", "5", "6", "7", "8", "9" } to index_selectors(9) => {
            "M-{}" => focus_workspace (REF);
            "M-S-{}" => client_to_workspace (REF);
        };
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut Wm, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut Wm, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    let mut wm = new_xcb_backed_window_manager(config, hooks, logging_error_handler())?;

    let home = env::var("HOME").unwrap();
    spawn(format!("{}/bin/scripts/penrose-startup.sh", home))?;

    wm.grab_keys_and_run(key_bindings, mouse_bindings)
}
