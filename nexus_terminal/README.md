# Nexus Terminal

A distributed collaborative terminal built in Rust using gRPC and Tonic.

## Overview

This project implements a remote shell that allows a client to connect to a server via gRPC and execute shell commands. It showcases three levels of gRPC communication:

1.  **Level 1: Unary RPC (`Execute`)** - Send a single command, wait for it to finish, and get the full output.
2.  **Level 2: Server Streaming (`WatchStream`)** - Server pushes periodic updates to the client (like a live ticker).
3.  **Level 3: Bi-directional Streaming (`RunLive`)** - A fully interactive remote shell where the client streams commands (via standard input) and the server streams back the output line-by-line in real-time.

## Project Structure

The project is organized as a Cargo Workspace containing three crates:

*   `terminal-proto`: The shared contract defining the gRPC service using Protocol Buffers (`proto/terminal.proto`). Generates the Rust code using `tonic-build`.
*   `server`: The gRPC server that actually executes shell commands on the host OS.
*   `client`: The gRPC client that connects to the server and provides an interactive command line interface.

## Requirements

*   Rust (cargo)
*   Protocol Buffers Compiler (`protoc`) installed on your system.
    *   Ubuntu/Debian: `sudo apt install protobuf-compiler`
    *   macOS: `brew install protobuf`

## How to Run

1.  **Start the Server:**
    Open a terminal, navigate to the project root, and run:
    ```bash
    cargo run --bin server
    ```
    The server will start listening on `[::1]:50051`.

2.  **Start the Client:**
    Open a *second* terminal, navigate to the project root, and run:
    ```bash
    cargo run --bin client
    ```

3.  **Interact:**
    Once the client connects, it will demonstrate Level 1 and Level 2 automatically. Then it will open the Level 3 interactive "Live Shell". Type any standard shell command (e.g., `ls -la`, `whoami`, `date`) and press Enter to see the live output from the server. Type `exit` or press `Ctrl+C` to quit.

## Limitations

Currently, every command typed in the live shell runs in a new subshell process (`sh -c`). This means persistent state like `cd` will not carry over between commands (e.g., `cd /tmp` followed by `ls` will list the original directory, not `/tmp`).

To run commands in different directories, chain them: `cd /path/to/dir && ls -la`.

*(A future Level 4 using a proper PTY backend would solve this!)*
