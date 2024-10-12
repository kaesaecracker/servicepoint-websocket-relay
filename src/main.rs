use std::env;
use std::sync::Arc;
use clap::Parser;
use futures_util::{future, StreamExt, TryStreamExt};
use futures_util::stream::{FuturesUnordered};
use log::{error, info};
use tokio::io;
use tokio::net::{TcpListener, TcpStream, UdpSocket};
use tokio_tungstenite::tungstenite::Message;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The address to listening on
    #[arg(short, long, default_value = "127.0.0.1:8080")]
    listen: String,

    /// The address to forward to
    #[arg(short, long, default_value = "127.0.0.1:2342")]
    forward: String,
}

struct App {
    listener: TcpListener,
    sender: Arc<UdpSocket>,
}

impl App {
    async fn new(args: Args) -> io::Result<Self> {
        let listener = TcpListener::bind(&args.listen).await?;
        info!("Listening on: {}", &args.listen);

        let sender = UdpSocket::bind("0.0.0.0:0").await?;
        if let Ok( addr) =  sender.local_addr() {
            info!("Sending on: {}", addr);            
        }
        sender.connect(&args.forward).await?;
        info!("Sending to: {}", &args.forward);
        
        let sender = Arc::new(sender);
        Ok(Self {
            listener,
            sender,
        })
    }

    async fn run(&self) -> io::Result<()> {
        let mut futures = FuturesUnordered::new();
        loop {
            let stream = match self.listener.accept().await {
                Ok((stream, _)) => stream,
                Err(e) => {
                    _ = futures.collect::<Vec<_>>();
                    return Err(e);
                }
            };
            futures.push(async {
                self.accept_connection(stream)
            });

            match futures.next().await {
                None => {}
                Some(e) => e.await,
            }
        }
    }

    async fn accept_connection(&self, stream: TcpStream) {
        let addr = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", addr);

        let ws_stream = match tokio_tungstenite::accept_async(stream).await {
            Ok(s) => s,
            Err(e) => {
                error!("Error during the websocket handshake occurred: {}", e);
                return;
            },
        };

        info!("New WebSocket connection: {}", addr);

        let (_, read) = ws_stream.split();
        let foo = read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
            .inspect_err(|e| error!("could not read from websocket: {}", e))
            .filter_map(|x| future::ready( x.ok()) )
            .map(move |x| self.forward_message(x));
    }

    async fn forward_message(&self, message: Message) -> io::Result<usize> {
        self.sender.send(&*message.into_data()).await
    }
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info")
    }

    let _ = env_logger::try_init();
    let args = Args::parse();

    let app = App::new(args).await?;
    app.run().await?;

    Ok(())
}
