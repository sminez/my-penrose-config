use penrose::{
    contrib::{actions::update_monitors_via_xrandr, extensions::dmenu::*},
    core::{bindings::KeyEventHandler, data_types::RelativePosition},
};

// use std::thread;

use crate::{Conn, Wm, MON_1, MON_2};

// A dmenu based power menu for common actions
pub fn power_menu() -> KeyEventHandler<Conn> {
    Box::new(move |wm: &mut Wm| {
        let options = vec!["lock", "logout", "restart-wm", "shutdown", "reboot"];
        let menu = DMenu::new(">>> ", options, DMenuConfig::default());
        let screen_index = wm.active_screen_index();

        if let Ok(MenuMatch::Line(_, choice)) = menu.run(screen_index) {
            match choice.as_ref() {
                "lock" => spawn!("dm-tool switch-to-greeter"),
                "logout" => spawn!("pkill -fi penrose"),
                "shutdown" => spawn!("sudo shutdown -h now"),
                "reboot" => spawn!("sudo reboot"),
                "restart-wm" => wm.exit(),
                _ => unimplemented!(),
            }
        } else {
            Ok(())
        }
    })
}

// Force redetection of monitors
pub fn redetect_monitors() -> KeyEventHandler<Conn> {
    Box::new(move |_: &mut Wm| update_monitors_via_xrandr(MON_1, MON_2, RelativePosition::Right))
}

// Run k to view snippets and open a url if one is available
pub fn k_open(float_class: &'static str) -> KeyEventHandler<Conn> {
    Box::new(move |_: &mut Wm| {
        // thread::spawn(move || spawn!("/usr/local/scripts/k-penrose.sh", float_class));
        spawn!("/usr/local/scripts/k-penrose.sh", float_class)
    })
}
