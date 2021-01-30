//! A penrose Draw backed implementation of dmenu with extensions
//!
//! TODO:
//!   - always map above
//!   - color overrides
//!   - password style input
use clap::Clap;
use penrose::{core::helpers::spawn, xcb::XcbDraw, PenroseError, Result};
use penrose_menu::*;

use std::{
    env,
    io::{self, Read},
};

const DEFAULT_FONT: &str = "ProFont For Powerline";

#[derive(Clap)]
#[clap(version = "1.0", name = "pmenu")]
struct Opts {
    /// Show line numbers
    #[clap(short, long)]
    line_numbers: bool,

    /// Maximum number of lines to display at once
    #[clap(short, long, default_value = "10")]
    n_lines: usize,

    /// Font to use
    #[clap(short, long, default_value = DEFAULT_FONT)]
    font: String,

    /// Prompt to use
    #[clap(short, long)]
    prompt: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    /// Run an executable from your $PATH
    Run,
    /// Select an option from stdin
    Select,
    // TODO: sort this out once issues with sending client messages are resolved
    // /// Focus the selected window
    // Windows,
}

pub fn find_executables() -> Result<Vec<String>> {
    let paths = env::var_os("PATH").ok_or(PenroseError::Raw("$PATH not set".into()))?;
    let mut executables = Vec::new();

    for dir in env::split_paths(&paths) {
        for res in dir.read_dir()? {
            let entry = res?;
            let meta = entry.metadata()?;
            if meta.is_file() {
                if let Some(exe) = entry.file_name().to_str().map(String::from) {
                    executables.push(exe)
                }
            }
        }
    }

    executables.sort();
    Ok(executables)
}

fn select(mut p: PMenu<XcbDraw>, opts: Opts) -> Result<()> {
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer).unwrap();
    let lines = buffer.trim().split('\n').map(|s| s.to_string()).collect();

    let res = p.get_selection_from_input(opts.prompt, lines, 0)?;

    if opts.line_numbers {
        if let PMenuMatch::Line(_, s) = res {
            println!("{}", s);
        }
    } else {
        match res {
            PMenuMatch::Line(_, s) => println!("{}", s),
            PMenuMatch::UserInput(s) => println!("{}", s),
            _ => (),
        }
    }

    Ok(())
}

fn run(mut p: PMenu<XcbDraw>, opts: Opts) -> Result<()> {
    let executables = find_executables()?;
    let res = p.get_selection_from_input(opts.prompt, executables, 0)?;
    let prog = match res {
        PMenuMatch::Line(_, s) => s,
        PMenuMatch::UserInput(s) => s,
        _ => return Ok(()),
    };

    spawn(prog)
}

// fn window_name(api: &Api, id: u32) -> Result<String> {
//     match api.get_prop(id, Atom::NetWmName.as_ref()) {
//         Ok(Prop::UTF8String(strs)) if !strs.is_empty() && strs[0].len() > 0 => Ok(strs[0].clone()),
//         _ => match api.get_prop(id, Atom::WmName.as_ref()) {
//             Ok(Prop::UTF8String(strs)) if !strs.is_empty() => Ok(strs[0].clone()),
//             _ => Ok(String::from("???")),
//         },
//     }
// }

// fn window(mut p: PMenu<XcbDraw>, opts: Opts) -> Result<()> {
//     let api = Api::new()?;

//     let ids = match api.current_clients() {
//         Err(_) => Vec::new(),
//         Ok(ids) => ids
//             .into_iter()
//             .filter(|&id| api.window_is_managed(id))
//             .collect(),
//     };

//     let names: Vec<String> = ids
//         .iter()
//         .map(|&id| window_name(&api, id))
//         .collect::<Result<Vec<_>>>()?;

//     let res = p.get_selection_from_input(opts.prompt, names, 0);
//     if let Ok(PMenuMatch::Line(i, _)) = res {
//         let id = ids[i];
//         api.mark_focused_window(id);
//         api.flush();
//     }

//     Ok(())
// }

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let drw = XcbDraw::new()?;

    let p = PMenu::new(
        drw,
        PMenuConfig {
            show_line_numbers: opts.line_numbers,
            n_lines: opts.n_lines,
            font: opts.font.clone(),
            ..PMenuConfig::default()
        },
    )?;

    match opts.subcmd {
        // SubCommand::Windows => window(p, opts),
        SubCommand::Select => select(p, opts),
        SubCommand::Run => run(p, opts),
    }
}
