//! A penrose Draw backed implementation of dmenu
use penrose::{
    contrib::extensions::{notify_send, DMenu, DMenuConfig, MenuMatch, NotifyConfig},
    xcb::XcbDraw,
    Result,
};
use penrose_menu::*;

use std::io::{self, Read};

use simplelog::{LevelFilter, SimpleLogger};

fn _pmenu() -> Result<()> {
    if let Err(e) = SimpleLogger::init(LevelFilter::Debug, simplelog::Config::default()) {
        panic!("unable to set log level: {}", e);
    };

    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer).unwrap();
    let lines = buffer.trim().split('\n').map(|s| s.to_string()).collect();

    let drw = XcbDraw::new()?;
    let mut p = PMenu::new(
        drw,
        PMenuConfig {
            show_line_numbers: true,
            ..PMenuConfig::default()
        },
    )?;

    match p.get_selection_from_input("what would you like to do?", lines, 0)? {
        PMenuMatch::Line(i, s) => println!("matched {} on line {}", s, i),
        PMenuMatch::UserInput(s) => println!("user input: {}", s),
        PMenuMatch::NoMatch => println!("no match"),
    }

    Ok(())
}

fn main() -> Result<()> {
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer).unwrap();
    let lines = buffer.trim().split('\n').map(|s| s.to_string()).collect();

    notify_send("hi!", "this should work", NotifyConfig::default())?;

    let menu = DMenu::new(
        ">>>",
        lines,
        DMenuConfig {
            password_input: true,
            ..DMenuConfig::default()
        },
    );

    match menu.run(0)? {
        MenuMatch::Line(i, s) => println!("matched '{}' on line '{}'", s, i),
        MenuMatch::UserInput(s) => println!("user input: '{}'", s),
        MenuMatch::NoMatch => println!("no match"),
    }

    Ok(())
}
