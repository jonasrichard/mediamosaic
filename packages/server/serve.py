#!/usr/bin/env python3

import http.server
import socketserver


class MyHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self) -> None:
        self.send_my_headers()
        http.server.SimpleHTTPRequestHandler.end_headers(self)

    def send_my_headers(self):
        self.send_header("Cache-Control", "no-cache, no-store, must-revalidate")
        self.send_header("Pragma", "no-cache")
        self.send_header("Expires", "0")


if __name__ == "__main__":
    with socketserver.TCPServer(("", 3000), MyHandler) as httpd:
        httpd.serve_forever()
