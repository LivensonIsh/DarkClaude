/**
 * DarkClaude lightweight i18n
 * - Full locales only (100 % traduites, pas de repli anglais visible)
 * - Persists darkclaude_lang in localStorage
 */
(function (global) {
  "use strict";

  const STORAGE_KEY = "darkclaude_lang";
  /** Langues 100 % traduites uniquement (pas de repli anglais visible). */
  const FULL_LOCALES = ["fr", "en"];
  const PARTIAL_LOCALES = [];
  const SUPPORTED = FULL_LOCALES.slice();
  const DEFAULT = "fr";

  const LANG_META = {
    fr: { label: "Français", code: "FR" },
    en: { label: "English", code: "EN" },
    es: { label: "Español", code: "ES" },
    it: { label: "Italiano", code: "IT" },
    de: { label: "Deutsch", code: "DE" },
    ru: { label: "Русский", code: "RU" }
  };

  let currentLang = DEFAULT;
  let catalog = {};
  let fallback = {};
  let enFallback = {};
  let ready = false;
  const waiters = [];

  function deepMerge(base, overlay) {
    if (!overlay || typeof overlay !== "object") return base;
    const out = Array.isArray(base) ? base.slice() : { ...base };
    for (const [k, v] of Object.entries(overlay)) {
      if (v && typeof v === "object" && !Array.isArray(v) && base && typeof base[k] === "object" && !Array.isArray(base[k])) {
        out[k] = deepMerge(base[k], v);
      } else {
        out[k] = v;
      }
    }
    return out;
  }

  function getByPath(obj, key) {
    if (!obj || !key) return undefined;
    if (Object.prototype.hasOwnProperty.call(obj, key) && typeof obj[key] === "string") {
      return obj[key];
    }
    const parts = String(key).split(".");
    let cur = obj;
    for (const p of parts) {
      if (cur == null || typeof cur !== "object") return undefined;
      cur = cur[p];
    }
    return typeof cur === "string" ? cur : undefined;
  }

  function interpolate(str, vars) {
    if (!vars) return str;
    return String(str).replace(/\{(\w+)\}/g, (_, k) =>
      vars[k] != null ? String(vars[k]) : `{${k}}`
    );
  }

  function t(key, vars) {
    const fromCur = getByPath(catalog, key);
    const fromEn = getByPath(enFallback, key);
    const fromFb = getByPath(fallback, key);
    if (fromCur == null && fromEn == null && fromFb == null) {
      /* Before locales load: never return the raw key (FOUTC). Callers keep HTML defaults. */
      if (!ready) return undefined;
      return key;
    }
    const raw = fromCur != null ? fromCur : fromEn != null ? fromEn : fromFb;
    return interpolate(raw, vars);
  }

  function revealI18n() {
    try {
      document.documentElement.classList.add("i18n-ready");
    } catch (_) {}
  }

  function markReady() {
    ready = true;
    revealI18n();
  }

  function detectLang() {
    try {
      const saved = localStorage.getItem(STORAGE_KEY);
      if (saved && SUPPORTED.includes(saved)) return saved;
    } catch (_) {}
    try {
      const nav = (navigator.language || navigator.userLanguage || "fr").toLowerCase();
      const short = nav.slice(0, 2);
      if (SUPPORTED.includes(short)) return short;
    } catch (_) {}
    return DEFAULT;
  }

  const LOCALE_BASE = (() => {
    try {
      if (document.currentScript && document.currentScript.src) {
        return new URL("../", document.currentScript.src).href;
      }
    } catch (_) {}
    return "/";
  })();

  /* Bump when locale JSON content changes (CDN / browser cache). */
  const LOCALE_VER = "dg-20260910e";

  function localeUrl(lang) {
    return `${LOCALE_BASE}locales/${lang}.json?v=${LOCALE_VER}`;
  }

  function partialLocaleUrl(lang) {
    return `${LOCALE_BASE}locales/partials/${lang}.json?v=${LOCALE_VER}`;
  }

  async function loadLocale(lang) {
    const res = await fetch(localeUrl(lang), { cache: "no-cache" });
    if (!res.ok) throw new Error("locale " + lang);
    return res.json();
  }

  async function loadPartialLocale(lang) {
    const res = await fetch(partialLocaleUrl(lang), { cache: "no-cache" });
    if (!res.ok) throw new Error("partial " + lang);
    return res.json();
  }

  async function loadCatalogFor(lang) {
    if (!Object.keys(enFallback).length) {
      try {
        enFallback = await loadLocale("en");
      } catch (_) {
        enFallback = {};
      }
    }
    if (lang === "fr") {
      return loadLocale("fr");
    }
    if (lang === "en") {
      return enFallback;
    }
    if (PARTIAL_LOCALES.includes(lang)) {
      try {
        const partial = await loadPartialLocale(lang);
        return deepMerge(enFallback, partial);
      } catch (_) {
        return enFallback;
      }
    }
    return loadLocale(lang);
  }

  function setAttr(el, attr, key) {
    if (!key) return;
    const val = t(key);
    if (val == null) return;
    el.setAttribute(attr, val);
  }

  function applyElement(el) {
    const key = el.getAttribute("data-i18n");
    if (key) {
      const val = t(key);
      if (val != null) {
        if (el.hasAttribute("data-i18n-html")) el.innerHTML = val;
        else el.textContent = val;
      }
    }
    const ph = el.getAttribute("data-i18n-placeholder");
    if (ph) {
      const phVal = t(ph);
      if (phVal != null) el.setAttribute("placeholder", phVal);
    }
    const aria = el.getAttribute("data-i18n-aria");
    if (aria) setAttr(el, "aria-label", aria);
    const title = el.getAttribute("data-i18n-title");
    if (title) setAttr(el, "title", title);
    const loginLabel = el.getAttribute("data-login-label");
    if (loginLabel != null && el.hasAttribute("data-i18n-login")) {
      const loginText = t("nav.login");
      if (loginText != null) {
        el.setAttribute("data-login-label", loginText);
        if (!el.classList.contains("hidden")) el.textContent = loginText;
      }
    }
  }

  function apply(root) {
    const scope = root || document;
    scope.querySelectorAll("[data-i18n], [data-i18n-placeholder], [data-i18n-aria], [data-i18n-title], [data-i18n-login]").forEach(applyElement);
    document.documentElement.setAttribute("lang", currentLang);
    syncSwitcher();
    try {
      document.dispatchEvent(new CustomEvent("darkclaude:i18n", { detail: { lang: currentLang } }));
    } catch (_) {}
  }

  function langMeta(lang) {
    return LANG_META[lang] || { label: String(lang).toUpperCase(), code: String(lang).toUpperCase() };
  }

  function closeAllLangMenus() {
    document.querySelectorAll("[data-lang-drop].is-open").forEach((drop) => {
      drop.classList.remove("is-open");
      const trigger = drop.querySelector("[data-lang-trigger]");
      const menu = drop.querySelector("[data-lang-menu]");
      if (trigger) trigger.setAttribute("aria-expanded", "false");
      if (menu) menu.hidden = true;
    });
    document.querySelectorAll(".topbar.is-lang-open").forEach((bar) => {
      bar.classList.remove("is-lang-open");
    });
  }

  function syncSwitcher() {
    document.querySelectorAll("[data-lang-drop]").forEach((drop) => {
      const current = langMeta(currentLang);
      const curEl = drop.querySelector(".dg-lang__current");
      if (curEl) curEl.textContent = current.label;
      drop.querySelectorAll("[data-lang]").forEach((btn) => {
        const active = btn.getAttribute("data-lang") === currentLang;
        btn.classList.toggle("is-active", active);
        btn.setAttribute("aria-selected", active ? "true" : "false");
      });
    });
    document.querySelectorAll(".settings-lang-list [data-lang]").forEach((btn) => {
      const active = btn.getAttribute("data-lang") === currentLang;
      btn.classList.toggle("is-active", active);
      btn.setAttribute("aria-selected", active ? "true" : "false");
    });
  }

  function switcherSettingsHtml() {
    const label = t("nav.lang") || "Langue";
    const title = t("settings.language") || "Langue";
    const desc = t("settings.languageDesc") || "Interface DarkClaude.";
    const opts = SUPPORTED.map((l) => {
      const m = langMeta(l);
      const active = l === currentLang ? " is-active" : "";
      const selected = l === currentLang ? ' aria-selected="true"' : ' aria-selected="false"';
      return (
        `<button type="button" class="settings-lang-option${active}" role="option" data-lang="${l}"${selected}>` +
        `<span class="settings-lang-option__code">${m.code}</span>` +
        `<span class="settings-lang-option__name">${m.label}</span>` +
        `</button>`
      );
    }).join("");
    return (
      `<section class="settings-section settings-section--lang">` +
      `<div class="settings-section__head">` +
      `<div class="settings-section__icon" aria-hidden="true"><svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75"><circle cx="12" cy="12" r="10"/><path d="M2 12h20"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg></div>` +
      `<div><h3 class="settings-section__title">${title}</h3>` +
      `<p class="settings-section__desc">${desc}</p></div></div>` +
      `<div class="settings-lang-list" data-lang-menu role="listbox" aria-label="${label}">${opts}</div>` +
      `</section>`
    );
  }

  function switcherHtml(extraClass) {
    const cls = extraClass ? ` dg-lang ${extraClass}` : " dg-lang";
    const label = t("nav.lang") || "Langue";
    const current = langMeta(currentLang);
    const opts = SUPPORTED.map((l) => {
      const m = langMeta(l);
      const active = l === currentLang ? " is-active" : "";
      const selected = l === currentLang ? ' aria-selected="true"' : ' aria-selected="false"';
      return (
        `<button type="button" class="dg-lang__option${active}" role="option" data-lang="${l}"${selected}>` +
        `<span class="dg-lang__code">${m.code}</span>` +
        `<span class="dg-lang__name">${m.label}</span>` +
        `</button>`
      );
    }).join("");
    return (
      `<div class="${cls.trim()}" role="group" aria-label="${label}">` +
      `<div class="dg-lang__drop" data-lang-drop>` +
      `<button type="button" class="dg-lang__trigger" data-lang-trigger aria-haspopup="listbox" aria-expanded="false" aria-label="${label}">` +
      `<span class="dg-lang__current">${current.label}</span>` +
      `<svg class="dg-lang__chev" width="12" height="12" viewBox="0 0 12 12" aria-hidden="true"><path fill="currentColor" d="M3 4.5 6 7.5 9 4.5"/></svg>` +
      `</button>` +
      `<div class="dg-lang__menu" data-lang-menu role="listbox" hidden>${opts}</div>` +
      `</div></div>`
    );
  }

  function injectSwitchers() {
    document.querySelectorAll(".top-actions").forEach((actions) => {
      if (actions.querySelector(".dg-lang")) return;
      const wrap = document.createElement("div");
      wrap.innerHTML = switcherHtml("dg-lang--nav");
      const node = wrap.firstElementChild;
      const login = actions.querySelector("#login-open, .nav-btn");
      if (login) actions.insertBefore(node, login);
      else actions.appendChild(node);
    });

    /* Mobile top bar: sélecteur langue entre logo et hamburger */
    document.querySelectorAll(".topbar-inner").forEach((inner) => {
      if (inner.querySelector(".dg-lang--bar")) return;
      const toggle =
        inner.querySelector("#mobile-nav-toggle") ||
        inner.querySelector(".mobile-nav-toggle");
      if (!toggle || !inner.contains(toggle) || toggle.parentElement !== inner) return;
      const wrap = document.createElement("div");
      wrap.innerHTML = switcherHtml("dg-lang--bar");
      const node = wrap.firstElementChild;
      inner.insertBefore(node, toggle);
    });

    const authBrand = document.querySelector(".auth-page .auth-brand");
    if (authBrand && !document.querySelector(".auth-page .dg-lang")) {
      const wrap = document.createElement("div");
      wrap.innerHTML = switcherHtml("dg-lang--auth");
      authBrand.insertAdjacentElement("afterend", wrap.firstElementChild);
    }

    const settingsBody = document.querySelector(".settings-panel__body");
    if (settingsBody) {
      settingsBody
        .querySelectorAll(".settings-section--lang, .dg-lang--settings")
        .forEach((node) => {
          const sec = node.closest(".settings-section");
          if (sec) sec.remove();
          else node.remove();
        });
      const wrap = document.createElement("div");
      wrap.innerHTML = switcherSettingsHtml();
      const section = wrap.firstElementChild;
      settingsBody.appendChild(section);
      apply(section);
    }

    syncSwitcher();
  }

  async function setLang(lang, opts) {
    const next = SUPPORTED.includes(lang) ? lang : DEFAULT;
    if (!Object.keys(fallback).length) {
      try {
        fallback = await loadLocale("fr");
      } catch (_) {
        fallback = {};
      }
    }
    catalog = await loadCatalogFor(next);
    currentLang = next;
    try {
      localStorage.setItem(STORAGE_KEY, next);
    } catch (_) {}
    apply();
    injectSwitchers();
    apply();
    if (opts && opts.reload) {
      location.reload();
      return;
    }
  }

  function onReady(fn) {
    if (ready) fn();
    else waiters.push(fn);
  }

  document.addEventListener("click", (e) => {
    const trigger = e.target && e.target.closest && e.target.closest("[data-lang-trigger]");
    if (trigger) {
      e.stopPropagation();
      const drop = trigger.closest("[data-lang-drop]");
      const menu = drop && drop.querySelector("[data-lang-menu]");
      const open = trigger.getAttribute("aria-expanded") === "true";
      closeAllLangMenus();
      if (!open && menu) {
        trigger.setAttribute("aria-expanded", "true");
        menu.hidden = false;
        drop.classList.add("is-open");
        const topbar = drop.closest(".topbar");
        if (topbar) topbar.classList.add("is-lang-open");
      }
      return;
    }
    const opt = e.target && e.target.closest && e.target.closest("[data-lang]");
    if (
      opt &&
      (opt.closest("[data-lang-menu]") || opt.closest(".settings-lang-list"))
    ) {
      const lang = opt.getAttribute("data-lang");
      closeAllLangMenus();
      if (lang && lang !== currentLang) setLang(lang);
      return;
    }
    closeAllLangMenus();
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeAllLangMenus();
  });

  async function boot() {
    currentLang = detectLang();
    document.documentElement.setAttribute("lang", currentLang);
    try {
      fallback = await loadLocale("fr");
    } catch (_) {
      fallback = {};
    }
    try {
      catalog = await loadCatalogFor(currentLang);
    } catch (_) {
      catalog = fallback;
      currentLang = "fr";
    }
    /* Catalogs loaded: t() may resolve keys before DOM apply */
    markReady();
    injectSwitchers();
    apply();
    waiters.splice(0).forEach((fn) => {
      try {
        fn();
      } catch (_) {}
    });
  }

  // Minimal CSS injection if stylesheet missing pieces
  function ensureStyles() {
    const styleId = "dg-i18n-style-v5";
    if (document.getElementById(styleId)) return;
    const legacy = document.getElementById("dg-i18n-style");
    if (legacy) legacy.remove();
    const s = document.createElement("style");
    s.id = styleId;
    s.textContent = `
/* Hide i18n nodes until first apply — nav/switcher only (keeps landing layout intact) */
html:not(.i18n-ready) .topbar [data-i18n],
html:not(.i18n-ready) .topbar [data-i18n-login],
html:not(.i18n-ready) .dg-lang [data-i18n],
html:not(.i18n-ready) [data-i18n-login]{opacity:0}
html.i18n-ready .topbar [data-i18n],
html.i18n-ready .topbar [data-i18n-login],
html.i18n-ready .dg-lang [data-i18n],
html.i18n-ready [data-i18n-login]{opacity:1}
.dg-lang{display:inline-flex;align-items:center;font-family:"DM Sans",system-ui,sans-serif;font-size:0.78rem;font-weight:600;user-select:none}
.dg-lang__drop{position:relative;display:inline-block}
.dg-lang__trigger{display:inline-flex;align-items:center;gap:6px;appearance:none;border:1px solid rgba(255,255,255,.12);background:rgba(6,6,7,.75);color:#e8e8ed;cursor:pointer;padding:6px 10px;border-radius:8px;line-height:1.2;font:inherit;font-weight:600;min-width:0;max-width:148px;transition:border-color .15s ease,color .15s ease,box-shadow .15s ease}
.dg-lang__trigger:hover,.dg-lang__drop.is-open .dg-lang__trigger{border-color:rgba(255,59,95,.35);color:#f5f5f7}
.dg-lang__trigger:focus-visible{outline:2px solid rgba(255,59,95,.45);outline-offset:1px}
.dg-lang__current{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
.dg-lang__chev{flex-shrink:0;opacity:.65;transition:transform .15s ease}
.dg-lang__drop.is-open .dg-lang__chev{transform:rotate(180deg)}
.dg-lang__menu{position:absolute;top:calc(100% + 4px);left:0;min-width:168px;padding:4px;background:#121214;border:1px solid rgba(255,255,255,.1);border-radius:10px;box-shadow:0 12px 32px rgba(0,0,0,.45);z-index:230;display:flex;flex-direction:column;gap:2px}
.dg-lang__menu[hidden]{display:none!important}
.dg-lang__option{display:flex;align-items:center;gap:10px;width:100%;appearance:none;border:0;background:transparent;color:#c8c8d0;cursor:pointer;padding:8px 10px;border-radius:7px;font:inherit;font-weight:500;text-align:left;transition:background .12s ease,color .12s ease}
.dg-lang__option:hover{background:rgba(255,59,95,.12);color:#f5f5f7}
.dg-lang__option.is-active{background:rgba(255,59,95,.18);color:#ff3b5f;font-weight:600}
.dg-lang__option:focus-visible{outline:2px solid rgba(255,59,95,.45);outline-offset:-2px}
.dg-lang__code{font-size:.68rem;font-weight:700;letter-spacing:.06em;opacity:.55;min-width:1.5em}
.dg-lang__option.is-active .dg-lang__code{opacity:.85;color:#ff3b5f}
.dg-lang--nav{margin-right:10px}
.dg-lang--bar{display:none}
.dg-lang--auth{position:fixed;top:18px;right:18px;z-index:120;padding:6px 8px;border-radius:10px;border:1px solid rgba(255,59,95,.18);background:rgba(6,6,7,.82);backdrop-filter:blur(12px)}
.dg-lang--side{width:100%;justify-content:center;margin:0 0 8px;padding:6px 4px;border-radius:10px;border:1px solid rgba(255,255,255,.06);background:rgba(255,255,255,.02)}
@media (max-width:980px){
  .dg-lang--nav{margin:8px 0 4px;width:100%;justify-content:center}
  .topbar-inner>.dg-lang--bar{display:inline-flex;flex-shrink:0;margin-left:auto;margin-right:4px;position:relative;z-index:20}
  .topbar-inner>.dg-lang--bar .dg-lang__trigger{max-width:112px;padding:8px 10px;font-size:0.72rem}
}
@media (max-width:380px){
  .topbar-inner>.dg-lang--bar .dg-lang__trigger{max-width:100px;font-size:0.68rem}
}
`;
    document.head.appendChild(s);
  }

  ensureStyles();
  /* Failsafe: reveal French HTML defaults if locale fetch hangs — do NOT set ready (avoids writing keys) */
  setTimeout(revealI18n, 2500);
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", boot);
  } else {
    boot();
  }

  global.DarkClaudeI18n = {
    t,
    apply,
    setLang,
    getLang: () => currentLang,
    isReady: () => ready,
    onReady,
    injectSwitchers,
    supported: SUPPORTED.slice()
  };
  global.t = t;
})(typeof window !== "undefined" ? window : globalThis);
