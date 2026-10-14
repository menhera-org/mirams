# MIRAMS

Menhera(R) Internet Resources Assignment Management System

Supported:

- Management of assignments for ASN, IPv4 & IPv6 addresses.
- Editing names, descriptions, and Public/Private visibility after creation.
- Overlapping assignment spaces with independent pools and assignments.

Authenticated `PUT` requests for spaces, pools, and assignments accept
`{"name": "Example", "description": "Details", "visibility": "private"}`.
Visibility can be `"public"` or `"private"`; omitting it preserves the current
setting. Private objects and descendants of private parents are visible only to
logged-in users. Pools must remain within their parent space and must not overlap
other pools in that space. Assignments must remain within their parent pool and
must not overlap other assignments in that pool.

API prefixes must have a valid length and use the network address (no host bits).
Nested API routes require the object to belong to every parent named in the URL.

Password changes revoke the user's existing API tokens. Existing databases
automatically migrate to schema version 2, enforcing unique usernames. If an
older password change created duplicate accounts, the migration keeps the latest
password and revokes tokens for that username; unrelated sessions remain valid.


## Installation

```bash
cargo install mirams
mirams -d path/to/mirams.db server -l 127.0.0.1:3001 </dev/null >/dev/null 2>&1 &
mirams -d path/to/mirams.db user-set-password --username some-user --password <SECRET_PASSWORD>

# log in as some-user!
```
