import { createContext, useContext, useEffect, useSyncExternalStore } from "react";
import type { ReactNode } from "react";
import {
  Link,
  Navigate,
  Outlet,
  useLocation,
  useNavigate,
  useParams,
} from "@tanstack/react-router";
import { useQuery } from "@tanstack/react-query";
import {
  ArrowLeft,
  ChevronDown,
  ChevronsUpDown,
  Github,
  Home,
  LogOut,
  Settings,
  Users,
} from "lucide-react";
import { api, client, sessionSnapshot, subscribeSession } from "../lib/api";
import { previewApi } from "../lib/preview";
import { queryClient, unitOptions, unitsOptions, userOptions } from "../lib/queries";
import type { Unit, UnitDataSource } from "../lib/types";
import { Avatar, Brand, ErrorState, LoadingState } from "./shared";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarHeader,
  SidebarInset,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarProvider,
  SidebarTrigger,
  useSidebar,
} from "./ui/sidebar";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import { TooltipProvider } from "./ui/tooltip";

interface WorkspaceContext {
  unit: Unit;
  source: UnitDataSource;
  preview: boolean;
  queryKey: readonly unknown[];
}
const context = createContext<WorkspaceContext | null>(null);

export function useWorkspace() {
  const value = useContext(context);
  if (!value) throw new Error("A unit workspace is required.");
  return value;
}

export function RequireSession({ children }: { children: ReactNode }) {
  const session = useSyncExternalStore(subscribeSession, sessionSnapshot);
  return session ? children : <Navigate to="/login" replace />;
}

export function UnitWorkspace() {
  const { unitId } = useParams({ strict: false });
  if (!unitId) return <Navigate to="/units" replace />;
  return unitId === "preview" ? (
    <LoadedWorkspace unitId={unitId} />
  ) : (
    <RequireSession>
      <LoadedWorkspace unitId={unitId} />
    </RequireSession>
  );
}

function LoadedWorkspace({ unitId }: { unitId: string }) {
  const preview = unitId === "preview";
  const source = preview ? previewApi : api;
  const options = unitOptions(source, unitId);
  const query = useQuery(options);
  const user = useQuery({ ...userOptions(), enabled: !preview });
  const location = useLocation();
  const label = location.pathname.endsWith("/roles")
    ? "Roles"
    : location.pathname.endsWith("/profile")
      ? "Profile"
      : location.pathname.endsWith("/overview")
        ? "Overview"
        : "Personnel";
  useEffect(() => {
    document.title = `${label}${query.data ? ` · ${query.data.display_name}` : ""} · Tactica`;
  }, [label, query.data]);
  return (
    <AppShell
      unit={query.data}
      preview={preview}
      accountName={preview ? "Preview" : (user.data?.display_name ?? user.data?.username)}
      label={label}
    >
      {query.isPending ? (
        <LoadingState />
      ) : query.isError ? (
        <ErrorState error={query.error} retry={() => void query.refetch()} />
      ) : (
        <context.Provider value={{ unit: query.data, source, preview, queryKey: options.queryKey }}>
          <Outlet key={unitId} />
        </context.Provider>
      )}
    </AppShell>
  );
}

type AppShellProps = {
  unit?: Unit;
  preview?: boolean;
  accountName?: string;
  label: string;
  children: ReactNode;
};

export function AppShell(props: AppShellProps) {
  return (
    <TooltipProvider>
      <SidebarProvider className="app-shell">
        <WorkspaceShell {...props} />
      </SidebarProvider>
    </TooltipProvider>
  );
}

