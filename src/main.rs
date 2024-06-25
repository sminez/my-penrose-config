//! My personal penrose config
use anyhow::Context;
use penrose::{
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
    actions::add_sticky_client_state,
    bar::status_bar,
    bindings::{mouse_bindings, raw_key_bindings},
    layouts::{layouts, PerScreenSpacingHook},
    GREY, INNER_PX, OUTER_PX, RED,
};
use tracing::subscriber::set_global_default;
use tracing_subscriber::{layer::SubscriberExt, FmtSubscriber};

use penrose::{
    core::State,
    x::{Atom, Prop, XConn, XEvent},
    Result,
};
pub fn event_hook<X: XConn>(event: &XEvent, _: &mut State<X>, x: &X) -> Result<bool> {
    let unmanaged: [&str; 2] = [
        Atom::NetWindowTypeDock.as_ref(),
        Atom::NetWindowTypeToolbar.as_ref(),
    ];

    if let XEvent::MapRequest(id) = event {
        let p = x.get_prop(*id, Atom::NetWmWindowType.as_ref())?;
        if let Some(Prop::Atom(atoms)) = p {
            if atoms.iter().any(|a| unmanaged.contains(&a.as_ref())) {
                x.map(*id)?;
                return Ok(false);
            }
        };
    }

    Ok(true)
}

fn main() -> anyhow::Result<()> {
    let builder = FmtSubscriber::builder()
        .with_env_filter("info")
        .with_writer(std::io::stdout)
        .with_filter_reloading();

    let reload_handle = builder.reload_handle();
    let journald_layer = tracing_journald::layer().context("unable to open journald socket")?;
    let subscriber = builder.finish().with(journald_layer);

    set_global_default(subscriber).context("unable to set a global tracing subscriber")?;

    let startup_hook = SpawnOnStartup::boxed("/usr/local/scripts/penrose-startup.sh");
    let manage_hook = manage_hooks![
        ClassName("floatTerm") => FloatingCentered::new(0.8, 0.6),
        ClassName("discord")  => SetWorkspace("9"),
    ];
    let layout_hook = PerScreenSpacingHook {
        inner_px: INNER_PX,
        outer_px: OUTER_PX,
    };

    let config = add_ewmh_hooks(Config {
        focused_border: RED.into(),
        normal_border: GREY.into(),
        default_layouts: layouts(),
        floating_classes: vec!["mpv-float".to_owned(), "stalonetray".to_owned()],
        manage_hook: Some(manage_hook),
        startup_hook: Some(startup_hook),
        layout_hook: Some(Box::new(layout_hook)),
        event_hook: Some(Box::new(event_hook)),
        ..Config::default()
    });

    let (nsp, toggle_scratch) = NamedScratchPad::new(
        "terminal",
        "st -c ScratchpadTerm",
        ClassName("ScratchpadTerm"),
        FloatingCentered::new(0.8, 0.8),
        true,
    );

    let conn = RustConn::new()?;
    let raw_bindings = raw_key_bindings(toggle_scratch, reload_handle);
    let key_bindings = parse_keybindings_with_xmodmap(raw_bindings)?;
    let wm = add_sticky_client_state(add_named_scratchpads(
        WindowManager::new(config, key_bindings, mouse_bindings(), conn)?,
        vec![nsp],
    ));

    let bar = status_bar()?;
    let wm = bar.add_to(wm);

    wm.run()?;

    Ok(())
}
