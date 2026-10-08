(function () {
  var B = /(^|\.)darkclaude\.online$/.test(location.hostname) ? "https://api.darkclaude.online" : "";
  var f = window.fetch;
  window.fetch = function (u, o) { if (typeof u === "string" && u.indexOf("/api/") === 0) u = B + u; return f.call(this, u, o); };
})();
