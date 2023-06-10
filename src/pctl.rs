//! A CLI for interacting with and debugging X11 related issues around penrose
use clap::{Parser, Subcommand};
use penrose::{
    x::{XConn, XConnExt},
    x11rb::RustConn,
};

fn kv_bold(k: &str, v: impl std::fmt::Debug) {
    println!("\\033[33m\\033[1m{k}:\\033[0m {v:?}")
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Dump out the query tree of the penrose root window
    DumpTree {
        /// Skip listing properties for detected clients
        #[arg(long, default_value = "false")]
        no_props: bool,
    },
}

/// Utility commands for penrose
#[derive(Debug, Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::DumpTree { no_props } => dump_tree(!no_props)?,
    }

    Ok(())
}

fn dump_tree(include_props: bool) -> anyhow::Result<()> {
    let conn = RustConn::new()?;

    for c in conn.existing_clients()? {
        let attrs = conn.get_window_attributes(c)?;
        let title = conn.window_title(c)?;
        let state = conn.get_wm_state(c)?;

        kv_bold("Xid", c);
        kv_bold("Title", title);
        kv_bold("WmState", state);
        kv_bold("Attrs", attrs);

        if include_props {
            kv_bold("Props:", "");
            for (k, v) in conn.all_props_for(c)? {
                println!("  {k}: {v:?}");
            }
        }

        println!();
    }

    Ok(())
}
