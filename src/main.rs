use i3ipc::{
    event::{inner::WindowChange, Event},
    reply::Node,
    I3Connection, I3EventListener, Subscription,
};

use daemonize::Daemonize;
use exitfailure::ExitFailure;
use failure::Error;
use signal_hook::{iterator::Signals, SIGHUP, SIGINT, SIGQUIT, SIGTERM};
use structopt::{
    clap::AppSettings::{ArgRequiredElseHelp, ColoredHelp},
    StructOpt,
};

use std::{process::exit, thread};

type NodeID = i64;

fn set_opacity(ipc: &mut I3Connection, node_id: NodeID, opacity: f32) -> Result<(), Error> {
    let command = format!("[con_id=\"{}\"] opacity {}", node_id, opacity);
    ipc.run_command(&command)?;
    Ok(())
}

fn find_focused(
    ipc: &mut I3Connection,
    root: Node,
    non_focused_opacity: f32,
) -> Result<Option<NodeID>, Error> {
    let mut focused = if root.focused {
        Some(root.id)
    } else {
        set_opacity(ipc, root.id, non_focused_opacity)?;
        None
    };

    for node in root.nodes {
        if let Some(id) = find_focused(ipc, node, non_focused_opacity)? {
            focused = Some(id);
        }
    }

    Ok(focused)
}

fn handle_signals() -> Result<(), Error> {
    let mut ipc = I3Connection::connect()?;
    let signals = Signals::new(&[SIGHUP, SIGINT, SIGQUIT, SIGTERM])?;

    signals.forever().next();
    let root = ipc.get_tree()?;
    find_focused(&mut ipc, root, 1.0)?;
    exit(0);
}

#[derive(StructOpt)]
#[structopt(raw(settings = "&[ColoredHelp, ArgRequiredElseHelp]"))]
struct Cli {
    /// Opacity to be used for non focused windows (in interval 0..1)
    opacity: f32,

    /// Run in the background.
    #[structopt(short, long)]
    daemonize: bool,
}

fn main() -> Result<(), ExitFailure> {
    let cli = Cli::from_args();

    if cli.daemonize {
        Daemonize::new().start()?;
    }

    let mut ipc = I3Connection::connect()?;

    let root = ipc.get_tree()?;
    let mut last_focused_id = find_focused(&mut ipc, root, cli.opacity)?
        .expect("Expected at least one window to be initially focused.");

    let mut listener = I3EventListener::connect()?;
    listener.subscribe(&[Subscription::Window])?;

    thread::spawn(handle_signals);

    for event in listener.listen() {
        if let Event::WindowEvent(info) = event? {
            if let WindowChange::Focus = info.change {
                let focused = info.container;
                set_opacity(&mut ipc, last_focused_id, cli.opacity)?;
                set_opacity(&mut ipc, focused.id, 1.0)?;
                last_focused_id = focused.id;
            }
        }
    }

    Ok(())
}
