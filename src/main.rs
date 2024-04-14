use clap::Parser;
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{channel, Sender};
use std::time::Instant;
use tokio::net::TcpStream;
use tokio::task;
use tokio::time::{sleep, Duration};

const IP_FALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
const MIN_PORT: u16 = 1;
const MAX_PORT: u16 = 65535;

#[derive(Parser)]
#[command(about = "Blazingly fast port sniffer", long_about=None)]
#[command(version)]
struct Args {
    // IP Address to scan (IPv4 or IPv6)
    #[arg(default_value_t=IP_FALLBACK)]
    ip: IpAddr,
    // Threads to spawn
    #[arg(short, long, default_value_t = 10)]
    threads: u16,
    // Start Port (> 0)
    #[arg(short, long, default_value_t=MIN_PORT)]
    start_port: u16,
    // End Port (<= 66635)
    #[arg(short, long, default_value_t=MAX_PORT)]
    end_port: u16,
}

// Scan the port.
async fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr) {
    // Attempts Connects to the address and the given port.
    if (TcpStream::connect(format!("{}:{}", addr, start_port)).await).is_ok() {
        print!(".");
        io::stdout().flush().unwrap();
        tx.send(start_port).unwrap();
    }

    sleep(Duration::from_millis(1000)).await;
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    // Initialize the channel.
    let (tx, rx) = channel();
    let now = Instant::now();
    // Iterate through all of the ports (based on user input) so that we can spawn a single task for each.
    // (Much faster than before because it uses green threads instead of OS threads.)
    for i in args.start_port..args.end_port {
        let tx = tx.clone();

        task::spawn(async move { scan(tx, i, args.ip).await });
    }
    // Create the vector for all of the outputs.
    let mut out = vec![];
    // Drop the tx clones.
    drop(tx);
    // Wait for all of the outputs to finish and push them into the vector.

    let elapsed = now.elapsed();
    for p in rx {
        out.push(p);
    }

    out.sort();
    println!("\nFinished in {:.2?}", elapsed);
    for v in out {
        // Iterate through the outputs and print them out as being open.
        println!("{} is open", v);
    }
}
