import init, { run_app } from '../public/file_link_frontend.js';
async function main() {
   await init('public/file_link_frontend_bg.wasm');
   run_app();
}
main()