# file-link
A simple full-stack Rust Webapplication facilitating peer-to-peer file transfers. Frontend compiled to WebAssembly with Yew, and both WebRTC and zlib functionalities are also compiled to WebAssembly, in order to enable an efficient P2P connectivity and data compression. Also integrates a simple Tokio signaling server.

[Live Demo](https://pkweber.de:8000)

### Dependencies:
zlib is provided as a git submodule, so clone this project with:
`git clone --recursive https://github.com/Paddel/file-link.git`
or download zlib for an already cloned repository with:
`git submodule update --init --recursive`

Install and add the following to your system's PATH environment variable:
- [Clang](https://github.com/llvm/llvm-project/releases) > 16.0.0
- [rollup](https://rollupjs.org/) > 3.28.0
- [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/) > 0.12.1

### Build/Run:
To build and run file-link execute `cargo run` in the root directory of the project.

## Credits
WebRtc integration is inspired by the example from the [Yew-WebRTC-Chat](https://github.com/codec-abc/Yew-WebRTC-Chat/blob/master/src/chat/web_rtc_manager.rs) repository by codec-abc.
The Slider Component is adapted from the example found in the [Yew's repository](https://github.com/yewstack/yew/blob/master/examples/boids/src/slider.rs).

## License
This project is licensed under the Apache-2.0 and MIT licenses. The full license text can be found in the root directory of the project.