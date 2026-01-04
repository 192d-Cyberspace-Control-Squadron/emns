---
icon: lucide/rocket
---

# Getting Started

## Introduction

borg-agent is a agent for the borg ecosystem. It purpose is to report, install, update, and uninstall software.

1) Agent-pull flow (simple + MECM-ish)

Every N minutes the agent does:

- POST /v1/checkin (device identity + basic facts).
- POST /v1/inventory (installed software snapshot or delta).
- GET /v1/policy?device_id=... (returns “assignments”).

For each assignment:

- evaluate applicability locally (rules).
- if needed: GET /v1/content/{payload_id}/url (signed URL).
- download → verify sha256 → stage in cache.
- run installer (silent).
- POST /v1/status (per attempt + logs summary).
- Sleep/backoff.

This avoids inbound firewall and scales well.
