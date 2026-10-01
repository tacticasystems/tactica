import { useMutation } from "@tanstack/react-query";

import { queryClient } from "../lib/queries";
import type { Role } from "../lib/types";

import { useWorkspace } from "../components/workspace-context";

export function useRoleReorder() {
  const { unit, source, queryKey } = useWorkspace();

  return useMutation({
    mutationFn: (roleIds: string[]) => source.reorderRoles(unit.id, roleIds),
    onMutate: async (roleIds) => {
      await queryClient.cancelQueries({ queryKey: [...queryKey, "roles"] });
      const previous = queryClient.getQueryData<Role[]>([...queryKey, "roles"]);
      const byId = new Map(previous?.map((item) => [item.id, item]));
      queryClient.setQueryData<Role[]>(
        [...queryKey, "roles"],
        roleIds.map((id, index) => ({ ...byId.get(id)!, position: roleIds.length - index - 1 })),
      );
      return { previous };
    },
    onSuccess: async (updated) => {
      queryClient.setQueryData([...queryKey, "roles"], updated);
      await queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
    onError: (_error, _order, context) => {
      if (context?.previous) queryClient.setQueryData([...queryKey, "roles"], context.previous);
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "roles"] });
      void queryClient.invalidateQueries({ queryKey: [...queryKey, "access"] });
    },
  });
}
