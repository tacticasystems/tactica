import {
  createRootRoute,
  createRoute,
  createRouter,
  Link,
  Navigate,
  Outlet,
  redirect,
} from "@tanstack/react-router";

import { sessionSnapshot } from "./lib/api";

import { Button } from "./components/ui/button";
import { AuthPage } from "./pages/auth";
import { UnitsPage } from "./pages/units";
import { MemberPage } from "./pages/member";
import { PersonnelPage } from "./pages/personnel";
import { AdminRanksPage } from "./pages/admin-ranks";
import { RankPage } from "./pages/rank";
import { RanksPage } from "./pages/ranks";
import { RolesPage } from "./pages/roles";
import { ProfilePage } from "./pages/profile";
import { OverviewPage } from "./pages/overview";
import { UnitWorkspace } from "./components/workspace";

const root = createRootRoute({
  component: Outlet,
  notFoundComponent: () => (
    <main className="standalone-state">
      <h1>Page not found</h1>
      <p>Choose a unit to return to your workspace.</p>
      <Link to="/units">Your units</Link>
    </main>
  ),
  errorComponent: ({ error, reset }) => (
    <main className="standalone-state">
      <h1>Something went wrong</h1>
      <p>{error instanceof Error ? error.message : "Please try again."}</p>
      <Button variant="outline" onClick={reset}>
        Try again
      </Button>
    </main>
  ),
});
const index = createRoute({
  getParentRoute: () => root,
  path: "/",
  component: () => <Navigate to={sessionSnapshot() ? "/units" : "/login"} replace />,
});
const login = createRoute({
  getParentRoute: () => root,
  path: "/login",
  component: () => <AuthPage />,
});
const register = createRoute({
  getParentRoute: () => root,
  path: "/register",
  component: () => <AuthPage register />,
});
const units = createRoute({ getParentRoute: () => root, path: "/units", component: UnitsPage });
const unit = createRoute({
  getParentRoute: () => root,
  path: "/units/$unitId",
  component: UnitWorkspace,
});
const unitIndex = createRoute({
  getParentRoute: () => unit,
  path: "/",
  component: () => {
    const { unitId } = unit.useParams();
    return <Navigate to="/units/$unitId/personnel" params={{ unitId }} replace />;
  },
});
const overview = createRoute({
  getParentRoute: () => unit,
  path: "/overview",
  component: OverviewPage,
});
const personnel = createRoute({
  getParentRoute: () => unit,
  path: "/personnel",
  component: PersonnelPage,
});
const member = createRoute({
  getParentRoute: () => unit,
  path: "/personnel/$memberId",
  component: MemberPage,
});
const editMember = createRoute({
  getParentRoute: () => unit,
  path: "/personnel/$memberId/edit",
  component: () => <MemberPage editing />,
});
const roles = createRoute({
  getParentRoute: () => unit,
  path: "/roles",
  component: RolesPage,
  validateSearch: (search: Record<string, unknown>): { roleId?: string } =>
    typeof search.roleId === "string" ? { roleId: search.roleId } : {},
});

const ranks = createRoute({
  getParentRoute: () => unit,
  path: "/ranks",
  component: RanksPage,
  beforeLoad: ({ search, params }) => {
    if (search.rankId)
      throw redirect({
        to: "/units/$unitId/ranks/$rankId",
        params: { unitId: params.unitId, rankId: search.rankId },
        search: {},
        replace: true,
      });
  },
  validateSearch: (search: Record<string, unknown>): { rankId?: string } =>
    typeof search.rankId === "string" ? { rankId: search.rankId } : {},
});

const rank = createRoute({
  getParentRoute: () => unit,
  path: "/ranks/$rankId",
  component: RankPage,
});
const adminRanks = createRoute({
  getParentRoute: () => unit,
  path: "/admin/ranks",
  component: AdminRanksPage,
  validateSearch: (search: Record<string, unknown>): { rankId?: string } =>
    typeof search.rankId === "string" ? { rankId: search.rankId } : {},
});

const profile = createRoute({
  getParentRoute: () => unit,
  path: "/profile",
  component: ProfilePage,
});

export const router = createRouter({
  routeTree: root.addChildren([
    index,
    login,
    register,
    units,
    unit.addChildren([
      unitIndex,
      overview,
      personnel,
      member,
      editMember,
      ranks,
      rank,
      adminRanks,
      roles,
      profile,
    ]),
  ]),
  defaultPreload: "intent",
});
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
