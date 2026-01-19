---
title: "`stencila site domain set`"
description: Set a custom domain for the site
---

Set a custom domain for the site

# Usage

```sh
stencila site domain set [OPTIONS] <DOMAIN>
```

# Examples

```bash
# Set custom domain for the current workspace's site
stencila site domain set example.com

# Set custom domain for another workspace's site
stencila site domain set example.com --path /path/to/workspace
```

# Arguments

| Name       | Description                            |
| ---------- | -------------------------------------- |
| `<DOMAIN>` | The custom domain to use for the site. |

# Options

| Name         | Description                                                     |
| ------------ | --------------------------------------------------------------- |
| `-p, --path` | Path to the workspace directory containing .stencila/site.yaml. |

# Setup Process

After running this command, you'll need to complete the following steps:

1. Add the CNAME record to your DNS
   The command will provide the exact record details (name and target)

2. Wait for DNS propagation (usually 5-30 minutes)
   DNS changes can take time to propagate globally

3. Check status: stencila site domain status
   Monitor the verification and SSL provisioning progress

Once the CNAME is detected, SSL will be provisioned automatically and
your site will go live.

# Troubleshooting

Domain status stuck on "Waiting for CNAME record to be configured":

1. Verify CNAME is configured correctly:
   dig example.com CNAME
   nslookup -type=CNAME example.com
   Should show your domain pointing to the CNAME target provided

2. Cloudflare DNS users:
   - Ensure CNAME is set to "DNS only" (gray cloud), NOT "Proxied" (orange cloud)
   - Proxied mode prevents domain verification and SSL provisioning
   - This setting must remain "DNS only" permanently, not just during setup

3. Check for conflicting DNS records:
   - Remove any A or AAAA records for the same hostname
   - Ensure no NS records delegating to a different DNS provider

4. Wait for DNS propagation:
   - DNS changes typically take 5-30 minutes (sometimes up to 48 hours)
   - Check propagation: https://dnschecker.org

5. Apex domain issues:
   - Some DNS providers don't support CNAME on apex/root domains
   - Consider using a subdomain (e.g., www.example.com) instead
