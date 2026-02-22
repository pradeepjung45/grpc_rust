use terminal_proto::terminal::remote_shell_server::{RemoteShell, RemoteShellServer};
use terminal_proto::terminal::{CommandRequest, CommandResponse};
use tonic::{transport::Server, Request, Response, Status};
use std::process::Command;

#[derive(Debug, Default)]
pub struct MyRemoteShell {}

#[tonic::async_trait]
impl RemoteShell for MyRemoteShell {
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
            Err(e) => {
                Err(Status::internal(format!("Failed to execute: {}", e)))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?; //[::1]means ipv6 localhost
    let service = MyRemoteShell::default();

    println!("Remote Shell listening on {}", addr);

    Server::builder()
        .add_service(RemoteShellServer::new(service))
        .serve(addr)
        .await?;

    Ok(())
}