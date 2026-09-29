// bindgen.js — the wasm-bindgen surface.
//
// The import below is the generated glue (wasm-visualizer/www/pkg/). It
// handles instantiation (including the `wbg` import module) and marshals
// every value across the boundary:
//   - `new SortRun(kind, n)`  -> Rust constructor, Result -> JS Error
//   - `.steps()` / `.initial()` -> Uint32Array / Int32Array (copies)
//   - `.algorithm_name()`     -> JS string
//   - `.replay_stats()`       -> Uint32Array, replay runs inside Rust

import init, { SortRun } from "./pkg/wasm_visualizer.js";

const $ = (id) => document.getElementById(id);
const ALGO_NAMES = ["Quicksort", "Mergesort", "Heapsort", "Insertion sort"];

const state = {
  runs: [], // SortRun instances for the stats table
  run: null, // the currently selected run for the canvas
  steps: [],
  display: [],
  pos: 0,
  playing: false,
  timer: null,
  lastPhase: null,
  lastA: -1,
  lastB: -1,
};

// --- stats table -----------------------------------------------------------
async function runAll() {
  const tbody = $("stats").querySelector("tbody");
  tbody.innerHTML = "";
  state.runs = [];

  for (let kind = 0; kind < 4; kind++) {
    const run = new SortRun(kind, 100);
    state.runs.push(run);
    const stats = run.replay_stats(); // [compares, swaps, writes]
    const row = document.createElement("tr");
    row.innerHTML = [
      `<td><b>${run.algorithm_name()}</b></td>`,
      `<td>${run.len()}</td>`,
      `<td>${run.step_count()}</td>`,
      `<td>${stats[0]}</td>`,
      `<td>${stats[1]}</td>`,
      `<td>${stats[2]}</td>`,
    ].join("");
    tbody.appendChild(row);
    run.free();
  }
  loadRun(Number($("algo").value));
}

// --- canvas replay (driven by bindgen-returned arrays) ---------------------
function loadRun(kind) {
  const run = new SortRun(kind, 100);
  state.run = run;
  state.steps = Array.from(run.steps()); // Uint32Array -> plain array
  state.display = Array.from(run.initial()); // Int32Array -> plain array
  state.pos = 0;
  state.lastPhase = null;
  stopPlayback();
  draw();
  $("runInfo").textContent = `${run.algorithm_name()} — ${run.len()} elements, ${state.steps.length} steps`;
}

function unpack(step) {
  return { phase: (step >>> 28) & 0xf, a: (step >>> 14) & 0x3fff, b: step & 0x3fff };
}

function stepForward() {
  if (state.pos >= state.steps.length) {
    stopPlayback();
    draw();
    return;
  }
  const { phase, a, b } = unpack(state.steps[state.pos]);
  state.pos += 1;
  if (phase === 1) {
    [state.display[a], state.display[b]] = [state.display[b], state.display[a]];
  } else if (phase === 2) {
    state.display[a] = b;
  }
  state.lastPhase = phase;
  state.lastA = a;
  state.lastB = b;
  draw();
}

function startPlayback() {
  if (state.pos >= state.steps.length) {
    state.pos = 0;
    state.display = Array.from(state.run.initial());
  }
  state.playing = true;
  $("play").textContent = "Pause";
  schedule();
}

function schedule() {
  state.timer = setTimeout(() => {
    if (!state.playing) return;
    stepForward();
    if (state.playing) schedule();
  }, 1000 / 120);
}

function stopPlayback() {
  state.playing = false;
  clearTimeout(state.timer);
  $("play").textContent = "Play";
}

function draw() {
  const canvas = $("canvas");
  const ctx = canvas.getContext("2d");
  const W = canvas.width;
  const H = canvas.height;
  ctx.clearRect(0, 0, W, H);
  const n = state.display.length;
  if (n === 0) return;
  const barW = W / n;
  const max = Math.max(...state.display, 1);
  const colors = { 0: "#3b82f6", 1: "#f59e0b", 2: "#22c55e" };
  for (let i = 0; i < n; i++) {
    const h = 8 + (state.display[i] / max) * (H - 30);
    ctx.fillStyle =
      state.lastPhase != null && (i === state.lastA || i === state.lastB)
        ? colors[state.lastPhase]
        : "#64748b";
    ctx.fillRect(i * barW + (barW > 3 ? barW * 0.15 : 0), H - h, barW > 3 ? barW * 0.7 : 1, h);
  }
}

// --- events ----------------------------------------------------------------
$("runAll").addEventListener("click", runAll);
$("algo").addEventListener("change", () => {
  if (state.run) state.run.free();
  loadRun(Number($("algo").value));
});
$("play").addEventListener("click", () => (state.playing ? stopPlayback() : startPlayback()));
$("step").addEventListener("click", () => {
  stopPlayback();
  stepForward();
});
$("reset").addEventListener("click", () => {
  stopPlayback();
  state.pos = 0;
  state.display = Array.from(state.run.initial());
  state.lastPhase = null;
  draw();
});

await init();
runAll();
