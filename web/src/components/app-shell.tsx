import type { ReactNode } from "react";

import type { Unit } from "../lib/types";

import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "./ui/breadcrumb";
import { SidebarInset, SidebarProvider, SidebarTrigger } from "./ui/sidebar";
import { TooltipProvider } from "./ui/tooltip";
import { WorkspaceSidebar } from "./workspace-sidebar";

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
  return (
    <>
      <a className="skip-link" href="#main-content">
        Skip to content
      </a>
      <WorkspaceSidebar unit={unit} preview={preview} accountName={accountName} />
      <SidebarInset className="workspace-body">
        <header className="topbar">
          <SidebarTrigger aria-label="Toggle navigation" />
          <Breadcrumb>
            <BreadcrumbList className="breadcrumb">
              {unit && (
                <>
                  <BreadcrumbItem>{unit.display_name}</BreadcrumbItem>
                  <BreadcrumbSeparator />
                </>
              )}
              <BreadcrumbItem>
                <BreadcrumbPage>{label}</BreadcrumbPage>
              </BreadcrumbItem>
            </BreadcrumbList>
          </Breadcrumb>
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
