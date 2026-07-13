import { Container, ContainerProxy, getRandom } from "@cloudflare/containers";

export { ContainerProxy };

const INSTANCE_COUNT = 3;
const RATE_LIMIT_SHARD_COUNT = 16;
const RATE_LIMIT_CLIENT_HEADER = "X-Rate-Limit-Client";
const RATE_LIMIT_RETENTION_MS = 25 * 60 * 60 * 1000;
const MAX_UPLOAD_BYTES = 25 * 1024 * 1024;
const MAX_REQUEST_BYTES = MAX_UPLOAD_BYTES + 1024 * 1024;
const MINUTE_LIMIT = 5;
const DAY_LIMIT = 50;

export interface Env {
  ASSETS: Fetcher;
  CONVERT_BACKEND: DurableObjectNamespace<ConvertBackend>;
  RATE_LIMITER: DurableObjectNamespace;
  STENCILA_API_KEY_SECRET: SecretsStoreSecret;
}

export interface BackendFetcher {
  fetch(request: Request): Promise<Response>;
}

export interface StartableBackend extends BackendFetcher {
  startAndWaitForPorts(options: {
    startOptions: { envVars: Record<string, string> };
  }): Promise<void>;
}

export type SelectBackend = (env: Env) => Promise<BackendFetcher>;

export class ConvertBackend extends Container {
  defaultPort = 8080;
  sleepAfter = "10m";
  enableInternet = false;
  allowedHosts = [
    "export.arxiv.org",
    "arxiv.org",
    "doi.org",
    "api.stencila.cloud",
    "biorxiv.org",
    "www.biorxiv.org",
    "medrxiv.org",
    "www.medrxiv.org",
    "www.ncbi.nlm.nih.gov",
    "pmc.ncbi.nlm.nih.gov",
    "ftp.ncbi.nlm.nih.gov",
  ];
}

export class ConvertRateLimitShard {
  constructor(
    private readonly state: DurableObjectState,
    _env: Env,
  ) {
    void _env;

    this.state.storage.sql.exec(`
      CREATE TABLE IF NOT EXISTS rate_limits (
        client_hash TEXT PRIMARY KEY,
        minute_window TEXT NOT NULL,
        minute_count INTEGER NOT NULL,
        day_window TEXT NOT NULL,
        day_count INTEGER NOT NULL,
        last_seen INTEGER NOT NULL
      )
    `);
    this.state.storage.sql.exec(`
      CREATE INDEX IF NOT EXISTS rate_limits_last_seen
      ON rate_limits (last_seen)
    `);
  }

  async fetch(request: Request): Promise<Response> {
    const clientHash = request.headers.get(RATE_LIMIT_CLIENT_HEADER);
    if (!clientHash || !/^[a-f0-9]{64}$/.test(clientHash)) {
      return json(
        {
          error: {
            code: "invalid_rate_limit_client",
            message: "Rate limit client identifier is missing or invalid",
          },
        },
        400,
      );
    }

    const now = Date.now();
    const result = this.state.storage.transactionSync(() => {
      const row = this.state.storage.sql
        .exec<RateLimitRow>(
          `SELECT minute_window, minute_count, day_window, day_count, last_seen
           FROM rate_limits
           WHERE client_hash = ?`,
          clientHash,
        )
        .toArray()[0];
      const decision = evaluateRateLimit(row, now);

      if (decision.result.ok) {
        this.state.storage.sql.exec(
          `INSERT INTO rate_limits (
             client_hash, minute_window, minute_count, day_window, day_count, last_seen
           ) VALUES (?, ?, ?, ?, ?, ?)
           ON CONFLICT (client_hash) DO UPDATE SET
             minute_window = excluded.minute_window,
             minute_count = excluded.minute_count,
             day_window = excluded.day_window,
             day_count = excluded.day_count,
             last_seen = excluded.last_seen`,
          clientHash,
          decision.record.minuteWindow,
          decision.record.minuteCount,
          decision.record.dayWindow,
          decision.record.dayCount,
          decision.record.lastSeen,
        );
      }

      return decision.result;
    });

    if ((await this.state.storage.getAlarm()) === null) {
      await this.state.storage.setAlarm(now + RATE_LIMIT_RETENTION_MS);
    }

    return json(result, result.ok ? 200 : 429, {
      "Retry-After": result.retryAfter.toString(),
    });
  }

  async alarm(): Promise<void> {
    const now = Date.now();
    this.state.storage.sql.exec(
      "DELETE FROM rate_limits WHERE last_seen <= ?",
      now - RATE_LIMIT_RETENTION_MS,
    );

    const oldest = this.state.storage.sql
      .exec<OldestRateLimitRow>(
        "SELECT MIN(last_seen) AS last_seen FROM rate_limits",
      )
      .one().last_seen;
    if (oldest !== null) {
      await this.state.storage.setAlarm(
        Math.max(now + 1_000, oldest + RATE_LIMIT_RETENTION_MS),
      );
    }
  }
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    return handleRequest(request, env);
  },
};

export async function handleRequest(
  request: Request,
  env: Env,
  selectBackend: SelectBackend = defaultSelectBackend,
): Promise<Response> {
  const url = new URL(request.url);

  if (url.pathname.startsWith("/api/")) {
    if (request.method === "POST" && url.pathname === "/api/convert") {
      const bodyLimit = checkContentLength(request);
      if (bodyLimit) {
        return bodyLimit;
      }

      const rateLimit = await checkRateLimit(request, env);
      if (!rateLimit.ok) {
        return json(
          {
            error: {
              code: "rate_limited",
              message: rateLimit.message,
            },
          },
          429,
          { "Retry-After": rateLimit.retryAfter.toString() },
        );
      }
    }

    const backend = await selectBackend(env);
    return backend.fetch(request);
  }

  return env.ASSETS.fetch(request);
}

