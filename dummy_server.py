#!/usr/bin/env python

import SimpleHTTPServer
import SocketServer
import logging

PORT = 8000

class GetHandler(SimpleHTTPServer.SimpleHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-type','text/html')
        self.end_headers()
        self.wfile.write("Hello World ! '{}'".format(self.path))
        return

for i in range(4):
    Handler = GetHandler
    httpd = SocketServer.TCPServer(("", PORT + i), Handler)
    httpd.serve_forever()