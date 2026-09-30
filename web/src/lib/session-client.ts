export interface TokenPair {
  access_token: string;
  refresh_token: string;
  expires_in: number;
}

export interface Session extends TokenPair {
  expires_at: number;
  session_id: string;
}

export interface SessionStore {
  read(): Session | null;
  write(session: Session | null): void;
}

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

type Lock = <T>(work: () => Promise<T>) => Promise<T>;

export class SessionClient {
  private refreshing: Promise<Session> | null = null;

  constructor(
    private readonly store: SessionStore,
    private readonly fetcher: typeof fetch = fetch,
    private readonly lock: Lock = (work) => work(),
  ) {}

  private async send(path: string, init: RequestInit = {}, token?: string) {
    const headers = new Headers(init.headers);
    if (init.body) headers.set("Content-Type", "application/json");
    if (token) headers.set("Authorization", `Bearer ${token}`);
    let response: Response;
    try {
      // Native window.fetch requires the global receiver, not this client.
      response = await this.fetcher.call(globalThis, `/api/v1${path}`, { ...init, headers });
    } catch (error) {
      if (error instanceof Error && error.name === "AbortError") throw error;
      console.error("Request failed:", error);
      throw new ApiError(0, "Something went wrong. Check your connection and try again.");
    }
    if (!response.ok) {
      let message = `Request failed (${response.status}). Try again.`;
      const text = await response.text();
      try {
        const body: unknown = JSON.parse(text);
        if (
          typeof body === "object" &&
          body !== null &&
          "message" in body &&
          typeof body.message === "string"
        ) {
          message = body.message;
        }
      } catch {
        /* Axum path and query rejections may be plain text. */
      }
      throw new ApiError(response.status, message);
    }
    return response;
  }

  private async refresh(failedToken?: string): Promise<Session> {
    if (this.refreshing) return this.refreshing;
    this.refreshing = this.lock(async () => {
      // Read inside the cross-tab lock: another tab may already have rotated.
      const current = this.store.read();
      if (!current) throw new ApiError(401, "Your session ended. Sign in again.");
      if (failedToken && current.access_token !== failedToken) return current;
      if (!failedToken && current.expires_at > Date.now() + 10_000) return current;
      try {
        const response = await this.send("/auth/refresh", {
          method: "POST",
          body: JSON.stringify({ refresh_token: current.refresh_token }),
        });
        const pair: TokenPair = await response.json();
        const next = {
          ...pair,
          session_id: current.session_id,
          expires_at: Date.now() + pair.expires_in * 1000,
        };
        this.store.write(next);
        return next;
      } catch (error) {
        if (error instanceof ApiError && error.status === 401) this.store.write(null);
        // Never retry refresh automatically: the server consumes its token.
        throw error;
      }
    }).finally(() => {
      this.refreshing = null;
    });
    return this.refreshing;
  }

  async request<T>(path: string, init: RequestInit = {}): Promise<T> {
    let session = this.store.read();
    if (!session) throw new ApiError(401, "Sign in to continue.");
    if (session.expires_at <= Date.now() + 10_000) session = await this.refresh();
    let response: Response;
    try {
      response = await this.send(path, init, session.access_token);
    } catch (error) {
      if (!(error instanceof ApiError) || error.status !== 401) throw error;
      const renewed = await this.refresh(session.access_token);
      try {
        response = await this.send(path, init, renewed.access_token);
      } catch (retryError) {
        if (retryError instanceof ApiError && retryError.status === 401) this.store.write(null);
        throw retryError;
      }
    }
    if (response.status === 204) return undefined as T;
    return response.json();
  }

  async login(username: string, password: string) {
    await this.lock(async () => {
      const response = await this.send("/auth/login", {
        method: "POST",
        body: JSON.stringify({ username, password }),
      });
      const pair: TokenPair = await response.json();
      this.store.write({
        ...pair,
        expires_at: Date.now() + pair.expires_in * 1000,
        session_id: crypto.randomUUID(),
      });
    });
  }

  async register(input: {
    username: string;
    email: string;
    password: string;
    password_confirm: string;
  }) {
    await this.send("/auth/register", { method: "POST", body: JSON.stringify(input) });
  }

  async logout() {
    await this.lock(async () => {
      const session = this.store.read();
      this.store.write(null);
      if (session)
        await this.send("/auth/logout", {
          method: "POST",
          body: JSON.stringify({ refresh_token: session.refresh_token }),
        });
    });
  }
}
