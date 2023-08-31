# file-link
A simple full-stack Rust Webapplication facilitating peer-to-peer file transfers. Frontend compiled to WebAssembly with Yew, and both WebRTC and zlib functionalities are also compiled to WebAssembly, in order to enable an efficient P2P connectivity and data compression. Also integrates a simple Tokio signaling server.

!GIF

### Dependencies:
Install and add the following to your system's PATH environment variable:
- Clang > 16.0.0
- openssl > OpenSSL 1.1
- rollup > 3.28.0
- wasm-pack > 0.12.1

## Credits
WebRtc integration is inspired by the example from the [Yew-WebRTC-Chat](https://github.com/codec-abc/Yew-WebRTC-Chat/blob/master/src/chat/web_rtc_manager.rs) repository by codec-abc.

Websocket functionality is based on the example from the [security-union's yew-websocket](https://github.com/security-union/yew-websocket/blob/master/src/websocket.rs)  repository.

The Slider Component is adapted from the example found in the [Yew's repository](https://github.com/yewstack/yew/blob/master/examples/boids/src/slider.rs).

## License
This project is licensed under the Apache-2.0 and MIT licenses. You can find the full license text in the root directory of the project.
