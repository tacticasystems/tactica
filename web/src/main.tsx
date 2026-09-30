import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { RouterProvider } from "@tanstack/react-router";
import { QueryClientProvider } from "@tanstack/react-query";
import { router } from "./router";
import { queryClient } from "./lib/queries";
import { sessionSnapshot, subscribeSession } from "./lib/api";
import "./global.css";
import "./app.css";

let previousSession = sessionSnapshot();
subscribeSession(() => {
  const nextSession = sessionSnapshot();
  if (nextSession !== previousSession) {
    previousSession = nextSession;
    queryClient.clear();
  }
});

const root = document.getElementById("root");
if (!root) throw new Error("Application root is missing.");
createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </StrictMode>,
);
