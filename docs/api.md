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
- `RoleSummary`: `id`, `unit_id`, `display_name`, `description`, `permissions`.

The member-role response contains role IDs that clients can resolve from
the unit's role listing. A member must belong to the unit in the URL. A member
from another unit returns 404, even when the caller belongs to both units.
User email addresses and authentication fields are never included in the roster.

## Pagination and errors

All collection reads above accept `offset` (default 0, minimum 0) and `limit`
(default 10, range 1–100). They sort by ID ascending; member-role assignments
sort by member ID and role ID. Empty collections and pages beyond the end return
200 with an empty array. Counts describe memberships, not the current page.

- 400: malformed IDs/query parameters or invalid pagination bounds.
- 401: missing/invalid token, or a token for a missing/inactive user.
- 403: authenticated caller is not a user with membership in the requested unit.
- 404: unit does not exist, or the member is absent from that unit.

Application errors use the existing `{ "code": "...", "message": "..." }`
format. Axum's malformed path/query rejections retain their default text bodies.

## Next endpoint batches

The endpoints below are planned, not implemented.

1. Role management: `POST /units/{unit_id}/roles`,
   `PATCH /units/{unit_id}/roles/{role_id}`, and
   `DELETE /units/{unit_id}/roles/{role_id}`.
2. Role assignment: `PUT /units/{unit_id}/members/{member_id}/roles/{role_id}`
   and `DELETE` on the same path. PUT should be idempotent.
3. Member lifecycle: join/invite/accept/leave flows, then rank changes.

All paths in these batches use the `/api/v1` prefix. Before implementing writes,
define permission bits, management checks, and a durable unit owner/admin
relationship. The current creator rank is not a sufficient authorization rule.
Decide how to prevent members from granting permissions they do not hold and how
to preserve at least one administrator. Member lifecycle also needs a decision
on open joining versus invitations/applications.

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
