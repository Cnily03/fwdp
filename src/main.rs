use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::net::{TcpListener, TcpStream};
pub mod logger;

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"))]
#[command(about = env!("CARGO_PKG_DESCRIPTION"))]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(author = env!("CARGO_PKG_AUTHORS"))]
struct Args {
    /// Target address to forward traffic to (ip:port)
    target: String,

    /// Listen address and port. Can be just a port (defaults to 0.0.0.0:port) or ip:port
    #[arg(short = 'L', long = "listen")]
    listen: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse target address
    let target_addr: SocketAddr = args
        .target
        .parse()
        .with_context(|| format!("invalid target address: {}", args.target))?;

    // Parse listen address
    let listen_addr: SocketAddr = parse_listen_address(&args.listen)
        .with_context(|| format!("invalid listen address: {}", args.listen))?;

    // Create TCP listener
    let listener = TcpListener::bind(listen_addr)
        .await
        .with_context(|| format!("failed to bind to {}", listen_addr))?;

    println!(
        "{} {} -> {}",
        "forward:".blue().bold(),
        listen_addr,
        target_addr
    );

    println!(
        "{} continuously recv listening on {}",
        "*".blue().bold(),
        listen_addr
    );

    let counter = AtomicU64::new(1);

    // Accept connections and handle them
    while let Ok((client_stream, client_addr)) = listener.accept().await {
        let id = counter.fetch_add(1, Ordering::SeqCst);
        record!([id], "{} new connection from {}", "+".green(), client_addr);

        let target_addr = target_addr.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(id, client_stream, target_addr).await {
                error!(
                    [id],
                    "error handling connection from {}: {}", client_addr, e
                );
            } else {
                record!([id], "{} connection from {} closed", "-".red(), client_addr);
            }
        });
    }

    Ok(())
}

fn parse_listen_address(listen: &str) -> Result<SocketAddr> {
    // If it contains a colon, treat it as ip:port
    if listen.contains(':') {
        listen.parse().with_context(|| "failed to parse as ip:port")
    } else {
        // Otherwise, treat it as just a port number and default to 0.0.0.0
        let port: u16 = listen
            .parse()
            .with_context(|| "Failed to parse port number")?;
        // warn!("no ip address specified, defaulting to 0.0.0.0:{}", port);
        Ok(SocketAddr::from(([0, 0, 0, 0], port)))
    }
}

macro_rules! fmt_addr_forward {
    ($id:expr, ($client_addr:expr, >>>, $target_addr:expr)) => {{
        use colored::*;
        // let pad_len = std::cmp::max($client_addr.len(), $target_addr.len());
        // let left_padded = format!("{:<width$}", $client_addr, width = pad_len);
        // let right_padded = format!("{:>width$}", $target_addr, width = pad_len);
        let left_padded = format!("{}", $client_addr);
        let right_padded = format!("{}", $target_addr);
        let left_padded = if $client_addr == "unknown" {
            left_padded.red().bold()
        } else {
            $crate::color_id!($id => "{}", left_padded).bold()
        };
        let right_padded = if $target_addr == "unknown" {
            right_padded.red().dimmed().bold()
        } else {
            right_padded.bright_black().bold()
        };

        format!("{} {} {}", left_padded, ">>>".blue(), right_padded)
    }};
    ($id:expr, ($client_addr:expr, <<<, $target_addr:expr)) => {{
        use colored::*;
        // let pad_len = std::cmp::max($client_addr.len(), $target_addr.len());
        // let left_padded = format!("{:<width$}", $client_addr, width = pad_len);
        // let right_padded = format!("{:>width$}", $target_addr, width = pad_len);
        let left_padded = format!("{}", $client_addr);
        let right_padded = format!("{}", $target_addr);
        let left_padded = if $target_addr == "unknown" {
            left_padded.red().dimmed().bold()
        } else {
            left_padded.bright_black().bold()
        };
        let right_padded = if $client_addr == "unknown" {
            right_padded.red().bold()
        } else {
            $crate::color_id!($id => "{}", right_padded).bold()
        };

        format!("{} {} {}", left_padded, "<<<".cyan(), right_padded)
    }};
}

macro_rules! copy_and_record {
    // r => w, a function to call, input bytes read
    ($reader:expr => $writer:expr, $callback:expr) => {{
        async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let mut buf = [0u8; 8192]; // 8KB buffer
            let mut total_bytes = 0u64;
            loop {
                let bytes_read = $reader.read(&mut buf).await?;
                if bytes_read == 0 {
                    break; // EOF
                }

                $writer.write_all(&buf[..bytes_read]).await?;
                total_bytes += bytes_read as u64;

                // Call callback for each packet
                $callback(bytes_read);
            }

            Ok::<u64, anyhow::Error>(total_bytes)
        }
    }};
}

async fn handle_connection(
    id: u64,
    mut client_stream: TcpStream,
    target_addr: SocketAddr,
) -> Result<()> {
    // Connect to the target server
    let mut target_stream = TcpStream::connect(target_addr)
        .await
        .with_context(|| format!("failed to connect to target {}", target_addr))?;

    let client_addr_str = extract_addr(&client_stream);
    let target_addr_str = extract_addr(&target_stream);

    let (mut client_read, mut client_write) = client_stream.split();
    let (mut target_read, mut target_write) = target_stream.split();

    let client_addr_clone = client_addr_str.clone();
    let target_addr_clone = target_addr_str.clone();
    let client_to_target = copy_and_record!(
        &mut client_read =>
        &mut target_write,
        |bytes_read| {
            record!(
                [id],
                "{} - {}",
                fmt_addr_forward!(id, (client_addr_clone, >>>, target_addr_clone)),
                format!("{} bytes", bytes_read).bright_black()
            );
        }
    );

    let target_to_client = copy_and_record!(
        &mut target_read =>
        &mut client_write,
        |bytes_read| {
            record!(
                [id],
                "{} - {}",
                fmt_addr_forward!(id, (client_addr_str, <<<, target_addr_str)).dimmed(),
                format!("{} bytes", bytes_read).bright_black()
            );
        }
    );

    tokio::select! {
        result = client_to_target => {
            match result {
                Ok(_) => {},
                Err(e) => error!([id], "error in client to target transfer: {}", e),
            }
        }
        result = target_to_client => {
            match result {
                Ok(_) => {},
                Err(e) => error!([id], "error in target to client transfer: {}", e),
            }
        }
    }

    Ok(())
}

fn extract_addr(stream: &TcpStream) -> String {
    stream
        .peer_addr()
        .map(|addr| addr.to_string())
        .unwrap_or_else(|_| "unknown".to_string())
}
