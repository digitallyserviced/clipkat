use snafu::{ResultExt, Snafu};
use structopt::StructOpt;
use tokio::runtime::Runtime;

use clipcat::{ClipboardData, ClipboardMonitor, ClipboardMonitorOptions, ClipboardType};

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not initialize tokio Runtime, error: {}", source))]
    InitializeTokioRuntime { source: std::io::Error },

    #[snafu(display("Could not create ClipboardMonitor, error: {}", source))]
    InitializeClipboardMonitor { source: clipcat::ClipboardError },

    #[snafu(display("Could not wait for clipboard event"))]
    WaitForClipboardEvent,

    #[snafu(display("Nothing to be monitored"))]
    MonitorNothing,
}

#[derive(Debug, StructOpt)]
#[structopt(name = clipcat::NOTIFY_PROGRAM_NAME)]
struct Command {
    #[structopt(subcommand)]
    subcommand: Option<SubCommand>,

    #[structopt(long = "id", help = "Print the ID of the clipboard entry")]
    show_id: bool,
    
    #[structopt(long = "content", help = "Print the content of the clipboard entry on a separate line from the id")]
    show_content: bool,

    #[structopt(long = "no-clipboard", help = "Does not monitor clipboard")]
    no_clipboard: bool,

    #[structopt(long = "no-primary", help = "Does not monitor primary")]
    no_primary: bool,

    #[structopt(long = "filter-min-size", help = "filter minimum sizes", default_value="1")]
    filter_min_size: usize,
}

#[derive(Debug, StructOpt)]
enum SubCommand {
    #[structopt(about = "Prints version information")]
    Version,

    #[structopt(about = "Outputs shell completion code for the specified shell (bash, zsh, fish)")]
    Completions { shell: structopt::clap::Shell },
}

impl Command {
    fn run(self) -> Result<(), Error> {
        match self.subcommand {
            Some(SubCommand::Version) => {
                Self::clap()
                    .write_long_version(&mut std::io::stdout())
                    .expect("Failed t write to stdout");
                return Ok(());
            }
            Some(SubCommand::Completions { shell }) => {
                Self::clap().gen_completions_to(
                    clipcat::NOTIFY_PROGRAM_NAME,
                    shell,
                    &mut std::io::stdout(),
                );
                return Ok(());
            }
            None => {}
        }

        let filter_min_size = self.filter_min_size;
        let show_id = self.show_id;
        let show_content = self.show_content;

        let enable_clipboard = !self.no_clipboard;
        let enable_primary = !self.no_primary;

        if !enable_clipboard && !enable_primary {
            return Err(Error::MonitorNothing);
        }

        let monitor_opts = ClipboardMonitorOptions {
            load_current: false,
            enable_clipboard,
            enable_primary,
            filter_min_size,
        };
        let monitor = ClipboardMonitor::new(monitor_opts).context(InitializeClipboardMonitor)?;
        let runtime = Runtime::new().context(InitializeTokioRuntime)?;
        runtime.block_on(async {
            let mut event_recv = monitor.subscribe();
            while let Ok(event) = event_recv.recv().await {
                match event.clipboard_type {
                    ClipboardType::Clipboard if enable_clipboard => {
                        if show_id {
                            println!("{:016x}", ClipboardData::compute_id(&event.data));
                        }
                        if show_content {
                            println!("{}", event.data);
                        }
                        continue
                    },
                    ClipboardType::Primary if enable_primary => {
                        if show_id {
                            println!("{:016x}", ClipboardData::compute_id(&event.data));
                        }
                        if show_content {
                            println!("{}", event.data);
                        }
                        continue
                    },
                    _ => continue,
                }
            }

            Err(Error::WaitForClipboardEvent)
        })?;

        Ok(())
    }
}

fn main() {
    let cmd = Command::from_args();
    if let Err(err) = cmd.run() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
