const $ = (id) => document.getElementById(id);

function i18nReady() {
  try {
    return !!(window.DarkClaudeI18n && typeof window.DarkClaudeI18n.isReady === "function" && window.DarkClaudeI18n.isReady());
  } catch (_) {
    return false;
  }
}

function tt(key, vars) {
  try {
    if (window.DarkClaudeI18n && typeof window.DarkClaudeI18n.t === "function") {
      const v = window.DarkClaudeI18n.t(key, vars);
      if (v != null) return v;
      /* Locales not loaded yet — do not return the raw key (FOUTC) */
      if (!i18nReady()) return undefined;
    }
  } catch (_) {}
  return key;
}

const chatEl = $("chat");

const formEl = $("chat-form");

const messageEl = $("message");

const sendBtn = $("send-btn");

const healthBtn = $("health-btn");

const statusEl = $("status");

const runtimeInfoEl = $("runtime-info");

const charCountEl = $("char-count");

const loginOpenBtn = $("login-open");

const logoutBtn = $("logout-btn");

const chatLockEl = $("chat-lock");

const authPillEl = $("auth-pill");

const requiresAuthEls = Array.from(document.querySelectorAll(".requires-auth"));

const requiresGuestEls = Array.from(document.querySelectorAll(".requires-guest"));

const pathname = window.location.pathname.toLowerCase();

const isClientPage = pathname.endsWith("/demo.html") || pathname === "/demo.html";

const openLoginButtons = Array.from(document.querySelectorAll(".open-login-btn"));

const metricCounters = Array.from(document.querySelectorAll("[data-count]"));

const SESSION_TOKEN_KEY = "darkclaude_session_token";
const USER_STORAGE_KEY = "darkclaude_user";

async function unregisterServiceWorkers() {
  if (!("serviceWorker" in navigator)) return;
  try {
    const regs = await navigator.serviceWorker.getRegistrations();
    await Promise.all(regs.map((reg) => reg.unregister()));
  } catch (_) {}
}

try {
  localStorage.removeItem("darkclaude_access_key");
} catch (_) {}

let sessionToken = localStorage.getItem(SESSION_TOKEN_KEY) || "";
let currentUser = null;
try {
  currentUser = JSON.parse(localStorage.getItem(USER_STORAGE_KEY) || "null");
} catch {
  currentUser = null;
}
let isAuthenticated = false;
let isPremium = computeIsPremium(currentUser);
let premiumPollTimer = null;

let emptyStateEl = null;
let conversations = [];
let activeConversationId = null;
let healthPollTimer = null;

const CONV_STORAGE_PREFIX = "darkclaude_conversations_v1";
const ACTIVE_CONV_PREFIX = "darkclaude_active_conversation_id_v1";
/** Legacy global keys — never load into a new account (cross-user leak). */
const LEGACY_CONV_STORAGE_KEY = "darkclaude_conversations_v1";
const LEGACY_ACTIVE_CONV_KEY = "darkclaude_active_conversation_id_v1";
const STORAGE_SOUND_SEND = "darkclaude_sound_send";

function conversationUserScope() {
  const id = currentUser?.id;
  if (id != null && String(id).trim()) return `id:${String(id).trim()}`;
  const username = String(currentUser?.username || currentUser?.name || "")
    .trim()
    .toLowerCase();
  if (username) return `u:${username}`;
  return null;
}

function convStorageKey() {
  const scope = conversationUserScope();
  return scope ? `${CONV_STORAGE_PREFIX}__${scope}` : null;
}

function activeConvStorageKey() {
  const scope = conversationUserScope();
  return scope ? `${ACTIVE_CONV_PREFIX}__${scope}` : null;
}

function discardLegacyGlobalConversationKeys() {
  try {
    localStorage.removeItem(LEGACY_CONV_STORAGE_KEY);
    localStorage.removeItem(LEGACY_ACTIVE_CONV_KEY);
  } catch {
    /* ignore */
  }
}

const MAX_MSG_LEN = 4000;
let maxMsgLen = MAX_MSG_LEN;
let CHAR_WARN_AT = Math.floor(maxMsgLen * 0.9);
let CHAR_DANGER_AT = Math.floor(maxMsgLen * 0.975);

function applyMaxMessageChars(n) {
  const v = Number(n);
  if (!Number.isFinite(v) || v < 1) return;
  maxMsgLen = Math.min(Math.max(Math.floor(v), 1), 32000);
  CHAR_WARN_AT = Math.floor(maxMsgLen * 0.9);
  CHAR_DANGER_AT = Math.floor(maxMsgLen * 0.975);
  if (messageEl) {
    messageEl.maxLength = maxMsgLen;
    refreshCharCount();
  }
}

const THEME_KEY = "darkclaude_theme";
const COOKIE_NOTICE_KEY = "darkclaude_cookie_notice_v1";
const CHAT_MODE_KEY = "darkclaude_chat_mode_v2";
const CHAT_MODE_KEY_LEGACY = "darkclaude_chat_mode_v1";
const WORKSPACE_KEY = "darkclaude_workspace_v1";
const HARDNESS_KEY = "darkclaude_hardness_v1";
const CHAT_MODES = {
  nolimit: {
    id: "nolimit",
    labelKey: "chat.modeNoLimit",
    label: "No Limit",
  },
  code: {
    id: "code",
    labelKey: "chat.modeCode",
    label: "Code",
  },
  analyse: {
    id: "analyse",
    labelKey: "chat.modeAnalyse",
    label: "Analyse",
    requiresUnlimited: true,
  },
};

const WORKSPACES = {
  code: { id: "code", labelKey: "ws.code", titleKey: "ws.codeTitle", label: "Code", title: "Atelier Code" },
  legal: { id: "legal", labelKey: "ws.legal", titleKey: "ws.legalTitle", label: "Juridique", title: "Dossier Juridique" },
  campaign: { id: "campaign", labelKey: "ws.campaign", titleKey: "ws.campaignTitle", label: "Campagne", title: "Campagne" },
  journal: { id: "journal", labelKey: "ws.journal", titleKey: "ws.journalTitle", label: "Journal", title: "Journal" },
};

function uiLocaleTag() {
  const lang =
    (window.DarkClaudeI18n && typeof window.DarkClaudeI18n.getLang === "function" && window.DarkClaudeI18n.getLang()) ||
    "fr";
  if (lang === "en") return "en-US";
  if (lang === "es") return "es-ES";
  if (lang === "it") return "it-IT";
  return "fr-FR";
}

function chatModeLabel(modeOrCfg) {
  const cfg =
    typeof modeOrCfg === "object" && modeOrCfg
      ? modeOrCfg
      : CHAT_MODES[normalizeChatMode(modeOrCfg)] || CHAT_MODES.nolimit;
  return tt(cfg.labelKey) || cfg.label || cfg.id;
}

function workspaceLabel(ws) {
  const cfg = typeof ws === "object" && ws ? ws : WORKSPACES[ws] || WORKSPACES.code;
  return tt(cfg.labelKey) || cfg.label || cfg.id;
}

function workspaceTitle(ws) {
  const cfg = typeof ws === "object" && ws ? ws : WORKSPACES[ws] || WORKSPACES.code;
  return tt(cfg.titleKey) || cfg.title || workspaceLabel(cfg);
}

let selectedWorkspace = "code";
let selectedHardness = 55;
let selectedProfile = "";
let artifactsOpen = false;

let activeChatAbort = null;
/** Après échec stream / réponse vide : préférer JSON non-stream pour la session. */
let preferNonStreamChat = false;
/** Empêche double submit (Entrée / clic) → réponses dupliquées. */
let chatRequestInFlight = false;
/** Image jointe (vision) — data URL + métadonnées. */
let pendingAttachImage = null;

function clearPendingAttachImage() {
  pendingAttachImage = null;
  const attachPreview = $("attach-preview");
  const attachPreviewImg = $("attach-preview-img");
  const attachPreviewName = $("attach-preview-name");
  if (attachPreview) attachPreview.hidden = true;
  if (attachPreviewImg) attachPreviewImg.removeAttribute("src");
  if (attachPreviewName) attachPreviewName.textContent = "";
  if (messageEl) messageEl.required = true;
}

function setPendingAttachImage(file, dataUrl) {
  pendingAttachImage = {
    name: file.name,
    mime: file.type || "image/jpeg",
    dataUrl
  };
  const attachPreview = $("attach-preview");
  const attachPreviewImg = $("attach-preview-img");
  const attachPreviewName = $("attach-preview-name");
  if (attachPreviewImg) attachPreviewImg.src = dataUrl;
  if (attachPreviewName) attachPreviewName.textContent = file.name;
  if (attachPreview) attachPreview.hidden = false;
  if (messageEl) messageEl.required = false;
}

function readFileAsDataUrl(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result || ""));
    reader.onerror = () => reject(reader.error || new Error("read failed"));
    reader.readAsDataURL(file);
  });
}

function isImageFile(file) {
  if (!file) return false;
  const mime = String(file.type || "").toLowerCase();
  if (/^image\/(jpeg|jpg|png|webp|gif)$/.test(mime)) return true;
  return /\.(jpe?g|png|webp|gif)$/i.test(String(file.name || ""));
}

/** Timeout client chat (modèles lents / TTFT Venice). Au-delà → « Délai dépassé », pas bulle vide. */
const CHAT_CLIENT_TIMEOUT_MS = 300000;
let selectedChatMode = "nolimit";

function chatModeStorageKey() {
  const scope = conversationUserScope();
  return scope ? `${CHAT_MODE_KEY}__${scope}` : CHAT_MODE_KEY;
}

function normalizeChatMode(mode) {
  if (!mode || typeof mode !== "string") return "nolimit";
  let m = mode.trim().toLowerCase();
  if (m === "no_limit" || m === "no-limit" || m === "aggressive") m = "nolimit";
  if (m === "developer") m = "code";
  if (m === "analyze") m = "analyse";
  return Object.prototype.hasOwnProperty.call(CHAT_MODES, m) ? m : "nolimit";
}

function getCurrentChatModeConfig() {
  const cfg = CHAT_MODES[normalizeChatMode(selectedChatMode)] || CHAT_MODES.nolimit;
  return { ...cfg, label: chatModeLabel(cfg) };
}

function computeIsUnlimited(user) {
  if (!user) return false;
  if (user.admin === true || String(user.username || user.name || "").toLowerCase() === "znano112") {
    return true;
  }
  const plan = String(user.plan || "").toLowerCase();
  return plan === "unlimited" || plan === "black" || plan === "lifetime";
}

function isAnalyseModeLocked() {
  return !computeIsUnlimited(currentUser);
}

/** Discovery / free preview : tous les sélecteurs de mode visibles mais bloqués. */
function areChatModesPreviewLocked() {
  return !isPremium;
}

function nudgeModePaywall(message) {
  if (!isAuthenticated) {
    openLoginModal();
    return;
  }
  if (message) showDgToast(message);
  openPaywallModal();
}

function loadChatModePreference() {
  try {
    const key = chatModeStorageKey();
    let raw = localStorage.getItem(key);
    if (!raw) {
      try {
        raw = localStorage.getItem(CHAT_MODE_KEY_LEGACY);
      } catch {
        raw = null;
      }
    }
    selectedChatMode = normalizeChatMode(raw);
  } catch {
    selectedChatMode = "nolimit";
  }
  if (selectedChatMode === "analyse" && isAnalyseModeLocked()) {
    selectedChatMode = "nolimit";
  }
  try {
    const ws = localStorage.getItem(WORKSPACE_KEY);
    selectedWorkspace = WORKSPACES[ws] ? ws : "code";
  } catch {
    selectedWorkspace = "code";
  }
  try {
    const h = Number(localStorage.getItem(HARDNESS_KEY));
    selectedHardness = Number.isFinite(h) ? Math.max(0, Math.min(100, Math.round(h))) : 55;
  } catch {
    selectedHardness = 55;
  }
}

function saveWorkspacePreference() {
  try {
    localStorage.setItem(WORKSPACE_KEY, selectedWorkspace);
  } catch {
    /* ignore */
  }
}

function saveHardnessPreference() {
  try {
    localStorage.setItem(HARDNESS_KEY, String(selectedHardness));
  } catch {
    /* ignore */
  }
}

function setWorkspace(id, { persistConv = true } = {}) {
  selectedWorkspace = WORKSPACES[id] ? id : "code";
  saveWorkspacePreference();
  document.querySelectorAll(".dos-ws").forEach((btn) => {
    btn.classList.toggle("is-active", btn.dataset.workspace === selectedWorkspace);
  });
  if (persistConv) {
    const conv = getActiveConversation();
    if (conv) {
      conv.workspace = selectedWorkspace;
      saveConversations();
    }
  }
}

function setHardness(value) {
  selectedHardness = Math.max(0, Math.min(100, Math.round(Number(value) || 0)));
  saveHardnessPreference();
  const slider = $("hardness-slider");
  const label = $("hardness-value");
  if (slider && Number(slider.value) !== selectedHardness) slider.value = String(selectedHardness);
  if (label) label.textContent = String(selectedHardness);
}

function setChatProfile(mode, profile = "", { userInitiated = false } = {}) {
  let next = normalizeChatMode(mode);
  // Paywall uniquement sur action utilisateur — jamais pendant sync/init (setupDarkOs).
  if (areChatModesPreviewLocked()) {
    if (userInitiated) {
      nudgeModePaywall(tt("toast.reservedPremium") || "Réservé Premium");
      return;
    }
    // Init / sync UI : peindre le mode courant sans ouvrir le modal.
  } else if (next === "analyse" && isAnalyseModeLocked()) {
    if (userInitiated) {
      showDgToast(tt("toast.reservedUnlimited") || "Réservé Unlimited");
      window.location.href = "./pricing.html";
      return;
    }
    next = "nolimit";
  }
  selectedChatMode = next;
  selectedProfile = String(profile || "").trim().toLowerCase();
  saveChatModePreference();
  syncFixedChatUi();
  syncComposerModeMenu();
  closeComposerModeMenu();
  document.querySelectorAll(".dos-profile").forEach((btn) => {
    const btnMode = normalizeChatMode(btn.dataset.mode || "nolimit");
    const btnProfile = String(btn.dataset.profile || "").trim().toLowerCase();
    let isActive = false;
    if (selectedProfile) {
      isActive = btnProfile === selectedProfile;
    } else {
      isActive = !btnProfile && btnMode === selectedChatMode;
    }
    btn.classList.toggle("is-active", isActive);
  });
}

function syncComposerModeMenu() {
  const cfg = getCurrentChatModeConfig();
  const label = $("composer-mode-label");
  if (label) label.textContent = cfg.label;
  const trigger = $("composer-mode-trigger");
  if (trigger) {
    trigger.setAttribute("aria-label", tt("chat.modeLabel", { label: cfg.label }));
    trigger.title = areChatModesPreviewLocked()
      ? tt("toast.reservedPremium") || "Réservé Premium"
      : cfg.label;
  }
  document.querySelectorAll(".composer-mode-option").forEach((btn) => {
    const mode = normalizeChatMode(btn.dataset.mode || "nolimit");
    const modeCfg = CHAT_MODES[mode] || CHAT_MODES.nolimit;
    btn.textContent = chatModeLabel(modeCfg);
    const needsUnlimited = btn.dataset.requiresUnlimited === "1" || CHAT_MODES[mode]?.requiresUnlimited;
    const isLocked = areChatModesPreviewLocked() || !!(needsUnlimited && isAnalyseModeLocked());
    const isSelected = mode === selectedChatMode;
    btn.classList.toggle("is-selected", isSelected);
    btn.classList.toggle("is-locked", isLocked);
    btn.setAttribute("aria-selected", isSelected ? "true" : "false");
    btn.setAttribute("aria-disabled", isLocked ? "true" : "false");
  });
}

