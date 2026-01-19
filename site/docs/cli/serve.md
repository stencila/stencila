---
title: "`stencila serve`"
description: Run the HTTP/Websocket server
---

Run the HTTP/Websocket server

# Usage

```sh
stencila serve [OPTIONS] [DIR]
```

# Arguments

| Name    | Description                                 |
| ------- | ------------------------------------------- |
| `[DIR]` | The directory to serve. Default value: `.`. |

# Options

| Name                      | Description                                                                                                                                                                                                                                         |
| ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `-a, --address <ADDRESS>` | The address to serve on. Default value: `127.0.0.1`.                                                                                                                                                                                                |
| `-p, --port <PORT>`       | The port to serve on. Default value: `9000`.                                                                                                                                                                                                        |
| `--no-auth <NO_AUTH>`     | Do not authenticate or authorize requests. Possible values: `true`, `false`.                                                                                                                                                                        |
| `--raw <RAW>`             | Should files be served raw? Possible values: `true`, `false`.                                                                                                                                                                                       |
| `--source <SOURCE>`       | Should `SourceMap` headers be sent? Possible values: `true`, `false`.                                                                                                                                                                               |
| `--sync <SYNC>`           | Whether and in which direction(s) to sync served documents. Possible values: `in`, `out`, `in-out`.                                                                                                                                                 |
| `--cors <CORS>`           | CORS policy level. Possible values: `none` (No CORS headers), `restrictive` (Allow only same-origin requests), `local` (Allow localhost and 127.0.0.1 origins only), `permissive` (Allow all origins, methods, and headers). Default value: `none`. |
