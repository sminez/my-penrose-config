use crate::KeyHandler;
use penrose::{
    core::actions::key_handler,
    custom_error,
    extensions::util::dmenu::{DMenu, DMenuConfig, MenuMatch},
    util::spawn,
};
use std::process::exit;
use tracing::warn;
use tracing_subscriber::{reload::Handle, EnvFilter};

// A dmenu based power menu for common actions
pub fn power_menu() -> KeyHandler {
    key_handler(|state, _| {
        let options = vec!["lock", "logout", "restart-wm", "shutdown", "reboot"];
        let menu = DMenu::new(">>> ", options, DMenuConfig::default());
        let screen_index = state.client_set.current_screen().index();

        if let Ok(MenuMatch::Line(_, choice)) = menu.run(screen_index) {
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
        let menu = DMenu::new("filter: ", options, DMenuConfig::default());
        let screen_index = state.client_set.current_screen().index();

        let new_filter = match menu.run(screen_index)? {
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
