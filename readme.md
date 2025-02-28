## Building a Group Chat WebSocket Server with Minimal Dependencies

### Demo

https://github.com/user-attachments/assets/51f9624b-5eba-451e-9a5f-17c2028f9508

### Motivation

I started this project on a whim to learn Rust and understand how web servers work under the hood. As a JavaScript developer, I've taken many things for granted and never truly understood how they work behind the scenes. For example, I had no idea that HTTP requests are just plain text!

I began with minimal knowledge of WebSocket protocols. ChatGPT initially pointed me to [tokio](https://tokio.rs/) and [warp](https://github.com/seanmonstar/warp). Warp offers a very concise way to handle WebSockets with elegant, magical syntax, but I felt it might prevent me from learning how things actually work. So, I decided to build this project without external dependencies—except for `sha1` and `base64` to encode the required token string.

### The Beginning

ChatGPT provided me with a short snippet to get started, and I was surprised by how simple everything initially seemed. I learned about the [WebSocket handshake](https://developer.mozilla.org/en-US/docs/Web/API/WebSockets_API/Writing_WebSocket_servers#the_websocket_handshake), which wasn’t too bad to implement. That’s when I realized that HTTP requests and responses are just plain text. To streamline my progress, I wrote a small module called `http` to handle parsing and formatting of requests and responses.

### The WebSocket Protocol

After fixing the code and running the server, I encountered an issue: messages sent from clients always appeared as gibberish. Some research led me to this [article](https://www.openmymind.net/WebSocket-Framing-Masking-Fragmentation-and-More/), where I learned that messages from clients are structured binary strings. These strings can handle various payload types, masking, dynamic sizing, and fragmentation.

Writing the parser turned out to be quite the task. I only implemented the features necessary to get my demo working. Using `stream.read_exact` to programmatically read the required number of bytes made handling the dynamic frame sizes easier. I bundled all this logic into a `ws` module. With that, I finally got client-server communication working properly.

### Nonblocking I/O

My next goal was to build a server capable of handling a group chat with multiple ongoing WebSocket connections. However, I quickly ran into a limitation: network I/O with `TcpStream` is blocking by default. Once the server established a WebSocket connection with a client, it stopped listening for new HTTP requests until the stream closed.

I learned about alternatives for handling this issue, such as using multiple threads or event loops. To stick with my no-external-dependency approach, I decided to handle it in the most rudimentary way possible—using an infinite loop that continuously checks for incoming client requests, much like a game loop.

To achieve this, I set the TCP listener and streams to nonblocking mode and stored all active WebSocket connections in a list. On each iteration of the loop, the server first listens for and handles new HTTP requests. It then loops over the list of open `TcpStream` objects to check for incoming messages. All incoming messages are collected into an array. Finally, the server loops over the list again to broadcast these messages to all connected clients. This approach successfully created a group chat system while still allowing new users to join.

A nasty bug I encountered was that when an incoming `TcpStream` was detected, calling `stream.read` still returned Null. I'm still not exactly sure why this happened. I suspected that my listener for incoming messages was busy looping, causing each iteration to run faster than the time required to complete the I/O operation. To test this, I introduced artificial delays using `thread::sleep`, and it fixed the issue. I'd love to hear any insights you might have about this.

### Handling Metadata

I wrote a simple web interface for the group chat demo and noticed a new limitation: from the client's perspective, it had no idea which client a broadcasted message belonged to. Only the server maintained this information. To address this, I decided to encode every message as a JSON string, allowing metadata to be sent along with the message. I also asked each client to input their username, which the server stored to identify the sender of each message. Additionally, the server now broadcasts a notification whenever a user joins or leaves, making the chat experience more transparent and interactive.
