"""Static server for web/dist with an SPA fallback.

    usage:  python3 tools/serve-dist.py web/dist 8900
            then open http://127.0.0.1:8900

Build dist first (needs Node 22 — see web/.nvmrc):
    export PATH="$HOME/.cargo/bin:$PATH"
    wasm-pack build crates/frees --release --target web --out-dir ../../web/src/wasm/pkg
    cd web && npm ci && npm run build


`python3 -m http.server` 404s on client-side routes like /help (a prior agent
hit exactly that), so anything without a file extension falls back to
index.html. .wasm is served as application/wasm so streaming instantiate works.
"""
import http.server
import os
import sys

root = sys.argv[1]
port = int(sys.argv[2])


class H(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        **http.server.SimpleHTTPRequestHandler.extensions_map,
        ".wasm": "application/wasm",
        ".js": "text/javascript",
        ".mjs": "text/javascript",
    }

    def __init__(self, *a, **k):
        super().__init__(*a, directory=root, **k)

    def do_GET(self):
        # `translate_path` already confines the result to `directory` (it
        # normalises away "..", drops absolute prefixes and rejects drive
        # letters), so the resolved path cannot escape the served root. The
        # containment check below is belt-and-braces, and makes that guarantee
        # explicit rather than inherited.
        resolved = os.path.realpath(self.translate_path(self.path))
        base = os.path.realpath(root)
        if os.path.commonpath((base, resolved)) != base:
            self.send_error(403, "Forbidden")
            return None
        if not os.path.exists(resolved) and "." not in os.path.basename(
            self.path.split("?")[0]
        ):
            self.path = "/index.html"  # SPA fallback for extensionless routes
        return super().do_GET()

    def log_message(self, fmt, *a):
        sys.stderr.write("%s\n" % (fmt % a))


http.server.ThreadingHTTPServer.allow_reuse_address = True
# Loopback-only, plain HTTP by design: this serves a local build for a
# browser test and never leaves the machine (the bind below is 127.0.0.1).
sys.stderr.write("serving %s on http://127.0.0.1:%d\n" % (root, port))  # NOSONAR
sys.stderr.flush()
http.server.ThreadingHTTPServer(("127.0.0.1", port), H).serve_forever()
