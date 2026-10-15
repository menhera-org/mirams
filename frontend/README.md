# MIRAMS Frontend

Successful logins save the username and API token in browser `localStorage` under
`mirams.session`. The app restores this login on startup, so it survives page
reloads and browser restarts. Logout removes the saved login. If browser storage
is unavailable, the login stays in memory for the current page.

The server build bundles the frontend with Dioxus and copies `frontend/assets`
into the root of `frontend-dist`, alongside the generated `index.html`. This
preserves stable URLs such as `/main.css` and `/icon.svg`, including their Brotli
variants. The files in `frontend-dist` are embedded in the server binary and
included in the release package.

The ASN, IPv4, and IPv6 pages list top-level assignments. Every assignment detail
page shows its children and ancestor breadcrumbs, with actions to add children,
edit metadata and visibility, move the whole subtree to a valid destination, or
delete the subtree. Dropdowns share the app's colors, typography, focus states,
and disabled states. See the root README for the unified API and schema 3
migration behavior.
