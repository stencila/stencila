---
title: Web Tools
description: Tools for fetching web pages and saving content locally, and the guard rules that protect against SSRF and credential exposure.
---

The web tool allows agents to fetch web pages and save content locally for further reading and analysis. Each URL is evaluated by the [web guard](#guard-rules) before the request is made.

## `web_fetch`

Fetches a URL, saves the content to `.stencila/cache/web/`, and converts HTML to Markdown with images extracted to a media directory.

| Parameter | Type | Required | Description |
| --------- | ---- | :------: | ----------- |
| `url` | string | ✅ | The URL to fetch |
| `raw` | boolean | | If true, save the response body as-is without conversion. Defaults to false |

### How it Works

1. **URL validation** — the URL must use the `http://` or `https://` scheme.

2. **HTTP caching** — responses are cached locally with full RFC 7234 compliance. Subsequent fetches of the same URL use conditional requests (`If-Modified-Since`, `If-None-Match`) and honor `304 Not Modified` responses.

3. **Content processing** — HTML pages are parsed and converted to Markdown. Images referenced in the page are downloaded in parallel (up to 8 concurrent, with retries) and saved alongside the Markdown file in a `media/` subdirectory. Image references in the Markdown are rewritten to point to the local copies.

4. **Output** — the tool returns a manifest listing the saved files with sizes and line counts, along with instructions to use `read_file`, `grep`, or `glob` to explore the content.

Responses are limited to 10 MB with a 30-second request timeout.

## Guard Rules

The web guard parses each URL, normalizes the host (ASCII case-fold, trailing dot strip) and path (consecutive slash collapse), then evaluates rules in most-specific-first order. Evaluation short-circuits on the first non-Allow verdict.

| Rule ID | Reason | Suggestion | Low | Medium | High |
| ------- | ------ | ---------- | :-: | :----: | :-: |
| `web.credential_url` | Metadata credential paths return IAM tokens and secrets that can be used for privilege escalation | Use the cloud provider's CLI for credential management | Deny | Deny | Deny |
| `web.metadata_endpoint` | Cloud metadata endpoints expose instance credentials and configuration | Access cloud credentials through the provider's CLI or SDK instead | Deny | Deny | Deny |
| `web.internal_network` | Fetching internal network addresses can expose services not meant for external access (SSRF) | Use a public URL, or access internal services through an appropriate API | Deny | Deny | Deny |
| `web.non_https` | Unencrypted HTTP requests can expose data in transit | Use `https://` instead of `http://` | Deny | Warn | Allow |
| `web.high_risk_port` | Port is associated with an infrastructure service not typically accessed via HTTP | Use the service's dedicated CLI or client library instead of HTTP | Deny | Warn | Allow |
| `web.domain_allowlist` | Domain is not in the agent's allowed domain list | Add the domain to `allowedDomains` in the agent definition, or use an allowed domain | Deny | Deny | Deny |
| `web.domain_denylist` | Domain is in the agent's disallowed domain list | Remove the domain from `disallowedDomains` if access is intended, or use a different source | Deny | Deny | Deny |
| `web.parse_failure` | URL could not be parsed | Provide a valid URL (e.g., `https://example.com/path`) | Deny | Deny | Deny |

### Metadata Hosts

Requests to these hosts trigger `web.metadata_endpoint` (or `web.credential_url` if the path also matches a credential prefix):

- `169.254.169.254` — AWS, Azure, most cloud providers
- `fd00:ec2::254` — AWS IMDSv2 IPv6 endpoint
- `metadata.google.internal` — GCP
- `100.100.100.200` — Alibaba Cloud

### Credential Path Prefixes

These URL path prefixes (on metadata hosts) trigger `web.credential_url`:

- `/latest/meta-data/iam/security-credentials` (AWS IMDSv1/v2)
- `/latest/api/token` (AWS IMDSv2)
- `/computeMetadata/v1/instance/service-accounts` (GCP)
- `/metadata/identity/oauth2/token` (Azure)
- `/latest/meta-data/ram/security-credentials` (Alibaba Cloud)

### Internal Network Detection

The following are considered internal network addresses and trigger `web.internal_network`:

- `localhost`
- `*.local`, `*.internal` hostname suffixes
- Loopback IPs: `127.0.0.0/8`, `::1`
- Private IPv4: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`
- Link-local: `169.254.0.0/16`, `fe80::/10`
- Shared address space: `100.64.0.0/10`
- IPv4-mapped IPv6: `::ffff:0:0/96` (when the mapped address is private)

### High-Risk Ports

These ports trigger `web.high_risk_port`:

| Port | Service |
| ---- | ------- |
| 22 | SSH |
| 23 | Telnet |
| 25 | SMTP |
| 135 | MS RPC |
| 139 | NetBIOS |
| 445 | SMB |
| 2375 | Docker daemon (unencrypted) |
| 2376 | Docker daemon (TLS) |
| 3306 | MySQL |
| 5432 | PostgreSQL |
| 5900 | VNC |
| 6379 | Redis |
| 6443 | Kubernetes API |
| 8200 | Vault |
| 8500 | Consul |
| 9200 | Elasticsearch |
| 27017 | MongoDB |
