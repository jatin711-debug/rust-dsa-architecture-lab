// tree.js — BST vs AVL visualizer driven by the raw wasm tree exports.
//
// The wasm records a full structure snapshot after every insert (see
// tree_recorder.rs for the packing). We replay the snapshots, compute a tree
// layout per snapshot, and lerp node positions between consecutive snapshots —
// so rotations animate as nodes gliding to their new homes.

"use strict";

const $ = (id) => document.getElementById(id);

// Surface any runtime error on the page for debugging.
window.addEventListener("error", (e) => {
  const s = $("status");
  if (s) s.textContent = "JS error: " + e.message;
});
window.addEventListener("unhandledrejection", (e) => {
  const s = $("status");
  if (s) s.textContent = "JS rejection: " + String(e.reason);
});

const state = {
  wasm: null,
  steps: { bst: [], avl: [] }, // parsed steps: [{ value, nodes: Map }]
  pos: 0,
  playing: false,
  raf: null,
  animT: 1, // 0..1 progress of the current step's lerp
  prevPos: { bst: null, avl: null },
  lastValue: -1,
};

const COLORS = { INSERT: "#22c55e", MOVED: "#f59e0b", ROOT: "#38bdf8", NODE: "#64748b", EDGE: "#334155" };

// --- wasm loading (same raw pattern as main.js) ---------------------------
async function loadWasm() {
  let module;
  try {
    const response = await fetch("wasm_visualizer.wasm");
    module = await WebAssembly.instantiateStreaming(response, {});
  } catch {
    const response = await fetch("wasm_visualizer.wasm");
    module = await WebAssembly.instantiate(await response.arrayBuffer(), {});
  }
  state.wasm = module.instance.exports;
  newTree();
}

// --- parsing ---------------------------------------------------------------
function parseTrack(steps32) {
  const out = [];
  let i = 0;
  while (i < steps32.length) {
    const header = steps32[i];
    // kind in top 4 bits, count in the 6 bits below it (n <= 40).
    const count = (header >>> 22) & 0x3f;
    const value = header & 0x3fffff;
    const nodes = new Map();
    for (let j = 0; j < count; j++) {
      const rec = steps32[i + 1 + j];
      const v = (rec >>> 20) & 0x3ff;
      const l = ((rec >>> 10) & 0x3ff) - 1;
      const r = (rec & 0x3ff) - 1;
      nodes.set(v, { left: l >= 0 ? l : null, right: r >= 0 ? r : null });
    }
    out.push({ value, nodes });
    i += 1 + count;
  }
  return out;
}

// --- layout ----------------------------------------------------------------
// x = in-order position, y = depth. Root = value nobody points at.
function layout(nodes) {
  const children = new Set();
  nodes.forEach(({ left, right }) => {
    if (left != null) children.add(left);
    if (right != null) children.add(right);
  });
  const root = [...nodes.keys()].find((v) => !children.has(v));
  const pos = new Map();
  let ino = 0;
  (function walk(v, depth) {
    if (v == null) return;
    const n = nodes.get(v);
    walk(n.left, depth + 1);
    pos.set(v, { x: ino++, y: depth });
    walk(n.right, depth + 1);
  })(root, 0);
  return { root, pos };
}

function treeHeight(nodes) {
  let maxDepth = 0;
  (function walk(v, depth) {
    if (v == null) return;
    maxDepth = Math.max(maxDepth, depth + 1);
    const n = nodes.get(v);
    walk(n.left, depth + 1);
    walk(n.right, depth + 1);
  })(layout(nodes).root, 0);
  return maxDepth;
}

// --- drawing ---------------------------------------------------------------
function drawTree(canvas, nodes, pos, highlights, animPos) {
  const ctx = canvas.getContext("2d");
  const W = canvas.width;
  const H = canvas.height;
  ctx.clearRect(0, 0, W, H);

  const n = nodes.size;
  if (n === 0) return;
  // Scale positions into the canvas with padding for node circles.
  const padX = 26;
  const padY = 30;
  const maxX = Math.max(...[...pos.values()].map((p) => p.x), 0);
  const maxY = Math.max(...[...pos.values()].map((p) => p.y), 0);
  const slotX = (W - 2 * padX) / Math.max(maxX, 1);
  const slotY = (H - 2 * padY) / Math.max(maxY, 1);
  const R = Math.min(13, slotX / 2.2);

  const px = (p) => padX + p.x * slotX;
  const py = (p) => padY + p.y * slotY;

  // Edges first (under the nodes).
  ctx.strokeStyle = COLORS.EDGE;
  ctx.lineWidth = 1.5;
  for (const [v, child] of nodes) {
    const from = pos.get(v);
    if (from == null) continue;
    for (const c of [child.left, child.right]) {
      if (c == null) continue;
      const to = pos.get(c);
      ctx.beginPath();
      ctx.moveTo(px(from), py(from));
      ctx.lineTo(px(to), py(to));
      ctx.stroke();
    }
  }

  // Nodes.
  for (const [v, child] of nodes) {
    const p = pos.get(v);
    if (p == null) continue;
    let color = COLORS.NODE;
    if (highlights.root === v) color = COLORS.ROOT;
    if (highlights.inserted === v) color = COLORS.INSERT;
    if (highlights.moved.has(v)) color = COLORS.MOVED;
    ctx.beginPath();
    ctx.arc(px(p), py(p), R, 0, Math.PI * 2);
    ctx.fillStyle = color;
    ctx.fill();
    ctx.strokeStyle = "#0b1222";
    ctx.lineWidth = 1.5;
    ctx.stroke();
    ctx.fillStyle = "#0b1222";
    ctx.font = "bold 11px system-ui";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(String(v), px(p), py(p));
  }
}