async function defaultSelectBackend(env: Env): Promise<BackendFetcher> {
  const backend = await getRandom(env.CONVERT_BACKEND, INSTANCE_COUNT);
  return startBackend(backend, env.STENCILA_API_KEY_SECRET);
}

export async function startBackend(
  backend: StartableBackend,
  apiKeySecret: SecretsStoreSecret,
): Promise<BackendFetcher> {
  const apiKey = await apiKeySecret.get();
  await backend.startAndWaitForPorts({
    startOptions: {
      envVars: {
        STENCILA_API_KEY: apiKey,
      },
    },
  });

  return backend;
}

function checkContentLength(request: Request): Response | undefined {
  const contentLength = request.headers.get("Content-Length");
  if (!contentLength) {
    return undefined;
  }

  const size = Number.parseInt(contentLength, 10);
  if (Number.isFinite(size) && size > MAX_REQUEST_BYTES) {
    return json(
      {
        error: {
          code: "input_too_large",
          message: "Input exceeds the 25 MiB public service limit",
        },
      },
      413,
    );
  }

  return undefined;
}

async function checkRateLimit(
  request: Request,
  env: Env,
): Promise<RateLimitResult> {
  const ip = request.headers.get("CF-Connecting-IP") ?? "unknown";
  const { clientHash, shard } = await rateLimitClient(ip);
  const id = env.RATE_LIMITER.idFromName(`shard-${shard}`);
  const stub = env.RATE_LIMITER.get(id);
  const response = await stub.fetch("https://rate-limit.local/check", {
    method: "POST",
    headers: { [RATE_LIMIT_CLIENT_HEADER]: clientHash },
  });

  return (await response.json()) as RateLimitResult;
}

export async function rateLimitClient(
  client: string,
): Promise<{ clientHash: string; shard: number }> {
  const digest = new Uint8Array(
    await crypto.subtle.digest("SHA-256", new TextEncoder().encode(client)),
  );
  const clientHash = Array.from(digest, (byte) =>
    byte.toString(16).padStart(2, "0"),
  ).join("");

  return {
    clientHash,
    shard: digest[0] % RATE_LIMIT_SHARD_COUNT,
  };
}

export interface RateLimitResult {
  ok: boolean;
  minuteCount: number;
  dayCount: number;
  retryAfter: number;
  message: string;
}

export interface RateLimitRecord {
  minuteWindow: string;
  minuteCount: number;
  dayWindow: string;
  dayCount: number;
  lastSeen: number;
}

interface RateLimitRow {
  [column: string]: SqlStorageValue;
  minute_window: string;
  minute_count: number;
  day_window: string;
  day_count: number;
  last_seen: number;
}

interface OldestRateLimitRow {
  [column: string]: SqlStorageValue;
  last_seen: number | null;
}

export function evaluateRateLimit(
  previous: RateLimitRecord | RateLimitRow | undefined,
  now: number,
): { result: RateLimitResult; record: RateLimitRecord } {
  const minuteWindow = Math.floor(now / 60_000).toString();
  const dayWindow = new Date(now).toISOString().slice(0, 10);
  const record =
    previous && "minute_window" in previous
      ? {
          minuteWindow: previous.minute_window,
          minuteCount: previous.minute_count,
          dayWindow: previous.day_window,
          dayCount: previous.day_count,
          lastSeen: previous.last_seen,
        }
      : previous;
  const minuteCount =
    record?.minuteWindow === minuteWindow ? record.minuteCount : 0;
  const dayCount = record?.dayWindow === dayWindow ? record.dayCount : 0;

  const nextRecord = (
    minuteCount: number,
    dayCount: number,
  ): RateLimitRecord => ({
    minuteWindow,
    minuteCount,
    dayWindow,
    dayCount,
    lastSeen: now,
  });

  if (minuteCount >= MINUTE_LIMIT) {
    return {
      result: {
        ok: false,
        minuteCount,
        dayCount,
        retryAfter: secondsUntilNextMinute(now),
        message: "Conversion rate limit exceeded",
      },
      record: nextRecord(minuteCount, dayCount),
    };
  }

  if (dayCount >= DAY_LIMIT) {
    return {
      result: {
        ok: false,
        minuteCount,
        dayCount,
        retryAfter: secondsUntilNextUtcDay(now),
        message: "Daily conversion limit exceeded",
      },
      record: nextRecord(minuteCount, dayCount),
    };
  }

  return {
    result: {
      ok: true,
      minuteCount: minuteCount + 1,
      dayCount: dayCount + 1,
      retryAfter: 0,
      message: "Allowed",
    },
    record: nextRecord(minuteCount + 1, dayCount + 1),
  };
}

function secondsUntilNextMinute(now: number): number {
  return Math.max(1, Math.ceil((60_000 - (now % 60_000)) / 1000));
}

function secondsUntilNextUtcDay(now: number): number {
  const date = new Date(now);
  const nextDay = Date.UTC(
    date.getUTCFullYear(),
    date.getUTCMonth(),
    date.getUTCDate() + 1,
  );
  return Math.max(1, Math.ceil((nextDay - now) / 1000));
}

function json(
  value: unknown,
  status = 200,
  headers: Record<string, string> = {},
): Response {
  return new Response(JSON.stringify(value), {
    status,
    headers: {
      "Content-Type": "application/json; charset=utf-8",
      ...headers,
    },
  });
}
