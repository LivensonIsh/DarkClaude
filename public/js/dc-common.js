const TK = "darkclaude_session_token", UK = "darkclaude_user";
const DC = {
  token: () => localStorage.getItem(TK) || "",
  save(token, user) { localStorage.setItem(TK, token); localStorage.setItem(UK, JSON.stringify(user)); },
  clear() { localStorage.removeItem(TK); localStorage.removeItem(UK); },
  async api(path, opts = {}) {
    const h = { "Content-Type": "application/json", ...(opts.headers || {}) };
    if (DC.token()) h["x-session-token"] = DC.token();
    const r = await fetch(path, { ...opts, headers: h, body: opts.body ? JSON.stringify(opts.body) : undefined });
    const data = await r.json().catch(() => ({}));
    if (!r.ok) { const e = new Error(data.error || data.details || "Erreur"); e.status = r.status; e.code = data.code; throw e; }
    return data;
  },
  msg(el, text, ok) { el.textContent = text || ""; el.className = "dc-msg " + (ok ? "ok" : "err"); },
  fmt: (n) => new Intl.NumberFormat("fr-FR").format(n)
};