function closeComposerModeMenu() {
  const dropdown = $("composer-mode-dropdown");
  const menu = $("composer-mode-menu");
  const trigger = $("composer-mode-trigger");
  if (dropdown) dropdown.classList.remove("is-open");
  if (menu) menu.hidden = true;
  if (trigger) trigger.setAttribute("aria-expanded", "false");
}

function openComposerModeMenu() {
  const dropdown = $("composer-mode-dropdown");
  const menu = $("composer-mode-menu");
  const trigger = $("composer-mode-trigger");
  if (!dropdown || !menu || !trigger) return;
  dropdown.classList.add("is-open");
  menu.hidden = false;
  trigger.setAttribute("aria-expanded", "true");
}

function syncChatModeSwitcherVisibility() {
  const dropdown = $("composer-mode-dropdown");
  if (!dropdown) return;
  // Discovery : chip visible mais grisé ; Premium : interactif
  const show = isAuthenticated;
  const previewLocked = areChatModesPreviewLocked();
  dropdown.hidden = !show;
  dropdown.classList.toggle("is-locked", show && previewLocked);
  if (show) {
    if (selectedChatMode === "analyse" && isAnalyseModeLocked()) {
      selectedChatMode = "nolimit";
      saveChatModePreference();
    }
    syncComposerModeMenu();
    syncFixedChatUi();
  } else {
    closeComposerModeMenu();
  }
}

function extractCodeArtifacts(text) {
  const src = String(text || "");
  const re = /```([a-zA-Z0-9_+-]*)\n([\s\S]*?)```/g;
  const out = [];
  let m;
  while ((m = re.exec(src)) !== null) {
    const lang = (m[1] || "code").trim() || "code";
    const code = String(m[2] || "").replace(/\n$/, "");
    if (code.trim().length < 8) continue;
    out.push({ id: newId(), lang, code });
  }
  return out;
}

function collectArtifactsFromConversation(conv) {
  const list = [];
  if (!conv || !Array.isArray(conv.messages)) return list;
  conv.messages.forEach((msg) => {
    if (!msg || msg.role !== "assistant") return;
    extractCodeArtifacts(msg.text).forEach((a) => list.push(a));
  });
  return list;
}

function renderArtifactsPanel(artifacts) {
  const body = $("artifacts-body");
  if (!body) return;
  body.innerHTML = "";
  if (!artifacts || !artifacts.length) {
    const empty = document.createElement("p");
    empty.className = "dos-artifacts__empty";
    empty.textContent = tt("chat.artifactsEmpty");
    body.appendChild(empty);
    return;
  }
  artifacts
    .slice()
    .reverse()
    .forEach((art) => {
      const card = document.createElement("article");
      card.className = "dos-artifact";
      const bar = document.createElement("div");
      bar.className = "dos-artifact__bar";
      const lang = document.createElement("span");
      lang.className = "dos-artifact__lang";
      lang.textContent = art.lang || "code";
      const copy = document.createElement("button");
      copy.type = "button";
      copy.className = "dos-artifact__copy";
      copy.textContent = tt("common.copy");
      copy.addEventListener("click", async () => {
        const ok = await copyTextToClipboard(art.code);
        if (ok) {
          copy.textContent = tt("common.copied") || "Copied";
          setTimeout(() => {
            copy.textContent = tt("common.copy") || "Copy";
          }, 1600);
        } else {
          showDgToast(tt("common.copyFailed") || "Unable to copy");
        }
      });
      bar.appendChild(lang);
      bar.appendChild(copy);
      const pre = document.createElement("pre");
      pre.textContent = art.code;
      card.appendChild(bar);
      card.appendChild(pre);
      body.appendChild(card);
    });
}

function refreshArtifactsFromActive() {
  renderArtifactsPanel(collectArtifactsFromConversation(getActiveConversation()));
}

function setArtifactsOpen(open) {
  artifactsOpen = !!open;
  const shell = document.querySelector(".app-shell--gpt");
  const toggle = $("artifacts-toggle");
  if (shell) shell.classList.toggle("has-artifacts", artifactsOpen);
  if (toggle) {
    toggle.classList.toggle("is-on", artifactsOpen);
    toggle.setAttribute("aria-pressed", artifactsOpen ? "true" : "false");
  }
  if (artifactsOpen) refreshArtifactsFromActive();
}

function setLocalFirstBadge(state) {
  const badge = $("local-first-badge");
  const label = $("local-first-label");
  if (!badge) return;
  badge.classList.toggle("is-offline", state === "offline" || state === "degraded");
  if (label) {
    if (state === "online") label.textContent = tt("chat.ready");
    else if (state === "degraded") label.textContent = tt("chat.degraded");
    else label.textContent = tt("chat.offline");
  }
  badge.title =
    state === "online"
      ? "Service prêt"
      : state === "degraded"
        ? "Réponse dégradée"
        : "Service hors ligne";
}

async function shareToDiscord() {
  const conv = getActiveConversation();
  let text = "";
  if (conv && Array.isArray(conv.messages)) {
    for (let i = conv.messages.length - 1; i >= 0; i--) {
      if (conv.messages[i].role === "assistant" && String(conv.messages[i].text || "").trim()) {
        text = String(conv.messages[i].text);
        break;
      }
    }
  }
  if (!text) {
    showDgToast(tt("toast.nothingToShare") || "Nothing to share yet");
    return;
  }
  const clipped = text.length > 1800 ? `${text.slice(0, 1800)}…` : text;
  const payload = `**DarkClaude**\n\n${clipped}`;
  if (await copyTextToClipboard(payload)) {
    showDgToast(tt("toast.copiedDiscord") || "Copied — paste into Discord");
  } else {
    showDgToast(tt("common.copyFailed") || "Unable to copy");
  }
  window.open("https://chat.whatsapp.com/LnuE7yJ0MhX2pPyrdMMbxD", "_blank", "noopener,noreferrer");
}

function setupDarkOs() {
  if (!document.body.classList.contains("app-chat")) return;

  setHardness(selectedHardness);
  setChatProfile(selectedChatMode, selectedProfile);

  document.querySelectorAll(".dos-profile").forEach((btn) => {
    if (btn.dataset.bound) return;
    btn.dataset.bound = "1";
    btn.addEventListener("click", () => {
      setChatProfile(btn.dataset.mode || "nolimit", btn.dataset.profile || "", {
        userInitiated: true
      });
    });
  });

  const slider = $("hardness-slider");
  if (slider && !slider.dataset.bound) {
    slider.dataset.bound = "1";
    slider.addEventListener("input", () => setHardness(slider.value));
  }

  const artToggle = $("artifacts-toggle");
  if (artToggle && !artToggle.dataset.bound) {
    artToggle.dataset.bound = "1";
    artToggle.addEventListener("click", () => setArtifactsOpen(!artifactsOpen));
  }
  const artClose = $("artifacts-close");
  if (artClose && !artClose.dataset.bound) {
    artClose.dataset.bound = "1";
    artClose.addEventListener("click", () => setArtifactsOpen(false));
  }

  const shareBtn = $("share-discord-btn");
  if (shareBtn && !shareBtn.dataset.bound) {
    shareBtn.dataset.bound = "1";
    shareBtn.addEventListener("click", () => {
      shareToDiscord();
    });
  }

  refreshArtifactsFromActive();
}

function saveChatModePreference() {
  try {
    localStorage.setItem(chatModeStorageKey(), normalizeChatMode(selectedChatMode));
  } catch {
    /* ignore */
  }
}

function sanitizeAssistantHtml(html) {
  if (typeof DOMPurify !== "undefined") {
    return DOMPurify.sanitize(String(html || ""), {
      ALLOWED_TAGS: [
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "p",
        "br",
        "hr",
        "strong",
        "b",
        "em",
        "i",
        "ul",
        "ol",
        "li",
        "pre",
        "code",
        "a",
        "blockquote",
        "span",
        "div",
      ],
      ALLOWED_ATTR: ["href", "title", "target", "rel", "class", "data-lang"],
    });
  }
  return String(html || "");
}

/** Rendu markdown assistant — marked sur le message entier, sinon fallback local solide. */
function renderAssistantMarkdown(text) {
  const raw = String(text || "").replace(/\r\n?/g, "\n");
  if (!raw) return "";

  const esc = (s) =>
    String(s)
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");

  const renderInlineMd = (escaped) =>
    escaped
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/__(.+?)__/g, "<strong>$1</strong>")
      .replace(/(^|[\s(])\*(?!\s)(.+?)(?!\s)\*(?=[\s).,!?:;]|$)/g, "$1<em>$2</em>")
      .replace(/`([^`\n]+)`/g, "<code>$1</code>");

  const flushParagraphLines = (lines) => {
    const parts = [];
    let para = [];
    let listType = null;
    let listItems = [];

    const flushPara = () => {
      if (!para.length) return;
      const joined = para.join("<br>");
      const onlyStrong = /^<strong>[^<]+<\/strong>$/.test(joined);
      parts.push(`<p${onlyStrong ? ' class="msg-md-title"' : ""}>${joined}</p>`);
      para = [];
    };

    const flushList = () => {
      if (!listType || !listItems.length) {
        listType = null;
        listItems = [];
        return;
      }
      const tag = listType;
      parts.push(`<${tag}>${listItems.map((li) => `<li>${li}</li>`).join("")}</${tag}>`);
      listType = null;
      listItems = [];
    };

    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed) {
        flushList();
        flushPara();
        continue;
      }
      const heading = trimmed.match(/^(#{1,6})\s+(.+)$/);
      if (heading) {
        flushList();
        flushPara();
        const level = Math.min(6, heading[1].length);
        parts.push(`<h${level}>${renderInlineMd(esc(heading[2].trim()))}</h${level}>`);
        continue;
      }
      const ul = trimmed.match(/^[-*+]\s+(.+)$/);
      const ol = trimmed.match(/^\d+[.)]\s+(.+)$/);
      if (ul || ol) {
        flushPara();
        const nextType = ul ? "ul" : "ol";
        if (listType && listType !== nextType) flushList();
        listType = nextType;
        listItems.push(renderInlineMd(esc((ul || ol)[1])));
        continue;
      }
      flushList();
      para.push(renderInlineMd(esc(trimmed)));
    }
    flushList();
    flushPara();
    return parts.join("");
  };

  /** Fallback local : gère fences + titres/listes/gras même sans marked. */
  const renderLocalMarkdown = (input) => {
    const lines = String(input || "").split("\n");
    const chunks = [];
    let inCode = false;
    let codeLang = "";
    let codeLines = [];
    let textLines = [];

    const flushText = () => {
      if (!textLines.length) return;
      chunks.push(flushParagraphLines(textLines));
      textLines = [];
    };

    const flushCode = () => {
      const lang = codeLang ? esc(codeLang) : "";
      const langAttr = lang ? ` data-lang="${lang}" class="language-${lang}"` : "";
      const codeClass = lang ? ` class="language-${lang}"` : "";
      chunks.push(`<pre${langAttr}><code${codeClass}>${esc(codeLines.join("\n"))}</code></pre>`);
      codeLines = [];
      codeLang = "";
    };

    for (const line of lines) {
      const fenceMatch = line.match(/^(\s*)```\s*([a-zA-Z0-9_+#-]*)\s*$/);
      if (fenceMatch) {
        if (!inCode) {
          flushText();
          inCode = true;
          codeLang = fenceMatch[2] || "";
        } else {
          flushCode();
          inCode = false;
        }
        continue;
      }
      if (inCode) codeLines.push(line);
      else textLines.push(line);
    }
    // Fence non fermée (stream / réponse tronquée) : afficher le code quand même
    if (inCode) flushCode();
    flushText();
    return chunks.join("");
  };

  // Toujours parser le message ENTIER avec marked (fences inclus).
  // Ne jamais échapper tout le message dès qu'un fence est détecté.
  let html = "";
  if (typeof marked !== "undefined") {
    try {
      html = marked.parse(raw, { breaks: true, gfm: true });
    } catch {
      html = "";
    }
  }
  if (!html) {
    html = renderLocalMarkdown(raw);
  }

  return sanitizeAssistantHtml(html);
}

function enhanceAssistantMarkdownTitles(container) {
  if (!container) return;
  container.querySelectorAll("p").forEach((p) => {
    const kids = Array.from(p.childNodes).filter(
      (n) => !(n.nodeType === 3 && !String(n.textContent || "").trim())
    );
    if (
      kids.length === 1 &&
      kids[0].nodeType === 1 &&
      kids[0].tagName === "STRONG" &&
      !String(kids[0].textContent || "").includes("\n")
    ) {
      p.classList.add("msg-md-title");
    }
  });
}

function enhanceAssistantCodeBlocks(container) {
  if (!container) return;
  const preBlocks = Array.from(container.querySelectorAll("pre"));
  preBlocks.forEach((pre) => {
    if (pre.parentElement?.classList?.contains("msg-code-wrap")) return;

    const wrap = document.createElement("div");
    wrap.className = "msg-code-wrap";

    const header = document.createElement("div");
    header.className = "msg-code-head";
    const codeEl = pre.querySelector("code");
    const rawLang = pre.getAttribute("data-lang") || codeEl?.className || "";
    const lang = String(rawLang)
      .replace(/^language-/, "")
      .trim()
      .toLowerCase();
    const langLabel = document.createElement("span");
    langLabel.className = "msg-code-lang";
    langLabel.textContent = lang || "code";

    const copyBtn = document.createElement("button");
    copyBtn.type = "button";
    copyBtn.className = "msg-code-copy";
    copyBtn.innerHTML =
      `<span class="msg-code-copy__icon" aria-hidden="true">⧉</span><span class="msg-code-copy__label">${tt("common.copy")}</span>`;
    copyBtn.setAttribute("aria-label", tt("chat.copyCode"));

    copyBtn.addEventListener("click", async () => {
      const innerCodeEl = pre.querySelector("code");
      const codeText = innerCodeEl ? innerCodeEl.textContent || "" : pre.textContent || "";
      const ok = await copyTextToClipboard(codeText);
      if (ok) {
        copyBtn.classList.add("is-copied");
        const lbl = copyBtn.querySelector(".msg-code-copy__label");
        if (lbl) lbl.textContent = tt("common.copied") || "Copied";
        setTimeout(() => {
          copyBtn.classList.remove("is-copied");
          const resetLbl = copyBtn.querySelector(".msg-code-copy__label");
          if (resetLbl) resetLbl.textContent = tt("common.copy") || "Copy";
        }, 1800);
      } else {
        showDgToast(tt("common.copyFailed") || "Unable to copy");
      }
    });

    if (typeof hljs !== "undefined" && codeEl && !codeEl.classList.contains("hljs")) {
      try {
        if (lang) codeEl.classList.add(`language-${lang}`);
        hljs.highlightElement(codeEl);
      } catch {
        /* ignore */
      }
    }

    pre.parentNode.insertBefore(wrap, pre);
    header.appendChild(langLabel);
    header.appendChild(copyBtn);
    wrap.appendChild(header);
    wrap.appendChild(pre);
  });
}

