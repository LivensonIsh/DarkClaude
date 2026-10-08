(function () {
  var API = "https://darkclaude-api.dynv6.net";
  var h = location.hostname;
  var local = h === "localhost" || h === "127.0.0.1" || /^\d+\.\d+\.\d+\.\d+$/.test(h);
  var B = local ? "" : API;
  var f = window.fetch;
  window.fetch = function (u, o) { if (typeof u === "string" && u.indexOf("/api/") === 0) u = B + u; return f.call(this, u, o); };
})();
