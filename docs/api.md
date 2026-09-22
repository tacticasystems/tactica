# Unit API

## Public unit reads

| Method | Path | Response |
| --- | --- | --- |
| GET | `/api/v1/units` | `{ "units": [UnitSummary] }` |
| GET | `/api/v1/units/{unit_id}` | `UnitSummary` |

`UnitSummary` includes `id`, `slug`, `display_name`, `icon_url`, `banner_url`,
`biography`, and `member_count`. The count is the total number of memberships in
that unit, independent of pagination. Units without members report zero.
Listing counts are fetched in one grouped query for the units on the page.

## Reads for unit members

These endpoints require a bearer token for an active user who currently belongs
to the requested unit. Membership in another unit is insufficient. Service
principals and non-member superusers do not bypass this rule.

| Method | Path | Response |
| --- | --- | --- |
| GET | `/api/v1/units/{unit_id}/members` | `{ "members": [MemberSummary] }` |
| GET | `/api/v1/units/{unit_id}/ranks` | `{ "ranks": [RankSummary] }` |
| GET | `/api/v1/units/{unit_id}/roles` | `{ "roles": [RoleSummary] }` |
| GET | `/api/v1/units/{unit_id}/members/{member_id}/roles` | `{ "role_ids": [UUID] }` |

- `MemberSummary`: `id`, `user_id`, `unit_id`, `rank_id`.
- `RankSummary`: `id`, `unit_id`, `slug`, `display_name`, `icon_url`, `description`.
- `RoleSummary`: `id`, `unit_id`, `display_name`, `description`, `permissions`, `position`, `kind`.

The member-role response contains role IDs that clients can resolve from
the unit's role listing. It always includes the implicit Everyone role for a
current member (subject to pagination), alongside explicit role assignments. A member must belong to the unit in the URL. A member
from another unit returns 404, even when the caller belongs to both units.
User email addresses and authentication fields are never included in the roster.

## Pagination and errors

All collection reads above accept `offset` (default 0, minimum 0) and `limit`
(default 10, range 1–100). Units, members, and ranks sort by ID ascending. Roles sort by position
descending (highest first); member-role assignments sort by member ID and role ID. Empty collections and pages beyond the end return
200 with an empty array. Counts describe memberships, not the current page.

- 400: malformed IDs/query parameters or invalid pagination bounds.
- 401: missing/invalid token, or a token for a missing/inactive user.
- 403: authenticated caller is not a user with membership in the requested unit.
- 404: unit does not exist, or the member is absent from that unit.

Application errors use the existing `{ "code": "...", "message": "..." }`
format. Axum's malformed path/query rejections retain their default text bodies.

## Role management

All writes require an active user with current membership in the unit. Role
creation, editing, deletion, and reordering require `ManageRoles`; assigning and
removing roles require `AssignRoles`. Administrator grants both permissions,
and the unit owner implicitly has all permissions. Ranks and global superuser
status do not grant unit permissions. The hierarchy rules below also apply.

| Method | Path | Response |
| --- | --- | --- |
| POST | `/api/v1/units/{unit_id}/roles` | 201 with `RoleSummary` |
| PATCH | `/api/v1/units/{unit_id}/roles/{role_id}` | 200 with `RoleSummary` |
| PATCH | `/api/v1/units/{unit_id}/roles/order` | 200 with `ListRolesResponse` |
| DELETE | `/api/v1/units/{unit_id}/roles/{role_id}` | 204 |
| PUT | `/api/v1/units/{unit_id}/members/{member_id}/roles/{role_id}` | 204 |
| DELETE | `/api/v1/units/{unit_id}/members/{member_id}/roles/{role_id}` | 204 |

POST accepts `display_name`, optional `description`, and `permissions` (default
0). Names are trimmed and must contain 1–100 characters. Descriptions may contain
up to 2000 characters. Permissions must contain only the defined bits (0–127). Unknown request fields
are rejected. Duplicate role names within a unit return 409.

