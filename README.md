# Morganite

![Morganite Chat Example](https://github.com/user-attachments/assets/713f6834-feea-450e-b965-b1ff788add70)

Morganite is a chat client for a [pseudo chat protocol](https://github.com/HAW-RN/protocol). It is written in Rust using Tokio TCP Sockets and Serde for serialization and deserialization of messages.

The protocol works in a serverless manner while also only having a direct connection to some of the clients, which requires routing using Distance Vector Routing with Poise Reverse & Split Horizon to mitigate routing loops.

It employs worker pools for handling incoming messages and sending messages to other clients. It also uses a timer to periodically send routing updates to other clients and channels to communicate between the workers and the main thread.

For the exact protocol specification see [HERE](https://github.com/HAW-RN/protocol).

## Usage

`cargo run <ip>:<port>` to bind to a specific address and port otherwise it will bind to a default address and port.

See `help` for a list of available commands.

## License

This project is licensed under EUPLv1.2 see [HERE](./LICENSE). It may not be used without adhering to the license or explicit permission from the authors. 

**This project is part of a university course. Unlicensed usage of this project to fulfill the course requirements is not allowed and would be considered plagiarism! We do not take any responsibility for the consequences of foolish actions and explicitly warn against it!**
