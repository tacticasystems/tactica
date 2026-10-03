# Generated resource routes

`createResourceRoutes(parent, path, idParam, definition)` creates sibling list,
detail, edit and delete-confirmation routes. The browser path and API template
can differ. Register `routes` for all operations, or select `list`, `view`, `edit`
and `remove` individually. Members does not register deletion because the data
source has no member deletion operation.

The members configuration lives in `src/resources/members.tsx`. It supplies its
columns, name/search functions and data source, then overrides detail and editing:

```tsx
const members: ResourceDefinition<Member, MemberContext> = {
  apiBase: "/units/{unit_id}/members",
  label: "Member",
  pluralLabel: "Members",
  name: (member) => member.display_name ?? member.username,
  fields: [
    {
      id: "member",
      label: "Member",
      in: ["list"],
      read: (member, context) => <MemberName member={member} unitId={context.unitId} />,
    },
  ],
  useResource: useMembersResource,
  editView: EditUnitMemberRoute,
};
const memberRoutes = createResourceRoutes(unitRoute, "/members", "memberId", members);
```

`read(record, context)` returns any React node. Callbacks needing only the record
can omit their second argument. `in` selects list columns, detail fields or edit
inputs. For automatic editing, include `"edit"` and supply `edit.value(record)`
and `edit.write(text)` returning a partial update. Only changed fields are sent;
`edit.required` opts into required input validation. Use an editor override for
relationships, structured inputs or field-specific permissions.

`useResource` keeps session/unit query scope, preview data sources, related queries
and permissions with the resource. It returns `source`, `queryKey` and `context`.
Optional `pending`, `error`, and `refetch` coordinate data required by field
renderers. `description`, `caption` and `total` customize the generated list;
`searchText` enables filtering the current page. Collections use the existing
20-record pagination convention. Generic list rows have a View action; set
`listActions: false` when a field already contains its own link.

`listView`, `viewView`, `editView` and `deleteView` replace the entire route view.
Overrides own their queries and mutations; the default resource hook does not
run for them. Members uses the generated list and preserves the specialized
member detail/editor, including rank/role authorization and draft reconciliation.

For plain REST collections, `createResourceSource(apiBase, params, collectionKey,
request)` resolves and encodes `{params}`, reads paginated envelopes and implements
GET/PATCH/DELETE. Bind `request` to the existing session client. Continue to supply
the preview data source when in preview mode. `key` defaults to the API template's
last segment; `itemKey` defaults to `${key}-item`. Override those when an existing
editor invalidates established cache keys, as members does with `itemKey: "member"`.

Automatic mutations require both a data-source method and explicit
`canEdit(record)` or `canDelete(record)` permission. The checks apply to direct
URLs as well as buttons; the server remains responsible for authorization.
Editing guards dirty drafts and pending saves. Deletion requires confirmation
on its own page and blocks navigation during the request. Mutations refresh the
supplied query scope, including after a failed request that may have committed.