function WorkspaceShell({ unit, preview = false, accountName, label, children }: AppShellProps) {
  const { setOpenMobile } = useSidebar();
  const location = useLocation();
  const navigate = useNavigate();
  const units = useQuery({ ...unitsOptions(), enabled: !preview && !!sessionSnapshot() });
  const close = () => setOpenMobile(false);
  const signOut = async () => {
    try {
      await client.logout();
    } catch {
      /* Local session ends even if server revocation is unavailable. */
    }
    queryClient.clear();
    await navigate({ to: "/login" });
  };
  const links = [
    { page: "overview", title: "Overview", icon: Home },
    { page: "personnel", title: "Personnel", icon: Users },
  ] as const;
  return (
    <>
      <a className="skip-link" href="#main-content">
        Skip to content
      </a>
      <Sidebar collapsible="offcanvas">
        <SidebarHeader className="workspace-sidebar-header">
          <SidebarMenu>
            <SidebarMenuItem>
              <DropdownMenu>
                <DropdownMenuTrigger asChild>
                  <SidebarMenuButton size="lg" className="unit-switcher" aria-label="Select unit">
                    <Avatar name={unit?.display_name ?? "Units"} url={unit?.icon_url} />
                    <span className="unit-switcher-copy">
                      <strong>{unit?.display_name ?? "Choose a unit"}</strong>
                      <span className="unit-select-detail">
                        {preview ? "Preview workspace" : "Unit workspace"}
                      </span>
                    </span>
                    <ChevronsUpDown className="unit-switcher-chevron" aria-hidden="true" />
                  </SidebarMenuButton>
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  className="unit-switcher-menu"
                  align="start"
                  side="bottom"
                  sideOffset={4}
                  aria-label="Your units"
                >
                  <DropdownMenuRadioGroup
                    value={unit?.id ?? ""}
                    onValueChange={(unitId) => {
                      close();
                      if (unitId !== unit?.id)
                        void navigate({ to: "/units/$unitId/personnel", params: { unitId } });
                    }}
                  >
                    {preview ? (
                      <DropdownMenuRadioItem value="preview">9 Rifles</DropdownMenuRadioItem>
                    ) : (
                      <>
                        {unit && !units.data?.some((item) => item.id === unit.id) && (
                          <DropdownMenuRadioItem value={unit.id}>
                            {unit.display_name}
                          </DropdownMenuRadioItem>
                        )}
                        {units.data?.map((item) => (
                          <DropdownMenuRadioItem key={item.id} value={item.id}>
                            {item.display_name}
                          </DropdownMenuRadioItem>
                        ))}
                      </>
                    )}
                  </DropdownMenuRadioGroup>
                  {!preview && units.isPending && (
                    <DropdownMenuItem disabled>Loading units…</DropdownMenuItem>
                  )}
                  {!preview && units.isError && (
                    <DropdownMenuItem onSelect={() => void units.refetch()}>
                      Retry loading units
                    </DropdownMenuItem>
                  )}
                  {!preview && !unit && units.isSuccess && !units.data.length && (
                    <DropdownMenuItem disabled>No units yet</DropdownMenuItem>
                  )}
                  <DropdownMenuSeparator />
                  <DropdownMenuItem asChild>
                    <Link to={preview ? "/login" : "/units"} onClick={close}>
                      <ArrowLeft />
                      <span>{preview ? "Back to sign in" : "All your units"}</span>
                    </Link>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            </SidebarMenuItem>
          </SidebarMenu>
        </SidebarHeader>
        <SidebarContent>
          <SidebarGroup className="workspace-sidebar-group">
            <nav aria-label="Unit navigation">
              <SidebarMenu className="gap-1">
                {unit ? (
                  <>
                    {links.map(({ page, title, icon: Icon }) => (
                      <SidebarMenuItem key={page}>
                        <SidebarMenuButton
                          asChild
                          isActive={location.pathname.endsWith(`/${page}`)}
                        >
                          <Link
                            to={`/units/$unitId/${page}`}
                            params={{ unitId: unit.id }}
                            onClick={close}
                          >
                            <Icon />
                            <span>{title}</span>
                          </Link>
                        </SidebarMenuButton>
                        {page === "personnel" && (
                          <SidebarMenuBadge>{unit.member_count}</SidebarMenuBadge>
                        )}
                      </SidebarMenuItem>
                    ))}
                    <SidebarMenuItem>
                      <details open className="administration-menu">
                        <SidebarMenuButton asChild>
                          <summary>
                            <Settings />
                            <span>Administration</span>
                            <ChevronDown className="administration-chevron" />
                          </summary>
                        </SidebarMenuButton>
                        <SidebarMenuSub>
                          <SidebarMenuSubItem>
                            <SidebarMenuSubButton
                              asChild
                              isActive={location.pathname.endsWith("/profile")}
                            >
                              <Link
                                to="/units/$unitId/profile"
                                params={{ unitId: unit.id }}
                                onClick={close}
                              >
                                <span>Profile</span>
                              </Link>
                            </SidebarMenuSubButton>
                          </SidebarMenuSubItem>
                          <SidebarMenuSubItem>
                            <SidebarMenuSubButton
                              asChild
                              isActive={location.pathname.endsWith("/roles")}
                            >
                              <Link
                                to="/units/$unitId/roles"
                                params={{ unitId: unit.id }}
                                onClick={close}
                              >
                                <span>Roles</span>
                              </Link>
                            </SidebarMenuSubButton>
                          </SidebarMenuSubItem>
                        </SidebarMenuSub>
                      </details>
                    </SidebarMenuItem>
                  </>
                ) : (
                  <SidebarMenuItem>
                    <SidebarMenuButton asChild isActive={location.pathname === "/units"}>
                      <Link to="/units" onClick={close}>
                        <Users />
                        <span>Your units</span>
                      </Link>
                    </SidebarMenuButton>
                  </SidebarMenuItem>
                )}
              </SidebarMenu>
            </nav>
          </SidebarGroup>
        </SidebarContent>
        <SidebarFooter className="workspace-sidebar-footer">
          <div className="account">
            <Avatar name={accountName ?? "Account"} size="small" />
            <div>
              <strong>{accountName ?? "Account"}</strong>
              <span>{preview ? "Preview data" : "Signed in"}</span>
            </div>
            {!preview && (
              <button
                type="button"
                className="icon-button"
                aria-label="Sign out"
                title="Sign out"
                onClick={() => void signOut()}
              >
                <LogOut size={17} />
              </button>
            )}
          </div>
          <div className="sidebar-brand-row">
            <Link to="/units" className="brand sidebar-footer-brand" onClick={close}>
              <Brand />
            </Link>
            <div className="sidebar-footer-links">
              <a
                href="https://github.com/tacticasystems"
                className="icon-button"
                aria-label="Tactica on GitHub"
                title="GitHub"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Github size={17} aria-hidden="true" />
              </a>
              <a
                href="https://discord.gg/gc3C2WR2cv"
                className="icon-button"
                aria-label="Discord (link coming soon)"
                title="Discord · coming soon"
                target="_blank"
                rel="noopener noreferrer"
              >
                {/* Discord mark from Simple Icons (CC0). */}
                <svg
                  width="17"
                  height="17"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path d="M20.317 4.3698a19.7913 19.7913 0 00-4.8851-1.5152.0741.0741 0 00-.0785.0371c-.211.3753-.4447.8648-.6083 1.2495-1.8447-.2762-3.68-.2762-5.4868 0-.1636-.3933-.4058-.8742-.6177-1.2495a.077.077 0 00-.0785-.037 19.7363 19.7363 0 00-4.8852 1.515.0699.0699 0 00-.0321.0277C.5334 9.0458-.319 13.5799.0992 18.0578a.0824.0824 0 00.0312.0561c2.0528 1.5076 4.0413 2.4228 5.9929 3.0294a.0777.0777 0 00.0842-.0276c.4616-.6304.8731-1.2952 1.226-1.9942a.076.076 0 00-.0416-.1057c-.6528-.2476-1.2743-.5495-1.8722-.8923a.077.077 0 01-.0076-.1277c.1258-.0943.2517-.1923.3718-.2914a.0743.0743 0 01.0776-.0105c3.9278 1.7933 8.18 1.7933 12.0614 0a.0739.0739 0 01.0785.0095c.1202.099.246.1981.3728.2924a.077.077 0 01-.0066.1276 12.2986 12.2986 0 01-1.873.8914.0766.0766 0 00-.0407.1067c.3604.698.7719 1.3628 1.225 1.9932a.076.076 0 00.0842.0286c1.961-.6067 3.9495-1.5219 6.0023-3.0294a.077.077 0 00.0313-.0552c.5004-5.177-.8382-9.6739-3.5485-13.6604a.061.061 0 00-.0312-.0286zM8.02 15.3312c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9555-2.4189 2.157-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.9555 2.4189-2.1569 2.4189zm7.9748 0c-1.1825 0-2.1569-1.0857-2.1569-2.419 0-1.3332.9554-2.4189 2.1569-2.4189 1.2108 0 2.1757 1.0952 2.1568 2.419 0 1.3332-.946 2.4189-2.1568 2.4189Z" />
                </svg>
              </a>
              <SidebarTrigger className="md:hidden" aria-label="Close navigation" />
            </div>
          </div>
        </SidebarFooter>
      </Sidebar>
      <SidebarInset className="workspace-body">
        <header className="topbar">
          <SidebarTrigger aria-label="Toggle navigation" />
          <div className="breadcrumb">
            {unit && (
              <>
                <span>{unit.display_name}</span>
                <span aria-hidden="true">/</span>
              </>
            )}
            <span>{label}</span>
          </div>
          {preview && <span className="preview-label">Preview</span>}
        </header>
        {preview && (
          <div className="preview-banner">
            9 Rifles preview. Changes stay in this browser session.
          </div>
        )}
        <div id="main-content" tabIndex={-1} className="main-content">
          {children}
        </div>
      </SidebarInset>
    </>
  );
}
