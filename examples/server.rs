use anyhow::Result;
use async_prost::AsyncProstStream;
use futures::prelude::*;
use rskv::{CommandRequest, CommandResponse, MemTable, Service};
use tokio::net::TcpListener;
use tracing::info;

use pyroscope::PyroscopeAgent;
use pyroscope_pprofrs::*;

fn complex() {
    // Your complex function implementation goes here
    println!("difficult");
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::set_var("RUST_LOG", "trace");

    tracing_subscriber::fmt::init();

    let agent = PyroscopeAgent::builder("http://127.0.0.1:4040", "rust-app")
        .backend(pprof_backend(PprofConfig::new().sample_rate(100)))
        .tags(vec![("Hostname", "pyroscope")])
        .build()
        .unwrap();

    let agent_running = agent.start().unwrap();

    let addr = "127.0.0.1:9000";
    let listener = TcpListener::bind(addr).await?;
    let svc = Service::new(MemTable::new());
    info!("Start listening on {}", addr);
    loop {
        let (stream, addr) = listener.accept().await?;
        info!("Client {:?} connected", addr);
        let svc = svc.clone();
        tokio::spawn(async move {
            let mut stream =
                AsyncProstStream::<_, CommandRequest, CommandResponse, _>::from(stream).for_async();
            while let Some(Ok(msg)) = stream.next().await {
                info!("Got a new command: {:?}", msg);
                // return a 404 response to client
                let resp = svc.execute(msg);
                stream.send(resp).await.unwrap();
            }
            info!("Client {:?} disconnected", addr);
        });
    }

    agent_running.stop();
    agent_running.shutdown();
}
