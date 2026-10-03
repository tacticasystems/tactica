import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { renderToString } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { createResourceRoutes } from "./resource-routes";
import type { ResourceDefinition } from "../lib/resource";
import { ApiError } from "../lib/session-client";

const person = { id: "one", name: "Example" };
async function renderRoute(
  path: string,
  overrides: Partial<ResourceDefinition<typeof person>> = {},
  missing = false,
) {
  const client = new QueryClient({
    defaultOptions: { queries: { retry: false, retryOnMount: false } },
  });
  client.setQueryData(["test", "people", 0], [person]);
  if (!missing) client.setQueryData(["test", "people-item", "one"], person);

  const root = createRootRoute({ component: Outlet });
  const parent = createRoute({
    getParentRoute: () => root,
    path: "/units/$unitId",
    component: Outlet,
  });
  const resource = createResourceRoutes(parent, "/people", "personId", {
    apiBase: "/people",
    label: "Person",
    pluralLabel: "People",
    name: (record: typeof person) => record.name,
    fields: [
      {
        id: "name",
        label: "Name",
        in: ["list", "view", "edit"],
        read: (record) => record.name,
        edit: { value: (record) => record.name, write: (name) => ({ name }) },
      },
    ],
    useResource: () => ({
      queryKey: ["test"],
      context: undefined,
      source: {
        list: async () => [person],
        get: async () => person,
        update: async () => {},
        remove: async () => {},
      },
      canEdit: () => true,
      canDelete: () => true,
    }),
    ...overrides,
  });
  if (missing) {
    await client
      .fetchQuery({
        queryKey: ["test", "people-item", "missing"],
        queryFn: async () => {
          throw new ApiError(404, "Not found");
        },
      })
      .catch(() => {});
  }

  const router = createRouter({
    routeTree: root.addChildren([parent.addChildren([...resource.routes])]),
    history: createMemoryHistory({ initialEntries: [path] }),
  });
  await router.load();
  return renderToString(
    <QueryClientProvider client={client}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  );
}

describe("generated resource routes", () => {
  it("renders list links using nested parent parameters", async () => {
    expect(await renderRoute("/units/unit/people")).toContain('href="/units/unit/people/one"');
  });
  it("renders read fields and permitted edit/delete links", async () => {
    const html = await renderRoute("/units/unit/people/one");
    expect(html).toContain("<dt>Name</dt>");
    expect(html).toContain('href="/units/unit/people/one/edit"');
    expect(html).toContain('href="/units/unit/people/one/delete"');
  });
  it("renders text editors and explicit delete confirmation", async () => {
    expect(await renderRoute("/units/unit/people/one/edit")).toContain('value="Example"');
    expect(await renderRoute("/units/unit/people/one/delete")).toContain("This cannot be undone.");
  });
  it("uses an override without invoking the default hook", async () => {
    expect(
      await renderRoute("/units/unit/people/one/edit", {
        editView: () => <p>Specialized editor</p>,
        useResource: () => {
          throw new Error("Default hook should not run");
        },
      }),
    ).toContain("Specialized editor");
  });
  it("denies direct mutation URLs and hides mutation links without explicit permission", async () => {
    const useResource = () => ({
      queryKey: ["test"],
      context: undefined,
      source: {
        list: async () => [person],
        get: async () => person,
        update: async () => {},
        remove: async () => {},
      },
    });
    const html = await renderRoute("/units/unit/people/one", { useResource });
    expect(html).not.toContain('href="/units/unit/people/one/edit"');
    expect(html).not.toContain('href="/units/unit/people/one/delete"');
    expect(await renderRoute("/units/unit/people/one/edit", { useResource })).toContain(
      "Editing unavailable",
    );
    expect(await renderRoute("/units/unit/people/one/delete", { useResource })).toContain(
      "Deletion unavailable",
    );
  });
  it("renders a missing-record state with a link back to the collection", async () => {
    const html = await renderRoute("/units/unit/people/missing", {}, true);
    expect(html).toContain("Person not found");
    expect(html).toContain('href="/units/unit/people"');
  });
});
