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
  const queued = new Set();
  function paint(background = false) {
    const focused = document.activeElement;
    const caret = focused && typeof focused.selectionStart === 'number'
      ? [focused.selectionStart, focused.selectionEnd, focused.selectionDirection] : null;
    const drafts = background ? [...root.querySelectorAll('input[id],select[id],textarea[id]')]
      .filter(e => !e.readOnly).map(e => [e.id, e.value, e.checked]) : [];
    const details = background ? [...root.querySelectorAll('details')].map(e => e.open) : [];
    const template = document.createElement('template');
    template.innerHTML = decoder.decode(new Uint8Array(api.memory.buffer, api.output_ptr(), api.output_len()));
    const previousStatus = root.querySelector('[data-editor-status]');
    const nextStatus = template.content.querySelector('[data-editor-status]');
    if (previousStatus?.parentElement === root && nextStatus?.parentNode === template.content) {
      // The live region stays connected while its surrounding editor is repainted.
      for (const node of [...root.childNodes]) if (node !== previousStatus) node.remove();
      let after = false;
      for (const node of [...template.content.childNodes]) {
        if (node === nextStatus) { after = true; continue; }
        root.insertBefore(node, after ? null : previousStatus);
      }
      for (const attribute of [...previousStatus.attributes]) {
        if (!nextStatus.hasAttribute(attribute.name)) previousStatus.removeAttribute(attribute.name);
      }
      for (const attribute of nextStatus.attributes) {
        if (previousStatus.getAttribute(attribute.name) !== attribute.value) previousStatus.setAttribute(attribute.name, attribute.value);
      }
      if (previousStatus.textContent !== nextStatus.textContent) previousStatus.textContent = nextStatus.textContent;
    } else root.replaceChildren(template.content);
    for (const [id, value, checked] of drafts) {
      const e = document.getElementById(id);
      if (e) { e.value = value; if ('checked' in e) e.checked = checked; }
    }
    if (background) root.querySelectorAll('details').forEach((e, i) => { e.open = details[i]; });
    const requested = !background && root.querySelector('[data-inspector-focus]')?.dataset.inspectorFocus;
    const next = (requested && (document.getElementById(requested) ?? document.getElementById('selection-heading')))
      || (focused?.id && document.getElementById(focused.id));
    if (next) { next.focus({preventScroll: background}); if (background && caret) next.setSelectionRange(...caret); }
    const canvas = root.querySelector('[data-canvas-revision]');
    const {canvasTicket: ticket, canvasPhase: phase} = canvas?.dataset ?? {};
    const key = `${ticket}/${phase}`;
    if (ticket && phase && !queued.has(key)) {
      queued.add(key);
      setTimeout(() => {
        queued.delete(key);
        const current = root.querySelector('[data-canvas-ticket]');
        if (current?.dataset.canvasTicket !== ticket || current.dataset.canvasPhase !== phase) return;
        try { send(`canvas_${phase}`, {ticket}, true); } catch (error) { report(error); }
      }, 0);
    }
  }
  function send(action, fields, background = false) {
    const input = encoder.encode(JSON.stringify({action, fields,
      entropy: Array.from(crypto.getRandomValues(new Uint8Array(api.creation_entropy_bytes())))}));
    if (input.length > 1048576) throw new Error('Input exceeds the 1 MiB editor limit.');
    api.input_reset();
    for (const byte of input) api.input_byte(byte);
    api.dispatch();
    paint(background);
  }
  async function requestCanvas(action) {
    const revision = root.querySelector('[data-canvas-revision]').dataset.canvasRevision;
    const labels = [...root.querySelectorAll('[data-canvas-label]')].map(e => [e.dataset.canvasLabel, e.textContent, e.getAttribute('style')]);
    if (labels.length > 32) throw new Error('Canvas layout supports at most 32 package/part nodes.');
    await document.fonts.ready;
    const host = document.createElement('div');
    host.style.cssText = 'position:fixed;left:-100000px;top:0;visibility:hidden;width:max-content';
    document.body.append(host);
    let observations;
    try {
      observations = labels.map(([id, text, style]) => {
        const span = document.createElement('span'); span.setAttribute('style', style); span.textContent = text;
        host.replaceChildren(span);
        const {width, height} = span.getBoundingClientRect();
        return {id, width, height};
      });
    } finally { host.remove(); }
    send(action, {labels: JSON.stringify(observations), revision}, true);
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
    if (button) {
      const action = button.dataset.action;
      if (action === 'layout' || action === 'rotate') requestCanvas(action).catch(report);
      else { try { send(action, {value: button.dataset.value ?? ''}); } catch (error) { report(error); } }
    }
  });
  for (const event of ['loading', 'loadingdone', 'loadingerror']) {
    document.fonts.addEventListener(event, () => {
      try { send('canvas_cancel', {}, true); } catch (error) { report(error); }
    });
  }
  api.initialize();
  paint();
} catch (error) {
  root.replaceChildren();
  const message = document.createElement('p');
  message.setAttribute('role', 'alert');
  message.textContent = `The Rust editor could not start: ${error.message}`;
  root.append(message);
}