function initTheme() {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    const useLight = saved === "light";
    document.documentElement.setAttribute("data-theme", useLight ? "light" : "dark");
  } catch {
    document.documentElement.setAttribute("data-theme", "dark");
  }
  const btn = $("theme-toggle");
  if (btn) {
    btn.textContent = document.documentElement.getAttribute("data-theme") === "light" ? "â—‘" : "â—";
  }
}

function toggleTheme() {
  const cur = document.documentElement.getAttribute("data-theme") === "light" ? "light" : "dark";
  const next = cur === "light" ? "dark" : "light";
  document.documentElement.setAttribute("data-theme", next === "light" ? "light" : "dark");
  try {
    localStorage.setItem(THEME_KEY, next);
  } catch {
    /* ignore */
  }
  const btn = $("theme-toggle");
  if (btn) btn.textContent = next === "light" ? "â—‘" : "â—";
}

function initCookieBanner() {
  const b = $("cookie-banner");
  const ok = $("cookie-accept");
  if (!b || !ok) return;
  try {
    if (localStorage.getItem(COOKIE_NOTICE_KEY)) {
      b.classList.add("hidden");
      return;
    }
  } catch {
    /* ignore */
  }
  b.classList.remove("hidden");
  ok.addEventListener("click", () => {
    try {
      localStorage.setItem(COOKIE_NOTICE_KEY, "1");
    } catch {
      /* ignore */
    }
    b.classList.add("hidden");
  });
}

function historyFilterQuery() {
  const el = $("chat-history-filter");
  if (!el || typeof el.value !== "string") return "";
  return el.value.trim().toLowerCase();
}

/** Affiche le nom du modèle réellement appelé (en-têtes du serveur). */
function applyResolvedModelHeaders(response) {
  if (!response || !response.ok) return;
  const model = response.headers.get("X-Resolved-Model");
  const modeHdr = response.headers.get("X-Chat-Mode");
  const pillModel = document.querySelector(".app-model-pill__model");
  if (!pillModel || !model) return;
  pillModel.textContent = model;
  if (modeHdr) pillModel.title = tt("chat.modeChatTitle", { mode: modeHdr }) || `Chat mode: ${modeHdr}`;
}

function syncFixedChatUi() {
  const labelEl = $("chat-mode-label");
  const cfg = getCurrentChatModeConfig();
  if (labelEl) labelEl.textContent = cfg.label;
  const top = $("topbar-mode-label");
  if (top) top.textContent = cfg.label;
  const chip = $("chip-mode-label");
  if (chip) chip.textContent = cfg.label;
  const composerLabel = $("composer-mode-label");
  if (composerLabel) composerLabel.textContent = cfg.label;
  const pill = document.querySelector(".app-model-pill");
  if (pill) pill.title = cfg.label;
  syncComposerModeMenu();
}

function setupFixedChatUi() {
  const modelPill = document.querySelector(".app-model-pill");
  syncFixedChatUi();
  if (modelPill && !modelPill.dataset.modeBound) {
    modelPill.dataset.modeBound = "1";
    modelPill.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      if (!isAuthenticated) {
        openLoginModal();
        return;
      }
      if (areChatModesPreviewLocked()) {
        nudgeModePaywall(tt("toast.reservedPremium") || "Réservé Premium");
        return;
      }
      const dropdown = $("composer-mode-dropdown");
      const trigger = $("composer-mode-trigger");
      if (dropdown && !dropdown.hidden && trigger) {
        dropdown.scrollIntoView({ behavior: "smooth", block: "nearest" });
        openComposerModeMenu();
      }
    });
  }
}

function setupChatModeUi() {
  if (!document.body.classList.contains("app-chat")) return;
  const dropdown = $("composer-mode-dropdown");
  const trigger = $("composer-mode-trigger");
  const menu = $("composer-mode-menu");

  if (trigger && !trigger.dataset.bound) {
    trigger.dataset.bound = "1";
    trigger.addEventListener("click", (e) => {
      e.preventDefault();
      e.stopPropagation();
      if (!isAuthenticated) {
        openLoginModal();
        return;
      }
      if (areChatModesPreviewLocked()) {
        nudgeModePaywall(tt("toast.reservedPremium") || "Réservé Premium");
        return;
      }
      const isOpen = dropdown && dropdown.classList.contains("is-open");
      if (isOpen) closeComposerModeMenu();
      else openComposerModeMenu();
    });
  }

  if (menu) {
    menu.querySelectorAll(".composer-mode-option").forEach((btn) => {
      if (btn.dataset.bound) return;
      btn.dataset.bound = "1";
      btn.addEventListener("click", (e) => {
        e.preventDefault();
        e.stopPropagation();
        const mode = normalizeChatMode(btn.dataset.mode || "nolimit");
        if (areChatModesPreviewLocked()) {
          closeComposerModeMenu();
          nudgeModePaywall(tt("toast.reservedPremium") || "Réservé Premium");
          return;
        }
        const needsUnlimited = btn.dataset.requiresUnlimited === "1" || CHAT_MODES[mode]?.requiresUnlimited;
        if (needsUnlimited && isAnalyseModeLocked()) {
          closeComposerModeMenu();
          showDgToast(tt("toast.reservedUnlimited") || "Réservé Unlimited");
          window.location.href = "./pricing.html";
          return;
        }
        setChatProfile(mode, "");
      });
    });
  }

  if (!document.documentElement.dataset.composerModeDocBound) {
    document.documentElement.dataset.composerModeDocBound = "1";
    document.addEventListener("click", (e) => {
      if (!dropdown || dropdown.hidden || !dropdown.classList.contains("is-open")) return;
      if (dropdown.contains(e.target)) return;
      closeComposerModeMenu();
    });
    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape") closeComposerModeMenu();
    });
  }

  syncChatModeSwitcherVisibility();
}

function saveActiveConversationId() {
  const key = activeConvStorageKey();
  if (!key) return;
  try {
    if (activeConversationId) {
      localStorage.setItem(key, activeConversationId);
    } else {
      localStorage.removeItem(key);
    }
    discardLegacyGlobalConversationKeys();
  } catch {
    /* ignore */
  }
}

