// main.js — the browser side of the Rust + WASM sort visualizer.
//
// Talks to the wasm module through its raw exports and linear memory:
//   - sort_new(n, kind)    starts a recording (Rust runs the algorithm)
//   - sort_initial_ptr()   -> Int32Array view of the pre-sort array
//   - sort_steps_ptr/len   -> Uint32Array of packed steps
//   - sort_len()           -> element count
//
// Step encoding (see wasm-visualizer/src/lib.rs):
//   step = (phase << 28) | (a << 14) | b
//   phase 0 = compare(a,b), 1 = swap(a,b), 2 = write value b at index a

"use strict";

const PHASE = { COMPARE: 0, SWAP: 1, WRITE: 2 };
const COLORS = {
  COMPARE: "#3b82f6",
  SWAP: "#f59e0b",
  WRITE: "#22c55e",
  BAR: "#64748b",
  SORTED: "#4ade80",
};

const canvas = document.getElementById("canvas");
const ctx = canvas.getContext("2d");
const W = canvas.width;
const H = canvas.height;

// --- state ---------------------------------------------------------------
const state = {
  wasm: null,
  initial: [], // snapshot from wasm memory (replay base)
  display: [], // what we draw (mutated by replay)
  steps: [], // Uint32Array from wasm memory
  pos: 0,
  playing: false,
  timer: null,
  lastPhase: null,
  lastA: -1,
  lastB: -1,
  counts: { compare: 0, swap: 0, write: 0 },
};

const $ = (id) => document.getElementById(id);

// --- wasm loading ---------------------------------------------------------
async function loadWasm() {
  let module;
  try {
    const response = await fetch("wasm_visualizer.wasm");
    module = await WebAssembly.instantiateStreaming(response, {});
  } catch {
    // Fallback for servers that don't send application/wasm.
    const response = await fetch("wasm_visualizer.wasm");
    const bytes = await response.arrayBuffer();
    module = await WebAssembly.instantiate(bytes, {});
  }
  state.wasm = module.instance.exports;
  newArray();
}

// --- array management -----------------------------------------------------
function newArray() {
  const n = Number($("size").value);
  const kind = Number($("algo").value);
  if (state.wasm.sort_new(n, kind) !== 0) {
    $("status").textContent = "wasm error: sort_new failed";
    return;
  }
  const memory = state.wasm.memory.buffer;
  const len = state.wasm.sort_len();
  const initial = new Int32Array(memory, state.wasm.sort_initial_ptr(), len);
  state.initial = Array.from(initial);
  state.display = state.initial.slice();
  state.steps = new Uint32Array(memory, state.wasm.sort_steps_ptr(), state.wasm.sort_steps_len());
  state.pos = 0;
  state.lastPhase = null;
  state.lastA = -1;
  state.lastB = -1;
  state.counts = { compare: 0, swap: 0, write: 0 };
  stopPlayback();
  $("status").textContent = "";
  draw();
  updateInfo();
}

// --- playback -------------------------------------------------------------
function unpack(step) {
  return {
    phase: (step >>> 28) & 0xf,
    a: (step >>> 14) & 0x3fff,
    b: step & 0x3fff,
  };
}

function stepForward() {
  if (state.pos >= state.steps.length) {
    stopPlayback();
    $("status").textContent = "Sorted ✓";
    draw();
    return;
  }
  const { phase, a, b } = unpack(state.steps[state.pos]);
  state.pos += 1;

  if (phase === PHASE.SWAP) {
    [state.display[a], state.display[b]] = [state.display[b], state.display[a]];
    state.counts.swap += 1;
  } else if (phase === PHASE.WRITE) {
    state.display[a] = b; // value packed into the b field
    state.counts.write += 1;
  } else {
    state.counts.compare += 1;
  }
  state.lastPhase = phase;
  state.lastA = a;
  state.lastB = b;
  draw();
  updateInfo();
  if (state.pos >= state.steps.length) {
    stopPlayback();
    $("status").textContent = "Sorted ✓";
    draw();
  }
}

function startPlayback() {
  if (state.pos >= state.steps.length) {
    state.pos = 0;
    state.display = state.initial.slice();
    state.counts = { compare: 0, swap: 0, write: 0 };
  }
  state.playing = true;
  $("playPause").textContent = "Pause";
  scheduleStep();
}

function scheduleStep() {
  const ms = 1000 / Number($("speed").value);
  state.timer = setTimeout(() => {
    if (!state.playing) return;
    stepForward();
    if (state.playing) scheduleStep();
  }, ms);
}

function stopPlayback() {
  state.playing = false;
  clearTimeout(state.timer);
  $("playPause").textContent = "Play";
}

// --- drawing --------------------------------------------------------------
function draw() {
  const n = state.display.length;
  const barW = W / n;
  ctx.clearRect(0, 0, W, H);

  // Sorted region: everything from the last heap "extract" boundary would be
  // algorithm-specific; simply tint bars that are already in final position
  // via a cheap heuristic: monotonic suffix of the display array.
  let sortedFrom = n;
  for (let i = n - 1; i >= 0; i--) {
    if (i === 0 || state.display[i - 1] <= state.display[i]) {
      sortedFrom = i;
    } else {
      break;
    }
  }

  const max = Math.max(...state.display, 1);
  for (let i = 0; i < n; i++) {
    const h = 8 + ((state.display[i] / max) * (H - 40));
    const x = i * barW + (barW > 3 ? barW * 0.15 : 0);
    const w = barW > 3 ? barW * 0.7 : 1;

    let color = COLORS.BAR;
    if (i >= sortedFrom && sortedFrom > 0) color = COLORS.SORTED;
    if (state.lastPhase === PHASE.COMPARE && (i === state.lastA || i === state.lastB)) {
      color = COLORS.COMPARE;
    }
    if (state.lastPhase === PHASE.SWAP && (i === state.lastA || i === state.lastB)) {
      color = COLORS.SWAP;
    }
    if (state.lastPhase === PHASE.WRITE && i === state.lastA) {
      color = COLORS.WRITE;
    }
    ctx.fillStyle = color;
    ctx.fillRect(x, H - h, w, h);
  }
}

function updateInfo() {
  $("stepInfo").textContent = `${state.pos} / ${state.steps.length}`;
  $("cmpCount").textContent = state.counts.compare;
  $("swpCount").textContent = state.counts.swap;
  $("wrtCount").textContent = state.counts.write;
  $("sizeLabel").textContent = $("size").value;
  $("speedLabel").textContent = $("speed").value;
}

// --- events ---------------------------------------------------------------
$("newArray").addEventListener("click", newArray);
$("playPause").addEventListener("click", () => (state.playing ? stopPlayback() : startPlayback()));
$("step").addEventListener("click", () => {
  stopPlayback();
  if (state.pos >= state.steps.length) {
    state.pos = 0;
    state.display = state.initial.slice();
    state.counts = { compare: 0, swap: 0, write: 0 };
  }
  stepForward();
});
$("reset").addEventListener("click", () => {
  stopPlayback();
  state.pos = 0;
  state.display = state.initial.slice();
  state.counts = { compare: 0, swap: 0, write: 0 };
  state.lastPhase = null;
  $("status").textContent = "";
  draw();
  updateInfo();
});
$("size").addEventListener("input", () => {
  $("sizeLabel").textContent = $("size").value;
});
$("speed").addEventListener("input", () => {
  $("speedLabel").textContent = $("speed").value;
});

loadWasm();
