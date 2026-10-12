"use strict";

// Rexit chat viewer. Everything is served by `rexit view` on localhost.

const PAGE = 200;
const GROUP_GAP_MS = 5 * 60 * 1000;

const $ = (id) => document.getElementById(id);
const els = {
  app: $("app"),
  list: $("list"),
  search: $("search"),
  summary: $("summary"),
  exported: $("exported"),
  messages: $("messages"),
  empty: $("empty"),
  chatName: $("chat-name"),
  chatMeta: $("chat-meta"),
  chatAvatar: $("chat-avatar"),
  back: $("back"),
  lightbox: $("lightbox"),
  lightboxImg: $("lightbox-img"),
};

const state = {
  rooms: [],
  room: null, // current room summary
  messages: [], // loaded messages of the current room
  start: 0, // index of messages[0] in the room
  total: 0,
  loading: false,
  stickToBottom: true,
  query: "",
  hits: null,
};

// ---------- helpers ----------

async function api(path) {
  const response = await fetch(path);
  if (!response.ok) throw new Error(`${path}: ${response.status}`);
  return response.json();
}

function el(tag, className, text) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function hue(name) {
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + ch.codePointAt(0)) | 0;
  return Math.abs(hash) % 360;
}

function avatar(name, node = el("div", "avatar")) {
  node.textContent = ([...name.trim()][0] || "?").toUpperCase();
  node.style.background = `hsl(${hue(name)} 32% 48%)`;
  node.hidden = false;
  return node;
}

const sameDay = (a, b) => a.toDateString() === b.toDateString();

function dayLabel(date) {
  const now = new Date();
  const yesterday = new Date(now);
  yesterday.setDate(now.getDate() - 1);
  if (sameDay(date, now)) return "Today";
  if (sameDay(date, yesterday)) return "Yesterday";
  const opts = { weekday: "long", day: "numeric", month: "long" };
  if (date.getFullYear() !== now.getFullYear()) opts.year = "numeric";
  return date.toLocaleDateString(undefined, opts);
}

function shortDate(date) {
  const now = new Date();
  if (sameDay(date, now)) return time(date);
  const days = (now - date) / 86400000;
  if (days < 6) return date.toLocaleDateString(undefined, { weekday: "short" });
  const opts = { day: "numeric", month: "short" };
  if (date.getFullYear() !== now.getFullYear()) opts.year = "2-digit";
  return date.toLocaleDateString(undefined, opts);
}

const time = (date) => date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });

const isGroupChat = (room) => room.members.length !== 1;

function preview(message, room) {
  if (!message) return "No messages";
  let text;
  if (message.kind === "image") text = "Photo";
  else if (message.kind === "deleted") text = "Message deleted";
  else text = message.text.replace(/\s+/g, " ");
  if (message.mine) return `You: ${text}`;
  if (isGroupChat(room) && message.author !== "N/A") return `${message.author}: ${text}`;
  return text;
}

