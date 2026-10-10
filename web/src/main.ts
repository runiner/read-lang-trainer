import init, { ping } from '../pkg/rus_wasm.js';

const app = document.querySelector<HTMLElement>('#app')!;
await init();
app.innerText = `rus-lang-trainer (wasm: ${ping()})`;