function newId() {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 9)}`;
}

function clearConversationsMemory() {
  conversations = [];
  activeConversationId = null;
}

function loadConversations() {
  const key = convStorageKey();
  if (!key) {
    conversations = [];
    return;
  }
  try {
    const raw = localStorage.getItem(key);
    const parsed = raw ? JSON.parse(raw) : null;
    conversations = Array.isArray(parsed) ? parsed : [];
  } catch {
    conversations = [];
  }
}

function saveConversations() {
  const key = convStorageKey();
  if (!key) return;
  try {
    localStorage.setItem(key, JSON.stringify(conversations));
    discardLegacyGlobalConversationKeys();
  } catch {
    // ignore quota errors
  }
}

/** Load only the current user's threads (empty if no session user). */
function hydrateClientConversations() {
  loadConversations();
  let savedActiveId = null;
  const activeKey = activeConvStorageKey();
  if (activeKey) {
    try {
      savedActiveId = localStorage.getItem(activeKey);
    } catch {
      savedActiveId = null;
    }
  }
  if (savedActiveId && conversations.some((c) => c.id === savedActiveId)) {
    activeConversationId = savedActiveId;
  } else if (conversations.length) {
    activeConversationId = conversations[0].id;
  } else {
    activeConversationId = null;
  }
  if (conversationUserScope()) {
    ensureConversation();
    saveActiveConversationId();
  }
  renderHistory();
  renderConversation(getActiveConversation());
}

function getActiveConversation() {
  return conversations.find((c) => c.id === activeConversationId) || null;
}

function ensureConversation() {
  if (activeConversationId && getActiveConversation()) return;
  const conv = {
    id: newId(),
    title: tt("chat.conversation"),
    createdAt: Date.now(),
    workspace: selectedWorkspace,
    messages: []
  };
  conversations.unshift(conv);
  activeConversationId = conv.id;
  saveConversations();
  saveActiveConversationId();
}

function formatHistoryDate(ts) {
  const d = new Date(ts || Date.now());
  if (Number.isNaN(d.getTime())) return "";
  const now = new Date();
  const sameDay = d.toDateString() === now.toDateString();
  if (sameDay) {
    return d.toLocaleTimeString(uiLocaleTag(), { hour: "2-digit", minute: "2-digit" });
  }
  return d.toLocaleDateString(uiLocaleTag(), { day: "numeric", month: "short" });
}

function showDgToast(msg) {
  const el = $("dg-toast");
  if (!el) return;
  el.textContent = String(msg || "");
  el.classList.add("is-on");
  clearTimeout(showDgToast._t);
  showDgToast._t = setTimeout(() => el.classList.remove("is-on"), 2200);
}

/** Clipboard API + fallback textarea (iOS / contextes non sécurisés). */
async function copyTextToClipboard(text) {
  const value = String(text ?? "");
  try {
    if (navigator.clipboard && typeof navigator.clipboard.writeText === "function") {
      await navigator.clipboard.writeText(value);
      return true;
    }
  } catch {
    /* fall through */
  }
  try {
    const ta = document.createElement("textarea");
    ta.value = value;
    ta.setAttribute("readonly", "");
    ta.style.cssText = "position:fixed;left:-9999px;top:0;opacity:0;";
    document.body.appendChild(ta);
    ta.focus();
    ta.select();
    ta.setSelectionRange(0, value.length);
    const ok = document.execCommand("copy");
    document.body.removeChild(ta);
    return Boolean(ok);
  } catch {
    return false;
  }
}

function renderHistory() {
  const historyEl = $("chat-history");
  if (!historyEl) return;

  historyEl.innerHTML = "";

  const q = historyFilterQuery();

  conversations.forEach((conv) => {
    if (q) {
      const blob = `${conv.title || ""} ${(conv.messages || []).map((m) => m.text).join(" ")}`.toLowerCase();
      if (!blob.includes(q)) return;
    }
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `gpt-history__item${conv.id === activeConversationId ? " is-active" : ""}`;
    btn.dataset.convId = conv.id;

    const icon = document.createElement("span");
    icon.className = "gpt-history__icon";
    icon.setAttribute("aria-hidden", "true");
    icon.innerHTML =
      '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>';

    const meta = document.createElement("div");
    meta.className = "gpt-history__meta";

    const title = document.createElement("span");
    title.className = "gpt-history__title";
    title.textContent = conv.title || tt("chat.conversation");

    const dateEl = document.createElement("span");
    dateEl.className = "gpt-history__date";
    const lastTs =
      (conv.messages && conv.messages.length && conv.messages[conv.messages.length - 1].ts) ||
      conv.createdAt;
    dateEl.textContent = formatHistoryDate(lastTs);

    meta.appendChild(title);
    meta.appendChild(dateEl);

    const del = document.createElement("button");
    del.type = "button";
    del.className = "gpt-history__del";
    del.dataset.convDel = conv.id;
    del.setAttribute("aria-label", tt("common.delete"));
    del.innerHTML =
      '<svg class="gpt-history__del-icon" xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.25" stroke-linecap="round" aria-hidden="true"><path d="M18 6 6 18M6 6l12 12"/></svg>';

    btn.appendChild(icon);
    btn.appendChild(meta);
    btn.appendChild(del);
    historyEl.appendChild(btn);
  });
  updateSidebarPremiumStats();
}

function planReminderKey(user) {
  if (!user) return "discovery";
  if (user.admin === true) return "admin";
  const p = String(user.plan || "").toLowerCase();
  if (p === "unlimited" || p === "black") return "unlimited";
  if (p === "lifetime") return "lifetime";
  if (p === "starter" || p === "monthly" || p === "premium" || p === "red") return "starter";
  if (computeIsUnlimited(user)) return "unlimited";
  if (computeIsPremium(user)) return "starter";
  return "discovery";
}

function planReminderLabel(user) {
  const key = planReminderKey(user);
  if (key === "discovery") return tt("plan.discovery");
  if (key === "admin") return tt("plan.admin");
  if (key === "unlimited") return tt("plan.unlimited");
  if (key === "lifetime") return tt("plan.lifetime");
  if (key === "starter") return tt("plan.starter");
  return tt("plan.discovery");
}

function updateSidebarPremiumStats() {
  const avatarEl = $("side-user-avatar");
  const metaEl = $("side-user-meta");
  const profileChip = $("chip-profile-btn");
  const name = (currentUser?.username || currentUser?.name || "D").trim();
  const letter = (name[0] || "D").toUpperCase();
  if (avatarEl) {
    avatarEl.textContent = letter;
    avatarEl.dataset.set = "1";
  }
  if (profileChip) profileChip.textContent = letter;
  if (metaEl) {
    const label = planReminderLabel(currentUser);
    metaEl.textContent = label;
    metaEl.classList.toggle("is-free", planReminderKey(currentUser) === "discovery");
  }
}

function renderConversation(conv) {
  if (!chatEl) return;
  chatEl.innerHTML = "";

  if (!conv || !Array.isArray(conv.messages) || conv.messages.length === 0) {
    if (emptyStateEl) emptyStateEl.classList.remove("is-hidden");
    refreshArtifactsFromActive();
    return;
  }

  conv.messages.forEach((m) => {
    const t = typeof m.ts === "number" ? m.ts : conv.createdAt || Date.now();
    addMessage(m.role, m.text, { persist: false, ts: t });
  });
  if (emptyStateEl) emptyStateEl.classList.add("is-hidden");
  refreshArtifactsFromActive();
}

function setActiveConversation(id) {
  activeConversationId = id;
  saveActiveConversationId();
  const conv = getActiveConversation();
  renderHistory();
  renderConversation(conv);
}



function setText(el, text) {

  if (el) el.textContent = text;

}

function setLoginStatus(text, variant = "neutral") {

  if (!loginStatusEl) return;

  loginStatusEl.textContent = text;

  loginStatusEl.classList.remove("is-error", "is-ok");

  if (variant === "error") loginStatusEl.classList.add("is-error");

  if (variant === "ok") loginStatusEl.classList.add("is-ok");

}



async function readJsonSafely(response) {

  const text = await response.text();

  try {

    return { data: JSON.parse(text), raw: text };

  } catch {

    return { data: null, raw: text };

  }

}



function formatMsgTime(ts) {
  if (ts == null || Number.isNaN(Number(ts))) return "—";
  const d = new Date(ts);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleString(uiLocaleTag(), {
    day: "2-digit",
    month: "short",
    hour: "2-digit",
    minute: "2-digit",
  });
}

const DARKCLAUDE_MARK_SRC = "/brand/darkgpt-mark.png?v=2";

function fillMsgAvatar(avatarEl, role) {
  if (!avatarEl) return;
  avatarEl.replaceChildren();
  avatarEl.classList.remove("msg-avatar--mark");
  avatarEl.setAttribute("aria-hidden", "true");
  if (role === "assistant") {
    avatarEl.classList.add("msg-avatar--mark");
    const img = document.createElement("img");
    img.className = "msg-avatar__mark";
    img.src = DARKCLAUDE_MARK_SRC;
    img.alt = "";
    img.decoding = "async";
    img.draggable = false;
    avatarEl.appendChild(img);
    return;
  }
  avatarEl.textContent = "V";
}

function addMessage(role, text, opts = {}) {
  const persist = opts.persist !== false;
  const ts = typeof opts.ts === "number" ? opts.ts : Date.now();
  const latencyMs = typeof opts.latencyMs === "number" ? opts.latencyMs : null;
  const imageUrl = typeof opts.imageUrl === "string" ? opts.imageUrl : "";

  if (!chatEl) return null;

  const div = document.createElement("div");
  div.className = `msg ${role}`;

  const avatar = document.createElement("div");
  avatar.className = "msg-avatar";
  fillMsgAvatar(avatar, role);

  const body = document.createElement("div");
  body.className = "msg-body";

  const label = document.createElement("div");
  label.className = "msg-label";
  label.textContent = role === "user" ? tt("common.you") : "DarkClaude";

  const bubble = document.createElement("div");
  bubble.className = "msg-bubble";
  if (role === "assistant") {
    bubble.classList.add("msg-bubble--md");
    bubble.innerHTML = renderAssistantMarkdown(text);
    enhanceAssistantMarkdownTitles(bubble);
    enhanceAssistantCodeBlocks(bubble);
  } else {
    if (imageUrl && /^data:image\//i.test(imageUrl)) {
      const img = document.createElement("img");
      img.className = "msg-attach-img";
      img.src = imageUrl;
      img.alt = tt("chat.attachedImageAlt") || "Attached image";
      img.loading = "lazy";
      bubble.appendChild(img);
    }
    if (text) {
      const span = document.createElement("div");
      span.className = "msg-user-text";
      span.textContent = text;
      bubble.appendChild(span);
    } else if (!imageUrl) {
      bubble.textContent = "";
    }
  }

  const meta = document.createElement("div");
  meta.className = "msg-meta";

  const timeEl = document.createElement("time");
  timeEl.className = "msg-time";
  const timeD = new Date(ts);
  timeEl.dateTime = Number.isNaN(timeD.getTime()) ? "" : timeD.toISOString();
  timeEl.textContent = formatMsgTime(ts);
  meta.appendChild(timeEl);

  if (role === "assistant" && latencyMs != null && latencyMs >= 0) {
    const lat = document.createElement("span");
    lat.className = "msg-latency";
    lat.textContent = `${(latencyMs / 1000).toFixed(1)}s`;
    meta.appendChild(lat);
  }

  const copyBtn = document.createElement("button");
  copyBtn.type = "button";
  copyBtn.className = "msg-action msg-copy";
  copyBtn.setAttribute("aria-label", tt("common.copy"));
  copyBtn.title = tt("common.copy");
  copyBtn.innerHTML =
    '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  copyBtn.addEventListener("click", async () => {
    const ok = await copyTextToClipboard(String(text || ""));
    if (ok) {
      copyBtn.classList.add("is-copied");
      copyBtn.title = tt("common.copied") || "Copied";
      showDgToast(tt("common.copied") || "Copied");
      setTimeout(() => {
        copyBtn.classList.remove("is-copied");
        copyBtn.title = tt("common.copy") || "Copy";
      }, 2000);
    } else {
      showDgToast(tt("common.copyFailed") || "Unable to copy");
    }
  });
  meta.appendChild(copyBtn);

  if (role === "assistant") {
    const regenBtn = document.createElement("button");
    regenBtn.type = "button";
    regenBtn.className = "msg-action msg-regen";
    regenBtn.setAttribute("aria-label", tt("chat.regen"));
    regenBtn.title = tt("chat.regen") || "Regenerate";
    regenBtn.innerHTML =
      '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" aria-hidden="true"><path d="M21 12a9 9 0 1 1-2.6-6.3"/><path d="M21 3v6h-6"/></svg>';
    regenBtn.addEventListener("click", () => {
      const conv = getActiveConversation();
      if (!conv || !Array.isArray(conv.messages)) return;
      let lastUser = "";
      for (let i = conv.messages.length - 1; i >= 0; i--) {
        if (conv.messages[i].role === "user") {
          lastUser = conv.messages[i].text;
          break;
        }
      }
      if (!lastUser || !messageEl || !formEl) return;
      messageEl.value = lastUser;
      refreshCharCount();
      formEl.requestSubmit();
    });
    meta.appendChild(regenBtn);
  }

  body.appendChild(label);
  body.appendChild(bubble);
  body.appendChild(meta);

  /* Ordre DOM stable pour la grille CSS (pas de row-reverse) */
  if (role === "user") {
    div.appendChild(body);
    div.appendChild(avatar);
  } else {
    div.appendChild(avatar);
    div.appendChild(body);
  }
  chatEl.appendChild(div);
  chatEl.scrollTop = chatEl.scrollHeight;

  if (emptyStateEl) emptyStateEl.classList.add("is-hidden");

  if (persist && document.body.classList.contains("app-chat")) {
    ensureConversation();
    const conv = getActiveConversation();
    if (conv) {
      conv.messages.push({ role, text, ts, latencyMs });
      if (!conv.workspace) conv.workspace = selectedWorkspace;
      if (role === "user" && (!conv.title || conv.title === "Conversation" || conv.title === tt("chat.conversation"))) {
        conv.title = text.length > 26 ? `${text.slice(0, 26)}…` : text;
      }
      saveConversations();
      renderHistory();
      if (role === "assistant") refreshArtifactsFromActive();
    }
  }

  return div;
}

function addThinkingMessage() {

  if (!chatEl) return null;

  const wrap = document.createElement("div");
  wrap.className = "msg assistant thinking";

  const avatar = document.createElement("div");
  avatar.className = "msg-avatar";
  fillMsgAvatar(avatar, "assistant");

  const body = document.createElement("div");
  body.className = "msg-body";

  const top = document.createElement("div");
  top.className = "msg-label";
  top.textContent = "DarkClaude";

  const bubble = document.createElement("div");
  bubble.className = "msg-bubble msg-bubble--thinking";

  const dots = document.createElement("span");
  dots.className = "thinking-dots";
  dots.innerHTML = "<span></span><span></span><span></span>";

  const label = document.createElement("span");
  label.className = "thinking-text";
  const phrases = [tt("chat.thinking1"), tt("chat.thinking2"), tt("chat.thinking3"), tt("chat.thinking4")];
  let phraseIdx = 0;
  label.textContent = phrases[0];

  const meta = document.createElement("span");
  meta.className = "thinking-meta";
  meta.textContent = "0s";

  bubble.appendChild(dots);
  bubble.appendChild(label);
  bubble.appendChild(meta);
  body.appendChild(top);
  body.appendChild(bubble);
  wrap.appendChild(avatar);
  wrap.appendChild(body);

  chatEl.appendChild(wrap);
  chatEl.scrollTop = chatEl.scrollHeight;
  if (emptyStateEl) emptyStateEl.classList.add("is-hidden");

  const started = Date.now();
  const timer = setInterval(() => {
    const secs = Math.max(0, Math.floor((Date.now() - started) / 1000));
    meta.textContent = `${secs}s`;
    if (secs > 0 && secs % 3 === 0) {
      phraseIdx = (phraseIdx + 1) % phrases.length;
      label.textContent = phrases[phraseIdx];
    }
  }, 250);

  return { el: wrap, timer, started };

}

function addOfflineMessage(opts = {}) {
  if (!chatEl) return null;
  const title = opts.title || tt("chat.offlineTitle");
  const desc =
    opts.desc || tt("chat.offlineDesc");
  const retryText = opts.retryText || tt("common.retry");

  const wrap = document.createElement("div");
  wrap.className = "msg assistant msg--offline";

  const avatar = document.createElement("div");
  avatar.className = "msg-avatar";
  fillMsgAvatar(avatar, "assistant");

  const body = document.createElement("div");
  body.className = "msg-body";

  const lab = document.createElement("div");
  lab.className = "msg-label";
  lab.textContent = "DarkClaude";

  const bubble = document.createElement("div");
  bubble.className = "msg-bubble msg-bubble--offline";

  const t = document.createElement("p");
  t.className = "msg-offline__title";
  t.textContent = title;

  const d = document.createElement("p");
  d.className = "msg-offline__desc";
  d.textContent = desc;

  const retry = document.createElement("button");
  retry.type = "button";
  retry.className = "msg-offline__retry";
  retry.textContent = retryText;
  retry.addEventListener("click", () => {
    if (opts.kind === "auth") {
      openLoginModal();
      return;
    }
    const conv = getActiveConversation();
    if (!conv || !Array.isArray(conv.messages) || !messageEl || !formEl) return;
    let lastUser = "";
    for (let i = conv.messages.length - 1; i >= 0; i--) {
      if (conv.messages[i].role === "user") {
        lastUser = conv.messages[i].text;
        break;
      }
    }
    if (!lastUser) return;
    messageEl.value = lastUser;
    refreshCharCount();
    formEl.requestSubmit();
  });

  bubble.appendChild(t);
  bubble.appendChild(d);
  bubble.appendChild(retry);
  body.appendChild(lab);
  body.appendChild(bubble);
  wrap.appendChild(avatar);
  wrap.appendChild(body);
  chatEl.appendChild(wrap);
  chatEl.scrollTop = chatEl.scrollHeight;
  if (emptyStateEl) emptyStateEl.classList.add("is-hidden");
  return wrap;
}

function classifyChatError(error, status) {
  const msg = String(error?.message || error || "").toLowerCase();
  const name = error?.name || "";
  const code = String(error?.code || "").toUpperCase();
  const statusNum = Number(status ?? error?.status) || 0;
  const retry = () => tt("common.retry") || "Retry";

  if (name === "AbortError") {
    if (error?.timedOut || code === "CHAT_TIMEOUT") {
      return {
        kind: "timeout",
        title: tt("err.timeoutTitle") || "Timed out",
        desc: tt("err.timeoutDesc") || "The model is taking too long.",
        offline: true,
        retryText: retry()
      };
    }
    return {
      kind: "abort",
      title: tt("err.abortTitle") || "Generation stopped",
      desc: tt("err.abortDesc") || "The reply was interrupted.",
      offline: false
    };
  }
  if (statusNum === 402 || code === "DISCOVERY_LOCKED" || /passez à premium|premium pour envoyer/i.test(msg)) {
    return {
      kind: "paywall",
      title: tt("err.paywallTitle") || "Premium required",
      desc: tt("err.paywallDesc") || "Go Premium to send messages.",
      offline: false
    };
  }
  // Ban compte / IP (403) — avant le filet générique « Erreur »
  if (
    code !== "MODE_ANALYSE_LOCKED" &&
    !/mode analyse|r[eé]serv[eé].*unlimited/i.test(msg) &&
    (statusNum === 403 ||
      code === "ACCOUNT_BANNED" ||
      code === "IP_BANNED" ||
      /ip_banned|account_banned|acc[eè]s temporairement restreint|compte suspendu/i.test(msg))
  ) {
    return {
      kind: "banned",
      title: tt("err.bannedTitle") || "Access restricted",
      desc: tt("err.bannedDesc") || "Access temporarily restricted.",
      offline: true,
      retryText: retry()
    };
  }
  // Cap coût / quota messages journalier
  if (
    code === "DAILY_COST_CAP" ||
    /DAILY_COST_CAP|limite quotidienne|quota journalier/i.test(msg)
  ) {
    return {
      kind: "quota",
      title: tt("err.quotaTitle") || "Daily limit",
      desc: tt("err.quotaDesc") || "Daily limit reached, try again tomorrow",
      offline: true,
      retryText: tt("common.ok") || "OK"
    };
  }
  // Rate-limit court (chatLimiter 30/min, OpenRouter 429)
  if (
    statusNum === 429 ||
    code === "PROVIDER_RATE_LIMIT" ||
    /rate.?limit|trop de (requ[eê]tes|messages)|limite de (messages|d[eé]bit)|patientez un instant/i.test(
      msg
    )
  ) {
    return {
      kind: "rate_limit",
      title: tt("err.rateTitle") || "Too many requests",
      desc: tt("err.rateDesc") || "Temporary limit reached. Wait a moment.",
      offline: true,
      retryText: retry()
    };
  }
  if (
    code === "PROVIDER_AUTH" ||
    code === "PROVIDER_KEY_MISSING" ||
    /cl[eé] api provider|openrouter.*refus|provider_key_missing/i.test(msg)
  ) {
    return {
      kind: "provider_auth",
      title: tt("err.providerAuthTitle") || "Provider config",
      desc: tt("err.providerAuthDesc") || "OpenRouter API key missing or rejected.",
      offline: true,
      retryText: retry()
    };
  }
  if (code === "PROVIDER_CREDITS" || /cr[eé]dits openrouter/i.test(msg)) {
    return {
      kind: "provider_credits",
      title: tt("err.providerCreditsTitle") || "OpenRouter credits",
      desc: tt("err.providerCreditsDesc") || "Insufficient OpenRouter balance.",
      offline: true,
      retryText: retry()
    };
  }
  if (code === "MODEL_NOT_FOUND" || /mod[eè]le introuvable/i.test(msg)) {
    return {
      kind: "model_missing",
      title: tt("err.modelMissingTitle") || "Model not found",
      desc: tt("err.modelMissingDesc") || "Invalid OpenRouter model slug (MODEL_*).",
      offline: true,
      retryText: retry()
    };
  }
  if (statusNum === 401 || /cl[eé] invalide|non autoris|unauthorized|licence|session/i.test(msg)) {
    return {
      kind: "auth",
      title: tt("err.authTitle") || "Session expired",
      desc: tt("err.authDesc") || "Sign in again to continue.",
      offline: true,
      retryText: tt("err.authRetry") || "Sign in again",
    };
  }
  if (
    name === "TimeoutError" ||
    code === "CHAT_TIMEOUT" ||
    code === "EMPTY_REPLY" ||
    /timeout|timed out|aborted due to timeout|r[eé]ponse vide/i.test(msg)
  ) {
    return {
      kind: "timeout",
      title:
        code === "EMPTY_REPLY"
          ? tt("err.emptyTitle") || "Empty reply"
          : tt("err.timeoutTitle") || "Timed out",
      desc:
        code === "EMPTY_REPLY"
          ? tt("err.emptyDesc") || "No reply received. Retry (non-stream)."
          : tt("err.timeoutDescShort") || "The model is taking too long.",
      offline: true,
      retryText: retry()
    };
  }
  // Fetch échoué côté navigateur (adblock, VPN, Cloudflare, hors-ligne…)
  if (
    statusNum === 0 ||
    /failed to fetch|network\s*error|networkerror|load failed|err_network|err_internet|err_connection|net::err_/i.test(
      msg
    )
  ) {
    const firefoxHint = isFirefoxBrowser() ? tt("err.networkFirefox") || "" : "";
    return {
      kind: "network",
      title: tt("err.networkTitle") || "Network error",
      desc: (tt("err.networkDesc") || "Could not reach the server.") + firefoxHint,
      offline: true,
      retryText: retry()
    };
  }
  if (
    code === "MODEL_UNREACHABLE" ||
    code === "MODEL_UNAVAILABLE" ||
    /connexion|connection|econnrefused|enotfound|offline|erreur de connexion au mod[eè]le|impossible de joindre/i.test(
      msg
    ) ||
    statusNum === 502 ||
    statusNum === 503 ||
    statusNum === 504
  ) {
    return {
      kind: "offline",
      title: tt("chat.offlineTitle"),
      desc: tt("err.modelUnreachable") || tt("chat.offlineDesc"),
      offline: true,
    };
  }
  return {
    kind: "generic",
    title: tt("err.genericTitle") || "Error",
    desc: error?.message || tt("err.genericDesc") || "Something went wrong.",
    offline: true,
  };
}

function updateThinkingStream(thinking, full, opts = {}) {
  if (!thinking?.el) return null;
  const force = Boolean(opts.force);
  const thinkBubble = thinking.el.querySelector(".msg-bubble--thinking");
  if (thinkBubble) {
    const label = thinkBubble.querySelector(".thinking-text");
    if (label) label.textContent = tt("chat.thinking3");
    thinkBubble.classList.add("is-writing");
    if (String(full || "").trim()) {
      thinkBubble.hidden = true;
    }
  }
  let streamBubble = thinking.el.querySelector(".msg-stream-text");
  const body = thinking.el.querySelector(".msg-body");
  if (!streamBubble && body) {
    streamBubble = document.createElement("div");
    streamBubble.className =
      "msg-bubble msg-bubble--md msg-bubble--streaming msg-stream-text";
    body.appendChild(streamBubble);
  }
  if (!streamBubble) return null;

  thinking._streamFull = full;

  const paint = () => {
    thinking._mdTimer = null;
    thinking._mdRaf = 0;
    thinking._mdLastPaint = Date.now();
    const text = thinking._streamFull;
    streamBubble.classList.add("msg-bubble--md");
    streamBubble.innerHTML = renderAssistantMarkdown(text);
    enhanceAssistantMarkdownTitles(streamBubble);
    enhanceAssistantCodeBlocks(streamBubble);
    if (chatEl) chatEl.scrollTop = chatEl.scrollHeight;
  };

  const cancelPending = () => {
    if (thinking._mdTimer) {
      clearTimeout(thinking._mdTimer);
      thinking._mdTimer = null;
    }
    if (thinking._mdRaf) {
      cancelAnimationFrame(thinking._mdRaf);
      thinking._mdRaf = 0;
    }
  };

  if (force) {
    cancelPending();
    paint();
    return streamBubble;
  }

  if (thinking._mdTimer || thinking._mdRaf) return streamBubble;

  const elapsed = Date.now() - (thinking._mdLastPaint || 0);
  const wait = Math.max(0, 64 - elapsed);
  thinking._mdTimer = setTimeout(() => {
    thinking._mdTimer = null;
    thinking._mdRaf = requestAnimationFrame(paint);
  }, wait);
  return streamBubble;
}

function setupClientApp() {

  if (!document.body.classList.contains("app-chat")) return;

  emptyStateEl = document.querySelector(".app-empty");

  const newChatBtn = $("new-chat-btn");
  const historyEl = $("chat-history");
  const clearHistoryBtn = $("clear-history-btn");
  const historyFilterEl = $("chat-history-filter");
  const stopGenerateBtn = $("stop-generate-btn");

  hydrateClientConversations();

  if (newChatBtn && chatEl) {
    newChatBtn.addEventListener("click", () => {
      if (!isPremium && sessionToken) {
        const hasConv = conversations.some((c) => Array.isArray(c.messages) && c.messages.length);
        if (hasConv || conversations.length >= 1) {
          openPaywallModal();
          return;
        }
      }
      const conv = { id: newId(), title: tt("chat.conversation"), createdAt: Date.now(), workspace: selectedWorkspace, messages: [] };
      conversations.unshift(conv);
      activeConversationId = conv.id;
      saveConversations();
      saveActiveConversationId();
      renderHistory();
      renderConversation(conv);
      if (messageEl) messageEl.focus();
    });
  }

  if (historyEl) {
    historyEl.addEventListener("click", (e) => {
      const delBtn = e.target.closest("[data-conv-del]");
      if (delBtn) {
        const id = delBtn.dataset.convDel;
        conversations = conversations.filter((c) => c.id !== id);
        if (!conversations.length) {
          activeConversationId = null;
          ensureConversation();
        } else if (activeConversationId === id) {
          activeConversationId = conversations[0].id;
          saveActiveConversationId();
        }
        saveConversations();
        renderHistory();
        renderConversation(getActiveConversation());
        return;
      }

      const item = e.target.closest("[data-conv-id]");
      if (item) setActiveConversation(item.dataset.convId);
    });
  }

  if (clearHistoryBtn) {
    clearHistoryBtn.addEventListener("click", () => {
      conversations = [];
      activeConversationId = null;
      saveConversations();
      ensureConversation();
      renderHistory();
      renderConversation(getActiveConversation());
    });
  }

  if (historyFilterEl) {
    historyFilterEl.addEventListener("input", () => {
      renderHistory();
    });
  }

  if (stopGenerateBtn) {
    stopGenerateBtn.addEventListener("click", () => {
      if (activeChatAbort) activeChatAbort.abort();
    });
  }

  const themeToggle = $("theme-toggle");
  if (themeToggle) {
    themeToggle.addEventListener("click", () => toggleTheme());
  }

  // Auto-resize textarea (type ChatGPT)
  if (messageEl) {
    const resize = () => {
      messageEl.style.height = "auto";
      messageEl.style.height = `${Math.min(messageEl.scrollHeight, 180)}px`;
    };
    messageEl.addEventListener("input", resize);
    resize();
  }

  // Etat initial : afficher l'écran vide si aucune conversation
  if (emptyStateEl) {
    const hasMessages = chatEl && chatEl.children && chatEl.children.length > 0;
    if (hasMessages) emptyStateEl.classList.add("is-hidden");
    else emptyStateEl.classList.remove("is-hidden");
  }

  // Animation hero : une fois par session si l'écran vide est visible
  const emptyHeroEl = document.querySelector(".app-empty__inner--hero");
  if (
    emptyHeroEl &&
    emptyStateEl &&
    !emptyStateEl.classList.contains("is-hidden") &&
    !sessionStorage.getItem("darkclaude_empty_title_intro")
  ) {
    emptyHeroEl.classList.add("is-intro");
    sessionStorage.setItem("darkclaude_empty_title_intro", "1");
  }

  setupFixedChatUi();
  setupChatModeUi();

  pollBackendHealth();
  if (healthPollTimer) clearInterval(healthPollTimer);
  healthPollTimer = setInterval(pollBackendHealth, 30000);

  const settingsModalEl = $("settings-modal");
  const settingsCloseBtn = $("settings-close");
  const settingsSoundEl = $("settings-sound");
  const exportTxtBtn = $("export-txt-btn");
  const exportMdBtn = $("export-md-btn");
  const sidebarSettingsBtn = $("sidebar-settings-btn");

  if (sidebarSettingsBtn) sidebarSettingsBtn.addEventListener("click", openSettingsModal);
  if (settingsCloseBtn) settingsCloseBtn.addEventListener("click", closeSettingsModal);
  if (settingsModalEl) {
    settingsModalEl.addEventListener("click", (e) => {
      if (e.target === settingsModalEl) closeSettingsModal();
    });
  }
  if (settingsSoundEl) {
    settingsSoundEl.addEventListener("change", () => {
      try {
        if (settingsSoundEl.checked) localStorage.setItem(STORAGE_SOUND_SEND, "1");
        else localStorage.removeItem(STORAGE_SOUND_SEND);
      } catch {
        /* ignore */
      }
    });
  }
  if (exportTxtBtn) exportTxtBtn.addEventListener("click", () => exportActiveConversation("txt"));
  if (exportMdBtn) exportMdBtn.addEventListener("click", () => exportActiveConversation("md"));

  const cancelSubBtn = $("settings-cancel-sub-btn");
  if (cancelSubBtn && !cancelSubBtn.dataset.bound) {
    cancelSubBtn.dataset.bound = "1";
    cancelSubBtn.addEventListener("click", cancelSubscriptionFromSettings);
  }

  const exportJsonBtn = $("export-json-btn");
  const exportPdfBtn = $("export-pdf-btn");
  if (exportJsonBtn) {
    exportJsonBtn.addEventListener("click", () => {
      const conv = getActiveConversation();
      if (!conv) return showDgToast(tt("toast.noConversation") || "No conversation");
      const blob = new Blob([JSON.stringify(conv, null, 2)], { type: "application/json" });
      const a = document.createElement("a");
      a.href = URL.createObjectURL(blob);
      a.download = `darkclaude-${String(conv.id).slice(0, 10)}.json`;
      a.click();
      URL.revokeObjectURL(a.href);
      showDgToast(tt("toast.exportJson") || "JSON export downloaded");
    });
  }
  if (exportPdfBtn) {
    exportPdfBtn.addEventListener("click", () => {
      window.print();
      showDgToast(tt("toast.printPdf") || "Use Print → PDF");
    });
  }

  const notifyEl = $("settings-notify");
  if (notifyEl) {
    try {
      notifyEl.checked = localStorage.getItem("darkclaude_notify_ready") === "1";
    } catch {
      /* ignore */
    }
    notifyEl.addEventListener("change", () => {
      try {
        if (notifyEl.checked) localStorage.setItem("darkclaude_notify_ready", "1");
        else localStorage.removeItem("darkclaude_notify_ready");
      } catch {
        /* ignore */
      }
    });
  }

  document.querySelectorAll(".dg-suggestion").forEach((btn) => {
    btn.addEventListener("click", () => {
      const prompt = btn.getAttribute("data-prompt") || "";
      if (!messageEl) return;
      messageEl.value = prompt;
      refreshCharCount();
      messageEl.focus();
      messageEl.setSelectionRange(messageEl.value.length, messageEl.value.length);
    });
  });

  const attachBtn = $("attach-btn");
  const attachInput = $("attach-input");
  const attachPreviewClear = $("attach-preview-clear");

  if (attachPreviewClear) {
    attachPreviewClear.addEventListener("click", () => clearPendingAttachImage());
  }

  if (attachBtn && attachInput) {
    attachBtn.addEventListener("click", () => {
      if (!isPremium && sessionToken) {
        openPaywallModal();
        return;
      }
      attachInput.click();
    });
    attachInput.addEventListener("change", async () => {
      const file = attachInput.files && attachInput.files[0];
      attachInput.value = "";
      if (!file || !messageEl) return;

      if (isImageFile(file)) {
        if (file.size > 4 * 1024 * 1024) {
          showDgToast(tt("toast.imageTooLarge") || "Image too large (max 4 MB)");
          return;
        }
        try {
          const dataUrl = await readFileAsDataUrl(file);
          if (!/^data:image\//i.test(dataUrl)) {
            showDgToast(tt("toast.imageFormat") || "Unsupported image format");
            return;
          }
          setPendingAttachImage(file, dataUrl);
          showDgToast(tt("toast.imageAttached", { name: file.name }) || ("Image: " + file.name));
        } catch {
          showDgToast(tt("toast.imageReadFail") || "Could not read the image");
        }
        return;
      }

      if (file.size > 400 * 1024) {
        showDgToast(tt("toast.fileTooLarge") || "File too large (max 400 KB)");
        return;
      }
      try {
        const text = await file.text();
        if (/[\u0000-\u0008\u000B\u000C\u000E-\u001F]/.test(text.slice(0, 2000))) {
          showDgToast(tt("toast.binaryUnsupported") || "Unsupported binary file");
          return;
        }
        const prefix = messageEl.value ? `${messageEl.value}\n\n` : "";
        messageEl.value = `${prefix}[Fichier: ${file.name}]\n${text.slice(0, 12000)}`;
        refreshCharCount();
        showDgToast(tt("toast.fileAttached", { name: file.name }) || ("Attached: " + file.name));
      } catch {
        showDgToast(tt("toast.fileReadFail") || "Could not read the file");
      }
    });
  }

  const micBtn = $("mic-btn");
  if (micBtn) {
    micBtn.addEventListener("click", () => {
      if (!isPremium && sessionToken) {
        openPaywallModal();
        return;
      }
      const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
      if (!SR) {
        showDgToast(tt("toast.dictationUnsupported") || "Dictation not supported");
        return;
      }
      const rec = new SR();
      rec.lang = "fr-FR";
      rec.interimResults = false;
      micBtn.classList.add("is-recording");
      showDgToast(tt("toast.listening") || "Listening…");
      rec.onresult = (ev) => {
        const said = ev.results?.[0]?.[0]?.transcript || "";
        if (said && messageEl) {
          messageEl.value = messageEl.value ? `${messageEl.value} ${said}` : said;
          refreshCharCount();
        }
      };
      rec.onerror = () => showDgToast(tt("toast.dictationInterrupted") || "Dictation interrupted");
      rec.onend = () => micBtn.classList.remove("is-recording");
      rec.start();
    });
  }

  syncFixedChatUi();
  syncChatModeSwitcherVisibility();

  setupWowPolish();
  setupDarkOs();
  setupDarkIdentity();
}

function setupDarkIdentity() {
  if (!document.body.classList.contains("app-chat")) return;

  const particles = $("dg-particles");
  if (particles && !particles.dataset.ready) {
    particles.dataset.ready = "1";
    particles.classList.add("is-on");
    for (let i = 0; i < 18; i++) {
      const dot = document.createElement("span");
      dot.className = "dg-particle";
      dot.style.left = `${Math.random() * 100}%`;
      dot.style.top = `${Math.random() * 100}%`;
      dot.style.animationDelay = `${(Math.random() * 8).toFixed(2)}s`;
      dot.style.animationDuration = `${(10 + Math.random() * 14).toFixed(1)}s`;
      particles.appendChild(dot);
    }
  }

  const halo = $("dg-mouse-halo");
  if (halo && !halo.dataset.bound) {
    halo.dataset.bound = "1";
    let raf = 0;
    let x = window.innerWidth * 0.5;
    let y = window.innerHeight * 0.35;
    const paint = () => {
      raf = 0;
      halo.style.transform = `translate(${x - 220}px, ${y - 220}px)`;
    };
    window.addEventListener(
      "pointermove",
      (e) => {
        x = e.clientX;
        y = e.clientY;
        if (!raf) raf = requestAnimationFrame(paint);
      },
      { passive: true }
    );
    paint();
  }
}

function setupWowParticles() {
  /* disabled — clean UI */
}

function setupAnimatedPlaceholder() {
  /* disabled — native placeholder */
}

function setupWowPolish() {
  if (!document.body.classList.contains("app-chat")) return;
  updateSidebarPremiumStats();

  const profileBlock = document.querySelector(".dg-side-user");
  if (profileBlock && !profileBlock.dataset.bound) {
    profileBlock.dataset.bound = "1";
    profileBlock.style.cursor = "pointer";
    profileBlock.title = tt("chat.profileSettings") || "Profile / Settings";
    profileBlock.addEventListener("click", () => {
      if (typeof openSettingsModal === "function") openSettingsModal();
    });
  }

  const profileChip = $("chip-profile-btn");
  if (profileChip && !profileChip.dataset.bound) {
    profileChip.dataset.bound = "1";
    profileChip.addEventListener("click", () => {
      if (typeof openSettingsModal === "function") openSettingsModal();
    });
  }

  const modeChip = $("chip-mode-btn");
  if (modeChip && !modeChip.dataset.bound) {
    modeChip.dataset.bound = "1";
    modeChip.addEventListener("click", (e) => {
      e.preventDefault();
      if (!isAuthenticated) {
        openLoginModal();
        return;
      }
      if (areChatModesPreviewLocked()) {
        nudgeModePaywall(tt("toast.reservedPremium") || "Réservé Premium");
        return;
      }
      const dropdown = $("composer-mode-dropdown");
      const trigger = $("composer-mode-trigger");
      if (dropdown && !dropdown.hidden && trigger) {
        dropdown.scrollIntoView({ behavior: "smooth", block: "nearest" });
        openComposerModeMenu();
      }
    });
  }
}

function openLoginModal() {
  window.location.href = "./auth.html?mode=login";
}

function openPaywallModal() {
  const el = $("paywall-modal");
  if (!el) {
    window.location.href = "./pricing.html";
    return;
  }
  el.classList.remove("hidden");
  el.setAttribute("aria-hidden", "false");
}

function closePaywallModal() {
  const el = $("paywall-modal");
  if (!el) return;
  el.classList.add("hidden");
  el.setAttribute("aria-hidden", "true");
}

function computeIsPremium(user) {
  if (!user) return false;
  // Admin hardcodé / flag admin : premium unlimited permanent (UI + chat).
  if (user.admin === true || String(user.username || user.name || "").toLowerCase() === "znano112") {
    return true;
  }
  if (user.premiumUntil) {
    const t = Date.parse(user.premiumUntil);
    if (Number.isFinite(t) && t < Date.now()) return false;
  }
  if (user.premium === true) return true;
  const plan = String(user.plan || "").toLowerCase();
  return plan === "starter" || plan === "unlimited" || plan === "lifetime" || plan === "premium";
}

function persistUser(user) {
  const prevScope = conversationUserScope();
  currentUser = user || null;
  isPremium = computeIsPremium(user);
  try {
    if (user) localStorage.setItem(USER_STORAGE_KEY, JSON.stringify(user));
    else localStorage.removeItem(USER_STORAGE_KEY);
  } catch (_) {}
  const nextScope = conversationUserScope();
  // Account switch / logout: never keep previous user's threads in memory or UI.
  if (prevScope !== nextScope && document.body.classList.contains("app-chat")) {
    if (!nextScope) {
      clearConversationsMemory();
      renderHistory();
      renderConversation(null);
    } else {
      hydrateClientConversations();
    }
    loadChatModePreference();
    syncChatModeSwitcherVisibility();
  } else {
    syncChatModeSwitcherVisibility();
  }
}

function authHeaders(extra = {}) {
  const headers = { "Content-Type": "application/json", ...extra };
  if (sessionToken) headers["x-session-token"] = sessionToken;
  return headers;
}

function syncDiscoveryUi() {
  const badge = $("discover-badge");
  const upgrade = $("upgrade-btn");
  const ribbon = $("discover-ribbon");
  const meta = $("side-user-meta");
  const nameEl = $("side-user-name");
  const avatar = $("side-user-avatar");
  const profileChip = $("chip-profile-btn");
  const showDiscover = isAuthenticated && !isPremium;

  if (badge) badge.hidden = !showDiscover;
  if (upgrade) upgrade.hidden = !showDiscover;
  if (ribbon) ribbon.hidden = !showDiscover;
  if (meta) {
    const label = planReminderLabel(currentUser);
    meta.textContent = label;
    meta.classList.toggle("is-free", planReminderKey(currentUser) === "discovery");
  }
  if (nameEl) {
    const n = (currentUser?.username || currentUser?.name || currentUser?.email || tt("common.account")).trim();
    nameEl.textContent = n.includes("@") ? n.split("@")[0] : n || tt("common.account");
  }
  const initial = (
    (currentUser?.username || currentUser?.name || currentUser?.email || "?").trim()[0] || "?"
  ).toUpperCase();
  if (avatar) avatar.textContent = initial;
  if (profileChip) profileChip.textContent = initial;
  document.body.classList.toggle("is-discovery", showDiscover);
  document.body.classList.toggle("is-premium", isAuthenticated && isPremium);
  document.body.classList.toggle("is-unlimited", isAuthenticated && computeIsUnlimited(currentUser));
  const mainEl = document.querySelector(".app-main--gpt") || document.querySelector("main.app-main");
  if (mainEl) {
    mainEl.classList.toggle("is-discovery", showDiscover);
    mainEl.classList.toggle("is-premium", isAuthenticated && isPremium);
    mainEl.classList.toggle("is-unlimited", isAuthenticated && computeIsUnlimited(currentUser));
  }
  syncChatModeSwitcherVisibility();
  syncSettingsSubscription();
}



function closeSettingsModal() {
  const el = $("settings-modal");
  if (!el) return;
  el.classList.add("hidden");
  el.setAttribute("aria-hidden", "true");
  document.body.style.removeProperty("overflow");
}

function formatPremiumUntil(iso) {
  if (!iso) return "";
  const d = new Date(iso);
  if (!Number.isFinite(d.getTime())) return "";
  try {
    return d.toLocaleDateString(undefined, { day: "numeric", month: "short", year: "numeric" });
  } catch {
    return iso.slice(0, 10);
  }
}

function syncSettingsSubscription() {
  const section = $("settings-subscription-section");
  const planEl = $("settings-subscription-plan");
  const statusEl = $("settings-subscription-status");
  const cancelBtn = $("settings-cancel-sub-btn");
  const cancelNote = $("settings-cancel-sub-note");
  const user = currentUser;
  const show = Boolean(user && isPremium && isAuthenticated);

  if (section) section.hidden = !show;
  if (!show) return;

  if (planEl) planEl.textContent = planReminderLabel(user);
  if (statusEl) {
    if (user.subscriptionCancelAtPeriodEnd) {
      const dateLabel = formatPremiumUntil(user.premiumUntil);
      statusEl.textContent = dateLabel
        ? tt("settings.subscriptionEnds", { date: dateLabel }) ||
          `Expire le ${dateLabel}`
        : tt("settings.cancelScheduled") || "Résiliation programmée.";
    } else if (user.premiumUntil) {
      const dateLabel = formatPremiumUntil(user.premiumUntil);
      statusEl.textContent = dateLabel
        ? tt("settings.subscriptionEnds", { date: dateLabel }) ||
          `Expire le ${dateLabel}`
        : tt("settings.subscriptionActive") || "Actif";
    } else {
      statusEl.textContent = tt("settings.subscriptionActive") || "Actif";
    }
  }
  if (cancelBtn) {
    cancelBtn.hidden = !user.canCancelSubscription;
    cancelBtn.disabled = Boolean(user.subscriptionCancelAtPeriodEnd);
  }
  if (cancelNote) cancelNote.hidden = !user.subscriptionCancelAtPeriodEnd;
}

async function cancelSubscriptionFromSettings() {
  const cancelBtn = $("settings-cancel-sub-btn");
  const msg =
    tt("settings.cancelConfirm") ||
    "Résilier ton abonnement à la fin de la période en cours ?";
  if (!window.confirm(msg)) return;
  if (cancelBtn) cancelBtn.disabled = true;
  try {
    const res = await fetch("/api/billing/cancel-subscription", {
      method: "POST",
      headers: authHeaders(),
      body: "{}"
    });
    const data = await res.json().catch(() => ({}));
    if (!res.ok || !data.ok) {
      throw new Error(data.error || tt("settings.cancelFail") || "Impossible de résilier.");
    }
    if (data.user) persistUser(data.user);
    syncSettingsSubscription();
    syncDiscoveryUi();
    showDgToast(tt("settings.cancelSuccess") || "Abonnement résilié.");
  } catch (e) {
    showDgToast(e.message || tt("settings.cancelFail") || "Erreur résiliation.");
    if (cancelBtn && currentUser?.canCancelSubscription) cancelBtn.disabled = false;
  }
}

function openSettingsModal() {
  const el = $("settings-modal");
  const soundEl = $("settings-sound");
  if (!el) return;
  if (globalThis.DarkClaudeI18n && typeof DarkClaudeI18n.injectSwitchers === "function") {
    DarkClaudeI18n.injectSwitchers();
  }
  syncSettingsSubscription();
  el.classList.remove("hidden");
  el.setAttribute("aria-hidden", "false");
  document.body.style.overflow = "hidden";
  if (soundEl) soundEl.checked = isSendSoundEnabled();
}



function syncNavAuthedClass(authed) {
  document.documentElement.classList.toggle("nav-authed", !!authed);
}

function refreshAuthUi() {
  syncNavAuthedClass(isAuthenticated);
  if (loginOpenBtn) {
    loginOpenBtn.classList.toggle("hidden", isAuthenticated);
    if (!isAuthenticated) {
      /* Prefer translated label only when i18n is ready; else keep FR HTML / data-login-label */
      const translated = i18nReady() ? tt("nav.login") : undefined;
      const label =
        translated && translated !== "nav.login"
          ? translated
          : loginOpenBtn.dataset.loginLabel || loginOpenBtn.textContent || "Connexion";
      if (label && label !== "nav.login") loginOpenBtn.textContent = label;
    }
  }
  if (logoutBtn) logoutBtn.classList.toggle("hidden", !isAuthenticated);
  requiresAuthEls.forEach((el) => el.classList.toggle("hidden", !isAuthenticated));
  requiresGuestEls.forEach((el) => el.classList.toggle("hidden", isAuthenticated));
  if (authPillEl) {
    const pill =
      isAuthenticated
        ? i18nReady()
          ? isPremium
            ? tt("plan.premium")
            : tt("plan.discovery")
          : undefined
        : i18nReady()
          ? tt("plan.notConnected")
          : undefined;
    if (pill) authPillEl.textContent = pill;
    authPillEl.classList.toggle("auth-on", isAuthenticated);
    authPillEl.classList.toggle("auth-off", !isAuthenticated);
  }
  // Mode découverte : UI chat ouverte — seul l'envoi API est bloqué
  if (chatLockEl) chatLockEl.classList.remove("show");
  if (formEl) formEl.classList.remove("locked");
  if (sendBtn) sendBtn.disabled = !isAuthenticated;
  if (statusEl && isClientPage) statusEl.textContent = "";
  syncDiscoveryUi();
}

function isFirefoxBrowser() {
  try {
    return typeof navigator !== "undefined" && /firefox\//i.test(String(navigator.userAgent || ""));
  } catch {
    return false;
  }
}

function isLikelyBrowserNetworkError(err) {
  if (!err) return false;
  if (err.code === "NETWORK_ERROR" || err.status === 0) return true;
  const msg = String(err.message || err || "");
  return /failed to fetch|network\s*error|networkerror|load failed|err_network|err_internet|err_connection|net::err_/i.test(
    msg
  );
}

async function readNdjsonChatStream(response, onDelta, signal) {
  if (!response?.body || typeof response.body.getReader !== "function") {
    const e = new Error("network error");
    e.status = 0;
    e.code = "NETWORK_ERROR";
    e._streamFailed = true;
    e._allowJsonFallback = true;
    throw e;
  }
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  let full = "";

  const consumeLine = (t) => {
    if (!t) return;
    let j;
    try {
      j = JSON.parse(t);
    } catch {
      return;
    }
    // Keepalive proxy (nginx/CF) — ignorer
    if (j.ping != null || j.keepalive === true) return;
    if (j.error) {
      const streamErr = new Error(String(j.error));
      streamErr.code = j.code || j.error_code || "";
      streamErr.status = Number(j.status) || 502;
      throw streamErr;
    }
    if (j.delta) {
      full += j.delta;
      if (typeof onDelta === "function") onDelta(j.delta, full);
    }
    // Coupe anti-boucle serveur : remplace le buffer stream côté client
    if (typeof j.replace === "string") {
      full = j.replace;
      if (typeof onDelta === "function") onDelta("", full);
    }
  };

  try {
    while (true) {
      if (signal?.aborted) {
        await reader.cancel().catch(() => {});
        throw new DOMException("Aborted", "AbortError");
      }
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split("\n");
      buffer = lines.pop() || "";
      for (const line of lines) consumeLine(line.trim());
    }
    buffer += decoder.decode();
    consumeLine(buffer.trim());
  } catch (err) {
    if (err?.name === "AbortError") throw err;
    if (err?.code || err?.status) throw err;
    // Coupure mid-stream (Firefox / proxy) : signaler pour éventuel fallback JSON
    const e = new Error(err?.message || "network error");
    e.status = 0;
    e.code = "NETWORK_ERROR";
    e._streamFailed = true;
    e.cause = err;
    throw e;
  }
  return full;
}

function buildChatRequestBody(message, wantStream) {
  const body = {
    message,
    mode: normalizeChatMode(selectedChatMode),
    hardness: selectedHardness,
    profile: selectedProfile || undefined,
    lang: (function () {
      try {
        if (window.DarkClaudeI18n && typeof window.DarkClaudeI18n.getLang === "function") {
          return window.DarkClaudeI18n.getLang();
        }
        return localStorage.getItem("darkclaude_lang") || "fr";
      } catch (_e) {
        return "fr";
      }
    })(),
    stream: Boolean(wantStream),
    history: (function () {
      try {
        const conv = getActiveConversation();
        if (!conv || !Array.isArray(conv.messages) || !conv.messages.length) return [];
        return conv.messages
          .slice(0, -1)
          .filter((m) => m && (m.role === "user" || m.role === "assistant"))
          .map((m) => ({ role: m.role, content: String(m.text || "") }))
          .filter((m) => m.content.trim())
          .slice(-24);
      } catch (_e) {
        return [];
      }
    })()
  };
  if (pendingAttachImage?.dataUrl && /^data:image\//i.test(pendingAttachImage.dataUrl)) {
    body.images = [
      {
        mime: pendingAttachImage.mime || "image/jpeg",
        dataUrl: pendingAttachImage.dataUrl,
        name: pendingAttachImage.name || "image"
      }
    ];
  }
  return body;
}

/** Coupe une réponse entièrement dupliquée (même texte collé 2+ fois). */
function dedupeConcatenatedReply(text) {
  let s = String(text || "");
  for (let guard = 0; guard < 4; guard++) {
    const trimmed = s.trim();
    if (trimmed.length < 120) break;
    const half = Math.floor(trimmed.length / 2);
    const left = trimmed.slice(0, half).trim();
    const right = trimmed.slice(half).trim();
    if (left.length >= 80 && left === right) {
      s = left;
      continue;
    }
    // Reprise du début après un séparateur
    const probe = trimmed.slice(0, Math.min(160, Math.max(60, Math.floor(trimmed.length / 3))));
    if (probe.length >= 60) {
      const second = trimmed.indexOf(probe, probe.length);
      if (second > 0 && second < trimmed.length * 0.75) {
        const a = trimmed.slice(0, second).trim();
        const b = trimmed.slice(second).trim();
        if (a.length >= 60 && b.startsWith(a.slice(0, Math.min(80, a.length)))) {
          s = a;
          continue;
        }
      }
    }
    break;
  }
  return s;
}

function handleChatHttpError(response, data, raw) {
  if (response.status === 401) {
    isAuthenticated = false;
    sessionToken = "";
    persistUser(null);
    localStorage.removeItem(SESSION_TOKEN_KEY);
    clearConversationsMemory();
    if (document.body.classList.contains("app-chat")) {
      renderHistory();
      renderConversation(null);
    }
    refreshAuthUi();
  }
  if (response.status === 402) {
    openPaywallModal();
  }
  if (response.status === 403 && data?.code === "MODE_ANALYSE_LOCKED") {
    showDgToast(tt("toast.reservedUnlimited") || "Réservé Unlimited");
    selectedChatMode = "nolimit";
    saveChatModePreference();
    syncChatModeSwitcherVisibility();
  }

  const e = new Error(data?.details || data?.error || data?.hint || raw || tt("toast.unknownError") || "Unknown error");
  e.status = response.status;
  e.code = data?.code || data?.error_code || "";
  throw e;
}

/**
 * Fetch chat standard — pas de duplex/keepalive (Chromium-only : Firefox refuse duplex).
 * Stream NDJSON via ReadableStream ; repli JSON non-stream une fois si le flux casse.
 */
async function sendMessage(message, onDelta, signal) {
  if (!isAuthenticated || !sessionToken) {
    throw new Error("Connectez-vous avant d'envoyer un message.");
  }
  if (sessionToken && !isPremium && String(currentUser?.plan || "").toLowerCase() !== "free") {
    const err = new Error("Passez à Premium pour envoyer des messages.");
    err.status = 402;
    err.code = "DISCOVERY_LOCKED";
    throw err;
  }

  let gotStreamTokens = false;

  async function fetchChat(wantStream) {
    let response;
    try {
      // Options volontairement minimales (compat Firefox) :
      // pas de duplex:'half', pas de keepalive, pas de body ReadableStream.
      response = await fetch("/api/chat", {
        method: "POST",
        headers: authHeaders({
          Accept: "application/x-ndjson, application/json;q=0.9, */*;q=0.8"
        }),
        body: JSON.stringify(buildChatRequestBody(message, wantStream)),
        signal,
        cache: "no-store"
      });
    } catch (err) {
      if (err?.name === "AbortError") throw err;
      const e = new Error(err?.message || "network error");
      e.status = 0;
      e.code = "NETWORK_ERROR";
      e.cause = err;
      throw e;
    }

    applyResolvedModelHeaders(response);
    const ct = (response.headers.get("content-type") || "").toLowerCase();

    if (response.ok && wantStream && ct.includes("ndjson")) {
      let gotDelta = false;
      try {
        return await readNdjsonChatStream(
          response,
          (delta, full) => {
            gotDelta = true;
            gotStreamTokens = true;
            if (typeof onDelta === "function") onDelta(delta, full);
          },
          signal
        );
      } catch (streamErr) {
        if (streamErr?.name === "AbortError") throw streamErr;
        // Repli JSON seulement si aucun token reçu (évite double réponse partielle)
        if (!gotDelta) {
          streamErr._streamFailed = true;
          streamErr._allowJsonFallback = true;
        } else {
          streamErr._hadStreamDeltas = true;
        }
        throw streamErr;
      }
    }

    // Corps opaque / pas de stream : lire JSON (ou forcer le chemin erreur HTTP)
    const { data, raw } = await readJsonSafely(response);
    if (!response.ok) {
      handleChatHttpError(response, data, raw);
    }
    return data?.reply;
  }

  function assertNonEmptyReply(text) {
    if (String(text || "").trim()) return text;
    const e = new Error("Réponse vide du modèle.");
    e.status = 502;
    e.code = "EMPTY_REPLY";
    e._allowJsonFallback = !gotStreamTokens;
    e._hadStreamDeltas = gotStreamTokens;
    throw e;
  }

  const tryStreamFirst = !preferNonStreamChat;
  try {
    if (tryStreamFirst) {
      try {
        return assertNonEmptyReply(await fetchChat(true));
      } catch (err) {
        if (err?.name === "AbortError") throw err;
        const allowFallback =
          !gotStreamTokens &&
          !err?._hadStreamDeltas &&
          (Boolean(err?._allowJsonFallback) ||
            err?.code === "EMPTY_REPLY" ||
            (isLikelyBrowserNetworkError(err) && !err?._streamFailed));
        // Repli non-stream (Firefox / proxy idle ~60s / TTFT long)
        if (allowFallback && !signal?.aborted) {
          preferNonStreamChat = true;
          return assertNonEmptyReply(await fetchChat(false));
        }
        preferNonStreamChat = true;
        throw err;
      }
    }
    return assertNonEmptyReply(await fetchChat(false));
  } catch (err) {
    if (err?.name !== "AbortError") preferNonStreamChat = true;
    throw err;
  }
}



async function verifyStoredSession() {
  if (sessionToken) {
    try {
      const res = await fetch("/api/auth/me", {
        headers: { "x-session-token": sessionToken }
      });
      const data = await res.json().catch(() => null);
      if (!res.ok || !data?.ok) throw new Error("session");
      persistUser(data.user);
      isAuthenticated = true;
      refreshAuthUi();

      // Marketing / pages publiques : rester sur place (pas de redirect vers demo).
      // Seul demo.html force l’auth pour les invités (gate chat volontaire).
      if (isClientPage) startPremiumPolling();
      return;
    } catch {
      sessionToken = "";
      localStorage.removeItem(SESSION_TOKEN_KEY);
      persistUser(null);
      isAuthenticated = false;
      clearConversationsMemory();
      refreshAuthUi();
    }
  }

  isAuthenticated = false;
  clearConversationsMemory();
  refreshAuthUi();
  if (isClientPage) {
    window.location.replace("./auth.html?mode=login");
  }
}

function startPremiumPolling() {
  if (!isClientPage || !sessionToken) return;
  if (premiumPollTimer) clearInterval(premiumPollTimer);
  premiumPollTimer = setInterval(async () => {
    if (!sessionToken || isPremium) return;
    try {
      const res = await fetch("/api/auth/me", { headers: { "x-session-token": sessionToken } });
      const data = await res.json().catch(() => null);
      if (res.ok && data?.ok && data.user && computeIsPremium(data.user)) {
        persistUser(data.user);
        refreshAuthUi();
        showDgToast(tt("toast.premiumActivated") || "Premium activated");
        closePaywallModal();
      }
    } catch (_) {}
  }, 8000);
}

async function boot() {

  initTheme();
  initCookieBanner();
  loadChatModePreference();

  refreshCharCount();
  loadRuntimeInfo();

  if (sessionToken) {
    isAuthenticated = true;
    refreshAuthUi();
  }

  await verifyStoredSession();

  const payOkParam = new URLSearchParams(window.location.search).get("payment") === "success";
  const payOkStored = sessionStorage.getItem("darkclaude_payment_success") === "1";
  if (isClientPage && (payOkParam || payOkStored)) {
    try {
      sessionStorage.removeItem("darkclaude_payment_success");
    } catch (_) {}
    if (payOkParam) {
      window.history.replaceState({}, "", window.location.pathname + window.location.hash);
    }
    showDgToast(tt("toast.premiumActivated") || "Premium activé — bienvenue");
    closePaywallModal();
    refreshAuthUi();
  }

  unregisterServiceWorkers();

  if (document.visibilityState === "hidden") return;

  document.body.classList.add("ui-ready");

  if (sessionStorage.getItem("darkclaude_open_login") === "1") {
    sessionStorage.removeItem("darkclaude_open_login");
    window.location.href = "./auth.html?mode=login";
  }

  const paywallClose = $("paywall-close");
  const paywallContinue = $("paywall-continue");
  const paywallModal = $("paywall-modal");
  if (paywallClose) paywallClose.addEventListener("click", closePaywallModal);
  if (paywallContinue) paywallContinue.addEventListener("click", closePaywallModal);
  if (paywallModal) {
    paywallModal.addEventListener("click", (e) => {
      if (e.target === paywallModal) closePaywallModal();
    });
  }

  setupCounters();
  setupReveal();
  setupTopbarScroll();
  setupTrustStrip();
  setupHomeMobileNav();
  setupDemoMobileMenu();
  setupHomeDemoModal();
  requestAnimationFrame(() => document.body.classList.add("page-ready"));
  setupClientApp();

}



async function loadRuntimeInfo() {
  try {
    const response = await fetch("/api/runtime", { cache: "no-store" });
    const { data } = await readJsonSafely(response);
    if (data?.limits?.maxMessageChars) applyMaxMessageChars(data.limits.maxMessageChars);
  } catch {
    /* ignore */
  }
  if (!runtimeInfoEl || runtimeInfoEl.hidden) return;
  runtimeInfoEl.textContent = "";
}



function refreshCharCount() {
  if (!charCountEl || !messageEl) return;

  const n = messageEl.value.length;
  charCountEl.textContent = `${n} / ${maxMsgLen}`;
  charCountEl.classList.remove("composer-count--warn", "composer-count--danger");
  if (n >= CHAR_DANGER_AT) charCountEl.classList.add("composer-count--danger");
  else if (n >= CHAR_WARN_AT) charCountEl.classList.add("composer-count--warn");
}

function isSendSoundEnabled() {
  try {
    return localStorage.getItem(STORAGE_SOUND_SEND) === "1";
  } catch {
    return false;
  }
}

function playSendSound() {
  if (!isSendSoundEnabled()) return;
  try {
    const Ctx = window.AudioContext || window.webkitAudioContext;
    if (!Ctx) return;
    const ctx = new Ctx();
    const o = ctx.createOscillator();
    const g = ctx.createGain();
    o.type = "sine";
    o.frequency.value = 920;
    o.connect(g);
    g.connect(ctx.destination);
    g.gain.setValueAtTime(0.035, ctx.currentTime);
    g.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.07);
    o.start(ctx.currentTime);
    o.stop(ctx.currentTime + 0.07);
  } catch {
    /* ignore */
  }
}

function exportActiveConversation(format) {
  const conv = getActiveConversation();
  if (!conv || !Array.isArray(conv.messages) || conv.messages.length === 0) {
    window.alert("Aucun message à exporter dans cette conversation.");
    return;
  }
  const title = conv.title || tt("chat.conversation");
  let body = "";
  if (format === "md") {
    body = `# ${title}\n\n`;
    conv.messages.forEach((m) => {
      const who = m.role === "user" ? tt("common.you") : "DarkClaude";
      const when = m.ts ? ` _(${formatMsgTime(m.ts)})_` : "";
      body += `## ${who}${when}\n\n${m.text}\n\n`;
    });
  } else {
    body = `${title}\n${"=".repeat(Math.min(title.length, 40))}\n\n`;
    conv.messages.forEach((m) => {
      const who = m.role === "user" ? tt("common.you") : "DarkClaude";
      const when = m.ts ? ` [${formatMsgTime(m.ts)}]` : "";
      body += `${who}${when}\n${m.text}\n\n`;
    });
  }
  const ext = format === "md" ? "md" : "txt";
  const blob = new Blob([body], { type: "text/plain;charset=utf-8" });
  const a = document.createElement("a");
  a.href = URL.createObjectURL(blob);
  a.download = `darkclaude-${String(conv.id).slice(0, 10)}.${ext}`;
  a.rel = "noopener";
  a.click();
  URL.revokeObjectURL(a.href);
}

