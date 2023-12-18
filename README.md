# Morganite

Morganite is a chat client for a pseudo chat protocol developed for my uni course. It is written in Rust using Tokio TCP Sockets. 

The protocol works in a serverless manner while also only having a direct connection to some of the clients, which requires routing using Distance Vector Routing with poison reverse and split horizon to mitigate routing loops.

Please note that this is a project for a course and is not intended for actual use, nor is it a good example of how to write asynchronous code in Rust.

## Usage

`cargo run <ip> <port> <username>`

## License

This project is licensed under EUPLv1.2 see [HERE](./LICENSE).