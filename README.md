<p align="center">
  <img src="assets/icon-256.png" width="120" alt="mikrotik" />
</p>

# mikrotik

MikroTik RouterOS powers MikroTik routers and switches; managed here over its API.

A first-party [orca](https://github.com/argyle-labs/orca) plugin (appliance integration).

This plugin **connects orca to an existing mikrotik install** — there's nothing to deploy here. Stand up mikrotik from the upstream project, then point orca at it.

---

## Run it without orca

Install mikrotik per the upstream project: <https://mikrotik.com/>. It listens on port `8728` by default; this plugin talks to that endpoint (host, credentials/token) — no container is deployed.


See [mikrotik-setup.md](docs/mikrotik-setup.md) for worked operator notes.

## With orca

orca drives this plugin through its generic surface — rich, mikrotik-specific data comes back in the typed `service.status` payload, never bespoke tools.

## Layout

- `src/` — the plugin (pure Rust): the `ServiceBackend` descriptor + `configure` / `status`.
- `docs/` — standalone operator notes.
- [CAPABILITIES.md](CAPABILITIES.md) — the service-backend contract checklist.
- `assets/` — plugin icon.
