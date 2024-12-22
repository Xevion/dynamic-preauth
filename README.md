# dynamic-preauth

I had an idea that executables could be pre-authenticated by the server before being ran, with no interaction from the user.

This is a proof of concept of that idea, built in Rust.

## How it works

1. The server is provided a fully built template executable on startup.

2. Users can 'authenticate' themselves and then download the executable.

3. Before the executable is served to the user, the server will inject the user's authentication token into the executable by modifying the binary.

4. The modified binary is then served to the user.

5. A constant time string in the binary, now modified, is used to authenticate the binary's requests to the server.

## Implementation

- The testing binary is built in Docker, and is a simple Rust binary that sends a GET request to the server with the authentication token.
