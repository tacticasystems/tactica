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
| GET | `/api/v1/units/{unit_id}/roles/{role_id}/members` | `{ "member_ids": [UUID] }` |

- `MemberSummary`: `id`, `user_id`, `unit_id`, `rank_id`, `username`, `display_name`, `icon_url`, `role_ids`.
- `RankSummary`: `id`, `unit_id`, `slug`, `display_name`, `icon_url`, `description`.
- `RoleSummary`: `id`, `unit_id`, `display_name`, `description`, `permissions`, `position`, `kind`.

The member-role response contains role IDs that clients can resolve from
the unit's role listing. It always includes the implicit Everyone role for a
current member (subject to pagination), alongside explicit role assignments. A member must belong to the unit in the URL. A member
from another unit returns 404, even when the caller belongs to both units.
User email addresses and authentication fields are never included in the roster.
Roster `role_ids` include all explicit roles and implicit Everyone, ordered
highest first. Assignments are loaded in one batch for the roster page.

The role-member response lists assigned membership IDs in ascending order, which
clients can resolve against the unit roster. Everyone returns all current unit
members, including those without explicit bindings. A missing role or a role
from another unit returns 404. The same pagination and member-only access rules
apply.

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

The next endpoint batch can use the remaining bits for unit settings,
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


## Unit profile

In the web app, **Administration → Profile** lets the unit owner, Administrator,
or a member with **Manage unit** permission edit the display name, unit handle,
and biography. **Save changes** saves the edited fields; **Cancel**
restores the saved values. Unsaved edits trigger a confirmation when navigating
away and a browser warning on reload or close. Failed saves retain the draft,
and icon or banner uploads do not discard it. Members without permission and preview
visitors see read-only profile fields.

`PATCH /api/v1/units/{unit_id}` requires bearer authentication and the same
owner/Administrator/ManageUnit access as icon uploads. Permissions are checked
again inside the database transaction that updates the profile. A successful
request returns 200 with the updated `UnitSummary`.

| Field | Validation after trimming surrounding whitespace |
| --- | --- |
| `display_name` | String of 1–100 characters, without control characters; cannot be null |
| `slug` (unit handle) | Unique string of 1–100 lowercase ASCII letters, numbers, and single hyphens; no leading or trailing hyphens; cannot be null |
| `biography` | Optional string of at most 5,000 characters, without null characters |
| `banner_url` | Optional HTTPS URL with a host and no credentials, or a path beginning with `/` but not `//`; at most 2,048 bytes, without backslashes or control characters |

Omitted fields remain unchanged. Explicit `null` or a blank string clears
`biography` or `banner_url`. At least one field must be supplied. Unknown fields
are rejected: this endpoint only edits the four profile fields above. Icon and
banner uploads use dedicated controls and endpoints. The web app manages banners through the upload control.

Invalid profile values return 400, missing authentication 401, insufficient
permission 403, a missing unit 404, and an already-used handle 409. The web form
shows save feedback and explains handle conflicts or permission loss.

## File uploads

In the web app, open **Administration → Profile**, select an image under
**Unit icon** or **Unit banner**, and choose **Upload icon** or **Upload banner**.
Uploading requires unit ownership, Administrator,
or the **Manage unit** permission. The control shows file validation errors,
upload progress, and success or failure feedback; a successful upload immediately
refreshes the displayed icon or banner preview. Uploads save separately from
**Save changes** and preserve unsaved text in the profile editor above.

For current uploaded artwork, choose **Remove icon** or **Remove banner**, then
confirm removal or choose **Cancel**. Removal requires the same **Manage unit**
access as uploading and uses the file DELETE endpoint below. Successful removal
refreshes the profile preview without discarding unsaved text. If removal fails,
the confirmation shows the error and lets you retry.

File bytes are stored through `tactica_files::FileStorage`, with filesystem and
S3 implementations. PostgreSQL stores ownership, filenames, size and content
metadata; API URLs remain stable across backends. Run the database migrations
before enabling these routes, including `20261001130000_file_banners` for banner
uploads.

The executable defaults to `TACTICA_FILE_BACKEND=filesystem` and
`TACTICA_FILE_ROOT=./uploads`. Use a persistent directory writable only by the
service account. For S3, set `TACTICA_FILE_BACKEND=s3`, `TACTICA_S3_BUCKET` and
`TACTICA_S3_REGION`. Supply AWS credentials using the object_store AWS credential
providers (environment credentials, web identity or instance/container identity).
The service requires GetObject, PutObject and DeleteObject on `files/*` in the
existing bucket. Keep the bucket private; downloads pass through the API, so
bucket CORS and public ACLs are unnecessary. `AWS_ENDPOINT` can select an
S3-compatible service; use HTTPS outside local testing. Changing the backend
does not migrate existing objects: copy `files/*` to the new backend first.

All uploads use `multipart/form-data` containing exactly one field named `file`,
with a filename. Original filenames are metadata, never filesystem paths or S3
keys. Files use generated IDs under `files/<file_id>` in either backend.

