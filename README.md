[![A screenshot of the header of the demo website](./.static/header.png)][demo]

[![Ran on Railway][railway-badge]][demo] [![Last Commit][last-commit-badge]][repo]

[railway-badge]: https://img.shields.io/badge/Railway-0B0D0E.svg?style=&logo=Railway&logoColor=white
[last-commit-badge]: https://img.shields.io/github/last-commit/Xevion/dynamic-preauth/master
[repo]: https://github.com/Xevion/dynamic-preauth

A proof of concept for server-side modification of executables for pre-authentication, built with Rust ([Salvo][salvo]).

---

I've had this idea for a while now, that you could 'pre-authenticate' an executable, right before it's downloaded by a user, by modifying a specific pattern of bytes within it.

- While incredibly complex, unorthodox, likely insecure, and makes code signing near impossible - I've often thought about the benefits of such a system.
- The primary benefit is that user authentication is instant and requires zero user interaction. Start the download, run the program - and you're already authenticated, without any user input or special files.

This project is a proof of concept for that idea, using Rust and Salvo to build a server that can inject a user's authentication token into an executable before it's served to them.

[![A screenshot of the demo section on the demo website](./.static/demo.png)][demo]

This demo allows a user to create new targets unique to their session (via a Cookie) that can be downloaded and ran.

When ran, a simple GET request will be made to the server, which will notify the user's browser via Websockets.

## How it works

1. At build time, the server has release builds for the major target platforms built. They are made available to the server at runtime.
2. At runtime, the server locates constant time variables within the executable, and remembers their location for later download.
3. When a user requests an executable, the server injects the user's authentication token into the executable, overwriting whatever was located at the remembered location.

Now, when the user runs the executable, it will have the user's authentication token embedded within it - no recompilation or sidecar files required.
The executable keeps a hash of the original values, so it knows if the value has been changed.

This application demonstrates the concept of authentication via Websockets. Downloading a new executable will create a new identifier, which is remembered by the server.

In the browser, all download identifiers are shown, and running any executable will tell the server to notify the browser of the download (a sound will play and a visual effect for the relevant identifier will appear).

## Docker

This application is carefully constructed via the [Dockerfile](Dockerfile), built with [Railway][railway] in mind.

- The [demo](./demo/src/main.rs) application is built for Windows and Linux x64 targets with the `rust:latest` image.
- The [server](./src/main.rs) is built for Linux with the `rust:alpine` image.
- The [frontend](./frontend) is built with `node:latest` and pre-compressed with Gzip, Brotli, and Zstd.
- The final application stage is ran on `alpine:latest`.

## Security

I am not a security engineer, and I've taken zero courses, cerifications, or training in any way. I am not qualified to make any claims about the security of this application.

However, this application is built with minimal attack surfaces, and the host is completely stateless. The Railway instance is public and linked (although, unfortunately, don't show any build logs). Upon restart, all session data is lost.

Sessions are not regularly purged (yet, see #5), and overall the server isn't super-well optimized. This is just a proof concept, closer to a silly idea than a serious demo/project.

[demo]: https://dynamic-preauth.xevion.dev?utm_source=github
[railway]: https://railway.app
[salvo]: https://salvo.rs
