Fixtures for testing `LexicalCodec`:

- `playground.lexical`: default content from https://playground.lexical.dev/ at 2025-01-07

- `ghost.koenig`: our Ghost test fixture at https://stencila.ghost.io/test-fixture/ which can be accessed (if a Stencila staff member) at https://stencila.ghost.io/ghost/api/admin/pages/677c6545db8aea00014b33e8/ in the browser or in the terminal e.g.

```sh
xh --session session.txt https://stencila.ghost.io/ghost/api/admin/pages/677c6545db8aea00014b3
3e8/ | jq '.pages[0].lexical | fromjson'
```

The [Koenig](https://koenig.ghost.org) (Ghost's editor) site is another place that is useful for generating JSON fixtures if more are needed.

