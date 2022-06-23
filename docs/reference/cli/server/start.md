<!-- Generated from doc comments in Rust. Do not edit. -->

# `start`: Start the server

## Usage

```sh
stencila server start [options] [home]
```

## Ports and addresses

Use the <url> argument to change the port and/or address that the server
listens on. This argument can be a partial, or complete, URL.

For example, to serve on port 8000 instead of the default port,

```sh
$ stencila server start :8000
```

To serve on all IPv4 addresses on the machine, instead of only `127.0.0.1`,

```sh
$ stencila server start 0.0.0.0
```

Or if you prefer, use a complete URL including the scheme e.g.

```sh
$ stencila server start http://127.0.0.1:9000
```

## Security

By default, the server requires authentication using JSON Web Token. A token is
printed as part of the server's URL at startup. To turn authorization off, for example
if you are using some other authentication layer in front of the server, use the `--insecure`
flag.

By default, this command will NOT run as a root (Linux/Mac OS/Unix) or administrator (Windows) user.
Use the `--root` option, with extreme caution, to allow to be run as root.

Most of these options can be set in the Stencila configuration file. See `stencila config get serve`

## Arguments

| Name   | Description                                     |
| ------ | ----------------------------------------------- |
| `home` | The home directory for the server to serve from |

## Options

| Name                                | Description                                                                                                                                                                               |
| ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--url -u <url>`                    | The URL to serve on. Defaults to the `STENCILA_SERVER_URL` environment variable, the value set in config or otherwise `http://127.0.0.1:9000`.                                            |
| `--key -k <key>`                    | Secret key to use for signing and verifying JSON Web Tokens. Defaults to the `STENCILA_SERVER_KEY` environment variable, the value set in config or otherwise a randomly generated value. |
| `--insecure`                        | Do not require a JSON Web Token to access the server. For security reasons (any client can access files and execute code) this should be avoided.                                         |
| `--traversal`                       | Allow traversal out of the server's home directory. For security reasons (clients can access any file on the filesystem) this should be avoided.                                          |
| `--root`                            | Allow root (Linux/Mac OS/Unix) or administrator (Windows) user to serve. For security reasons (clients may be able to execute code as root) this should be avoided.                       |
| `--max-inactivity <max-inactivity>` | The maximum number of seconds of inactivity before the server shutsdown.                                                                                                                  |
| `--max-duration <max-duration>`     | The maximum number of seconds that the server should run for.                                                                                                                             |
| `--log-requests`                    | Log each request.                                                                                                                                                                         |

## Global options

| Name                        | Description                                                                                                                                          |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--help`                    | Print help information.                                                                                                                              |
| `--version`                 | Print version information.                                                                                                                           |
| `--as <format>`             | Format to display output values (if possible).                                                                                                       |
| `--json`                    | Display output values as JSON (alias for `--as json`).                                                                                               |
| `--yaml`                    | Display output values as YAML (alias for `--as yaml`).                                                                                               |
| `--md`                      | Display output values as Markdown if possible (alias for `--as md`).                                                                                 |
| `--interact -i`             | Enter interactive mode (with any command and options as the prefix).                                                                                 |
| `--debug`                   | Print debug level log events and additional diagnostics. Equivalent to setting `--log-level=debug` and `--log-format=detail` and overrides the both. |
| `--log-level <log-level>`   | The minimum log level to print. One of: `trace`, `debug`, `info`, `warn`, `error`, `never`                                                           |
| `--log-format <log-format>` | The format to print log events. One of: `simple`, `detail`, `json`                                                                                   |
