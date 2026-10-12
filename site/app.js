(function(){
  "use strict";

  fetch("device.svg")
    .then(function(r){ return r.text(); })
    .then(function(txt){
      var holder = document.createElement("div");
      holder.innerHTML = txt;
      var defs = holder.querySelector("svg");
      if (!defs) return;
      document.body.insertBefore(defs, document.body.firstChild);
      var uses = document.querySelectorAll("use");
      for (var i = 0; i < uses.length; i++) {
        var h = uses[i].getAttribute("href");
        uses[i].removeAttribute("href");
        uses[i].setAttribute("href", h);
      }
    })
    .catch(function(){});
})();

(function(){
  "use strict";

  var SHELLS = [
    { name:"graphite", light:"#6a6d75", base:"#3c3e44", shade:"#1c1d21", btn:"#4a4c52" },
    { name:"silver",   light:"#eceef1", base:"#c3c7cd", shade:"#868b93", btn:"#b8bcc2" },
    { name:"pink",     light:"#f6d3dd", base:"#e2a3b5", shade:"#a26a7b", btn:"#d993a7" },
    { name:"blue",     light:"#d6e4f0", base:"#93b0c9", shade:"#5a7690", btn:"#86a3bd" }
  ];

  var want = (location.search.match(/[?&]shell=([^&]+)/) || [])[1];
  var shell = SHELLS[Math.floor(Math.random() * SHELLS.length)];
  if (want) {
    want = decodeURIComponent(want).replace(/\+/g, " ").toLowerCase();
    for (var si = 0; si < SHELLS.length; si++) {
      if (SHELLS[si].name === want) { shell = SHELLS[si]; break; }
    }
  }

  var root = document.documentElement.style;
  root.setProperty("--shell-light", shell.light);
  root.setProperty("--shell-base",  shell.base);
  root.setProperty("--shell-shade", shell.shade);
  root.setProperty("--btn",         shell.btn);

  var pose = (location.search.match(/[?&]pose=(flat|hero)/) || [])[1];
  if (pose) {
    var heroDevEl = document.querySelector(".hero-device .device");
    if (heroDevEl) heroDevEl.className = "device pose-" + pose;
  }

  var reduced = window.matchMedia("(prefers-reduced-motion: reduce)").matches;


  // Media slots. Every clip and still names a file in media/; until that file exists, a labelled
  // gray card stands in for it, so recordings can be dropped in later without touching the HTML.

  var SCREEN = { phone: [1020, 450], fold: [940, 840] };

  function esc(s){
    return String(s).replace(/[&<>"']/g, function(c){
      return { "&":"&amp;", "<":"&lt;", ">":"&gt;", "\"":"&quot;", "'":"&#39;" }[c];
    });
  }

  function placeholder(name, note, w, h){
    var cx = w / 2, cy = h / 2;
    var u = Math.min(w, h) / 10;
    // Text shrinks to fit the card's width (about 0.55em per character).
    function fit(size, text){ return Math.min(size, w * .88 / (Math.max(1, text.length) * .55)); }
    var nameSize = fit(u * .55, "media/" + name);
    var noteSize = note ? fit(u * .42, note) : 0;
    var svg =
      '<svg xmlns="http://www.w3.org/2000/svg" width="' + w + '" height="' + h + '" viewBox="0 0 ' + w + ' ' + h + '">' +
      '<rect width="100%" height="100%" fill="#2a2a31"/>' +
      '<rect x="' + u * .4 + '" y="' + u * .4 + '" width="' + (w - u * .8) + '" height="' + (h - u * .8) + '" rx="' + u * .3 + '"' +
        ' fill="none" stroke="#4d4d57" stroke-width="' + u * .06 + '" stroke-dasharray="' + u * .3 + ' ' + u * .2 + '"/>' +
      '<g transform="translate(' + cx + ' ' + (cy - u * 1.1) + ')" fill="none" stroke="#8a8b97" stroke-width="' + u * .1 + '" stroke-linejoin="round">' +
        '<rect x="' + -u * .9 + '" y="' + -u * .55 + '" width="' + u * 1.3 + '" height="' + u * 1.1 + '" rx="' + u * .15 + '"/>' +
        '<path d="M' + u * .4 + ' ' + -u * .2 + ' L' + u * .9 + ' ' + -u * .5 + ' V' + u * .5 + ' L' + u * .4 + ' ' + u * .2 + ' Z"/>' +
      '</g>' +
      '<text x="' + cx + '" y="' + (cy + u * .55) + '" fill="#f4f5f9" font-family="ui-monospace,Menlo,Consolas,monospace"' +
        ' font-size="' + nameSize + '" text-anchor="middle">media/' + esc(name) + '</text>' +
      (note ? '<text x="' + cx + '" y="' + (cy + u * 1.35) + '" fill="#bcbdc9" font-family="Open Sans,sans-serif"' +
        ' font-size="' + noteSize + '" text-anchor="middle">' + esc(note) + '</text>' : '') +
      '</svg>';
    return "data:image/svg+xml;charset=utf-8," + encodeURIComponent(svg);
  }

  // Sets `el`'s poster (a video) or src (an img) to media/<still>, or the placeholder when it is
  // missing. `token` guards against a slow probe landing after the step has already moved on.
  function stillOrPlaceholder(el, still, label, note, w, h, attr){
    var token = {};
    el._probe = token;
    var probe = new Image();
    probe.onload = function(){ if (el._probe === token) el.setAttribute(attr, "media/" + still); };
    probe.onerror = function(){ if (el._probe === token) el.setAttribute(attr, placeholder(label, note, w, h)); };
    probe.src = "media/" + still;
  }

  function stillName(clip){ return clip.replace(/\.mp4$/, ".webp"); }

  // A video whose clip is missing gives up its src and keeps the poster, the placeholder card.
  function guardVideo(v){
    v.addEventListener("error", function(){
      v.removeAttribute("src");
      v.load();
    });
  }

  var imgs = document.querySelectorAll("img.ph-img");
  for (var ii = 0; ii < imgs.length; ii++) {
    (function(img){
      var ratio = (img.getAttribute("data-ratio") || "3/2").split("/");
      var h = 600, w = Math.round(h * Number(ratio[0]) / Number(ratio[1]));
      stillOrPlaceholder(img, img.getAttribute("data-media"), img.getAttribute("data-media"),
                         img.getAttribute("data-note"), w, h, "src");
    })(imgs[ii]);
  }


  function setDevice(dev, kind){
    if (!dev) return;
    dev.setAttribute("data-device", kind === "fold" ? "fold" : "phone");
  }

  function playSafe(v){
    var p = v.play();
    if (p && p.catch) p.catch(function(){});
  }

  var heroVid = document.querySelector(".hero-screen");
  if (heroVid) {
    guardVideo(heroVid);
    var heroClip = heroVid.getAttribute("data-clip");
    var heroSize = SCREEN.phone;
    stillOrPlaceholder(heroVid, stillName(heroClip), heroClip, heroVid.getAttribute("data-note"),
                       heroSize[0], heroSize[1], "poster");
    if (reduced) {
      heroVid.style.opacity = "1";
    } else {
      setTimeout(function(){
        heroVid.style.opacity = "1";
        if (heroVid.getAttribute("src")) playSafe(heroVid);
      }, 900);
      heroVid.src = "media/" + heroClip;
      heroVid.load();
    }
  }


  var vid   = document.querySelector(".guide-screen");
  var gdev  = document.querySelector(".guide-stage .device");
  var steps = Array.prototype.slice.call(document.querySelectorAll(".step"));
  if (vid) guardVideo(vid);

  var current = null;

  var playRetries = [];
  function ensurePlaying(){
    if (reduced || !vid || !vid.getAttribute("src") || !vid.paused) return;
    playSafe(vid);
  }
  function nudgePlayback(){
    for (var i = 0; i < playRetries.length; i++) clearTimeout(playRetries[i]);
    playRetries = [];
    ensurePlaying();
    playRetries.push(setTimeout(ensurePlaying, 220), setTimeout(ensurePlaying, 800));
  }
  if (vid) {
    vid.addEventListener("loadeddata", ensurePlaying);
    vid.addEventListener("canplay", ensurePlaying);
  }

  function show(step){
    if (!vid || !step || step === current) return;
    current = step;
    var at = 0;
    for (var i = 0; i < steps.length; i++) {
      var on = steps[i] === step;
      steps[i].classList.toggle("is-active", on);
      if (on) at = i;
    }
    if (stepAt) stepAt.textContent = String(at + 1);
    if (prevBtn) prevBtn.disabled = at === 0;
    if (nextBtn) nextBtn.disabled = at === steps.length - 1;

    var kind = step.getAttribute("data-device") || "phone";
    setDevice(gdev, kind);
    var size = SCREEN[kind] || SCREEN.phone;

    var clip = step.getAttribute("data-clip");
    if (!clip) { vid.style.opacity = "0"; vid.removeAttribute("src"); vid.load(); return; }
    var note = step.getAttribute("data-note");
    stillOrPlaceholder(vid, stillName(clip), clip, note, size[0], size[1], "poster");
    vid.style.opacity = "1";
    if (reduced) {
      vid.removeAttribute("src");
      vid.load();
      return;
    }
    var path = "media/" + clip;
    if (vid.getAttribute("src") === path) { nudgePlayback(); return; }
    vid.src = path;
    vid.load();
    nudgePlayback();
  }

  var stepList = document.querySelector(".guide-steps");
  var prevBtn  = document.getElementById("step-prev");
  var nextBtn  = document.getElementById("step-next");
  var stepAt   = document.getElementById("step-at");
  var stepOf   = document.getElementById("step-of");
  if (stepList && steps.length) stepList.classList.add("is-live");
  if (stepOf) stepOf.textContent = String(steps.length);

  function stepBy(delta){
    var at = steps.indexOf(current);
    var to = at + delta;
    if (at < 0 || to < 0 || to >= steps.length) return;
    show(steps[to]);
    setHash(steps[to].id);
  }
  if (prevBtn) prevBtn.addEventListener("click", function(){ stepBy(-1); });
  if (nextBtn) nextBtn.addEventListener("click", function(){ stepBy(1); });

  var guidePanel = document.getElementById("guide");
  if (guidePanel) guidePanel.addEventListener("keydown", function(e){
    if (e.target.closest("[role=tablist]")) return;
    if (e.key === "ArrowLeft") { e.preventDefault(); stepBy(-1); }
    else if (e.key === "ArrowRight") { e.preventDefault(); stepBy(1); }
  });


  function wire(list){
    var tabs = Array.prototype.slice.call(list.querySelectorAll("[role=tab]"));
    var group = {
      tabs: tabs,
      panels: tabs.map(function(t){ return document.getElementById(t.getAttribute("aria-controls")); }),
      select: function(tab, focus){
        var i = tabs.indexOf(tab);
        if (i < 0) return;
        for (var k = 0; k < tabs.length; k++) {
          var on = k === i;
          tabs[k].setAttribute("aria-selected", on ? "true" : "false");
          tabs[k].tabIndex = on ? 0 : -1;
          if (group.panels[k]) group.panels[k].hidden = !on;
        }
        if (hero) hero.hidden = true;
        if (tabs[i].scrollIntoView && list.scrollWidth > list.clientWidth + 1) {
          try { tabs[i].scrollIntoView({ inline: "center", block: "nearest" }); } catch (e) {}
        }
        if (focus) tabs[i].focus();
      }
    };
    for (var i = 0; i < tabs.length; i++) {
      (function(tab){
        tab.addEventListener("click", function(){
          group.select(tab, false);
          setHash(tab.getAttribute("aria-controls"));
        });
      })(tabs[i]);
    }
    list.addEventListener("keydown", function(e){
      var at = tabs.indexOf(document.activeElement);
      if (at < 0) return;
      var to = -1;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") to = (at + 1) % tabs.length;
      else if (e.key === "ArrowLeft" || e.key === "ArrowUp") to = (at - 1 + tabs.length) % tabs.length;
      else if (e.key === "Home") to = 0;
      else if (e.key === "End") to = tabs.length - 1;
      if (to < 0) return;
      e.preventDefault();
      group.select(tabs[to], true);
      setHash(tabs[to].getAttribute("aria-controls"));
    });
    return group;
  }

  var hero = document.getElementById("hero");
  var groups = Array.prototype.slice.call(document.querySelectorAll("[role=tablist]")).map(wire);


  var menuBtn = document.getElementById("menu-btn");
  var tabsNav = document.getElementById("section-tabs");
  function setMenu(open){
    if (!menuBtn || !tabsNav) return;
    tabsNav.classList.toggle("is-open", open);
    menuBtn.setAttribute("aria-expanded", open ? "true" : "false");
  }
  if (menuBtn && tabsNav) {
    menuBtn.addEventListener("click", function(e){
      e.stopPropagation();
      setMenu(!tabsNav.classList.contains("is-open"));
    });
    tabsNav.addEventListener("click", function(){ setMenu(false); });
    document.addEventListener("click", function(e){
      if (e.target !== menuBtn && !menuBtn.contains(e.target) && !tabsNav.contains(e.target)) {
        setMenu(false);
      }
    });
    document.addEventListener("keydown", function(e){
      if (e.key !== "Escape" || !tabsNav.classList.contains("is-open")) return;
      setMenu(false);
      menuBtn.focus();
    });
  }

  function showHero(){
    var grp = groups[0];
    if (!grp) return;
    for (var k = 0; k < grp.tabs.length; k++) {
      grp.tabs[k].setAttribute("aria-selected", "false");
      grp.tabs[k].tabIndex = k === 0 ? 0 : -1;
    }
    for (var q = 0; q < grp.panels.length; q++) {
      if (grp.panels[q]) grp.panels[q].hidden = true;
    }
    if (hero) hero.hidden = false;
  }

  var mark = document.querySelector(".mast-mark");
  if (mark) mark.addEventListener("click", function(e){
    e.preventDefault();
    showHero();
    setHash("hero");
  });


  var routing = false;
  function setHash(id){
    if (!id) return;
    routing = true;
    if (history.replaceState) history.replaceState(null, "", "#" + id);
    else location.hash = id;
    routing = false;
  }

  function reveal(id){
    var el = id && document.getElementById(id);
    if (!el) return false;
    if (el === hero) { showHero(); return true; }
    var chain = [];
    for (var node = el; node && node !== document.body; node = node.parentNode) {
      for (var g = 0; g < groups.length; g++) {
        var at = groups[g].panels.indexOf(node);
        if (at >= 0) chain.unshift({ group: groups[g], tab: groups[g].tabs[at] });
      }
    }
    for (var c = 0; c < chain.length; c++) chain[c].group.select(chain[c].tab, false);
    if (el.classList.contains("step")) show(el);
    return true;
  }

  window.addEventListener("hashchange", function(){
    if (routing) return;
    reveal(location.hash.slice(1));
  });

  if (steps.length) show(steps[0]);
  reveal(location.hash.slice(1));

  var say = document.querySelector(".say");
  var voice = document.getElementById("say-slot");
  if (say && voice) {
    say.addEventListener("click", function(){
      voice.currentTime = 0;
      var playing = voice.play();
      if (playing && playing.catch) playing.catch(function(){});
    });
    voice.addEventListener("play", function(){ say.classList.add("is-playing"); });
    ["pause", "ended"].forEach(function(e){
      voice.addEventListener(e, function(){ say.classList.remove("is-playing"); });
    });
  }

})();
