```terminal
$ mitmproxy
```

```terminal
$ SSL_CERT_FILE=~/.mitmproxy/mitmproxy-ca.pem HTTPS_PROXY=localhost:8080 cargo run -- submit --key abcd1234 https://example.com
```
