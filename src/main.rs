#![deny(clippy::all)]
/**
 * My personal penrose config (build from the head of develop)
 */

#[macro_use]
extern crate penrose;

use penrose::{
    contrib::extensions::Scratchpad,
    core::{
        bindings::MouseEvent,
        helpers::{index_selectors, spawn, spawn_for_output},
        layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
        manager::WindowManager,
    },
    draw::{dwm_bar, TextStyle},
    xcb::{new_xcb_connection, XcbDraw},
    Backward, Config, Forward, Less, More, Result, Selector,
};
use simplelog::{LevelFilter, SimpleLogger};
use std::env;

const HEIGHT: usize = 18;
const PROFONT: &str = "ProFont For Powerline";

const BLACK: u32 = 0x282828ff;
const GREY: u32 = 0x3c3836ff;
const WHITE: u32 = 0xebdbb2ff;
const BLUE: u32 = 0x458588ff;

fn main() -> Result<()> {
    // -- logging --
    SimpleLogger::init(LevelFilter::Info, simplelog::Config::default())?;

    let mut config = Config::default();

    // -- top level config constants --
    config.workspaces = vec!["1", "2", "3", "4", "5", "6", "7", "8", "9"];
    config.floating_classes = &["rofi", "dmenu", "dunst", "polybar", "pinentry-gtk-2"];

    // -- hooks --
    let sp = Scratchpad::new("st", 0.8, 0.8);
    sp.register(&mut config);

    config.hooks.push(Box::new(dwm_bar(
        Box::new(XcbDraw::new()?),
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
        &config.workspaces,
    )?));

    // -- layouts --
    let follow_focus_conf = LayoutConf {
        floating: false,
        gapless: true,
        follow_focus: true,
        allow_wrapping: true,
    };
    let n_main = 1;
    let ratio = 0.6;
    config.layouts = vec![
        Layout::new("[side]", LayoutConf::default(), side_stack, n_main, ratio),
        Layout::new("[botm]", LayoutConf::default(), bottom_stack, n_main, ratio),
        Layout::new("[mono]", follow_focus_conf, monocle, n_main, ratio),
    ];

    // -- bindings --
    let home = env::var("HOME").unwrap();
    let script = format!("{}/bin/scripts/power-menu.sh", home);
    let power_menu = Box::new(move |wm: &mut WindowManager| {
        if let Ok(o) = spawn_for_output(&script) {
            if o.as_str() == "restart-wm\n" {
                wm.exit();
            }
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
        "M-A-m" => run_external!("xrandr --output HDMI-1 --auto --right-of eDP-1 ");
        "M-A-d" => run_internal!(detect_screens);
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
        Press Right + [Meta] => |wm: &mut WindowManager, _: &MouseEvent| wm.cycle_workspace(Forward),
        Press Left + [Meta] => |wm: &mut WindowManager, _: &MouseEvent| wm.cycle_workspace(Backward)
    };

    // -- init & run --
    let conn = new_xcb_connection()?;
    let mut wm = WindowManager::init(config, &conn);

    spawn(format!("{}/bin/scripts/penrose-startup.sh", home));
    wm.grab_keys_and_run(key_bindings, mouse_bindings);

    Ok(())
}
