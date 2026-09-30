# Mock server for the Misskey / Mastodon / Bluesky APIs. Prints every request it gets.
# python3 dev/mock.py, then: SOCIAL_POSTER_CONFIG=dev/config.toml cargo run -- "hello"
import json
from http.server import BaseHTTPRequestHandler, HTTPServer

RESPONSES = {
    "/api/notes/create": {"createdNote": {"id": "mock-note-1"}},
    "/api/v1/statuses": {"url": "http://localhost:8787/@me/1"},
    "/xrpc/com.atproto.server.createSession": {"did": "did:plc:mock", "accessJwt": "mock-jwt"},
    "/xrpc/com.atproto.repo.createRecord": {"uri": "at://did:plc:mock/app.bsky.feed.post/mockrkey"},
}

class H(BaseHTTPRequestHandler):
    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", 0)))
        print(f"POST {self.path}  auth={self.headers.get('Authorization')}\n  {body.decode()}", flush=True)
        res = RESPONSES.get(self.path)
        if self.path == "/api/v1/statuses" and self.headers.get("Authorization") != "Bearer ok":
            res = None  # token other than "ok" -> 401
        if json.loads(body or b'{}').get("i") == "broken":
            res = {"unexpected": True}  # misskey token "broken" -> 200 with a wrong shape
        self.send_response(200 if res else 401)
        self.send_header("Content-Type", "application/json")
        self.end_headers()
        self.wfile.write(json.dumps(res or {"error": "unauthorized"}).encode())

    def log_message(self, *a):
        pass

print("mock on http://localhost:8787", flush=True)
HTTPServer(("127.0.0.1", 8787), H).serve_forever()
