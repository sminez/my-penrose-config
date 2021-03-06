#![deny(clippy::all, rust_2018_idioms)]
/// My personal penrose config (build from the head of develop)

#[macro_use]
extern crate penrose;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate penrose_sminez;

use penrose::{
    contrib::{
        actions::focus_or_spawn,
        extensions::{dmenu::*, Scratchpad},
        hooks::{AutoSetMonitorsViaXrandr, ManageExistingClients},
    },
    core::{
        bindings::{KeyEventHandler, MouseEvent},
        config::Config,
        data_types::RelativePosition,
        helpers::index_selectors,
        hooks::Hooks,
        layout::{bottom_stack, monocle, side_stack, Layout, LayoutConf},
        ring::Selector,
    },
    draw::{dwm_bar, Color, TextStyle},
    logging_error_handler,
    xcb::{new_xcb_backed_window_manager, XcbDraw},
    Backward, Forward, Less, More, Result,
};
use tracing_subscriber::{self, filter::EnvFilter, prelude::*};

use penrose_sminez::{
    actions::{k_open, power_menu, redetect_monitors},
    hooks::{CenterFloat, StartupScript},
    Conn, Wm, BLACK, BLUE, BROWSER, FLOAT_CLASS, FOLLOW_FOCUS_CONF, FONT, GREY, HEIGHT, MON_1,
    MON_2, QT_CONSOLE, TERMINAL, WHITE,
};

use std::convert::TryFrom;

fn main() -> Result<()> {
    // NOTE: Setting up tracing with dynamic filter updating inline as getting the type for
    // the reload Handle to work is a massive pain... this really should be in its own method
    // somewhere as the example here: https://github.com/tokio-rs/tracing/blob/master/examples/examples/tower-load.rs
    // _really_ seems to show that Handle only has a single type param, but when I try it in here
    // it complains about needing a second (phantom data) param as well?
    let tracing_builder = tracing_subscriber::fmt()
        .json() // JSON logs
        .flatten_event(true)
        .with_env_filter("info")
        .with_filter_reloading();

    let reload_handle = tracing_builder.reload_handle();
    tracing_builder.finish().init();

    // Use dmenu to set a new tracing filter while penrose is running.
    // Syntax for the filters themselves can be found at `doc_url`
    let set_trace_filter = Box::new(move |wm: &mut Wm| {
        let options = vec!["show_docs", "trace", "debug", "info"];
        let doc_url = "https://docs.rs/tracing-subscriber/0.2.16/tracing_subscriber/filter/struct.EnvFilter.html";
        let menu = DMenu::new("filter: ", options, DMenuConfig::default());
        let new_filter = match menu.run(wm.active_screen_index())? {
            MenuMatch::Line(_, selection) if &selection == "show docs" => {
                return spawn!(format!("qutebrowser {}", doc_url));
            }
            MenuMatch::Line(_, level) => level,
            MenuMatch::UserInput(custom) => custom,
            MenuMatch::NoMatch => return Ok(()),
        };

        warn!(?new_filter, "attempting to update tracing filter");
        let f = new_filter
            .parse::<EnvFilter>()
            .map_err(|e| perror!("invalid filter: {}", e))?;
        warn!("reloading tracing handle");
        reload_handle
            .reload(f)
            .map_err(|e| perror!("unable to set filter: {}", e))
    }) as KeyEventHandler<Conn>;

    // Now build the rest of the config

    let floating_classes = vec![
        "rofi",
        "penrose-menu",
        "dmenu",
        "dunst",
        "pinentry-gtk-2",
        FLOAT_CLASS,
    ];
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

    let bar = dwm_bar(
        XcbDraw::new()?,
        HEIGHT,
        &TextStyle {
            font: FONT.to_string(),
            point_size: 8,
            fg: Color::try_from(WHITE)?,
            bg: Some(Color::try_from(BLACK)?),
            padding: (2.0, 2.0),
        },
        Color::try_from(BLUE)?, // highlight
        Color::try_from(GREY)?, // empty_ws
        config.workspaces().clone(),
    )?;
    // bar.widgets.push(Box::new(StaloneTray::new(18, 0x282828)?));

    let hooks: Hooks<Conn> = vec![
        ManageExistingClients::new(),
        AutoSetMonitorsViaXrandr::new(MON_1, MON_2, RelativePosition::Right),
        CenterFloat::new(FLOAT_CLASS, 0.9),
        sp.get_hook(),
        Box::new(bar),
        Box::new(StartupScript::new("/usr/local/scripts/penrose-startup.sh")),
    ];

    let key_bindings = gen_keybindings! {
        // Program launch
        "M-semicolon" => run_external!("rofi-apps");
        "M-Return" => run_external!(TERMINAL);
        "M-C-b" => focus_or_spawn(BROWSER, BROWSER);
        "M-C-p" => focus_or_spawn(QT_CONSOLE, QT_CONSOLE);

        // actions
        "M-A-s" => run_external!("screenshot");
        "M-A-l" => run_external!("dm-tool switch-to-greeter");
        "M-A-m" => redetect_monitors();
        "M-A-t" => set_trace_filter;
        "M-A-Escape" => power_menu();
        "M-slash" => sp.toggle();
        "M-S-slash" => k_open(FLOAT_CLASS);

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
    wm.grab_keys_and_run(key_bindings, mouse_bindings)
}
