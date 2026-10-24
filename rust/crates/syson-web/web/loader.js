// SPDX-License-Identifier: EPL-2.0
// Browser platform glue only. All actions, validation, model data, history and
// HTML generation live in Rust. No Java/TypeScript model runtime is invoked.
const root = document.getElementById('app');
try {
  const bytes = await (await fetch('syson_web.wasm')).arrayBuffer();
  const {instance} = await WebAssembly.instantiate(bytes, {});
  const api = instance.exports;
  const decoder = new TextDecoder();
  const encoder = new TextEncoder();
  function paint() {
    const focused = document.activeElement?.id;
    root.innerHTML = decoder.decode(new Uint8Array(api.memory.buffer, api.output_ptr(), api.output_len()));
    if (focused) document.getElementById(focused)?.focus();
  }
  function send(action, fields) {
    const input = encoder.encode(JSON.stringify({action, fields,
      entropy: Array.from(crypto.getRandomValues(new Uint8Array(api.creation_entropy_bytes())))}));
    if (input.length > 1048576) throw new Error('Input exceeds the 1 MiB editor limit.');
    api.input_reset();
    for (const byte of input) api.input_byte(byte);
    api.dispatch();
    paint();
  }
  function report(error) {
    root.querySelector('[data-platform-error]')?.remove();
    const message = document.createElement('p');
    message.dataset.platformError = '';
    message.setAttribute('role', 'alert');
    message.textContent = error.message;
    root.append(message);
  }
  root.addEventListener('submit', event => {
    event.preventDefault();
    const form = event.target;
    try { send(form.dataset.action, Object.fromEntries(new FormData(form))); } catch (error) { report(error); }
  });
  root.addEventListener('click', event => {
    const button = event.target.closest('button[data-action]');
    if (button) { try { send(button.dataset.action, {value: button.dataset.value ?? ''}); } catch (error) { report(error); } }
  });
  api.initialize();
  paint();
} catch (error) {
  root.replaceChildren();
  const message = document.createElement('p');
  message.setAttribute('role', 'alert');
  message.textContent = `The Rust editor could not start: ${error.message}`;
  root.append(message);
}
