import { describe, expect, it, vi } from "vitest";
import { ApiError, SessionClient } from "./session-client";
import type { Session, SessionStore } from "./session-client";

function fixture() {
  let session: Session | null = {
    session_id: "test-session",
    access_token: "old-access",
    refresh_token: "old-refresh",
    expires_in: 300,
    expires_at: Date.now() - 1000,
  };
  const store: SessionStore = {
    read: () => session,
    write: (next) => {
      session = next;
    },
  };
  let consumed = false;
  const fetcher = vi.fn<typeof fetch>(async (url, init) => {
    if (url === "/api/v1/auth/refresh") {
      if (consumed) return Response.json({ message: "Refresh token reused" }, { status: 401 });
      consumed = true;
      await Promise.resolve();
      return Response.json({
        access_token: "new-access",
        refresh_token: "new-refresh",
        expires_in: 300,
      });
    }
    if (new Headers(init?.headers).get("Authorization") !== "Bearer new-access")
      return Response.json({ message: "Expired token" }, { status: 401 });
    return Response.json({ ok: true });
  });
  return { store, fetcher };
}

describe("rotating sessions", () => {
  it("accepts a successful role deletion with no response body", async () => {
    const { store } = fixture();
    const original = store.read();
    if (!original) throw new Error("Fixture session missing");
    store.write({ ...original, expires_at: Date.now() + 300_000 });
    const fetcher = vi.fn<typeof fetch>(async () => new Response(null, { status: 204 }));
    const client = new SessionClient(store, fetcher);
    await expect(
      client.request<void>("/units/unit/roles/role", { method: "DELETE" }),
    ).resolves.toBeUndefined();
    expect(fetcher).toHaveBeenCalledWith(
      "/api/v1/units/unit/roles/role",
      expect.objectContaining({ method: "DELETE" }),
    );
  });
  it("uses the browser receiver when sending a registration request", async () => {
    const store: SessionStore = { read: () => null, write: () => undefined };
    const fetcher = vi.fn<typeof fetch>(async function (this: unknown) {
      if (this !== globalThis) throw new TypeError("Illegal invocation");
      return Response.json({ id: "registered-user" });
    });
    const client = new SessionClient(store, fetcher);
    await expect(
      client.register({
        username: "registration-fixture",
        email: "fixture@example.invalid",
        password: "test-only-password",
        password_confirm: "test-only-password",
      }),
    ).resolves.toBeUndefined();
    expect(fetcher).toHaveBeenCalledWith(
      "/api/v1/auth/register",
      expect.objectContaining({ method: "POST" }),
    );
  });
  it("shares one rotation across concurrent requests and saves the replacement token", async () => {
    const { store, fetcher } = fixture();
    const client = new SessionClient(store, fetcher);
    const results = await Promise.all(
      Array.from({ length: 8 }, () => client.request<{ ok: boolean }>("/auth/me")),
    );
    expect(results.every((value) => value.ok)).toBe(true);
    expect(fetcher.mock.calls.filter(([url]) => url === "/api/v1/auth/refresh")).toHaveLength(1);
    expect(store.read()?.refresh_token).toBe("new-refresh");
  });

  it("reads tokens inside a shared lock so separate tab clients do not reuse refresh tokens", async () => {
    const { store, fetcher } = fixture();
    let tail: Promise<unknown> = Promise.resolve();
    const lock = <T>(work: () => Promise<T>): Promise<T> => {
      const next = tail.then(work);
      tail = next.catch(() => undefined);
      return next;
    };
    const first = new SessionClient(store, fetcher, lock);
    const second = new SessionClient(store, fetcher, lock);
    await Promise.all([first.request("/auth/me"), second.request("/auth/me")]);
    expect(fetcher.mock.calls.filter(([url]) => url === "/api/v1/auth/refresh")).toHaveLength(1);
  });

  it("refreshes once after concurrent 401 responses even before local expiry", async () => {
    const { store, fetcher } = fixture();
    const original = store.read();
    if (!original) throw new Error("Fixture session missing");
    store.write({ ...original, expires_at: Date.now() + 300_000 });
    const client = new SessionClient(store, fetcher);
    await Promise.all([client.request("/auth/me"), client.request("/auth/me")]);
    expect(fetcher.mock.calls.filter(([url]) => url === "/api/v1/auth/refresh")).toHaveLength(1);
  });

  it("clears a revoked session without retrying its consumed refresh token", async () => {
    const { store } = fixture();
    const fetcher = vi.fn<typeof fetch>(async () =>
      Response.json({ message: "Revoked" }, { status: 401 }),
    );
    const client = new SessionClient(store, fetcher);
    await expect(client.request("/auth/me")).rejects.toMatchObject({ status: 401 });
    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(store.read()).toBeNull();
  });

  it("does not sign out or retry rotation automatically after a network failure", async () => {
    const { store } = fixture();
    const fetcher = vi.fn<typeof fetch>(async () => {
      throw new TypeError("offline");
    });
    const client = new SessionClient(store, fetcher);
    await expect(client.request("/auth/me")).rejects.toBeInstanceOf(ApiError);
    expect(fetcher).toHaveBeenCalledTimes(1);
    expect(store.read()).not.toBeNull();
  });

  it("handles a plain-text rejection without treating it as an invalid JSON success", async () => {
    const { store } = fixture();
    const original = store.read();
    if (!original) throw new Error("Fixture session missing");
    store.write({ ...original, expires_at: Date.now() + 300_000 });
    const client = new SessionClient(
      store,
      async () => new Response("Malformed UUID", { status: 400 }),
    );
    await expect(client.request("/units/invalid")).rejects.toMatchObject({ status: 400 });
  });
});

describe("file uploads", () => {
  it("keeps multipart bodies and browser-generated boundaries after session refresh", async () => {
    const { store, fetcher } = fixture();
    const client = new SessionClient(store, fetcher);
    const body = new FormData();
    body.append("file", new Blob(["image bytes"], { type: "image/png" }), "icon.png");
    await client.request("/units/unit/icon", { method: "POST", body });
    const upload = fetcher.mock.calls.find(([url]) => url === "/api/v1/units/unit/icon");
    expect(upload?.[1]?.body).toBe(body);
    const headers = new Headers(upload?.[1]?.headers);
    expect(headers.get("Authorization")).toBe("Bearer new-access");
    expect(headers.has("Content-Type")).toBe(false);
    const refresh = fetcher.mock.calls.find(([url]) => url === "/api/v1/auth/refresh");
    expect(new Headers(refresh?.[1]?.headers).get("Content-Type")).toBe("application/json");
  });
});
