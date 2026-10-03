import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  Outlet,
  RouterProvider,
} from "@tanstack/react-router";
import { createRoot } from "react-dom/client";
import { createResourceRoutes } from "../../src/components/resource-routes";
import { ApiError } from "../../src/lib/session-client";
import type { ResourceDefinition } from "../../src/lib/resource";
import "../../src/global.css";
import "../../src/app.css";

// Disposable in-memory records exercise default views without touching the API.
let people = Array.from({ length: 21 }, (_, index) => ({
  id: String(index),
  name: `Person ${index}`,
}));
const resource: ResourceDefinition<(typeof people)[number]> = {
  apiBase: "/people",
  label: "Person",
  pluralLabel: "People",
  name: (person) => person.name,
  searchText: (person) => person.name,
  fields: [
    {
      id: "name",
      label: "Name",
      in: ["list", "view", "edit"],
      read: (person) => person.name,
      edit: { value: (person) => person.name, write: (name) => ({ name }), required: true },
    },
  ],
  useResource: () => ({
    queryKey: ["test"],
    context: undefined,
    total: people.length,
    canEdit: (person) => person.id !== "1",
    canDelete: (person) => person.id !== "1",
    source: {
      list: async (offset) => people.slice(offset, offset + 20),
      get: async (id) => {
        const person = people.find((person) => person.id === id);
        if (!person) throw new ApiError(404, "Not found");
        return { ...person };
      },
      update: async (id, input) => {
        if (id === "1") throw new ApiError(403, "Forbidden");
        if (input.name === "fail") throw new Error("Save failed. Try again.");
        people = people.map((person) => (person.id === id ? { ...person, ...input } : person));
      },
      remove: async (id) => {
        if (id === "1") throw new ApiError(403, "Forbidden");
        people = people.filter((person) => person.id !== id);
      },
    },
  }),
};
const root = createRootRoute({ component: Outlet });
const unit = createRoute({ getParentRoute: () => root, path: "/units/$unitId", component: Outlet });
const routes = createResourceRoutes(unit, "/people", "personId", resource);
const router = createRouter({
  routeTree: root.addChildren([unit.addChildren([...routes.routes])]),
  history: createMemoryHistory({
    initialEntries: [
      new URLSearchParams(window.location.search).get("route") ?? "/units/test/people",
    ],
  }),
});
const client = new QueryClient({
  defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
});
createRoot(document.getElementById("root")!).render(
  <QueryClientProvider client={client}>
    <main className="p-6">
      <RouterProvider router={router} />
    </main>
  </QueryClientProvider>,
);
