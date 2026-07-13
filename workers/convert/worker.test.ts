import { describe, expect, it, vi } from "vitest";

import type { Env, RateLimitRecord } from "./worker";

vi.mock("cloudflare:workers", () => ({
  DurableObject: class {},
  WorkerEntrypoint: class {},
}));

const { evaluateRateLimit, handleRequest, rateLimitClient, startBackend } =
  await import("./worker");

function fakeEnv(options: {
  assetResponse?: Response;
  rateLimitResponse?: Response;
  rateLimiterFetch?: (
    input: RequestInfo | URL,
    init?: RequestInit,
  ) => Promise<Response>;
  rateLimiterIdFromName?: (name: string) => DurableObjectId;
}): Env {
  const rateLimitResponse =
    options.rateLimitResponse ??
    new Response(
      JSON.stringify({
        ok: true,
        minuteCount: 1,
        dayCount: 1,
        retryAfter: 0,
        message: "Allowed",
      }),
    );

  return {
    ASSETS: {
      fetch: async () => options.assetResponse ?? new Response("asset"),
    } as unknown as Fetcher,
    CONVERT_BACKEND: {
      idFromName: () => ({}) as DurableObjectId,
      get: () =>
        ({
          fetch: async () => new Response("backend"),
        }) as unknown as DurableObjectStub,
    } as unknown as Env["CONVERT_BACKEND"],
    RATE_LIMITER: {
      idFromName:
        options.rateLimiterIdFromName ?? (() => ({}) as DurableObjectId),
      get: () =>
        ({
          fetch: options.rateLimiterFetch ?? (async () => rateLimitResponse),
        }) as unknown as DurableObjectStub,
    } as unknown as DurableObjectNamespace,
    STENCILA_API_KEY_SECRET: {
      get: async () => "test-stencila-api-key",
    },
  };
}

describe("startBackend", () => {
  it("passes the Secrets Store API key to the container", async () => {
    const startAndWaitForPorts = vi.fn(async () => undefined);
    const backend = {
      fetch: async () => new Response("backend"),
      startAndWaitForPorts,
    };

    const selected = await startBackend(backend, {
      get: async () => "secret-stencila-api-key",
    });

    expect(selected).toBe(backend);
    expect(startAndWaitForPorts).toHaveBeenCalledWith({
      startOptions: {
        envVars: {
          STENCILA_API_KEY: "secret-stencila-api-key",
        },
      },
    });
  });
});

describe("evaluateRateLimit", () => {
  it("allows requests below minute and day limits", async () => {
    const { result } = evaluateRateLimit(undefined, Date.UTC(2026, 6, 9, 0, 0));

    expect(result.ok).toBe(true);
    expect(result.minuteCount).toBe(1);
    expect(result.dayCount).toBe(1);
  });

  it("throttles after five conversions in a minute", async () => {
    const now = Date.UTC(2026, 6, 9, 0, 0);
    let record: RateLimitRecord | undefined;

    for (let index = 0; index < 5; index += 1) {
      record = evaluateRateLimit(record, now).record;
    }

    const { result } = evaluateRateLimit(record, now);
    expect(result.ok).toBe(false);
    expect(result.retryAfter).toBeGreaterThan(0);
  });

  it("resets counters across minute and day windows", async () => {
    const first = evaluateRateLimit(undefined, Date.UTC(2026, 6, 9, 23, 59));
    const second = evaluateRateLimit(first.record, Date.UTC(2026, 6, 10, 0, 1));

    expect(first.result.minuteCount).toBe(1);
    expect(first.result.dayCount).toBe(1);
    expect(second.result.minuteCount).toBe(1);
    expect(second.result.dayCount).toBe(1);
  });
});

describe("rateLimitClient", () => {
  it("maps clients deterministically onto one of sixteen shards", async () => {
    const first = await rateLimitClient("192.0.2.1");
    const second = await rateLimitClient("192.0.2.1");

    expect(first).toEqual(second);
    expect(first.clientHash).toMatch(/^[a-f0-9]{64}$/);
    expect(first.shard).toBeGreaterThanOrEqual(0);
    expect(first.shard).toBeLessThan(16);
  });
});

describe("handleRequest", () => {
  it("serves static assets outside api paths", async () => {
    const response = await handleRequest(
      new Request("https://convert.stencila.dev/"),
      fakeEnv({ assetResponse: new Response("index") }),
    );

    expect(await response.text()).toBe("index");
  });

  it("proxies api requests to the selected backend", async () => {
    const response = await handleRequest(
      new Request("https://convert.stencila.dev/api/health"),
      fakeEnv({}),
      async () => ({
        fetch: async () => new Response("proxied"),
      }),
    );

    expect(await response.text()).toBe("proxied");
  });

  it("allows multipart overhead above the upload limit", async () => {
    const response = await handleRequest(
      new Request("https://convert.stencila.dev/api/convert", {
        method: "POST",
        headers: { "Content-Length": String(25 * 1024 * 1024 + 1024) },
      }),
      fakeEnv({}),
      async () => ({
        fetch: async () => new Response("proxied"),
      }),
    );

    expect(await response.text()).toBe("proxied");
  });

  it("rejects requests above the multipart request limit", async () => {
    const response = await handleRequest(
      new Request("https://convert.stencila.dev/api/convert", {
        method: "POST",
        headers: { "Content-Length": String(26 * 1024 * 1024 + 1) },
      }),
      fakeEnv({}),
    );

    expect(response.status).toBe(413);
  });

  it("returns 429 for throttled conversion requests", async () => {
    const response = await handleRequest(
      new Request("https://convert.stencila.dev/api/convert", {
        method: "POST",
        headers: { "CF-Connecting-IP": "192.0.2.1" },
      }),
      fakeEnv({
        rateLimitResponse: new Response(
          JSON.stringify({
            ok: false,
            minuteCount: 5,
            dayCount: 5,
            retryAfter: 30,
            message: "Conversion rate limit exceeded",
          }),
        ),
      }),
      async () => ({
        fetch: async () => new Response("should not proxy"),
      }),
    );

    expect(response.status).toBe(429);
    expect(response.headers.get("Retry-After")).toBe("30");
  });

  it("routes hashed client identifiers to fixed limiter shards", async () => {
    const idFromName = vi.fn(() => ({}) as DurableObjectId);
    const rateLimiterFetch = vi.fn(
      async (input: RequestInfo | URL, init?: RequestInit) => {
        const request = new Request(input, init);
        expect(request.headers.get("X-Rate-Limit-Client")).toMatch(
          /^[a-f0-9]{64}$/,
        );
        return new Response(
          JSON.stringify({
            ok: true,
            minuteCount: 1,
            dayCount: 1,
            retryAfter: 0,
            message: "Allowed",
          }),
        );
      },
    );

    await handleRequest(
      new Request("https://convert.stencila.dev/api/convert", {
        method: "POST",
        headers: { "CF-Connecting-IP": "192.0.2.1" },
      }),
      fakeEnv({ rateLimiterFetch, rateLimiterIdFromName: idFromName }),
      async () => ({ fetch: async () => new Response("proxied") }),
    );

    expect(idFromName).toHaveBeenCalledOnce();
    expect(idFromName).toHaveBeenCalledWith(
      expect.stringMatching(/^shard-(?:[0-9]|1[0-5])$/),
    );
  });
});
