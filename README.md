# MIRAMS

Menhera(R) Internet Resources Assignment Management System

MIRAMS manages ASN ranges and IPv4/IPv6 prefixes as assignments. Any assignment
can contain more assignments, with no fixed number of levels. Children must use
the same resource type, fit within their parent's range, and not overlap their
siblings. Top-level assignments may overlap.

Every assignment has a name, description, and Public/Private visibility. Private
assignments and all their descendants are visible only to logged-in users.
Assignments can be moved with their entire subtree to another containing
assignment, or back to the top level. Moves reject cycles and sibling overlaps.

## API

The unified API replaces the old space/pool/assignment endpoints:

- `GET /api/v1/assignment?resource_type=asn` lists top-level ASN assignments.
  `resource_type=ipv4` and `resource_type=ipv6` select the other types; omitting
  the filter lists every type.
- `GET /api/v1/assignment?parent_id=123` lists immediate children of assignment 123.
- `GET /api/v1/assignment/123` reads an assignment.
- `GET /api/v1/assignment/123/ancestors` lists ancestors, root first.
- `POST /api/v1/assignment` creates an assignment. For example:

  ```json
  {
    "parent_id": null,
    "name": "Private ASNs",
    "description": "Available ASNs",
    "visibility": "public",
    "resource_type": "asn",
    "asn_from": 65000,
    "asn_to": 65534
  }
  ```

  Set `parent_id` to an existing assignment ID to create a child. IPv4 requests
  use `resource_type: "ipv4"`, `ipv4_prefix` (four address octets), and
  `ipv4_prefix_len`. IPv6 requests use `resource_type: "ipv6"`, `ipv6_prefix`
  (sixteen address octets), and `ipv6_prefix_len`. Prefixes must be valid network
  addresses with no host bits. A single ASN is a range with equal start and end.
- `PUT /api/v1/assignment/123` updates metadata:
  `{"name":"Example","description":"Details","visibility":"private"}`.
  Omit visibility to preserve it. This endpoint does not resize ranges or move
  assignments.
- `GET /api/v1/assignment/123/move-targets` lists valid destinations.
- `POST /api/v1/assignment/123/move` with `{"parent_id":456}` moves the assignment
  and its descendants. Explicit `{"parent_id":null}` promotes it to the top
  level. IDs, ranges, metadata, and descendants are retained by a move.
- `DELETE /api/v1/assignment/123` deletes the complete subtree.

Responses use `assignment` for one object and `assignments` for a list. Writes
and destination lists require a bearer token from `POST /api/v1/login`. Public
reads remain available without authentication. Password changes revoke the
user's existing tokens.

## Existing databases

The first database open automatically migrates schemas 1 and 2 to schema 3 in
one transaction. Old spaces become roots, pools become their children, and
assignments become grandchildren. Names, descriptions, ranges, visibility, and
parent relationships are preserved. Single ASN assignments become one-number
ranges, and legacy IP prefixes with host bits are normalized to their network
addresses. User accounts and valid API tokens are retained; the schema 1 account
repair still keeps the latest password for duplicate usernames and revokes
those duplicate accounts' tokens.

Assignment IDs may change because the nine old tables had independent ID
namespaces. The old API endpoints and detail-page URLs are replaced by the
unified API and `/assignment/:id/` pages. Reopening a migrated database preserves
the new IDs and does not repeat the migration. Older server versions cannot open
schema 3 databases.

Historical sibling overlaps are preserved during migration; new assignments and
moves enforce the overlap rules. If a legacy child falls outside its parent,
migration stops with the offending resource type and old ID, and the transaction
rolls back without modifying the original database.

## Installation

```bash
cargo install mirams
mirams -d path/to/mirams.db server -l 127.0.0.1:3001 </dev/null >/dev/null 2>&1 &
mirams -d path/to/mirams.db user-set-password --username some-user --password <SECRET_PASSWORD>

# log in as some-user!
```
