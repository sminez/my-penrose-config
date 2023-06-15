use crate::KeyHandler;
use penrose::{
    builtin::actions::key_handler,
    core::{State, WindowManager},
    custom_error,
    extensions::util::dmenu::{DMenu, DMenuConfig, MenuMatch},
    util::spawn,
    x::{XConn, XConnExt},
    x11rb::RustConn,
    Result, Xid,
};
use std::process::exit;
use tracing::warn;
use tracing_subscriber::{reload::Handle, EnvFilter};

// A dmenu based power menu for common actions
pub fn power_menu() -> KeyHandler {
    key_handler(|state, _| {
        let options = vec!["lock", "logout", "restart-wm", "shutdown", "reboot"];
        let screen_index = state.client_set.current_screen().index();
        let menu = DMenu::new(&DMenuConfig::with_prompt(">>> "), screen_index);

        if let Ok(MenuMatch::Line(_, choice)) = menu.build_menu(options) {
            match choice.as_ref() {
                "lock" => spawn("xflock4"),
                "logout" => spawn("pkill -fi penrose"),
                "shutdown" => spawn("sudo shutdown -h now"),
                "reboot" => spawn("sudo reboot"),
                "restart-wm" => exit(0), // Wrapper script then handles restarting us
                _ => unimplemented!(),
            }
        } else {
            Ok(())
        }
    })
}

const TRACING_DOC_URL: &str =
    "https://docs.rs/tracing-subscriber/0.2.16/tracing_subscriber/filter/struct.EnvFilter.html";

// Use dmenu to set a new tracing filter while penrose is running.
// Syntax for the filters themselves can be found at `doc_url`
pub fn set_tracing_filter<L, S>(handle: Handle<L, S>) -> KeyHandler
where
    L: From<EnvFilter> + 'static,
    S: 'static,
{
    key_handler(move |state, _| {
        let options = vec!["show_docs", "trace", "debug", "info"];
        let screen_index = state.client_set.current_screen().index();
        let menu = DMenu::new(&DMenuConfig::with_prompt("filter: "), screen_index);

        let new_filter = match menu.build_menu(options)? {
            MenuMatch::Line(_, selection) if &selection == "show docs" => {
                return spawn(format!("qutebrowser {}", TRACING_DOC_URL));
            }
            MenuMatch::Line(_, level) => level,
            MenuMatch::UserInput(custom) => custom,
            MenuMatch::NoMatch => return Ok(()),
        };

        warn!(?new_filter, "attempting to update tracing filter");
        let f = new_filter
            .parse::<EnvFilter>()
            .map_err(|e| custom_error!("invalid filter: {}", e))?;
        warn!("reloading tracing handle");
        handle
            .reload(f)
            .map_err(|e| custom_error!("unable to set filter: {}", e))
    })
}

// Run k to view snippets and open a url if one is available
pub fn k_open(float_class: &'static str) -> KeyHandler {
    key_handler(move |_, _| spawn(format!("/usr/local/scripts/k-penrose.sh {float_class}")))
}

struct StickyClientState(Vec<Xid>);

pub fn add_sticky_client_state<X>(mut wm: WindowManager<X>) -> WindowManager<X>
where
    X: XConn + 'static,
{
    wm.state.add_extension(StickyClientState(Vec::new()));
    wm.state.config.compose_or_set_refresh_hook(refresh_hook);

    wm
}

fn refresh_hook<X: XConn>(state: &mut State<X>, x: &X) -> Result<()> {
    let s = state.extension::<StickyClientState>()?;
    let t = state.client_set.current_tag().to_string();
    let mut need_refresh = false;

    // clear out any clients we were tracking that are no longer in state
    s.borrow_mut().0.retain(|id| state.client_set.contains(id));

    for client in s.borrow().0.iter() {
        if state.client_set.tag_for_client(client) != Some(&t) {
            state.client_set.move_client_to_tag(client, &t);
            need_refresh = true;
        }
    }

    // we guard against refreshing only when clients were on the wrong screen
    // so that we don't get into an infinite loop from calling refresh from
    // inside of a refresh hook
    if need_refresh {
        x.refresh(state)?;
    }

    Ok(())
}

pub fn toggle_sticky_client() -> KeyHandler {
    key_handler(|state, x: &RustConn| {
        let _s = state.extension::<StickyClientState>()?;
        let mut s = _s.borrow_mut();

        if let Some(&id) = state.client_set.current_client() {
            if s.0.contains(&id) {
                s.0.retain(|&elem| elem != id);
            } else {
                s.0.push(id);
            }

            drop(s);
            x.refresh(state)?;
        }

        Ok(())
    })
}
