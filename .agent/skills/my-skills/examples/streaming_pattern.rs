// Reference Pattern for Bi-directional gRPC Streaming
// This is the core architectural pattern to prevent deadlocks and ensure smooth full-duplex communication.

/*
THE PATTERN:
1. Extract the inbound stream from the request.
2. Create an MPSC channel for the outbound stream.
3. Spawn a background task to handle reading/writing.
4. Return the receiver end of the channel immediately to the client.
*/

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};
use std::pin::Pin;
use futures::Stream;

// Define your generated types here
struct ClientMsg {}
struct ServerMsg {}

type ServiceStream = Pin<Box<dyn Stream<Item = Result<ServerMsg, Status>> + Send>>;

pub async fn handle_bidi_stream(
    request: Request<Streaming<ClientMsg>>
) -> Result<Response<ServiceStream>, Status> {
    
    // 1. Get the river of incoming messages
    let mut inbound = request.into_inner();

    // 2. Create the outbound pipe (adjust buffer size as needed, e.g. 32 for live terminal, larger for bulk data)
    let (tx, rx) = mpsc::channel::<Result<ServerMsg, Status>>(32);

    // 3. Spawn background worker
    tokio::spawn(async move {
        // ... Initialization logic (e.g. spawn PTY, open files, db connections) ...

        // Wait for messages from client
        while let Some(msg) = inbound.message().await.unwrap_or(None) {
            
            // ... Process the ClientMsg ...

            // Send a response back
            let res = ServerMsg {};
            if tx.send(Ok(res)).await.is_err() {
                // The client disconnected; break to clean up resources
                break;
            }
        }
    });

    // 4. Convert receiver to stream and return immediately
    let out_stream = ReceiverStream::new(rx);
    Ok(Response::new(Box::pin(out_stream)))
}