| Method | Path | Access / result |
| --- | --- | --- |
| POST | `/api/v1/units/{unit_id}/files` | Active member; 201 file summary |
| GET | `/api/v1/units/{unit_id}/files?offset=0&limit=10` | Active member; paginated `{ "files": [...] }` |
| GET | `/api/v1/units/{unit_id}/files/{file_id}` | Active member; attachment download |
| DELETE | `/api/v1/units/{unit_id}/files/{file_id}` | Uploader or ManageUnit; 204 |
| POST | `/api/v1/units/{unit_id}/icon` | ManageUnit (including owner); 201 file summary and updates unit `icon_url` |
| GET | `/api/v1/units/{unit_id}/icon/{file_id}` | Public; PNG icon |
| POST | `/api/v1/units/{unit_id}/banner` | ManageUnit (including owner); 201 file summary and updates unit `banner_url` |
| GET | `/api/v1/units/{unit_id}/banner/{file_id}` | Public; PNG banner |

A summary contains `id`, `unit_id`, `uploaded_by`, `filename`, `content_type`,
`size`, `url`, and `created_at`. Deleting an uploader account preserves unit files
and artwork, setting `uploaded_by` to null; only unit managers may delete those
files afterward. Apply `20261001140000_nullable_file_uploader` to enable this policy.
Its rollback refuses to restore non-null attribution while deleted-account rows remain.
Attachment downloads require bearer
authentication, use `application/octet-stream`, `Content-Disposition: attachment`,
`nosniff` and `private, no-store`. Do not navigate directly to private URLs in a
browser: fetch them with the session bearer token and download the response blob.

Attachments are limited to 10 MiB. Icons accept PNG, JPEG or WebP up to 2 MiB and
2048×2048 pixels; they are decoded and re-encoded as PNG, scaled to fit 512×512.
Banners accept PNG, JPEG or WebP up to 5 MiB and 4096×4096 pixels; they are
decoded and re-encoded as PNG, scaled to fit 1920×1080 while preserving aspect ratio.
SVG and invalid images are rejected. Empty files and multiple fields are rejected.
Upload routes allow an additional 64 KiB of multipart overhead; ordinary API
requests retain their 1 MiB limit. Oversized requests return 413. Invalid input
returns 400, missing authentication 401, insufficient access 403, and missing or
cross-unit file IDs 404. The library's ApiState requires `with_file_storage` to
enable byte operations; otherwise those endpoints return 503.

Icon and banner writes recheck permissions and update the unit URL and metadata
atomically. Both are public profile artwork; previous uploads remain available by ID until
explicitly deleted using the file DELETE route (which requires ManageUnit for
icons and banners). Deleting the current artwork clears its unit URL; deleting
old artwork does not clear a newer icon or banner. Icons and banners are excluded
from attachment lists; retain returned IDs if you need to remove old artwork.

Storage and PostgreSQL cannot share a transaction. A failed metadata write
triggers object cleanup; deletion removes metadata first, making the object
inaccessible through the API, then deletes its bytes. Cleanup failures log the
file ID for retry. Process interruption, request timeout, or unit deletion can
leave unreferenced objects; operators must periodically reconcile `files/*`
against the `files` table, allowing a grace period for uploads in progress.
There is no automated orphan collector or per-unit storage quota yet.

Example (use a current access token):

```sh
curl -H "Authorization: Bearer $ACCESS_TOKEN" \
  -F 'file=@report.pdf' \
  "http://localhost:8080/api/v1/units/$UNIT_ID/files"
```


### Garage

Garage uses the S3 backend with path-style requests. Set these alongside the
normal API settings (the region must match Garage's `s3_api.s3_region`):

```dotenv
TACTICA_FILE_BACKEND=s3
TACTICA_S3_BUCKET=tactica-uploads
TACTICA_S3_REGION=garage
AWS_ENDPOINT=https://s3.example.test
AWS_ACCESS_KEY_ID=<Garage access key ID>
AWS_SECRET_ACCESS_KEY=<Garage secret key>
```

Create the bucket and grant the Garage key read/write permissions first. Use
`AWS_ALLOW_HTTP=true` only for a local HTTP development endpoint. Keep secrets out
of committed files. See the [Garage quick start](https://garagehq.deuxfleurs.fr/documentation/quick-start/).

### Storage integration tests

`cargo test -p tactica-files` runs a shared storage contract against a temporary
filesystem directory and a real `dxflrs/garage:v2.3.0` container managed by
`testcontainers`. Docker must be available, just as for the PostgreSQL tests.
Garage starts with a single-node layout and a disposable bucket/key, waits for
its health endpoint, and uses randomly mapped host ports. Tests do not use or
change the developer's AWS credentials, and containers/directories are removed
on drop.

Both backends exercise missing reads, idempotent deletion, binary round trips,
readback through a separate client, replacement, object isolation and empty
objects. Garage additionally verifies that an invalid secret cannot write and
that authentication errors are not reported as missing files.

`cargo test -p tactica-api --test files` tests upload limits, icon normalization,
membership and ownership checks, cross-unit isolation, downloads and icon URL
updates using PostgreSQL plus filesystem storage.
