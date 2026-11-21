// Injected before the page's own scripts. It gives the unchanged HTML two native powers:
//   1. localStorage that is saved to a file (survives restarts, not tied to any browser profile)
//   2. a clipboard that writes real rich text (HTML + plain) through the operating system
(() => {
  "use strict";
  const INIT = __STATE__;
  const post = (o) => { try { window.ipc.postMessage(JSON.stringify(o)); } catch (e) { /* no bridge: ignore */ } };

  /* ---------- settings storage ---------- */
  const store = new Map(Object.entries(INIT));
  const shim = {
    get length() { return store.size; },
    key(i) { const k = Array.from(store.keys())[i]; return k === undefined ? null : k; },
    getItem(k) { k = String(k); return store.has(k) ? store.get(k) : null; },
    setItem(k, v) { k = String(k); v = String(v); store.set(k, v); post({ t: "set", k, v }); },
    removeItem(k) { k = String(k); store.delete(k); post({ t: "del", k }); },
    clear() { store.clear(); post({ t: "clear" }); }
  };
  try {
    Object.defineProperty(window, "localStorage", { value: shim, configurable: true });
  } catch (e) {
    // Fallback: keep the real storage but mirror every change to the file.
    try {
      for (const [k, v] of store) window.localStorage.setItem(k, v);
      const proto = Storage.prototype, set = proto.setItem, del = proto.removeItem, clr = proto.clear;
      proto.setItem = function (k, v) { set.call(this, k, v); post({ t: "set", k: String(k), v: String(v) }); };
      proto.removeItem = function (k) { del.call(this, k); post({ t: "del", k: String(k) }); };
      proto.clear = function () { clr.call(this); post({ t: "clear" }); };
    } catch (e2) { /* storage unavailable: the page already copes with that */ }
  }

  /* ---------- clipboard ---------- */
  const pending = new Map();
  let seq = 0;
  window.__clipDone = (id, ok) => {
    const p = pending.get(id);
    if (!p) return;
    pending.delete(id);
    if (ok) p.res(); else p.rej(new Error("clipboard unavailable"));
  };
  const send = (payload) => new Promise((res, rej) => {
    const id = ++seq;
    pending.set(id, { res, rej });
    post(Object.assign({ t: "clip", id }, payload));
    setTimeout(() => { if (pending.delete(id)) rej(new Error("clipboard timeout")); }, 4000);
  });
  class NativeClipboardItem {
    constructor(data) { this.data = data; this.types = Object.keys(data); }
    async getType(type) { return this.data[type]; }
  }
  const blobText = async (item, type) => {
    if (!item.types.includes(type)) return null;
    const b = await item.getType(type);
    return typeof b === "string" ? b : await b.text();
  };
  const clipboard = {
    writeText: (text) => send({ text: String(text) }),
    write: async (items) => {
      const it = items[0];
      const html = await blobText(it, "text/html");
      const text = (await blobText(it, "text/plain")) || "";
      return send({ html, text });
    },
    readText: () => Promise.reject(new Error("not supported")),
    read: () => Promise.reject(new Error("not supported"))
  };
  window.ClipboardItem = NativeClipboardItem;
  try {
    Object.defineProperty(Navigator.prototype, "clipboard", { get: () => clipboard, configurable: true });
  } catch (e) {
    try { Object.defineProperty(navigator, "clipboard", { value: clipboard, configurable: true }); } catch (e2) { /* keep the page's own fallbacks */ }
  }
})();
