# MIRAMS Frontend

Successful logins save the username and API token in browser `localStorage` under
`mirams.session`. The app restores this login on startup, so it survives page
reloads and browser restarts. Logout removes the saved login. If browser storage
is unavailable, the login stays in memory for the current page.