PATCH accepts the same fields, all optional. Omitted fields are preserved;
`description: null` clears the description. An empty patch returns the unchanged
role with an updated modification timestamp. Role IDs and unit IDs cannot be
changed by PATCH. A missing role returns 404 on PATCH or DELETE.

Assignment PUT is idempotent, including simultaneous requests. Assignment DELETE
returns 204 if the assignment is already absent. Both require the member and
role to exist in the unit named in the URL, otherwise they return 404. Deleting
a role cascades to its assignments.

Every unit has a required `owner_id`, recorded when it is created. Ownership is
separate from role assignments, so deleting roles cannot strip the owner's
management authority. Generic unit updates do not transfer ownership. Owner
account deletion is restricted while it owns units. This migration assumes an
empty, undeployed database; it does not infer ownership for existing rows.

## Built-in roles

Every unit is created atomically with two roles, identified by the stable `kind`
field rather than their display names. Ordinary roles have `kind: "custom"`.

| Kind | Name | Initial permissions | Behavior |
| --- | --- | --- | --- |
| `administrator` | Administrator | Administrator (1) | Immutable, undeletable, pinned highest; explicitly assignable |
| `everyone` | Everyone | None (0) | Immutable name/description, editable permissions, undeletable, pinned at position 0; applies to every member |

Both descriptions are initially null and immutable. Administrator rejects all
PATCH requests, including permission edits; Everyone accepts permission edits
only. Violating built-in protections returns 403, even for the owner. Clients
cannot set `kind` in create/update requests.

Everyone is implicit: no individual assignment rows are created, and attempts
to assign or remove it return 403. Current and future members receive its
permissions automatically; former members do not. Its permissions still obey
ManageRoles, hierarchy, and grant restrictions when edited. Ordinary role edits
and assignments continue to obey the same hierarchy rules.

Administrator is not automatically assigned to the owner or any other member.
The owner always gets authority from `owner_id`. Assigning Administrator grants
all permission checks to another member while preserving role hierarchy. Since
it is pinned highest, only the owner can assign or remove the built-in
Administrator role. Custom roles may still grant the Administrator permission
bit and obey their own positions.

The built-in role migration follows the project's empty-database assumption.
Unit deletion cascades to both built-ins and their explicit assignments.

## Permission bits and hierarchy

| Permission | Bit | Value | Meaning |
| --- | --- | --- | --- |
| Administrator | 0 | 1 | Implicitly grants every permission, including future permissions |
| ManageUnit | 1 | 2 | Manage the unit profile and settings |
| ManageRoles | 2 | 4 | Create, edit, delete, and reorder roles |
| AssignRoles | 3 | 8 | Assign and remove roles on members |
| ManageRanks | 4 | 16 | Manage organizational rank definitions |
| AssignRanks | 5 | 32 | Assign ranks to members |
| ManageMembers | 6 | 64 | Edit display fields on member profiles |

Masks are combined using bitwise OR across **all** the member's roles in that
unit. Ranks are purely organizational. There is no implicit permission from
membership or from holding all ordinary bits: Administrator is a separate bit.
Unknown bits and negative masks are rejected. The shared `tactica-permissions`
crate is re-exported by `tactica_api_types::v1::roles` for Rust clients.

`ManageUnit`, `ManageRanks`, `AssignRanks`, and `ManageMembers` are defined for
future write endpoints. `ManageMembers` does not mean kick, ban, or manage role
assignments. The currently implemented writes enforce ManageRoles/AssignRoles.

Higher `position` values rank above lower values. Each unit has a unique order.
A member can edit, delete, assign, remove, or reorder only roles **strictly
below** their highest assigned role. A role with zero permissions still counts
when determining that ceiling. Administrator grants permission checks but does
not bypass this hierarchy. The owner bypasses the hierarchy and grant limits.

