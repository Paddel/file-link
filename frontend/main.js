import init, { run_app } from '../target/wasm/file_link_frontend.js';
async function main() {
   await init('/public/js/file_link_frontend_bg.wasm');
   run_app();
}
main()