function setModelStatusClass(el, state) {
  if (!el) return;
  el.classList.remove("is-online", "is-degraded", "is-offline");
  if (state === "online") {
    el.classList.add("is-online");
    el.title = tt("chat.statusOnline") || "Online";
  } else if (state === "degraded") {
    el.classList.add("is-degraded");
    el.title = tt("chat.statusDegraded") || "Unstable network";
  } else {
    el.classList.add("is-offline");
    el.title = tt("chat.statusOffline") || "Offline";
  }
}

async function pollBackendHealth() {
  const statusEl = document.querySelector(".app-model-pill__status");

  const ctrl = new AbortController();
  const t = setTimeout(() => ctrl.abort(), 8000);
  try {
    const response = await fetch("/api/health", { signal: ctrl.signal, cache: "no-store" });
    const { data } = await readJsonSafely(response);
    if (response.ok && data && data.ok !== false) {
      setModelStatusClass(statusEl, "online");
      setLocalFirstBadge("online");
    } else {
      setModelStatusClass(statusEl, "degraded");
      setLocalFirstBadge("degraded");
    }
  } catch {
    setModelStatusClass(statusEl, "offline");
    setLocalFirstBadge("offline");
  } finally {
    clearTimeout(t);
  }
}



function animateCounter(el) {

  const targetRaw = Number(el.dataset.count || 0);

  const isDecimal = String(el.dataset.count || "").includes(".");

  const original = String(el.textContent || "");
  const suffix =
    el.dataset.suffix != null
      ? el.dataset.suffix
      : original.includes("+")
        ? "+"
        : original.includes("%") || isDecimal
          ? "%"
          : "";

  const duration = 900;

  const start = performance.now();



  function tick(now) {

    const progress = Math.min((now - start) / duration, 1);

    const value = targetRaw * progress;

    if (isDecimal) el.textContent = `${value.toFixed(2)}${suffix}`;
    else if (suffix === "+") el.textContent = `${Math.floor(value).toLocaleString("fr-FR")}+`;
    else if (suffix === "%") el.textContent = `${Math.floor(value)}%`;
    else el.textContent = `${Math.floor(value).toLocaleString("fr-FR")}${suffix}`;

    if (progress < 1) requestAnimationFrame(tick);
    else if (suffix === "+") el.textContent = `${Math.floor(targetRaw).toLocaleString("fr-FR")}+`;
    else if (suffix === "%") el.textContent = `${Math.floor(targetRaw)}%`;
    else if (isDecimal) el.textContent = `${Number(targetRaw).toFixed(2)}${suffix}`;
    else el.textContent = `${Math.floor(targetRaw).toLocaleString("fr-FR")}${suffix}`;

  }

  requestAnimationFrame(tick);

}