// --- state machine ---------------------------------------------------------
function computePositions(track) {
  const nodes = state.steps[track][state.pos].nodes;
  return layout(nodes);
}

function highlightsFor(track) {
  const nodes = state.steps[track][state.pos].nodes;
  const moved = new Set();
  if (state.pos > 0) {
    const prevNodes = state.steps[track][state.pos - 1].nodes;
    for (const v of nodes.keys()) {
      const prev = prevNodes.get(v);
      const cur = nodes.get(v);
      if (!prev) continue; // newly inserted, handled separately
      if (prev.left !== cur.left || prev.right !== cur.right) moved.add(v);
    }
  }
  return { root: layout(nodes).root, inserted: state.lastValue, moved };
}

function drawAll() {
  for (const track of ["bst", "avl"]) {
    const canvas = $(track === "bst" ? "canvasBst" : "canvasAvl");
    const nodes = state.steps[track][state.pos].nodes;
    const layoutNow = layout(nodes);
    // Lerp between the previous layout and the current one.
    const pos = new Map();
    const prev = state.prevPos[track];
    const t = state.animT;
    for (const [v, p] of layoutNow.pos) {
      const q = prev && prev.get(v);
      pos.set(v, q ? { x: q.x + (p.x - q.x) * t, y: q.y + (p.y - q.y) * t } : p);
    }
    drawTree(canvas, nodes, pos, highlightsFor(track));
  }
  updateInfo();
}

function advanceStep() {
  state.pos += 1;
  if (state.pos >= state.steps.bst.length) {
    state.pos = state.steps.bst.length - 1;
    stopPlayback();
    $("status").textContent = "Done — all values inserted";
  }
  state.animT = 0;
  const t = state.pos;
  state.prevPos.bst = layout(state.steps.bst[Math.min(t, state.steps.bst.length - 1)].nodes).pos;
  state.prevPos.avl = layout(state.steps.avl[Math.min(t, state.steps.avl.length - 1)].nodes).pos;
  state.lastValue = state.steps.bst[state.pos].value;
  drawAll();
}

function tick() {
  if (!state.playing) return;
  const duration = Number($("speed").value);
  state.animT = Math.min(1, state.animT + 16 / duration);
  if (state.animT >= 1) {
    // Snap to the next step's layout and advance.
    state.prevPos.bst = layout(state.steps.bst[state.pos].nodes).pos;
    state.prevPos.avl = layout(state.steps.avl[state.pos].nodes).pos;
    state.animT = 1;
    drawAll();
    if (state.playing) advanceStep();
  } else {
    drawAll();
  }
  if (state.playing) state.raf = requestAnimationFrame(tick);
}

function startPlayback() {
  if (state.pos >= state.steps.bst.length - 1) resetPlayback();
  state.playing = true;
  $("playPause").textContent = "Pause";
  if (state.raf == null) state.raf = requestAnimationFrame(tick);
}

function stopPlayback() {
  state.playing = false;
  cancelAnimationFrame(state.raf);
  state.raf = null;
  $("playPause").textContent = "Play";
}

function resetPlayback() {
  stopPlayback();
  state.pos = 0;
  state.animT = 1;
  state.lastValue = -1;
  state.prevPos.bst = null;
  state.prevPos.avl = null;
  $("status").textContent = "";
  drawAll();
}

// --- setup ----------------------------------------------------------------
function newTree() {
  const n = Number($("size").value);
  const sorted = Number($("seq").value);
  if (state.wasm.tree_init(n, sorted) !== 0) {
    $("status").textContent = "wasm error: tree_init failed";
    return;
  }
  const memory = state.wasm.memory.buffer;
  state.steps.bst = parseTrack(
    new Uint32Array(memory, state.wasm.tree_steps_ptr(0), state.wasm.tree_steps_len(0))
  );
  state.steps.avl = parseTrack(
    new Uint32Array(memory, state.wasm.tree_steps_ptr(1), state.wasm.tree_steps_len(1))
  );
  // Visibility of each panel per the view mode.
  const mode = $("mode").value;
  $("boxBst").style.display = mode === "avl" ? "none" : "";
  $("boxAvl").style.display = mode === "bst" ? "none" : "";
  resetPlayback();
}

function updateInfo() {
  $("stepInfo").textContent = `${state.pos} / ${state.steps.bst.length - 1}`;
  $("lastInsert").textContent = state.lastValue >= 0 ? String(state.lastValue) : "—";
  $("sizeLabel").textContent = $("size").value;
  $("speedLabel").textContent = $("speed").value;
  // Live height comparison — the money shot of this page.
  const bstH = treeHeight(state.steps.bst[state.pos].nodes);
  const avlH = treeHeight(state.steps.avl[state.pos].nodes);
  $("heightBst").textContent = `height ${bstH}`;
  $("heightAvl").textContent = `height ${avlH}`;
}

$("newTree").addEventListener("click", newTree);
$("playPause").addEventListener("click", () => (state.playing ? stopPlayback() : startPlayback()));
$("step").addEventListener("click", () => {
  stopPlayback();
  if (state.pos >= state.steps.bst.length - 1) resetPlayback();
  advanceStep();
});
$("reset").addEventListener("click", resetPlayback);
$("mode").addEventListener("change", () => {
  const mode = $("mode").value;
  $("boxBst").style.display = mode === "avl" ? "none" : "";
  $("boxAvl").style.display = mode === "bst" ? "none" : "";
  drawAll();
});
$("size").addEventListener("input", () => ($("sizeLabel").textContent = $("size").value));
$("speed").addEventListener("input", () => ($("speedLabel").textContent = $("speed").value));

loadWasm();
