# `api/auth` worker

This Cloudflare Worker provides the following endpoints at `api/auth`:

- [signup](https://cloud.stencila.io/api/auth/signup)

- [signin](https://cloud.stencila.io/api/auth/signin)

- [signout](https://cloud.stencila.io/api/auth/signout)

- [connect?app=<app>&user=<user>](https://cloud.stencila.io/api/auth/connect) for connecting GitHub, Google etc to your Stencila account

These all redirect to https://accounts.stencila.io (hosted by our OAuth provider https://kinde.com). In addition, this worker has an OAuth `callback` endpoint to fetch and verify the user access token.

To develop and test locally, change the following line in `src/index.ts` to,

```ts
const OAUTH_REDIRECT_URL = "http://localhost:8787/api/auth/callback";
```

And set up to a `.dev.vars` file with,

```
OAUTH_CLIENT_SECRET=xxxxxxxxxxxxxx
```

Then run,

```sh
npm run dev
```

To deploy a new version,

```sh
npm run deploy
```