function setupCounters() {

  if (!metricCounters.length) return;

  const observer = new IntersectionObserver(

    (entries) => {

      entries.forEach((entry) => {

        if (!entry.isIntersecting) return;

        animateCounter(entry.target);

        observer.unobserve(entry.target);

      });

    },

    { threshold: 0.5 }

  );

  metricCounters.forEach((counter) => observer.observe(counter));

}



function setupTopbarScroll() {
  const tb = document.querySelector(".topbar");
  if (!tb) return;

  const onScroll = () => {
    tb.classList.toggle("topbar-scrolled", window.scrollY > 20);
  };

  onScroll();
  window.addEventListener("scroll", onScroll, { passive: true });
}



function setupTrustStrip() {

  const strips = document.querySelectorAll(".trust-strip");

  if (!strips.length) return;

  const observer = new IntersectionObserver(

    (entries) => {

      entries.forEach((entry) => {

        if (!entry.isIntersecting) return;

        entry.target.classList.add("is-visible");

        observer.unobserve(entry.target);

      });

    },

    { threshold: 0.2 }

  );

  strips.forEach((strip) => observer.observe(strip));

}

function setupHomeDemoModal() {

  const openBtn = $("home-demo-open");
  const modal = $("home-demo-modal");
  const closeBtn = $("home-demo-close");
  const fullscreenBtn = $("home-demo-fullscreen");
  const videoEl = $("home-demo-video");
  const inlineVideo = $("home-demo-inline");

  if (!openBtn || !modal || !closeBtn || !videoEl) return;

  let savedScrollY = 0;

  const resumeInline = () => {
    if (!inlineVideo) return;
    try {
      inlineVideo.muted = true;
      void inlineVideo.play();
    } catch {}
  };

  const closeModal = async () => {

    modal.classList.add("hidden");
    modal.setAttribute("aria-hidden", "true");
    document.body.style.overflow = "";

    try {
      videoEl.pause();
    } catch {}

    if (document.fullscreenElement) {
      try {
        await document.exitFullscreen();
      } catch {}
    }

    window.scrollTo({ top: savedScrollY, behavior: "auto" });
    resumeInline();

  };

  openBtn.addEventListener("click", async () => {

    savedScrollY = window.scrollY || window.pageYOffset || 0;
    modal.classList.remove("hidden");
    modal.setAttribute("aria-hidden", "false");
    document.body.style.overflow = "hidden";

    if (inlineVideo) {
      try {
        inlineVideo.pause();
      } catch {}
      try {
        videoEl.currentTime = inlineVideo.currentTime || 0;
      } catch {}
    }

    try {
      await videoEl.play();
    } catch {}

  });

  if (fullscreenBtn) {
    fullscreenBtn.addEventListener("click", async () => {
      if (videoEl.requestFullscreen) {
        try {
          await videoEl.requestFullscreen();
        } catch {}
      }
    });
  }

  closeBtn.addEventListener("click", () => {
    closeModal();
  });

  modal.addEventListener("click", (e) => {
    if (e.target === modal) closeModal();
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !modal.classList.contains("hidden")) {
      closeModal();
    }
  });

  if (inlineVideo) {
    inlineVideo.addEventListener("click", () => {
      openBtn.click();
    });
    resumeInline();
  }

}

