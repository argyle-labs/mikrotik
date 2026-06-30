# mikrotik — ServiceBackend contract checklist

Driven by the generic `service.*` surface (no per-plugin tools). `[ ]` =
scaffolded stub. Modalities: **device**.

## ServiceBackend methods
- [ ] `provider` / `modalities` / `default_port` (declared)
- [ ] `deploy(modality)` — docker/podman/lxc/vm as applicable
- [ ] `backup`
- [ ] `restore`
- [ ] `configure` — service-specific config
- [ ] `status` — health/diagnostics
- [ ] connect/sync handled generically by the toolkit (endpoint registry + peer sync)

## Deploy modalities
- n/a docker
- n/a podman
- n/a lxc
- n/a vm
- [ ] device API (no deploy)
- n/a host
