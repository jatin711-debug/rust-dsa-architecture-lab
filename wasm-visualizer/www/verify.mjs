// Smoke test: instantiate the real .wasm in Node and verify the recorded
// steps replay to a sorted array — exactly what main.js does in the browser.
//
// Run: node verify.mjs

"use strict";

import { readFile } from "node:fs/promises";

const ALGOS = ["quicksort", "mergesort", "heapsort", "insertion"];

async function main() {
  const bytes = await readFile(new URL("wasm_visualizer.wasm", import.meta.url));
  const { instance } = await WebAssembly.instantiate(bytes, {});
  const wasm = instance.exports;
  const memory = () => wasm.memory.buffer;

  let failures = 0;

  for (const size of [1, 2, 10, 100, 400]) {
    for (let kind = 0; kind < ALGOS.length; kind++) {
      const rc = wasm.sort_new(size, kind);
      if (rc !== 0) {
        console.log(`FAIL sort_new(${size}, ${ALGOS[kind]}) -> ${rc}`);
        failures++;
        continue;
      }
      const len = wasm.sort_len();
      const initial = Array.from(new Int32Array(memory(), wasm.sort_initial_ptr(), len));
      const steps = Array.from(new Uint32Array(memory(), wasm.sort_steps_ptr(), wasm.sort_steps_len()));

      // Replay, mirroring main.js exactly.
      const display = initial.slice();
      let compares = 0, swaps = 0, writes = 0;
      for (const step of steps) {
        const phase = (step >>> 28) & 0xf;
        const a = (step >>> 14) & 0x3fff;
        const b = step & 0x3fff;
        if (a >= size || b >= size) {
          console.log(`FAIL ${ALGOS[kind]}(${size}): step index out of bounds a=${a} b=${b}`);
          failures++;
          break;
        }
        if (phase === 0) compares++;
        else if (phase === 1) { [display[a], display[b]] = [display[b], display[a]]; swaps++; }
        else if (phase === 2) { display[a] = b; writes++; }
        else {
          console.log(`FAIL ${ALGOS[kind]}(${size}): unknown phase ${phase}`);
          failures++;
          break;
        }
      }

      const sorted = display.every((v, i) => i === 0 || display[i - 1] <= v);
      const finalWasm = Array.from(new Int32Array(memory(), wasm.sort_final_ptr(), len));
      const matches = JSON.stringify(display) === JSON.stringify(finalWasm);

      if (!sorted || !matches) {
        console.log(`FAIL ${ALGOS[kind]}(${size}): sorted=${sorted} replay==wasm=${matches}`);
        failures++;
      } else {
        console.log(
          `ok   ${ALGOS[kind].padEnd(9)} n=${String(size).padStart(3)}  ` +
          `steps=${String(steps.length).padStart(6)} cmp=${compares} swp=${swaps} wrt=${writes}`
        );
      }
      wasm.sort_free();
    }
  }

  // Error paths.
  if (wasm.sort_new(0, 0) !== -1) { console.log("FAIL: n=0 accepted"); failures++; }
  if (wasm.sort_new(4001, 0) !== -1) { console.log("FAIL: n=4001 accepted"); failures++; }
  if (wasm.sort_new(10, 99) !== -1) { console.log("FAIL: bad kind accepted"); failures++; }
  if (wasm.sort_len() !== 0) { console.log("FAIL: stale recorder after errors"); failures++; }

  // --- tree exports: BST vs AVL on the same sequence -----------------------
  console.log("\n--- tree tracks ---");

  // Decode a track's flat stream into snapshots of { value -> {left, right} }.
  function parseTrack(steps32) {
    const out = [];
    let i = 0;
    while (i < steps32.length) {
      const header = steps32[i];
      const count = (header >>> 22) & 0x3f; // kind is in the top 4 bits
      const nodes = new Map();
      for (let j = 0; j < count; j++) {
        const rec = steps32[i + 1 + j];
        const v = (rec >>> 20) & 0x3ff;
        const l = ((rec >>> 10) & 0x3ff) - 1;
        const r = (rec & 0x3ff) - 1;
        nodes.set(v, { left: l >= 0 ? l : null, right: r >= 0 ? r : null });
      }
      out.push(nodes);
      i += 1 + count;
    }
    return out;
  }

  function rootOf(nodes) {
    const children = new Set();
    nodes.forEach(({ left, right }) => {
      if (left != null) children.add(left);
      if (right != null) children.add(right);
    });
    return [...nodes.keys()].find((v) => !children.has(v));
  }

  function heightOf(nodes) {
    let max = 0;
    (function walk(v, depth) {
      if (v == null) return;
      max = Math.max(max, depth + 1);
      const n = nodes.get(v);
      walk(n.left, depth + 1);
      walk(n.right, depth + 1);
    })(rootOf(nodes), 0);
    return max;
  }

  function inOrderOf(nodes) {
    const out = [];
    (function walk(v) {
      if (v == null) return;
      const n = nodes.get(v);
      walk(n.left);
      out.push(v);
      walk(n.right);
    })(rootOf(nodes));
    return out;
  }

  for (const [label, n, sorted] of [
    ["sorted n=25", 25, 1],
    ["random n=20", 20, 0],
    ["sorted n=40", 40, 1],
  ]) {
    if (wasm.tree_init(n, sorted) !== 0) {
      console.log(`FAIL tree_init(${n}, ${sorted})`);
      failures++;
      continue;
    }
    const bst = parseTrack(new Uint32Array(memory(), wasm.tree_steps_ptr(0), wasm.tree_steps_len(0)));
    const avl = parseTrack(new Uint32Array(memory(), wasm.tree_steps_ptr(1), wasm.tree_steps_len(1)));

    // Every snapshot must be a valid sorted tree (rotations included).
    let snapshotsOk = bst.length === n && avl.length === n;
    for (const track of [bst, avl]) {
      for (let s = 0; s < track.length && snapshotsOk; s++) {
        const order = inOrderOf(track[s]);
        const sortedOk = order.every((v, i) => i === 0 || order[i - 1] <= v);
        if (!sortedOk) snapshotsOk = false;
      }
    }

    const bstH = heightOf(bst[n - 1]);
    const avlH = heightOf(avl[n - 1]);
    const bound = Math.ceil(1.45 * Math.log2(n)) + 1; // AVL height guarantee
    const sameValues =
      JSON.stringify(inOrderOf(bst[n - 1])) === JSON.stringify(inOrderOf(avl[n - 1]));

    if (!snapshotsOk || !sameValues || avlH > bound) {
      console.log(`FAIL ${label}: snapshotsOk=${snapshotsOk} sameValues=${sameValues} bstH=${bstH} avlH=${avlH} (bound ${bound})`);
      failures++;
    } else {
      console.log(
        `ok   ${label.padEnd(12)} snapshots=${bst.length}  bstH=${String(bstH).padStart(2)}  avlH=${String(avlH).padStart(2)} (<= ${bound})`
      );
    }
    wasm.tree_free();
  }

  console.log(failures === 0 ? "\nALL SMOKE TESTS PASSED" : `\n${failures} FAILURES`);
  process.exit(failures === 0 ? 0 : 1);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