function setupHomeMobileNav() {
  const toggleBtn = $("mobile-nav-toggle");
  const closeBtn = $("mobile-nav-close");
  const topbar = document.querySelector(".topbar");
  const panel = document.querySelector(".topbar-panel");
  const backdrop = $("mobile-nav-backdrop");
  if (!toggleBtn || !topbar || !panel) return;

  const closeMenu = () => {
    topbar.classList.remove("is-mobile-nav-open");
    document.body.classList.remove("is-mobile-nav-open");
    toggleBtn.setAttribute("aria-expanded", "false");
    toggleBtn.setAttribute("aria-label", tt("nav.openMenu"));
    if (backdrop) backdrop.setAttribute("aria-hidden", "true");
  };

  const openMenu = () => {
    topbar.classList.add("is-mobile-nav-open");
    document.body.classList.add("is-mobile-nav-open");
    toggleBtn.setAttribute("aria-expanded", "true");
    toggleBtn.setAttribute("aria-label", tt("nav.closeMenu"));
    if (backdrop) backdrop.setAttribute("aria-hidden", "false");
  };

  toggleBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    if (topbar.classList.contains("is-mobile-nav-open")) closeMenu();
    else openMenu();
  });

  if (closeBtn) {
    closeBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      closeMenu();
    });
  }

  if (backdrop) {
    backdrop.addEventListener("click", closeMenu);
  }

  document.addEventListener("click", (e) => {
    if (window.innerWidth > 980) return;
    if (!topbar.classList.contains("is-mobile-nav-open")) return;
    if (!topbar.contains(e.target) && e.target !== backdrop) closeMenu();
  });

  panel.addEventListener("click", (e) => {
    const link = e.target.closest("a");
    if (link) closeMenu();
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeMenu();
  });

  window.addEventListener("resize", () => {
    if (window.innerWidth > 980) closeMenu();
  });
}

