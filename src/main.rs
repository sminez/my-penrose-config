#![deny(clippy::all)]
/**
 * My personal penrose config (build from the head of develop)
 */

#[macro_use]
extern crate penrose;

#[macro_use]
extern crate log;

use penrose::{
    contrib::extensions::Scratchpad,
    core::{
        bindings::MouseEvent,
        config::Config,
        helpers::{index_selectors, spawn, spawn_for_output},
        hooks::Hooks,
        layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
        manager::WindowManager,
        ring::Selector,
    },
    draw::{dwm_bar, TextStyle},
    xcb::{new_xcb_backed_window_manager, Api, XcbConnection, XcbDraw},
    Backward, Forward, Less, More, Result,
};
use penrose_sminez::actions::{update_monitors_via_xrandr, XrandrMonitorPosition::Right};
use simplelog::{LevelFilter, SimpleLogger};
use std::env;

const DEBUG_ENV_VAR: &str = "PENROSE_DEBUG";

const PROFONT: &str = "ProFont For Powerline";
const HEIGHT: usize = 18;

const BLACK: u32 = 0x282828ff;
const WHITE: u32 = 0xebdbb2ff;
const GREY: u32 = 0x3c3836ff;
const BLUE: u32 = 0x458588ff;

const RATIO: f32 = 0.6;
const N_MAIN: u32 = 1;
const FOLLOW_FOCUS_CONF: LayoutConf = LayoutConf {
    floating: false,
    gapless: true,
    follow_focus: true,
    allow_wrapping: true,
};

// NOTE: Declaring these as type aliases here so that changing XConn impls at a later date
//       is simply a case of updating these definitions (for the most part)
type Conn = XcbConnection<Api>;
type Wm = WindowManager<Conn>;

// Helper function

fn set_log_level() {
    let log_level = match env::var(DEBUG_ENV_VAR) {
        Ok(val) if &val == "true" => LevelFilter::Debug,
        _ => LevelFilter::Info,
    };
    if let Err(e) = SimpleLogger::init(log_level, simplelog::Config::default()) {
        panic!("unable to set log level: {}", e);
    };
}

fn main() -> Result<()> {
    set_log_level();

    let mut config = Config::default();
    config
        .workspaces(vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"])
        .floating_classes(vec!["rofi", "dmenu", "dunst", "polybar", "pinentry-gtk-2"])
        .layouts(vec![
            Layout::new("[side]", LayoutConf::default(), side_stack, N_MAIN, RATIO),
            Layout::new("[botm]", LayoutConf::default(), bottom_stack, N_MAIN, RATIO),
            Layout::new("[mono]", FOLLOW_FOCUS_CONF, monocle, N_MAIN, RATIO),
        ]);

    let sp = Scratchpad::new("st", 0.8, 0.8);
    let hooks: Hooks<Conn> = vec![
        sp.get_hook(),
        Box::new(dwm_bar(
            XcbDraw::new()?,
            HEIGHT,
            &TextStyle {
                font: PROFONT.to_string(),
                point_size: 11,
                fg: WHITE.into(),
                bg: Some(BLACK.into()),
                padding: (2.0, 2.0),
            },
            BLUE, // highlight
            GREY, // empty_ws
            config.workspaces.clone(),
        )?),
    ];

    let home = env::var("HOME").unwrap();
    let script = format!("{}/bin/scripts/power-menu.sh", home);

    let power_menu = Box::new(move |wm: &mut Wm| {
        if let Ok(o) = spawn_for_output(&script) {
            if o.as_str() == "restart-wm\n" {
                wm.exit();
            }
        }
    });

    let redetect_monitors = Box::new(move |_: &mut Wm| {
        spawn("updating screens");
        if let Err(e) = update_monitors_via_xrandr("eDP-1", "HDMI-2", Right) {
            error!("unable to set monitors via xrandr: {}", e);
        }
    });

    let key_bindings = gen_keybindings! {
        // Program launch
        "M-semicolon" => run_external!("rofi-apps");
        "M-Return" => run_external!("st");

        // actions
        "M-A-s" => run_external!("screenshot");
        "M-A-k" => run_external!("toggle-kb-for-tada");
        "M-A-l" => run_external!("lock-screen");
        "M-A-m" => redetect_monitors;
        "M-slash" => sp.toggle();

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
        "M-A-Escape" => power_menu;

        refmap [ config.ws_range() ] in {
            "M-{}" => focus_workspace [ index_selectors(config.workspaces.len()) ];
            "M-S-{}" => client_to_workspace [ index_selectors(config.workspaces.len()) ];
        };
    };

    let mouse_bindings = gen_mousebindings! {
        Press Right + [Meta] => |wm: &mut Wm, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut Wm, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    let mut wm = new_xcb_backed_window_manager(config, hooks)?;

    spawn(format!("{}/bin/scripts/penrose-startup.sh", home));
    wm.grab_keys_and_run(key_bindings, mouse_bindings);

    Ok(())
}
