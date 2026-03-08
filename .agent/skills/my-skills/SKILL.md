---
name: Terminal Dev Skill
description: Core architecture, libraries, and design patterns for building the Nexus Distributed Collaborative Terminal using Rust and gRPC.
---

# Nexus Terminal Architecture & Patterns

This project is a multi-tier Rust workspace that implements a distributed, collaborative remote terminal using gRPC.

## 1. Core Technologies
- **Rust**: Systems programming language for high performance and strict type safety.
- **Tonic**: gRPC over HTTP/2 implementation in Rust. Requires `tokio` for async.
- **Prost**: Protocol Buffers implementation for serializing structured data.
- **Tokio**: The underlying asynchronous runtime handling green threads, networking, and channels.
- **Portable-PTY** (Upcoming): Cross-platform crate to allocate pseudo-terminals for stateful shell sessions.

## 2. Project Structure
The project is a standard Cargo workspace containing three crates:
- `terminal-proto`: The source of truth. Contains `proto/terminal.proto` and a `build.rs` using `tonic-build` to compile it.
- `server`: The host machine that actually runs the shell executable. Depends on `terminal-proto` and `tokio`.
- `client`: The user interface that connects to the server and handles user input/output formatting.

## 3. The Protocol Evolution (Levels 1-3)
We implemented the network layer progressively:

*   **Level 1: Unary RPC (`Execute`)**
    *   **Pattern:** Request -> Wait -> Response.
    *   **Usage:** Fire-and-forget shell commands. Blocks until the command finishes.
*   **Level 2: Server Streaming (`WatchStream`)**
    *   **Pattern:** Request -> Open Connection -> Server pushes multiple responses.
    *   **Usage:** Live tickers or logs (e.g., `tail -f`).
    *   **Implementation:** The server spawns a `tokio::spawn` task, writes to an `mpsc::channel`, and converts the `Receiver` to a `ReceiverStream` for the client.
*   **Level 3: Bi-directional Streaming (`RunLive`)**
    *   **Pattern:** Client Stream <-> Server Stream simultaneously.
    *   **Usage:** Interactive stateless shell. Client sends standard input lines, server returns standard output lines.
    *   **Implementation:** Client uses `tokio_stream::iter` (or async stdin reads via `mpsc`) to stream input. Server reads `request.into_inner().message().await` in a loop, runs `std::process::Command`, and streams lines back.

## 4. Current Limitations (Statelessness)
In Level 3, every incoming command line from the client is fed to a **new OS process**:
```rust
Command::new("sh").arg("-c").arg(&cmd).output()
```
Because a new process is spawned for every command, **state does not persist**. 
- `cd /tmp` will change the directory of that temporary `sh` process, which Immediately dies. The next command runs from the original working directory.
- Interactive shells like `vim` or Python REPLs will crash or freeze because they expect an interactive TTY device, not simple piped standard input.

## 5. Next Steps: Level 4 (The Stateful PTY)
To fix the stateless issue, we must move away from `Command::new("sh")` and instead allocate a **Pseudo-Terminal (PTY)** on the server when a client connects.

### The PTY Pattern
1.  **Allocate a PTY System:** Using the `portable-pty` crate, the server allocates a virtual screen size and shell (e.g., `/bin/bash`).
2.  **Split Streams:** The PTY produces a Reader (giving us the shell's output bytes) and a Writer (letting us funnel the client's keystrokes into the shell).
3.  **Bridge gRPC and PTY:**
    *   *Task A:* As `LiveInput` arrives from the gRPC stream, write the bytes into the PTY Writer.
    *   *Task B:* A loop constantly reading bytes from the PTY Reader and bundling them into `LiveOutput` messages to stream back over gRPC.

## General Agent Directives
When working on this project:
- Always check `terminal-proto/proto/terminal.proto` as the single source of truth for the API.
- Re-run `cargo build` in the workspace root if the proto file changes, to ensure `tonic-build` regenerates the types.
- Ensure that the client handles async blocking gracefully (e.g., `tokio::spawn` for reading `stdin`).
- When introducing `portable-pty`, be mindful of byte conversions. A PTY speaks raw bytes (`Vec<u8>`), which we must convert to strings or transmit as raw bytes in the proto.
