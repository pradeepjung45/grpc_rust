// Reference Implementation for PTY Integration in Rust
// Uses the `portable-pty` crate.

use anyhow::Result;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};
use std::io::{Read, Write};

/// This demonstrates how to spawn a PTY and get readers/writers.
pub fn spawn_pty() -> Result<(Box<dyn Read + Send>, Box<dyn Write + Send>)> {
    // 1. Initialize the PTY System
    let pty_system = NativePtySystem::default();

    // 2. Define the PTY Size (columns, rows, pixel dimensions)
    let size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };

    // 3. Open the PTY Pair (Master and Slave)
    let pair = pty_system.openpty(size)?;

    // 4. Set up the shell command to run inside the PTY
    let mut cmd = CommandBuilder::new("bash");
    
    // Crucial: define the environment, especially TERM for color support
    cmd.env("TERM", "xterm-256color");

    // 5. Spawn the command attached to the slave end
    let _child = pair.slave.spawn_command(cmd)?;

    // 6. We do not need the slave end anymore in the host process
    drop(pair.slave);

    // 7. Get the reader and writer from the master end
    // The master end is what our gRPC Server will interact with.
    let reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;

    Ok((reader, writer))
}

/*
USAGE IN SERVER:
When handling `run_live(stream LiveInput)`:
1. Call `spawn_pty()` to get `pty_reader` and `pty_writer`.
2. Spawn Task A: Reads chunks from `pty_reader` -> sends `LiveOutput` to client.
3. Spawn Task B: Reads `LiveInput` from client -> writes strings to `pty_writer`.

WARNING:
- Reading from `pty_reader` requires a raw byte buffer (e.g. `[u8; 1024]`).
- Do NOT use `BufReader::read_line()` because PTY output arrives in raw byte chunks (often including ANSI escape codes for colors/cursor movement).
*/