// Text with clickable links, built from text nodes only
const URL_RE = /https?:\/\/[^\s<]+[^\s<.,:;"')\]!?]/g;
function linkify(text, parent) {
  let last = 0;
  for (const match of text.matchAll(URL_RE)) {
    parent.append(text.slice(last, match.index));
    const a = el("a", null, match[0]);
    a.href = match[0];
    a.target = "_blank";
    a.rel = "noopener noreferrer";
    parent.append(a);
    last = match.index + match[0].length;
  }
  parent.append(text.slice(last));
}

// Text with the search term marked
function highlight(text, query, parent) {
  const lower = text.toLowerCase();
  const q = query.toLowerCase();
  let at = lower.indexOf(q);
  // Keep the match visible in a one-line preview
  let from = at > 40 ? at - 30 : 0;
  if (from > 0) parent.append("…");
  let last = from;
  while (at !== -1) {
    parent.append(text.slice(last, at));
    parent.append(el("mark", null, text.slice(at, at + q.length)));
    last = at + q.length;
    at = lower.indexOf(q, last);
  }
  parent.append(text.slice(last));
}

// ---------- sidebar ----------

function roomItem(room, { active, previewNode, date, href }) {
  const item = el("button", "item" + (active ? " active" : ""));
  item.type = "button";
  item.addEventListener("click", () => (location.hash = href));
  item.append(avatar(room.name));
  const body = el("div", "item-body");
  const top = el("div", "item-top");
  top.append(el("span", "item-name", room.name));
  if (date) top.append(el("span", "item-date", shortDate(date)));
  body.append(top, previewNode);
  item.append(body);
  return item;
}

function renderList() {
  els.list.replaceChildren();
  const q = state.query.trim().toLowerCase();
  const rooms = q ? state.rooms.filter((r) => r.name.toLowerCase().includes(q)) : state.rooms;

  if (q) els.list.append(el("div", "section-title", "Chats"));
  for (const room of rooms) {
    const last = room.last;
    els.list.append(
      roomItem(room, {
        active: state.room && state.room.key === room.key,
        previewNode: el("div", "item-preview", preview(last, room)),
        date: last && new Date(last.timestamp),
        href: `#/${room.key}`,
      }),
    );
  }
  if (q && !rooms.length) els.list.append(el("div", "list-empty", "No chats found"));
  if (!q && !rooms.length) els.list.append(el("div", "list-empty", "This export has no chats"));

  if (q && state.hits) {
    els.list.append(el("div", "section-title", `Messages (${state.hits.length})`));
    for (const hit of state.hits) {
      const room = state.rooms.find((r) => r.key === hit.room);
      if (!room) continue;
      const node = el("div", "item-preview");
      if (hit.message.mine) node.append("You: ");
      highlight(hit.message.text, state.query.trim(), node);
      els.list.append(
        roomItem(room, {
          previewNode: node,
          date: new Date(hit.message.timestamp),
          href: `#/${hit.room}/${hit.index}`,
        }),
      );
    }
    if (!state.hits.length) els.list.append(el("div", "list-empty", "No messages found"));
  }
}

let searchTimer;
els.search.addEventListener("input", () => {
  state.query = els.search.value;
  clearTimeout(searchTimer);
  if (state.query.trim().length < 2) {
    state.hits = null;
    renderList();
    return;
  }
  renderList();
  const query = state.query;
  searchTimer = setTimeout(async () => {
    const hits = await api(`/api/search?q=${encodeURIComponent(query.trim())}`);
    if (query === state.query) {
      state.hits = hits;
      renderList();
    }
  }, 200);
});

// ---------- chat ----------

function renderMessages() {
  const frag = document.createDocumentFragment();
  if (state.start > 0) frag.append(el("div", "loading", "Loading earlier messages…"));

  const group = isGroupChat(state.room);
  let prev = null;
  let groupNode = null;

  state.messages.forEach((message, i) => {
    const date = new Date(message.timestamp);
    const prevDate = prev && new Date(prev.timestamp);
    const newDay = !prev || !sameDay(date, prevDate);
    const who = (m) => m.sender || m.author;
    const newGroup = newDay || who(prev) !== who(message) || date - prevDate > GROUP_GAP_MS;

    if (newDay) frag.append(el("div", "day", dayLabel(date)));
    if (newGroup) {
      if (groupNode) groupNode.lastChild.classList.add("last");
      groupNode = el("div", "group " + (message.mine ? "mine" : "theirs"));
      if (!message.mine && group && message.author !== "N/A") {
        groupNode.append(el("div", "author", message.author));
      }
      frag.append(groupNode);
    }

    const bubble = el("div", "bubble " + message.kind);
    if (newGroup) bubble.classList.add("first");
    bubble.dataset.index = state.start + i;
    bubble.title = date.toLocaleString();
    const stamp = el("span", "time", time(date));

    if (message.kind === "text") {
      linkify(message.text, bubble);
      bubble.append(stamp);
    } else if (message.kind === "image") {
      if (message.image) {
        const img = el("img");
        img.src = message.image;
        img.alt = "Image";
        img.loading = "lazy";
        img.addEventListener("load", keepBottom);
        img.addEventListener("click", () => openLightbox(message.image));
        bubble.append(img);
      } else {
        bubble.append(el("div", "missing", "Image not downloaded. Export again with --images to see it."));
      }
      bubble.append(stamp);
    } else {
      bubble.append("This message was deleted");
      bubble.append(stamp);
    }

    groupNode.append(bubble);
    prev = message;
  });
  if (groupNode) groupNode.lastChild.classList.add("last");

  if (state.start + state.messages.length < state.total) {
    frag.append(el("div", "loading", "Loading newer messages…"));
  }
  els.messages.replaceChildren(frag);
}

function keepBottom() {
  if (state.stickToBottom) els.messages.scrollTop = els.messages.scrollHeight;
}

async function loadPage(before) {
  const room = state.room;
  const page = await api(`/api/rooms/${encodeURIComponent(room.key)}/messages?before=${before}&limit=${PAGE}`);
  if (room !== state.room) return null; // switched chats meanwhile
  state.total = page.total;
  return page;
}

async function openRoom(key, focusIndex) {
  const room = state.rooms.find((r) => r.key === key);
  if (!room) {
    showEmpty();
    return;
  }
  state.room = room;
  state.loading = true;
  els.app.classList.add("chat-open");
  els.empty.hidden = true;
  els.chatName.textContent = room.name;
  els.chatMeta.textContent = isGroupChat(room)
    ? `${room.members.length} people · ${room.count} messages`
    : `${room.count} messages`;
  avatar(room.name, els.chatAvatar);
  document.title = `${room.name} · Rexit`;
  renderList();

  const before = focusIndex === undefined ? room.count : Math.min(room.count, focusIndex + PAGE / 2);
  const page = await loadPage(before);
  if (!page) return;
  state.messages = page.messages;
  state.start = page.start;
  renderMessages();

  if (focusIndex === undefined) {
    state.stickToBottom = true;
    keepBottom();
  } else {
    state.stickToBottom = false;
    const target = els.messages.querySelector(`[data-index="${focusIndex}"]`);
    if (target) {
      target.scrollIntoView({ block: "center" });
      target.classList.add("flash");
    }
  }
  state.loading = false;
}

async function loadOlder() {
  if (state.loading || state.start === 0) return;
  state.loading = true;
  const page = await loadPage(state.start);
  if (page) {
    const before = els.messages.scrollHeight - els.messages.scrollTop;
    state.messages = page.messages.concat(state.messages);
    state.start = page.start;
    renderMessages();
    els.messages.scrollTop = els.messages.scrollHeight - before;
  }
  state.loading = false;
}

async function loadNewer() {
  const end = state.start + state.messages.length;
  if (state.loading || end >= state.total) return;
  state.loading = true;
  const page = await loadPage(Math.min(state.total, end + PAGE));
  if (page) {
    const top = els.messages.scrollTop;
    state.messages = state.messages.concat(page.messages.slice(end - page.start));
    renderMessages();
    els.messages.scrollTop = top;
  }
  state.loading = false;
}

els.messages.addEventListener("scroll", () => {
  const m = els.messages;
  const fromBottom = m.scrollHeight - m.scrollTop - m.clientHeight;
  state.stickToBottom = fromBottom < 40 && state.start + state.messages.length >= state.total;
  if (m.scrollTop < 300) loadOlder();
  if (fromBottom < 300) loadNewer();
});

function showEmpty() {
  state.room = null;
  els.app.classList.remove("chat-open");
  els.chatName.textContent = "";
  els.chatMeta.textContent = "";
  els.chatAvatar.hidden = true;
  els.messages.replaceChildren(els.empty);
  els.empty.hidden = false;
  document.title = "Rexit";
  renderList();
}

// ---------- lightbox ----------

function openLightbox(src) {
  els.lightboxImg.src = src;
  els.lightbox.hidden = false;
}
els.lightbox.addEventListener("click", () => (els.lightbox.hidden = true));

// ---------- navigation ----------

function route() {
  const [key, index] = location.hash.replace(/^#\/?/, "").split("/");
  if (!key) showEmpty();
  else openRoom(decodeURIComponent(key), index === undefined ? undefined : Number(index));
}

els.back.addEventListener("click", () => (location.hash = ""));
window.addEventListener("hashchange", route);

document.addEventListener("keydown", (event) => {
  const typing = event.target === els.search;
  if (event.key === "Escape") {
    if (!els.lightbox.hidden) els.lightbox.hidden = true;
    else if (state.query) {
      els.search.value = "";
      els.search.dispatchEvent(new Event("input"));
    }
  } else if (event.key === "/" && !typing) {
    event.preventDefault();
    els.search.focus();
  } else if (event.altKey && (event.key === "ArrowUp" || event.key === "ArrowDown")) {
    event.preventDefault();
    const at = state.rooms.findIndex((r) => state.room && r.key === state.room.key);
    const next = event.key === "ArrowDown" ? at + 1 : Math.max(at - 1, 0);
    if (state.rooms[next]) location.hash = `#/${state.rooms[next].key}`;
  }
});

// ---------- start ----------

(async function start() {
  const [info, rooms] = await Promise.all([api("/api/info"), api("/api/rooms")]);
  state.rooms = rooms;
  els.summary.textContent = `${info.rooms} chats · ${info.messages.toLocaleString()} messages`;
  if (info.exported_at) {
    els.exported.textContent = `Exported ${new Date(info.exported_at).toLocaleString()}`;
  } else {
    els.exported.textContent = "Older export: re-export for names and sides";
  }
  route();
})().catch((error) => {
  els.list.replaceChildren(el("div", "list-empty", `Could not load the export (${error.message})`));
});
