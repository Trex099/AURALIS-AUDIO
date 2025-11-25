use clap::Parser;
use anyhow::Result;
use auralis_core::PipeWireClient;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    list: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args = Args::parse();

    if args.list {
        println!("Initializing PipeWire Client...");
        let (tx, rx) = std::sync::mpsc::channel();
        let (_cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let _client = PipeWireClient::new(tx, cmd_rx)?;
        
        println!("Listening for events (Ctrl+C to stop)...");
        // Keep the main thread alive to let the background thread run
        loop {
            if let Ok(event) = rx.try_recv() {
                println!("Discovered Event: {:?}", event);
            }
            thread::sleep(Duration::from_millis(100));
        }
    }

    Ok(())
}