Creating a custom role requires ManageRoles and inserts it at position 1,
just above Everyone, shifting existing non-Everyone roles upward. A caller with
only Everyone cannot create a role above their own position. A non-owner can grant only permission bits they already
hold. Editing a role checks newly added bits against the caller's effective
permissions; unchanged bits may be preserved and existing bits may be removed.
Assignment checks the role's position, not whether the caller holds each bit
in that already-existing role. This follows the agreed Discord-style distinction
between role hierarchy and granting new permissions. See
[Discord's role permission rules](https://support.discord.com/hc/en-us/articles/214836687-Discord-Roles-and-Permissions).

`PATCH /api/v1/units/{unit_id}/roles/order` accepts
`{ "role_ids": ["lowest-role-id", "...", "highest-role-id"] }`.
The list must contain every role in the unit exactly once, including the two
built-ins; missing, duplicate, or foreign IDs return 400. Everyone must remain
first (lowest) and Administrator last (highest), including for owner requests. For non-owners, the caller's highest assigned role
and every role above it must remain at the same place in the order. Roles below
that boundary may be rearranged. Owner requests may rearrange all custom roles between the built-ins.
The response contains roles highest-first, matching GET. Position cannot be
changed via the ordinary role PATCH endpoint.

Authorization, hierarchy checks, and mutations run in one transaction with a
unit row lock. Concurrent role edits, assignments, and reorders are serialized
for that unit. Permissions are read from the database for every write, not
cached in JWTs; revocation is effective on the next request. The raw database
CRUD stores are for trusted internal use; request handlers use
`UnitRoleManagementStore` for atomic authorization and mutation.

The permission/order migration assumes an empty, undeployed database. Apply it
before using the new permission checks. It also constrains database permission
masks and role positions.

The next endpoint batch can use the remaining bits for unit profile/settings,
rank management, rank assignment, and member display fields. Member onboarding
and removal need separate rules, plus ownership transfer and protection against
removing the owner.

## Authentication sessions

| Method | Path | Request | Response |
| --- | --- | --- | --- |
| POST | `/api/v1/auth/login` | `{ "username": "...", "password": "..." }` | Token pair |
| POST | `/api/v1/auth/refresh` | `{ "refresh_token": "..." }` | Replacement token pair |
| POST | `/api/v1/auth/logout` | `{ "refresh_token": "..." }` | 204, no body |

A token pair contains `token_type: "Bearer"`, `access_token`, `refresh_token`,
and `expires_in: 300` (the access token lifetime in seconds). Login and refresh
responses include `Cache-Control: no-store` and `Pragma: no-cache`.
Refresh and logout do not require an access token; the refresh token is the
credential. Refresh tokens cannot be used as bearer access tokens.

Each login starts an independent session with a fixed 30-day expiration.
Refreshing does not extend that deadline. Refresh tokens contain 256 random
bits, and only SHA-256 hashes are stored in the database. Every successful
refresh consumes the presented token and returns a replacement. Clients must
save the replacement and serialize refresh requests for each login session.

Reusing a consumed token returns 401 and revokes that session, including its
latest replacement. Retried or concurrent refresh requests can therefore require
a fresh login. This follows the rotation/replay model described in
[RFC 9700 section 4.14.2](https://www.rfc-editor.org/rfc/rfc9700.html#section-4.14.2).
Unknown, expired, revoked, malformed, or inactive-user tokens also return 401.
An attempted refresh for an inactive account revokes its session permanently.
Deleted users' sessions are removed by cascading foreign keys.

Logout revokes the session identified by either its current token or a consumed
ancestor. It is idempotent and returns 204 for unknown tokens too. Other login
sessions are unaffected. Already-issued access JWTs remain valid until their
short expiration; logout does not maintain an access-token denylist.

The `refresh_sessions` and `refresh_tokens` migration must be applied before
using these endpoints. Consumed token hashes must be retained until their
session expires to detect replay. Expired sessions may be pruned by deleting
`refresh_sessions` rows with `expires_at <= CURRENT_TIMESTAMP`; their token
hashes cascade. Automated pruning is not included in this change.
