# Morganite

Morganite is a chat client for a [pseudo chat protocol](https://github.com/HAW-RN/protocol). It is written in Rust using Tokio TCP Sockets and Serde for serialization and deserialization of messages.

The protocol works in a serverless manner while also only having a direct connection to some of the clients, which requires routing using Distance Vector Routing with Poise Reverse & Split Horizon to mitigate routing loops.

It employs worker pools for handling incoming messages and sending messages to other clients. It also uses a timer to periodically send routing updates to other clients and channels to communicate between the workers and the main thread.

For the exact protocol specification see [HERE](https://github.com/HAW-RN/protocol).

## Usage

`cargo run <ip>:<port>` to bind to a specific address and port otherwise it will bind to a default address and port.

See `help` for a list of available commands.

## License

This project is licensed under EUPLv1.2 see [HERE](./LICENSE). It may not be used without adhering to the license or explicit permission from the authors.