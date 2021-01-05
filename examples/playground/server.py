#!/usr/bin/env python3

# We need a HTTP server for serving the JS/WASM files. Apparently browsers will
# refuse to accept the WASM if it simply comes from the local file system.
# I think that for Python versions 3.9+, the default http.server no longer
# deduces MIME types from file extensions. Thus, we need this slightly modified
# server.

# The code was adapted from:
# https://gist.github.com/HaiyangXu/ec88cbdce3cdbac7b8d5

import sys
import os
import http.server
import socketserver

os.chdir(sys.argv[1])

PORT = 8080

class HttpRequestHandler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {
        '': 'application/octet-stream',
        '.html': 'text/html',
        '.png': 'image/png',
        '.css':	'text/css',
        '.js':'application/x-javascript',
        '.wasm': 'application/wasm',
    }

httpd = socketserver.TCPServer(("127.0.0.1", PORT), HttpRequestHandler)

try:
    print(f"Serving on http://127.0.0.1:{PORT}")
    httpd.serve_forever()
except KeyboardInterrupt:
    pass