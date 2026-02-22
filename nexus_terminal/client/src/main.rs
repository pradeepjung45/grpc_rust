use terminal_proto::terminal::remote_shell_client::RemoteShellClient;
use terminal_proto::terminal::LiveInput;
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = RemoteShellClient::connect("http://[::1]:50051").await?;

    println!("=== Nexus Live Shell ===");
    println!("Type commands and press Enter. Type 'exit' to quit.\n");

    
    let (tx, rx) = mpsc::channel::<LiveInput>(32);

    
    tokio::spawn(async move {
        
        let stdin = BufReader::new(io::stdin());
        let mut lines = stdin.lines();

        
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim() == "exit" {
                break;
            }
            
            if tx.send(LiveInput { input_line: line }).await.is_err() {
                break;
            }
        }
     
    });


    let input_stream = ReceiverStream::new(rx);

    
    let response = client.run_live(input_stream).await?;
    let mut output_stream = response.into_inner();

  
    while let Some(output) = output_stream.message().await? {
        println!("{}", output.output_line);
    }

    println!("\nSession closed.");
    Ok(())
}