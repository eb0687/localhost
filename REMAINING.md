1. Body Size Limits (DONE)

- Use client_max_body_size.
- Reject oversized Content-Length and chunked bodies.
- Add 413 Payload Too Large.

2. Custom Error Pages (DONE)

- Use each selected virtual server’s error_pages.
- Serve configured pages for 400, 403, 404, 405, 413, 500.
- Keep fallback default generated HTML pages.

3. HTTP Method Behavior

- Make configured allowed_methods work cleanly.
- Add correct Allow header for 405.
- Fix file handlers that still internally assume only GET.

4. Uploads

- Implement POST upload handling.
- Save request body to configured upload location.
- Verify downloaded file is not corrupted.

5. DELETE

- Implement DELETE for configured file routes.
- Return correct status codes: 204, 403, 404, 405, etc.

6. CGI

- Execute one CGI type.
- Use fork/exec.
- Pass body through stdin.
- Set CGI env vars, especially PATH_INFO.
- Support chunked and unchunked request bodies.

7. Config Completeness

- Improve config names to match README more closely if desired:
- server_address instead of host
- route methods
- route default_file
- CGI by extension
- Validate bad config cases more thoroughly.

8. Static/Browsing Polish

- Serve a real static website.
- Verify CSS/JS/images.
- Improve directory/index behavior.

9. Tests

- Add unit tests for config validation and host routing.
- Add integration tests for GET/POST/DELETE, upload, redirects, error pages, CGI.
- Add malformed request tests.

10. Audit Readiness
    Run siege -b 127.0.0.1:8080.
    Check hanging connections.
    Check memory/fd leaks.
    Document test commands and architecture.
