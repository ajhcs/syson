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
  let canvasGeneration = {};
  // A required cancellation is an invalidation barrier, never a queued edit.
  // `flushed` means attempted once, NOT acknowledged successful cancellation.
  let requiredCancellation = null;
  // Presentation state lives only in this page, outside model/canvas history.
  let canvasView = 'actual';
  let actualPan = {left: 0, top: 0};
  function rememberCanvasPan() {
    const viewport = root.querySelector('[data-canvas-viewport]');
    if (viewport?.dataset.canvasView === 'actual') {
      actualPan = {left: viewport.scrollLeft, top: viewport.scrollTop};
    }
  }
  function applyCanvasView() {
    const viewport = root.querySelector('[data-canvas-viewport]');
    const svg = viewport?.querySelector('svg');
    if (!svg) return;
    const width = Math.max(svg.viewBox.baseVal.width, 300);
    viewport.dataset.canvasView = canvasView;
    // At actual size, one diagram unit is one CSS pixel. Fit is explicit.
    svg.style.minWidth = canvasView === 'actual' ? `${width}px` : '0px';
    svg.style.maxWidth = canvasView === 'actual' ? `${width}px` : 'none';
    for (const button of root.querySelectorAll('button[data-canvas-view]')) {
      button.setAttribute('aria-pressed', String(button.dataset.canvasView === canvasView));
    }
    viewport.scrollLeft = canvasView === 'actual' ? actualPan.left : 0;
    viewport.scrollTop = canvasView === 'actual' ? actualPan.top : 0;
  }
  // One page-local presentation cache; no document JSON, model or history copy.
  const draftLimits = {fields: 64, details: 32, utf8: 1048576, utf16: 2097152, id: 128};
  let documentEpoch = 0;
  let recoveryDrafts = null;
  let presentationTrusted = true;
  let inspectorDisclosureDefaults = new WeakMap();
  const normal = host => !!host.querySelector('#export-json');
  const selected = host => host.querySelector('.tree button[aria-pressed="true"]')?.dataset.value ?? '';
  function formKey(element) {
    const form = element.closest('form');
    return form ? `${form.dataset.action ?? ''}/${form.dataset.attribute ?? ''}` : '';
  }
  function scope(element) {
    const owner = element.closest('form')?.dataset.action;
    return owner === 'rename' || owner === 'attribute' ? 'inspector'
      : owner === 'geometry_import' ? 'geometry' : 'document';
  }
  function detailKey(element) {
    return element.id ? `id:${element.id}`
      : element.querySelector('form[data-action]') ? `form:${formKey(element.querySelector('form[data-action]'))}` : '';
  }
  function utf8Size(text) {
    let bytes = 0;
    for (let i = 0; i < text.length; i++) {
      const unit = text.charCodeAt(i);
      if (unit < 128) bytes++;
      else if (unit < 2048) bytes += 2;
      else if (unit >= 0xd800 && unit <= 0xdbff && i + 1 < text.length
        && text.charCodeAt(i + 1) >= 0xdc00 && text.charCodeAt(i + 1) <= 0xdfff) { bytes += 4; i++; }
      else bytes += 3; // Includes TextEncoder's replacement for a lone surrogate.
      if (bytes > draftLimits.utf8) break;
    }
    return bytes;
  }
  // Native textarea values normalize CRLF and lone CR to LF; defaultValue
  // retains CR decoded from character references. Compare without allocating
  // another model-sized normalized String before the draft quota is checked.
  function sameTextareaDefault(value, initial) {
    if (value === initial) return true;
    let current = 0;
    for (let index = 0; index < initial.length; index++) {
      let unit = initial.charCodeAt(index);
      if (unit === 13) {
        if (initial.charCodeAt(index + 1) === 10) index++;
        unit = 10;
      }
      if (value.charCodeAt(current++) !== unit) return false;
    }
    return current === value.length;
  }
  function rememberField(element) {
    if (element === document.activeElement) return true;
    if (element.tagName === 'TEXTAREA') return !sameTextareaDefault(element.value, element.defaultValue);
    if (element.tagName !== 'SELECT') return element.value !== element.defaultValue
      || ('checked' in element && element.checked !== element.defaultChecked);
    // Creation selections remain presentation choices even when native defaults
    // were derived from the current Rust selection; recovery clears selection.
    if (['kind', 'parent', 'definition'].includes(element.id) && formKey(element) === 'stage/') return true;
    let initial = element.options[0]?.value ?? '';
    for (const option of element.options) if (option.defaultSelected) initial = option.value;
    return element.value !== initial;
  }
  function rememberInspectorDefaults() {
    inspectorDisclosureDefaults = new WeakMap();
    const inspector = root.querySelector('.inspector');
    if (!inspector) return;
    const walker = document.createTreeWalker(inspector, NodeFilter.SHOW_ELEMENT);
    let count = 0;
    for (let element = walker.nextNode(); element; element = walker.nextNode()) {
      if (element.tagName !== 'DETAILS') continue;
      if (count++ === draftLimits.details) break;
      inspectorDisclosureDefaults.set(element, element.open);
    }
  }
  function captureDrafts() {
    if (!normal(root) || !presentationTrusted) return null;
    const snapshot = {epoch: documentEpoch, selected: '', fields: [], details: [], focus: null};
    let bytes = 0, units = 0;
    const ids = new Set(), detailIds = new Set();
    const refuse = () => { throw new Error('Draft preservation limit exceeded (64 fields, 32 disclosures, 1 MiB UTF-8 / 2 MiB UTF-16). No action was sent; shorten/reset drafts, reset inspector disclosures to their initial state, or choose Document root as Parent first.'); };
    const charge = text => {
      // Check length before scanning/copying into retained records or encoding.
      if (text.length > draftLimits.utf16 / 2) refuse();
      units += text.length * 2; bytes += utf8Size(text);
      if (units > draftLimits.utf16 || bytes > draftLimits.utf8) refuse();
    };
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);
    for (let element = walker.nextNode(); element; element = walker.nextNode()) {
      if (element.matches('input[id],select[id],textarea[id]') && !element.readOnly && !element.disabled) {
        if (!rememberField(element)) continue;
        if (snapshot.fields.length === draftLimits.fields || element.id.length > draftLimits.id || ids.has(element.id) || document.getElementById(element.id) !== element) refuse();
        const owner = formKey(element), type = element.type ?? '', value = element.value;
        if (owner.length > draftLimits.id || type.length > draftLimits.id) refuse();
        charge(element.id); charge(owner); charge(type); charge(value);
        ids.add(element.id);
        const record = {id: element.id, tag: element.tagName, type, owner, scope: scope(element), value, checked: element.checked};
        snapshot.fields.push(record);
        if (element === document.activeElement) snapshot.focus = {id: record.id,
          caret: typeof element.selectionStart === 'number' ? [element.selectionStart, element.selectionEnd, element.selectionDirection] : null};
      } else if (element.tagName === 'DETAILS') {
        const inspector = !!element.closest('.inspector');
        if (inspector && element.open === inspectorDisclosureDefaults.get(element) && !element.contains(document.activeElement)) continue;
        const key = detailKey(element);
        if (!key) continue; // Unkeyed disclosures have no stable restoration contract.
        if (snapshot.details.length === draftLimits.details || key.length > draftLimits.id || detailIds.has(key)) refuse();
        charge(key); detailIds.add(key);
        snapshot.details.push({key, open: element.open, scope: inspector ? 'inspector' : 'document'});
      }
    }
    if (snapshot.fields.some(record => record.scope === 'inspector') || snapshot.details.some(record => record.scope === 'inspector')) {
      // Model identities are arbitrary strings, not bounded DOM control IDs.
      snapshot.selected = selected(root);
      charge(snapshot.selected);
    }
    return snapshot;
  }
  function restoreDrafts(snapshot, inspector, geometry) {
    if (!snapshot || snapshot.epoch !== documentEpoch) return null;
    const permitted = record => (record.scope !== 'inspector' || inspector && snapshot.selected === selected(root))
      && (record.scope !== 'geometry' || geometry);
    let focus = null;
    for (const record of snapshot.fields) {
      const element = document.getElementById(record.id);
      if (!permitted(record) || !element || !root.contains(element) || element.readOnly
        || element.tagName !== record.tag || element.type !== record.type || formKey(element) !== record.owner) continue;
      if (element.tagName === 'SELECT' && !Array.prototype.some.call(element.options, option => option.value === record.value)) continue;
      element.value = record.value;
      if ('checked' in element) element.checked = record.checked;
      if (snapshot.focus?.id === record.id) focus = {element, caret: snapshot.focus.caret};
    }
    const disclosures = new Map(snapshot.details.filter(permitted).map(record => [record.key, record]));
    if (disclosures.size) {
      const walker = document.createTreeWalker(root, NodeFilter.SHOW_ELEMENT);
      for (let element = walker.nextNode(); element; element = walker.nextNode()) {
        if (element.tagName !== 'DETAILS') continue;
        const key = detailKey(element), record = disclosures.get(key);
        if (record) { element.open = record.open; disclosures.delete(key); }
        if (!disclosures.size) break;
      }
    }
    return focus;
  }
  function paint(background = false, context = null) {
    rememberCanvasPan();
    const focused = document.activeElement;
    const template = document.createElement('template');
    template.innerHTML = decoder.decode(new Uint8Array(api.memory.buffer, api.output_ptr(), api.output_len()));
    const previousStatus = root.querySelector('[data-editor-status]');
    const nextStatus = template.content.querySelector('[data-editor-status]');
    const wasNormal = normal(root), isNormal = normal(template.content);
    const acknowledged = isNormal && nextStatus?.classList.contains('status') && !nextStatus.classList.contains('error');
    const action = context?.action;
    const reset = acknowledged && ['import', 'confirm_discard'].includes(action);
    if (reset) { documentEpoch++; recoveryDrafts = null; }
    const preserveDrafts = !reset && (background || nextStatus?.classList.contains('error'));
    if (wasNormal && !isNormal && context?.snapshot) {
      recoveryDrafts = context.snapshot;
      // A submitted edit is represented by Rust pending state, not a latent draft.
      recoveryDrafts.fields = recoveryDrafts.fields.filter(record => record.owner !== context.submitted);
      if (!recoveryDrafts.fields.some(record => record.id === recoveryDrafts.focus?.id)) recoveryDrafts.focus = null;
    }
    const recovered = !wasNormal && isNormal && recoveryDrafts;
    const geometryActions = ['stage', 'rename', 'attribute', 'cancel', 'recover_preview', 'discard_document', 'cancel_discard', 'select', 'canvas_cancel', 'import', 'confirm_discard'];
    if (action && !geometryActions.includes(action)) {
      for (const snapshot of [recoveryDrafts, context?.snapshot]) {
        if (snapshot) snapshot.fields = snapshot.fields.filter(record => record.scope !== 'geometry');
      }
    }
    if (action === 'recover_preview' && recoveryDrafts) {
      recoveryDrafts.fields = recoveryDrafts.fields.filter(record => record.scope !== 'inspector');
      recoveryDrafts.details = recoveryDrafts.details.filter(record => record.scope !== 'inspector');
    }
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
    rememberInspectorDefaults();
    const snapshot = recovered ? recoveryDrafts : preserveDrafts ? context?.snapshot : null;
    const restoredFocus = restoreDrafts(snapshot, !recovered, true);
    if (isNormal) recoveryDrafts = null;
    applyCanvasView();
    const requested = !background && root.querySelector('[data-inspector-focus]')?.dataset.inspectorFocus;
    const next = restoredFocus?.element || (requested && (document.getElementById(requested) ?? document.getElementById('selection-heading')))
      || (focused?.id && document.getElementById(focused.id));
    if (next) {
      // Returning focus must remain reachable even if a restored disclosure closed it.
      for (let ancestor = next.parentElement; ancestor && ancestor !== root; ancestor = ancestor.parentElement) {
        if (ancestor.tagName === 'DETAILS') ancestor.open = true;
      }
      next.focus({preventScroll: background});
      if (restoredFocus?.caret && next === restoredFocus.element) next.setSelectionRange(...restoredFocus.caret);
    }
    presentationTrusted = true;
    const canvas = root.querySelector('[data-canvas-revision]');
    const {canvasTicket: ticket, canvasPhase: phase} = canvas?.dataset ?? {};
    const key = `${ticket}/${phase}`;
    if (!requiredCancellation && ticket && phase && !queued.has(key)) {
      queued.add(key);
      const generation = canvasGeneration;
      setTimeout(() => {
        if (generation !== canvasGeneration || requiredCancellation) return;
        queued.delete(key);
        const current = root.querySelector('[data-canvas-ticket]');
        if (current?.dataset.canvasTicket !== ticket || current.dataset.canvasPhase !== phase) return;
        try { send(`canvas_${phase}`, {ticket}, true); } catch (error) { report(error); }
      }, 0);
    }
    return acknowledged;
  }
  function send(action, fields, background = false, submitted = '') {
    // Refuse before dispatch/repaint when any valid draft cannot fit; old DOM is
    // still connected and its data remains available for explicit correction.
    if (normal(root)) recoveryDrafts = null;
    const snapshot = captureDrafts();
    if (['import', 'confirm_discard'].includes(action) && documentEpoch === Number.MAX_SAFE_INTEGER) throw new Error('Presentation document identity exhausted; reload before replacing the document.');
    const input = encoder.encode(JSON.stringify({action, fields,
      entropy: Array.from(crypto.getRandomValues(new Uint8Array(api.creation_entropy_bytes())))}));
    if (input.length > 1048576) throw new Error('Input exceeds the 1 MiB editor limit.');
    if (requiredCancellation && action !== 'canvas_cancel') {
      const recoveryControl = ['recover_preview', 'discard_document', 'cancel_discard', 'confirm_discard'].includes(action);
      if (!requiredCancellation.flushed && (normal(root) || !recoveryControl)) send('canvas_cancel', {}, true);
      // Fallback hides Rust's action status. Keep its controls reachable without
      // repeated cancel dispatches, which would reset discard confirmation.
      if (requiredCancellation && !recoveryControl) {
        throw new Error('Canvas cancellation is pending. Drafts were retained. Use Cancel canvas work or the recovery controls before editing.');
      }
    }
    api.input_reset();
    for (const byte of input) api.input_byte(byte);
    try {
      if (action === 'canvas_cancel' && requiredCancellation) requiredCancellation.flushed = true;
      api.dispatch();
      const acknowledged = paint(background || ['canvas_undo', 'canvas_redo', 'canvas_cancel'].includes(action), {action, snapshot, submitted});
      if (acknowledged && ['canvas_cancel', 'recover_preview', 'confirm_discard'].includes(action)) requiredCancellation = null;
    } catch (error) {
      recoveryDrafts = null; presentationTrusted = false;
      throw new Error(`Editor output became uncertain; cached drafts were cleared. ${error.message}`);
    }
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
    try { send(form.dataset.action, Object.fromEntries(new FormData(form)), false, formKey(form)); } catch (error) { report(error); }
  });
  root.addEventListener('click', event => {
    const viewButton = event.target.closest('button[data-canvas-view]');
    if (viewButton && ['actual', 'fit'].includes(viewButton.dataset.canvasView)) {
      rememberCanvasPan();
      canvasView = viewButton.dataset.canvasView;
      applyCanvasView();
      return; // No WASM dispatch, rerender, document edit or history entry.
    }
    const button = event.target.closest('button[data-action]');
    if (button) {
      const action = button.dataset.action;
      if (action === 'layout' || action === 'rotate') requestCanvas(action).catch(report);
      else { try { send(action, {value: button.dataset.value ?? ''}); } catch (error) { report(error); } }
    }
  });
  for (const event of ['loading', 'loadingdone', 'loadingerror']) {
    document.fonts.addEventListener(event, () => {
      canvasGeneration = {}; queued.clear(); requiredCancellation = {flushed: false};
      if (!normal(root)) {
        report(new Error('Canvas work was invalidated; cancellation is pending until recovery or confirmed discard. Drafts were retained.'));
        return; // Do not reset Rust's armed fallback discard confirmation.
      }
      try { send('canvas_cancel', {}, true); }
      catch (error) { report(new Error(`Canvas work was invalidated; cancellation is pending. ${error.message}`)); }
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
