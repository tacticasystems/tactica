import { SiGithub, SiDiscord } from "@icons-pack/react-simple-icons";
import { useQuery } from "@tanstack/react-query";
import { Link, useLocation, useNavigate } from "@tanstack/react-router";
import {
  ArrowLeft,
  ChevronDown,
  ChevronsUpDown,
  ChevronsUp,
  Home,
  LogOut,
  Settings,
  Users,
} from "lucide-react";

import { client, sessionSnapshot } from "../lib/api";
import { queryClient, unitsOptions } from "../lib/queries";
import type { Unit } from "../lib/types";

import { Avatar } from "./avatar";
import { Brand } from "./brand";
import { Button } from "./ui/button";
import { Collapsible, CollapsibleTrigger, CollapsibleContent } from "./ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuBadge,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarTrigger,
  useSidebar,
} from "./ui/sidebar";

type WorkspaceSidebarProps = { unit?: Unit; preview: boolean; accountName?: string };

export function WorkspaceSidebar({ unit, preview, accountName }: WorkspaceSidebarProps) {
  return (
    <Sidebar collapsible="offcanvas">
      <UnitSwitcher unit={unit} preview={preview} />
      <UnitNavigation unit={unit} />
      <AccountFooter preview={preview} accountName={accountName} />
    </Sidebar>
  );
}

function UnitSwitcher({ unit, preview }: Pick<WorkspaceSidebarProps, "unit" | "preview">) {
  const { setOpenMobile } = useSidebar();
  const navigate = useNavigate();
  const units = useQuery({ ...unitsOptions(), enabled: !preview && !!sessionSnapshot() });
  const close = () => setOpenMobile(false);

  return (
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
  );
}

function UnitNavigation({ unit }: Pick<WorkspaceSidebarProps, "unit">) {
  const { setOpenMobile } = useSidebar();
  const location = useLocation();
  const close = () => setOpenMobile(false);
  const links = [
    { page: "overview", title: "Overview", icon: Home },
    { page: "personnel", title: "Personnel", icon: Users },
    { page: "ranks", title: "Ranks", icon: ChevronsUp },
  ] as const;

  return (
    <SidebarContent>
      <SidebarGroup className="workspace-sidebar-group">
        <nav aria-label="Unit navigation">
          <SidebarMenu className="gap-1">
            {unit ? (
              <>
                {links.map(({ page, title, icon: Icon }) => (
                  <SidebarMenuItem key={page}>
                    <SidebarMenuButton asChild isActive={location.pathname.endsWith(`/${page}`)}>
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
                  <Collapsible defaultOpen className="administration-menu">
                    <CollapsibleTrigger asChild>
                      <SidebarMenuButton>
                        <Settings />
                        <span>Administration</span>
                        <ChevronDown className="administration-chevron" />
                      </SidebarMenuButton>
                    </CollapsibleTrigger>
                    <CollapsibleContent>
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
                    </CollapsibleContent>
                  </Collapsible>
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
  );
}

function AccountFooter({
  preview,
  accountName,
}: Pick<WorkspaceSidebarProps, "preview" | "accountName">) {
  const { setOpenMobile } = useSidebar();
  const navigate = useNavigate();
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

  return (
    <SidebarFooter className="workspace-sidebar-footer">
      <div className="account">
        <Avatar name={accountName ?? "Account"} size="small" />
        <div>
          <strong>{accountName ?? "Account"}</strong>
          <span>{preview ? "Preview data" : "Signed in"}</span>
        </div>
        {!preview && (
          <Button
            variant="ghost"
            size="icon"
            type="button"
            className="icon-button"
            aria-label="Sign out"
            title="Sign out"
            onClick={() => void signOut()}
          >
            <LogOut size={17} />
          </Button>
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
            <SiGithub size={17} aria-hidden="true" />
          </a>
          <a
            href="https://discord.gg/gc3C2WR2cv"
            className="icon-button"
            aria-label="Discord (link coming soon)"
            title="Discord · coming soon"
            target="_blank"
            rel="noopener noreferrer"
          >
            <SiDiscord size={17} aria-hidden="true" />
          </a>
          <SidebarTrigger className="md:hidden" aria-label="Close navigation" />
        </div>
      </div>
    </SidebarFooter>
  );
}
