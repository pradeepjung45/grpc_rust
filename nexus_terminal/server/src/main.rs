use terminal_proto::terminal::remote_shell_server::{RemoteShell, RemoteShellServer};
use terminal_proto::terminal::{CommandRequest, CommandResponse, StreamRequest, StreamResponse, LiveInput, LiveOutput};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use std::process::Command;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use std::pin::Pin;
use futures::Stream;

#[derive(Debug, Default)]
pub struct MyRemoteShell {}

#[tonic::async_trait]
impl RemoteShell for MyRemoteShell {
    // ─── Level 1: Basic Remote Shell ──────────────────────────────────────────
    async fn execute(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let cmd_string = request.into_inner().command;
        println!("Running command: {}", cmd_string);

        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd_string)
            .output();

        match output {
            Ok(output) => {
                let result = String::from_utf8_lossy(&output.stdout).to_string();
                let error = String::from_utf8_lossy(&output.stderr).to_string();
                let full_output = format!("{}{}", result, error);
                
                Ok(Response::new(CommandResponse {
                    output: full_output,
                    exit_code: output.status.code().unwrap_or(-1),
                }))
            }
            Err(e) => Err(Status::internal(format!("Failed to execute: {}", e))),
        }
    }

    // ─── Level 2: Server Streaming ────────────────────────────────────────────

    type WatchStreamStream = Pin<Box<dyn Stream<Item = Result<StreamResponse, Status>> + Send>>;

    async fn watch_stream(
        &self,
        request: Request<StreamRequest>,
    ) -> Result<Response<Self::WatchStreamStream>, Status> {
        let target = request.into_inner().target;
        println!("Starting stream for: '{}'", target);

        let (tx, rx) = mpsc::channel(10);

        tokio::spawn(async move {
            for i in 1..=10 {
                let msg = StreamResponse {
                    update: format!("[{}] Event #{}: Server tick!", target, i),
                    event_id: i,
                };
                if tx.send(Ok(msg)).await.is_err() {
                    println!("Client disconnected early. Stopping stream.");
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
            println!("Stream for '{}' complete.", target);
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }

    // ─── Level 3: Bi-directional Streaming ────────────────────────────────────

    type RunLiveStream = Pin<Box<dyn Stream<Item = Result<LiveOutput, Status>> + Send>>;

    async fn run_live(
        &self,
        request: Request<Streaming<LiveInput>>,
    ) -> Result<Response<Self::RunLiveStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(input) = inbound.message().await.unwrap_or(None) {
                let cmd = input.input_line;
                println!("LiveShell running: '{}'", cmd);

                let output = Command::new("sh")
                    .arg("-c")
                    .arg(&cmd)
                    .output();

                match output {
                    Ok(out) => {
                        let result = String::from_utf8_lossy(&out.stdout).to_string();
                        let error  = String::from_utf8_lossy(&out.stderr).to_string();
                        let full   = format!("{}{}", result, error);

                        for line in full.lines() {
                            let msg = LiveOutput {
                                output_line: line.to_string(),
                            };
                            if tx.send(Ok(msg)).await.is_err() {
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(Status::internal(e.to_string()))).await;
                    }
                }
            }
            println!("Client closed the live session.");
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = MyRemoteShell::default();

    println!("Remote Shell listening on {}", addr);

    Server::builder()
        .add_service(RemoteShellServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}