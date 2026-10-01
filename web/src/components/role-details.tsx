import { useState, type ReactNode } from "react";

import type { Access, Role } from "../lib/types";

import { RoleMembers } from "./role-members";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "./ui/tabs";

export function RoleDetails({
  role,
  access,
  pending,
  children,
}: {
  role?: Role;
  access: Access;
  pending: boolean;
  children: ReactNode;
}) {
  const [tab, setTab] = useState("permissions");
  const activeTab = role ? tab : "permissions";

  return (
    <Tabs className="role-details" value={activeTab} onValueChange={setTab}>
      {role && (
        <TabsList className="role-view-switch" aria-label="Role details">
          <TabsTrigger value="permissions">Permissions</TabsTrigger>
          <TabsTrigger value="members">Members</TabsTrigger>
        </TabsList>
      )}
      {/* Keep the editor mounted so switching tabs preserves unsaved permissions. */}
      <TabsContent value="permissions" forceMount hidden={activeTab !== "permissions"}>
        {children}
      </TabsContent>
      {role && (
        <TabsContent value="members">
          <RoleMembers key={role.id} role={role} access={access} pending={pending} />
        </TabsContent>
      )}
    </Tabs>
  );
}
