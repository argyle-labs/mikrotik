# MikroTik Plugin — Device Setup

Operator notes for connecting orca's `mikrotik` plugin to a RouterOS device.
This plugin manages RouterOS devices over the RouterOS API; the notes below
cover only what is needed to make that connection work.

---

## Prerequisites

- A RouterOS device reachable from the host running orca.
- The RouterOS **API service** enabled (default port **8728**, or **8729**
  for API-SSL).
- A user account on the device with permission to perform the operations you
  intend to manage through the plugin.

---

## 1. Enable the RouterOS API

The plugin talks to RouterOS over its API service. Enable it on the device
(via WinBox, WebFig, or SSH):

```routeros
# Enable the plain API service on the default port
/ip service set api disabled=no port=8728

# Or enable API-SSL (recommended when the device is reachable off-link)
/ip service set api-ssl disabled=no port=8729
```

Restrict the service to trusted source networks so the API is not exposed
more widely than necessary:

```routeros
/ip service set api address=<orca-host-or-subnet>
```

Confirm the service is listening:

```routeros
/ip service print
```

---

## 2. Create an API user

Use a dedicated account for the plugin rather than the built-in `admin`
account. Grant it only the group/permissions it needs:

```routeros
/user add name=orca-api password="<strong-password>" group=full
```

For read-only monitoring, use a group with reduced privileges (e.g. `read`)
instead of `full`.

---

## 3. Configure the plugin connection

The plugin needs the device's API endpoint and credentials:

| Setting    | Value                                      |
|------------|--------------------------------------------|
| Host / IP  | The RouterOS device's management address   |
| Port       | `8728` (API) or `8729` (API-SSL)           |
| Username   | The API account created above              |
| Password   | That account's password                    |
| TLS        | Enable when using API-SSL (port 8729)      |

Store the password as an orca secret rather than in plaintext configuration.

---

## 4. Verify connectivity

From the orca host, confirm the API port is reachable before wiring up the
plugin:

```bash
# Plain API
nc -vz <device-ip> 8728

# API-SSL
nc -vz <device-ip> 8729
```

If the port is filtered, check the RouterOS firewall and the `address=`
restriction on the API service.

---

## Troubleshooting

| Problem                                | Check                                                                 |
|----------------------------------------|-----------------------------------------------------------------------|
| Connection refused on 8728/8729        | API service disabled — enable it with `/ip service set api disabled=no`. |
| Connection times out                   | RouterOS firewall or the `address=` restriction is blocking the host. |
| Authentication fails                   | Verify the API username/password and that the user group grants access. |
| TLS handshake errors on 8729           | Ensure API-SSL is enabled and a valid certificate is assigned to the service. |

### Useful RouterOS diagnostics

```routeros
# Show API service state, port, and address restriction
/ip service print

# Show configured users and their groups
/user print

# Show firewall rules that may block the API port
/ip firewall filter print
```
