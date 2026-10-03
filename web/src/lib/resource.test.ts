import { describe, expect, it, vi } from "vitest";
import { createResourceSource, resolveResourcePath } from "./resource";

describe("resource transport", () => {
  it("encodes template parameters and fails before using unresolved paths", () => {
    expect(resolveResourcePath("/units/{unit_id}/members", { unit_id: "a/b ?" })).toBe(
      "/units/a%2Fb%20%3F/members",
    );
    expect(() => resolveResourcePath("/units/{unit_id}/members", {})).toThrow(
      "Missing resource parameter: unit_id",
    );
  });
  it("reads paginated envelopes and uses cancellable reads, PATCH and DELETE", async () => {
    const spy = vi.fn(async (_path: string, _init?: RequestInit) => ({ members: [{ id: "one" }] }));
    const request = async <R>(path: string, init?: RequestInit): Promise<R> =>
      (await spy(path, init)) as R;
    const source = createResourceSource<{ id: string }>(
      "/units/{unit_id}/members",
      { unit_id: "unit" },
      "members",
      request,
    );
    const signal = new AbortController().signal;
    await expect(source.list(20, signal)).resolves.toEqual([{ id: "one" }]);
    expect(spy).toHaveBeenLastCalledWith("/units/unit/members?offset=20&limit=20", { signal });
    await source.get("a/b", signal);
    expect(spy).toHaveBeenLastCalledWith("/units/unit/members/a%2Fb", { signal });
    await source.update!("a/b", { id: "new" });
    expect(spy).toHaveBeenLastCalledWith("/units/unit/members/a%2Fb", {
      method: "PATCH",
      body: '{"id":"new"}',
    });
    await source.remove!("a/b");
    expect(spy).toHaveBeenLastCalledWith("/units/unit/members/a%2Fb", { method: "DELETE" });
  });
  it("rejects malformed collection envelopes", async () => {
    const request = async <R>(): Promise<R> => ({ other: [] }) as R;
    await expect(createResourceSource("/members", {}, "members", request).list(0)).rejects.toThrow(
      "invalid collection",
    );
  });
});
