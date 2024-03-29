//! My personal penrose config
use penrose::{
    builtin::hooks::SpacingHook,
    core::{bindings::parse_keybindings_with_xmodmap, Config, WindowManager},
    extensions::hooks::{
        add_ewmh_hooks, add_named_scratchpads,
        manage::{FloatingCentered, SetWorkspace},
        NamedScratchPad, SpawnOnStartup,
    },
    manage_hooks,
    x::query::ClassName,
    x11rb::RustConn,
};
use penrose_sminez::{
    actions::add_sticky_client_state, bar::status_bar, bindings::raw_key_bindings,
    layouts::layouts, BAR_HEIGHT_PX, INNER_PX, OUTER_PX,
};
use std::collections::HashMap;
use tracing_subscriber::{self, prelude::*};

fn main() -> anyhow::Result<()> {
    // NOTE: Setting up tracing with dynamic filter updating inline as getting the type for
    // the reload Handle to work is a massive pain... this really should be in its own method
    // somewhere as the example here: https://github.com/tokio-rs/tracing/blob/master/examples/examples/tower-load.rs
    // _really_ seems to show that Handle only has a single type param, but when I try it in here
    // it complains about needing a second (phantom data) param as well?
    let tracing_builder = tracing_subscriber::fmt()
        // .json() // JSON logs
        // .flatten_event(true)
        .with_env_filter("info")
        .with_filter_reloading();

    let reload_handle = tracing_builder.reload_handle();
    tracing_builder.finish().init();

    let startup_hook = SpawnOnStartup::boxed("/usr/local/scripts/penrose-startup.sh");
    let manage_hook = manage_hooks![
        ClassName("floatTerm") => FloatingCentered::new(0.8, 0.6),
        ClassName("discord")  => SetWorkspace("9"),
    ];
    let layout_hook = SpacingHook {
        inner_px: INNER_PX,
        outer_px: OUTER_PX,
        top_px: BAR_HEIGHT_PX,
        bottom_px: 0,
    };

    let config = add_ewmh_hooks(Config {
        default_layouts: layouts(),
        floating_classes: vec!["mpv-float".to_owned()],
        manage_hook: Some(manage_hook),
        startup_hook: Some(startup_hook),
        layout_hook: Some(Box::new(layout_hook)),
        ..Config::default()
    });

    // Create a new named scratchpad and toggle handle for use in keybindings.
    let (nsp, toggle_scratch) = NamedScratchPad::new(
        "terminal",
        "st -c StScratchpad",
        ClassName("StScratchpad"),
        FloatingCentered::new(0.8, 0.8),
        true,
    );

    let (nsp_py, toggle_scratch_py) = NamedScratchPad::new(
        "qt-console",
        "jupyter-qtconsole",
        ClassName("jupyter-qtconsole"),
        FloatingCentered::new(0.8, 0.8),
        true,
    );

    let conn = RustConn::new()?;
    let raw_bindings = raw_key_bindings(toggle_scratch, toggle_scratch_py, reload_handle);
    let key_bindings = parse_keybindings_with_xmodmap(raw_bindings)?;

    // Initialise the required state extension and hooks for handling the named scratchpad
    let wm = add_sticky_client_state(add_named_scratchpads(
        WindowManager::new(config, key_bindings, HashMap::new(), conn)?,
        vec![nsp, nsp_py],
    ));

    let bar = status_bar()?;
    let wm = bar.add_to(wm);

    wm.run()?;

    Ok(())
}
