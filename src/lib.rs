#![warn(clippy::all)]
#![warn(future_incompatible, rust_2024_compatibility)]
use penrose::{core::bindings::KeyEventHandler, x11rb::RustConn};

pub mod actions;
pub mod bar;
pub mod bindings;
pub mod layouts;
pub mod schedule;

pub type KeyHandler = Box<dyn KeyEventHandler<RustConn>>;

pub const FONT: &str = "ProFontIIx Nerd Font";

// Gruvbox
// pub const BLACK: u32 = 0x282828ff; // #282828
// pub const WHITE: u32 = 0xebdbb2ff; // #ebdbb2
// pub const GREY: u32 = 0x3c3836ff; //  #3c3836
// pub const BLUE: u32 = 0x458588ff; //  #458588

// Kanagawa
// https://github.com/rebelot/kanagawa.nvim?tab=readme-ov-file#color-palette
pub const BLACK: u32 = 0x252535ff; // #252535
pub const WHITE: u32 = 0xdcd7baff; // #dcd7ba
pub const GREY: u32 = 0x363646ff; //  #363646
pub const BLUE: u32 = 0x658594ff; //  #658594
pub const RED: u32 = 0xc34043ff; //   #C34043

pub const MAX_MAIN: u32 = 1;
pub const RATIO: f32 = 0.6;
pub const RATIO_STEP: f32 = 0.1;
pub const OUTER_PX: u32 = 5;
pub const INNER_PX: u32 = 5;

pub const DEBUG_ENV_VAR: &str = "PENROSE_DEBUG";

pub const MON_1: &str = "eDP-1";
pub const MON_2: &str = "HDMI-2";
