// A tiny dependency-free static file server for the visualizer.
// Run: node server.js  (serves http://localhost:8000)
//
// The `application/wasm` MIME type matters: WebAssembly.instantiateStreaming
// requires it (main.js falls back gracefully if missing).

"use strict";

const http = require("http");
const fs = require("fs");
const path = require("path");

const ROOT = __dirname;
const PORT = process.env.PORT || 8000;

const MIME = {
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".wasm": "application/wasm",
  ".css": "text/css",
  ".png": "image/png",
  ".svg": "image/svg+xml",
};

http
  .createServer((req, res) => {
    let urlPath = decodeURIComponent(req.url.split("?")[0]);
    if (urlPath === "/") urlPath = "/index.html";
    const file = path.normalize(path.join(ROOT, urlPath));
    if (!file.startsWith(ROOT)) {
      res.writeHead(403);
      res.end("forbidden");
      return;
    }
    fs.readFile(file, (err, data) => {
      if (err) {
        res.writeHead(404);
        res.end("not found");
        return;
      }
      res.writeHead(200, {
        "Content-Type": MIME[path.extname(file)] || "application/octet-stream",
        // Lets wasm use SharedArrayBuffer in the future (atomics).
        "Cross-Origin-Opener-Policy": "same-origin",
        "Cross-Origin-Embedder-Policy": "require-corp",
      });
      res.end(data);
    });
  })
  .listen(PORT, () => {
    console.log(`Sort visualizer running at http://localhost:${PORT}`);
  });
