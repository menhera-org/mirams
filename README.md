# MIRAMS

Menhera.org Internet Resources Assignment Management System

Supported:

- Management of assignments for ASN, IPv4 & IPv6 addresses.


## Installation

```bash
cargo install mirams
mirams -d path/to/mirams.db server -l 127.0.0.1:3001 </dev/null >/dev/null 2>&1 &
mirams -d path/to/mirams.db user-set-password --username some-user --password <SECRET_PASSWORD>

# log in as some-user!
```