function setupDemoMobileMenu() {
  if (!document.body.classList.contains("app-chat")) return;

  const toggleBtn = $("demo-mobile-menu-toggle");
  const closeBtn = $("demo-mobile-menu-close");
  const scrim = $("demo-mobile-scrim");
  const sidebar = document.querySelector(".gpt-sidebar");
  if (!toggleBtn || !sidebar) return;

  const MQ = 960;

  const setScrim = (open) => {
    if (!scrim) return;
    scrim.setAttribute("aria-hidden", open ? "false" : "true");
  };

  const setScrollLock = (lock) => {
    document.documentElement.style.overflow = lock ? "hidden" : "";
    document.body.style.overflow = lock ? "hidden" : "";
  };

  const closeMenu = () => {
    document.body.classList.remove("demo-mobile-menu-open");
    toggleBtn.setAttribute("aria-expanded", "false");
    setScrim(false);
    setScrollLock(false);
  };

  const openMenu = () => {
    document.body.classList.add("demo-mobile-menu-open");
    toggleBtn.setAttribute("aria-expanded", "true");
    setScrim(true);
    setScrollLock(true);
  };

  toggleBtn.addEventListener("click", (e) => {
    e.stopPropagation();
    if (document.body.classList.contains("demo-mobile-menu-open")) closeMenu();
    else openMenu();
  });

  if (closeBtn) {
    closeBtn.addEventListener("click", (e) => {
      e.stopPropagation();
      closeMenu();
    });
  }

  if (scrim) {
    scrim.addEventListener("click", () => closeMenu());
  }

  sidebar.addEventListener("click", (e) => {
    const clickable = e.target.closest("a, button, [data-conv-id], [data-conv-del]");
    if (clickable && clickable !== closeBtn && window.innerWidth <= MQ) closeMenu();
  });

  document.addEventListener("click", (e) => {
    if (window.innerWidth > MQ) return;
    if (!document.body.classList.contains("demo-mobile-menu-open")) return;
    if (sidebar.contains(e.target) || toggleBtn.contains(e.target)) return;
    if (scrim && e.target === scrim) return; /* handled above */
    closeMenu();
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape") closeMenu();
  });

  window.addEventListener("resize", () => {
    if (window.innerWidth > MQ) closeMenu();
  });
}



function setupReveal() {

  const targets = Array.from(

    document.querySelectorAll(

      ".section .container, .page-hero .container, .chat-shell, .benefit-card, .feature-card, .price-card, .testimonial-card, .compare-card, .cta-card"

    )

  );

  if (!targets.length) return;



  targets.forEach((el) => el.classList.add("reveal"));

  const observer = new IntersectionObserver(

    (entries) => {

      entries.forEach((entry) => {

        if (!entry.isIntersecting) return;

        entry.target.classList.add("in-view");

        observer.unobserve(entry.target);

      });

    },

    { threshold: 0.15 }

  );

  targets.forEach((el) => observer.observe(el));

}



if (formEl && messageEl && sendBtn) {

  formEl.addEventListener("submit", async (e) => {

    e.preventDefault();

    if (chatRequestInFlight) return;

    const text = messageEl.value.trim();
    const imagePayload = pendingAttachImage;

    if (!text && !imagePayload) return;

    if (!isAuthenticated) {
      openLoginModal();
      return;
    }

    chatRequestInFlight = true;
    const displayText = text || (imagePayload ? (tt("chat.analyzeImage") || "Analyze this image.") : "");
    addMessage("user", displayText, {
      imageUrl: imagePayload?.dataUrl || ""
    });
    playSendSound();

    messageEl.value = "";
    clearPendingAttachImage();
    refreshCharCount();

    sendBtn.disabled = true;
    const stopBtn = $("stop-generate-btn");
    if (stopBtn) stopBtn.classList.remove("hidden");
    activeChatAbort = new AbortController();
    const signal = activeChatAbort.signal;
    let chatTimedOut = false;
    const chatTimeoutId = setTimeout(() => {
      chatTimedOut = true;
      try {
        activeChatAbort.abort();
      } catch {
        /* ignore */
      }
    }, CHAT_CLIENT_TIMEOUT_MS);

    const thinking = addThinkingMessage();
    let streamed = false;
    const t0 = thinking?.started || Date.now();

    // Mode découverte : animation puis paywall (pas d'appel API)
    if (!isPremium && sessionToken) {
      clearTimeout(chatTimeoutId);
      await new Promise((r) => setTimeout(r, 1100));
      if (thinking?.timer) clearInterval(thinking.timer);
      if (thinking?.el) thinking.el.remove();
      openPaywallModal();
      sendBtn.disabled = false;
      if (stopBtn) stopBtn.classList.add("hidden");
      activeChatAbort = null;
      chatRequestInFlight = false;
      return;
    }

    // Remet l'image le temps de l'envoi (clearPending l'a déjà enlevée de l'UI)
    if (imagePayload) pendingAttachImage = imagePayload;

    try {

      const reply = await sendMessage(
        displayText,
        (_d, full) => {
          streamed = true;
          if (thinking?.timer) {
            clearInterval(thinking.timer);
            thinking.timer = null;
          }
          updateThinkingStream(thinking, dedupeConcatenatedReply(full));
        },
        signal
      );

      const replyClean = dedupeConcatenatedReply(reply);
      if (streamed && thinking) {
        updateThinkingStream(thinking, replyClean, { force: true });
      }
      if (thinking?.timer) clearInterval(thinking.timer);
      if (thinking?.el) thinking.el.remove();
      const latencyMs = Date.now() - t0;
      const replyText = String(replyClean || "").trim();
      if (!replyText) {
        const emptyErr = new Error("Réponse vide du modèle.");
        emptyErr.code = "EMPTY_REPLY";
        emptyErr.status = 502;
        throw emptyErr;
      }
      addMessage("assistant", replyText, { latencyMs });
      try {
        if (localStorage.getItem("darkclaude_notify_ready") === "1" && document.hidden && "Notification" in window) {
          if (Notification.permission === "granted") new Notification("DarkClaude", { body: "Réponse prête" });
          else if (Notification.permission !== "denied") Notification.requestPermission();
        }
      } catch {
        /* ignore */
      }
      pollBackendHealth();

    } catch (error) {

      if (thinking?.timer) clearInterval(thinking.timer);
      if (thinking?._mdTimer) {
        clearTimeout(thinking._mdTimer);
        thinking._mdTimer = null;
      }
      if (thinking?._mdRaf) {
        cancelAnimationFrame(thinking._mdRaf);
        thinking._mdRaf = 0;
      }

      // Coupure mid-stream : conserver le texte déjà reçu (ne pas wipe la bulle)
      const partial = String(thinking?._streamFull || "").trim();
      const keepPartial = streamed && partial.length > 0;
      if (keepPartial) {
        try {
          updateThinkingStream(thinking, partial, { force: true });
        } catch {
          /* ignore */
        }
        if (thinking?.el) thinking.el.remove();
        addMessage("assistant", partial, { latencyMs: Date.now() - t0 });
      } else if (thinking?.el) {
        thinking.el.remove();
      }

      if (chatTimedOut && error?.name === "AbortError") {
        error.timedOut = true;
        error.code = "CHAT_TIMEOUT";
      }
      const info = classifyChatError(error, error?.status);
      if (keepPartial) {
        if (info.kind === "abort") {
          showDgToast(tt("toast.generationStopped") || "Generation stopped");
        } else if (
          info.kind === "network" ||
          info.kind === "offline" ||
          info.kind === "timeout" ||
          isLikelyBrowserNetworkError(error)
        ) {
          showDgToast(tt("toast.connectionInterrupted") || "Connection interrupted");
        } else if (info.kind !== "paywall") {
          showDgToast(info.title || tt("toast.connectionInterrupted") || "Connection interrupted");
        }
        if (info.offline && info.kind !== "abort") {
          addOfflineMessage({
            title: info.title || tt("err.genericTitle") || "Error",
            desc: info.desc || error.message || "Une erreur est survenue.",
            retryText: info.retryText,
            kind: info.kind
          });
        }
      } else if (info.kind === "abort") {
        addMessage("assistant", "Génération arrêtée.");
      } else if (info.kind === "paywall") {
        /* modal déjà ouverte dans sendMessage */
      } else if (info.offline) {
        addOfflineMessage(info);
      } else {
        addOfflineMessage({
          title: info.title || tt("err.genericTitle") || "Error",
          desc: info.desc || error.message || "Une erreur est survenue.",
          retryText: info.retryText,
          kind: info.kind
        });
      }
      pollBackendHealth();

    } finally {

      clearTimeout(chatTimeoutId);
      activeChatAbort = null;
      pendingAttachImage = null;
      if (stopBtn) stopBtn.classList.add("hidden");
      sendBtn.disabled = !isAuthenticated;
      chatRequestInFlight = false;
      void streamed;

    }

  });



  messageEl.addEventListener("keydown", (e) => {

    if (e.key === "Enter" && !e.shiftKey) {

      e.preventDefault();

      formEl.requestSubmit();

    }

  });

  messageEl.addEventListener("input", refreshCharCount);

}



if (healthBtn) {

  healthBtn.addEventListener("click", async () => {

    setText(statusEl, "Vérification du service…");

    try {

      const response = await fetch("/api/health");

      const data = await response.json();

      if (!response.ok || !data.ok) {
        throw new Error(data?.hint || data?.message || "Service indisponible");
      }

      setText(statusEl, "Service DarkClaude disponible.");

    } catch (error) {

      setText(statusEl, `Service indisponible : ${error.message}`);

    }

  });

}



if (loginOpenBtn) loginOpenBtn.addEventListener("click", openLoginModal);

if (logoutBtn) {

  logoutBtn.addEventListener("click", async () => {
    try {
      if (sessionToken) {
        await fetch("/api/auth/logout", {
          method: "POST",
          headers: { "x-session-token": sessionToken }
        });
      }
    } catch (_) {}
    sessionToken = "";
    isAuthenticated = false;
    persistUser(null);
    localStorage.removeItem(SESSION_TOKEN_KEY);
    clearConversationsMemory();
    if (premiumPollTimer) clearInterval(premiumPollTimer);
    await unregisterServiceWorkers();
    window.location.replace("/");
  });

}

openLoginButtons.forEach((btn) => btn.addEventListener("click", openLoginModal));

if (chatEl && !document.body.classList.contains("app-chat")) {
  addMessage("assistant", "DarkClaude est prêt. Écrivez votre message.");
}

document.addEventListener("keydown", (e) => {
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "k") {
    if (messageEl && document.body.classList.contains("app-chat")) {
      e.preventDefault();
      messageEl.focus();
    }
    return;
  }
  if (e.key !== "Escape") return;
  const settingsM = $("settings-modal");
  if (settingsM && !settingsM.classList.contains("hidden")) {
    closeSettingsModal();
    e.preventDefault();
    return;
  }
  if (messageEl && messageEl.value.trim()) {
    messageEl.value = "";
    refreshCharCount();
    messageEl.dispatchEvent(new Event("input", { bubbles: true }));
  }
});

boot();

document.addEventListener("click", (e) => {
  const t = e.target;
  if (!(t instanceof Element)) return;
  if (t.closest("#sidebar-settings-btn")) {
    e.preventDefault();
    openSettingsModal();
  }
});

document.addEventListener("darkclaude:i18n", () => {
  try {
    if (typeof refreshAuthUi === "function") refreshAuthUi();
    if (typeof renderHistory === "function") renderHistory();
    if (typeof updateSidebarPremiumStats === "function") updateSidebarPremiumStats();
    if (typeof syncDiscoveryUi === "function") syncDiscoveryUi();
    if (typeof syncFixedChatUi === "function") syncFixedChatUi();
    if (typeof syncComposerModeMenu === "function") syncComposerModeMenu();
    const profileBlock = document.querySelector(".dg-side-user");
    if (profileBlock) profileBlock.title = tt("chat.profileSettings") || "Profile / Settings";
  } catch (_e) {}
});
