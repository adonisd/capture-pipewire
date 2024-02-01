# Capture a Screencast in Rust using pipeWire

This example demonstrates how to create (using XDG) and capture a screencast in Rust using pipeWire.
Tested on Wayland (Gnome DE), but should work on X11 as well.
Requires the `xdg-desktop-portal` package to be installed.

## Modules

### ashpd

This module provides access to the `ashpd` library, which is used for creating screencasts.

## Functions

### create_screencast_stream

This asynchronous function creates a screencast stream. It first creates a screencast proxy and a session. Then it selects the sources for the screencast, which are the monitor and the window. After that, it starts the screencast and gets the response. The first stream from the response is then returned.

## Main Function

The main function is asynchronous and it creates a screencast stream. If the creation of the screencast stream fails, it will panic with a message.
Then it uses the stream to create a screencast source. After that, it creates a screencast sink. Then it creates a screencast node and connects the source and the sink to it. Finally, it activates the node.

## Dependencies

- ashpd: This library is used for creating screencasts.
- tokio: This library is used for writing asynchronous code in Rust.
- pipewire: This library is used for multimedia processing.

## Usage

To run the program, use the command `cargo run` in the terminal.
