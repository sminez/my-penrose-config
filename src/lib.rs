#![warn(clippy::all)]
#![warn(future_incompatible, rust_2018_idioms)]

#[macro_use]
extern crate penrose;

use penrose::{core::layout::LayoutConf, xcb::XcbConnection, WindowManager};

pub mod actions;
pub mod hooks;

// NOTE: Declaring these as type aliases here so that changing XConn impls at a later date
//       is simply a case of updating these definitions (for the most part)
pub type Conn = XcbConnection;
pub type Wm = WindowManager<Conn>;

#[macro_export]
macro_rules! layout {
    { $name:expr, $func:expr } => {
        Layout::new($name, LayoutConf::default(), $func, penrose_sminez::N_MAIN, penrose_sminez::RATIO)
    };
    { $name:expr, $conf:expr, $func:expr } => {
        Layout::new($name, $conf, $func, penrose_sminez::N_MAIN, penrose_sminez::RATIO)
    };
}

pub const DEBUG_ENV_VAR: &str = "PENROSE_DEBUG";

pub const PROFONT: &str = "ProFont For Powerline";
pub const HEIGHT: usize = 18;

pub const BLACK: &str = "#282828";
pub const WHITE: &str = "#ebdbb2";
pub const GREY: &str = "#3c3836";
pub const BLUE: &str = "#458588";

pub const RATIO: f32 = 0.6;
pub const N_MAIN: u32 = 1;
pub const FOLLOW_FOCUS_CONF: LayoutConf = LayoutConf {
    floating: false,
    gapless: true,
    follow_focus: true,
    allow_wrapping: true,
};

pub const FLOAT_CLASS: &str = "floating";
pub const MON_1: &str = "eDP-1";
pub const MON_2: &str = "HDMI-2";

pub const TERMINAL: &str = "alacritty";
pub const BROWSER: &str = "qutebrowser";
pub const QT_CONSOLE: &str = "jupyter-qtconsole";
