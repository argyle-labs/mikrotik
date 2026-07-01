# MikroTik Switch Setup — <your-domain> Homelab

**Switches:** 2× MikroTik L2 switches (RouterOS)
**Role:** VLAN-aware L2 switching only — no routing, no DHCP, no firewall
**Router:** OPNsense at `<ip>` handles all routing, DHCP, firewall, VPN

---

## Management Reality Check

OPNsense **cannot manage** MikroTik switches or UniFi APs. There is no OPNsense plugin or built-in feature for this. Each device has its own management interface:

| Device | Management Method | URL / Access |
|--------|------------------|-------------|
| MikroTik switch | WebFig (browser) or WinBox | `http://<switch-ip>` or WinBox to switch IP |
| MikroTik switch | SSH (RouterOS CLI) | `ssh admin@<switch-ip>` |
| UniFi APs | UniFi controller | `https://<ip>:8443` |
| OPNsense | Web UI + SSH | `https://<ip>` / `ssh root@<ip>` |

What OPNsense **can** do with the switches:
- See their MAC addresses and IPs in DHCP leases (give them static leases)
- Monitor via SNMP if you enable SNMP on the switches (optional — see [Part 5](#part-5-snmp-optional))
- Firewall/control traffic to/from switch management IPs

**Bottom line:** configure the switches correctly once, then you rarely need to touch them again. OPNsense handles everything meaningful.

---

## Table of Contents

- [Part 1: Initial Access](#part-1-initial-access)
- [Part 2: Management IP and Static Lease](#part-2-management-ip-and-static-lease)
- [Part 3: VLAN Bridge Configuration](#part-3-vlan-bridge-configuration)
  - [Port Layout](#port-layout)
  - [Bridge and VLAN Setup Commands](#bridge-and-vlan-setup-commands)
- [Part 4: Second Switch (Daisy-Chained)](#part-4-second-switch-daisy-chained)
- [Part 5: SNMP (Optional — OPNsense Monitoring)](#part-5-snmp-optional)
- [Part 6: Security Hardening](#part-6-security-hardening)
- [Troubleshooting](#troubleshooting)

---

## Part 1: Initial Access

MikroTik switches ship with IP `<ip>` and username `admin` / no password.

**WebFig** (browser, works everywhere):
```
http://<ip>
```

**WinBox** (native GUI, Windows/Wine/macOS via Wine):
- Download from [mikrotik.com/download](https://mikrotik.com/download)
- Can discover switches by MAC address even before IP is set

**SSH** (after IP is set):
```bash
ssh admin@<switch-ip>
```

> If the switch has already been configured by a previous owner or was reset, default credentials may vary. Check the sticker on the device for the default IP/password.

### Factory Reset

If you need to start fresh:
1. Hold the **Reset button** while powering on
2. Hold until the LED flashes, then release
3. Switch reboots to factory defaults (`<ip>`, `admin`, no password)

---

## Part 2: Management IP and Static Lease

Each switch needs a static management IP on the LAN (<ip>/24) so you can always reach it.

### Assign management IPs

Current switch addresses (static DHCP reservations in Kea, entries in [network-map.md](network-map.md)):

| Switch | Hostname | IP | MAC | Role |
|--------|----------|----|-----|------|
| Primary (uplink to OPNsense) | <host> | <ip> | <mac> | Main switch |
| Secondary (daisy-chained) | <host> | <ip> | <mac> | Secondary switch |

### Configure IP on the switch

On each switch via SSH or WebFig terminal:

```routeros
# Remove the default <ip> address
/ip address remove [find address="<ip>/24"]

# Add management IP (replace X with the switch number: 5 or 6)
# Primary switch: <ip>, hostname <host>
# Secondary switch: <ip>, hostname <host>
/ip address add address=<ip>/24 interface=bridge1   # adjust for sw2

# Set default gateway (OPNsense)
/ip route add gateway=<ip>

# Set DNS
/ip dns set servers=<ip>

# Set hostname (<host> or <host>)
/system identity set name=<host>
```

After setting the IP, reconnect to the new address. Static DHCP reservations are already set in Kea for both switches (see [config/dhcp/kea-reservations-lan.csv](../../config/dhcp/kea-reservations-lan.csv)).

---

## Part 3: VLAN Bridge Configuration

This is the main configuration. The switches run in **bridge mode with VLAN filtering** — the standard RouterOS approach for L2 switching with VLANs.

VLANs to carry:

| VLAN ID | Network | Purpose |
|---------|---------|---------|
| 1 (native/untagged) | <ip>/24 | LAN — primary network |
| 20 (tagged) | <ip>/24 | IoT |
| 30 (tagged) | <ip>/24 | Guest |

### Port Layout

Label your ports before running commands. Common layouts:

**Primary switch (<host>):**

| Port | Connection | Mode |
|------|-----------|------|
| ether1 | OPNsense (uplink) | Trunk: native VLAN 1 + tagged 20, 30 |
| ether2 | Secondary switch (sw2) or AP | Trunk: native VLAN 1 + tagged 20, 30 |
| ether3–ether5 | UniFi APs | Trunk: native VLAN 1 + tagged 20, 30 |
| ether6–ether8 | Servers (<host>, <host>, <host>) | Access: VLAN 1 untagged only |
| ether9–ether10 | General hosts | Access: VLAN 1 untagged only |

Update this table with your actual port assignments before running commands.

### Bridge and VLAN Setup Commands

Run these on the switch via SSH. Adjust port names (`etherX`) to match your actual port assignments.

**Step 1: Create/verify bridge with VLAN filtering**

```routeros
# Check if bridge already exists
/interface bridge print

# If bridge1 doesn't exist, create it
/interface bridge add name=bridge1 vlan-filtering=no comment="main bridge"

# Important: leave vlan-filtering=no until all ports and VLANs are configured.
# Enabling it early will lock you out if the management IP isn't set up first.
```

**Step 2: Add all ports to the bridge**

```routeros
# Add trunk ports (uplink, inter-switch, AP ports)
# frame-types=admit-all means tagged and untagged frames are accepted
/interface bridge port
add bridge=bridge1 interface=ether1 pvid=1 frame-types=admit-all comment="uplink to OPNsense"
add bridge=bridge1 interface=ether2 pvid=1 frame-types=admit-all comment="to sw2 or AP"
add bridge=bridge1 interface=ether3 pvid=1 frame-types=admit-all comment="AP1"
add bridge=bridge1 interface=ether4 pvid=1 frame-types=admit-all comment="AP2"

# Add access ports (hosts, servers) — untagged VLAN 1 only
# frame-types=admit-only-untagged means only untagged frames in; tags stripped on egress
/interface bridge port
add bridge=bridge1 interface=ether6 pvid=1 frame-types=admit-only-untagged-and-priority-tagged comment="<host>"
add bridge=bridge1 interface=ether7 pvid=1 frame-types=admit-only-untagged-and-priority-tagged comment="<host>"
add bridge=bridge1 interface=ether8 pvid=1 frame-types=admit-only-untagged-and-priority-tagged comment="<host>"
```

**Step 3: Configure the VLAN table**

```routeros
# VLAN 1 (management/LAN)
# tagged=bridge1 means the CPU (switch management) can access VLAN 1
# untagged= lists all access ports that carry native VLAN 1
/interface bridge vlan
add bridge=bridge1 vlan-ids=1 \
    tagged=bridge1,ether1,ether2,ether3,ether4 \
    untagged=ether6,ether7,ether8,ether9,ether10

# VLAN 20 (IoT) — only trunk ports; no access ports for IoT on this switch
add bridge=bridge1 vlan-ids=20 \
    tagged=ether1,ether2,ether3,ether4

# VLAN 30 (Guest) — only trunk ports
add bridge=bridge1 vlan-ids=30 \
    tagged=ether1,ether2,ether3,ether4
```

**Step 4: Set management IP on the bridge (VLAN 1)**

```routeros
# The management IP should be on bridge1 directly (which participates in VLAN 1 via tagged=bridge1 above)
# <host>: <ip>, <host>: <ip>
/ip address add address=<ip>/24 interface=bridge1
/ip route add gateway=<ip>
/ip dns set servers=<ip>
```

**Step 5: Enable VLAN filtering**

```routeros
# Enable VLAN filtering — do this LAST after all ports and VLANs are configured
# This is the step that enforces VLAN isolation. Do NOT do this before step 3 or you lose access.
/interface bridge set bridge1 vlan-filtering=yes
```

**Step 6: Verify**

```routeros
# Check bridge VLAN table
/interface bridge vlan print

# Check port settings
/interface bridge port print detail

# Check your management IP is still reachable (ping from a LAN host)
# ping <ip>

# Check bridge is forwarding
/interface bridge host print
```

---

## Part 4: Second Switch (Daisy-Chained)

If <host> is connected to <host> via a trunk port (not directly to OPNsense), the setup is identical with one difference: the uplink port on <host> goes to <host>, not OPNsense.

On **<host>**, run the same commands as Part 3, but:
- `ether1` = trunk to <host> (not to OPNsense)
- Management IP: `<ip>/24`
- Hostname: `<host>`

The uplink port on <host> connects to a trunk port on <host> (ether2 in the example layout above). Both ends must be trunk ports carrying VLANs 1, 20, 30.

Verify the inter-switch trunk is passing all VLANs:
```routeros
# On <host> — check ether2 is in the VLAN table for all three VLANs
/interface bridge vlan print where tagged~"ether2"
```

---

## Part 5: SNMP (Optional)

Enabling SNMP on the switches lets you monitor port counters, link status, and traffic from network monitoring tools (e.g., Observium, LibreNMS, Grafana+SNMP exporter). OPNsense itself is not a monitoring platform, but it can receive SNMP traps if you configure it.

**Enable SNMP on each switch:**

```routeros
/snmp set enabled=yes contact="admin@<your-domain>" location="homelab"

# Set a community string (treat as a password — use something non-default)
/snmp community
set [ find default=yes ] name=homelab-ro addresses=<ip>/24 read-access=yes write-access=no
```

This restricts SNMP queries to your LAN subnet only.

**Optional: Add firewall rule on OPNsense to restrict SNMP**

In OPNsense under Firewall > Rules > LAN, the SNMP port (UDP 161) is not exposed to the WAN by default. No extra rule needed unless you want to explicitly block SNMP between VLANs (good idea — add a rule blocking IoT/Guest → SNMP on switch management IPs).

---

## Part 6: Security Hardening

```routeros
# Change admin password (run on each switch)
/user set admin password="<strong-password>"

# Add SSH public key (optional but recommended)
/user ssh-keys import user=admin public-key-file=flash/id_rsa.pub

# Disable services you don't need
/ip service
set telnet disabled=yes
set ftp disabled=yes
set www disabled=yes      # disables plain-HTTP WebFig; use WinBox or SSH instead
set api disabled=yes
set api-ssl disabled=yes
set winbox disabled=no    # keep if you use WinBox
set ssh disabled=no

# Restrict SSH and WinBox to LAN only
/ip service
set ssh address=<ip>/24
set winbox address=<ip>/24

# Disable neighbor discovery on trunk/external-facing ports (security best practice)
/ip neighbor discovery-settings set discover-interface-list=none

# Disable MAC server (WinBox discovery) on trunk ports
/tool mac-server set allowed-interface-list=none
/tool mac-server mac-winbox set allowed-interface-list=none
```

---

## Integration with UniFi APs

UniFi APs plug into trunk ports on the MikroTik (ether3/ether4 in the example layout). The switch passes all VLAN-tagged traffic through those ports, and the UniFi controller (at `<ip>:8443`) manages which SSID maps to which VLAN tag.

The MikroTik switch does not know about SSIDs. It just passes whatever tagged frames the AP sends. See [unifi-setup.md → Part 2](unifi-setup.md#part-2-mikrotik-switch-trunk-ports-for-aps) for the AP-specific trunk port configuration.

**AP port requirements (already covered in Part 3 above):**
- `frame-types=admit-all` (accept tagged frames from the AP)
- `pvid=1` (native VLAN for management traffic and the primary LAN SSID)
- Tagged in VLAN 20 and VLAN 30 in the bridge VLAN table

---

## Troubleshooting

| Problem | Check |
|---------|-------|
| Lost management access after enabling VLAN filtering | Connect via WinBox using MAC address (not IP) from the same L2 segment. Verify `tagged=bridge1` is set for VLAN 1 in the bridge VLAN table. |
| Hosts on VLAN 20 or 30 get no DHCP | Check the bridge VLAN table includes the uplink port as tagged for that VLAN. Check OPNsense VLAN interface and DHCP are enabled. |
| Can't ping switch from LAN | Confirm `bridge1` has the management IP assigned (`/ip address print`). Confirm VLAN 1 has `tagged=bridge1`. Confirm IP route to `<ip>` exists. |
| AP on trunk port can't reach IoT SSID clients | Verify the AP's switch port has VLAN 20 tagged in the bridge VLAN table. Verify `frame-types=admit-all` on that port. |
| Two switches can't pass VLAN traffic between them | The inter-switch port must be a trunk on **both** switches — tagged for VLANs 20 and 30, and included in VLAN 1 as tagged. Check both ends. |
| RouterOS version too old | `/system routeros update download` then `/system reboot` |

### Useful diagnostic commands

```routeros
# Show all bridge ports and their VLAN config
/interface bridge port print detail

# Show VLAN table
/interface bridge vlan print

# Show MAC address table (what's learned on each port)
/interface bridge host print

# Show interface traffic counters
/interface print stats

# Show active DHCP leases (if DHCP client is running for management)
/ip dhcp-client print

# Show current IP addresses
/ip address print

# Test connectivity from switch
/tool ping <ip>
```

---

## Notes on SwitchOS (SwOS)

If your MikroTik switch runs **SwOS** (CSS series, RB260 series) instead of RouterOS, the CLI commands above do not apply. SwOS uses a web-only interface at `http://<switch-ip>` with a different configuration model. Key differences:

- VLAN configuration is under **VLANs** tab in the web UI
- Port isolation is under **Ports** tab
- No SSH access (web UI only)
- No scripting support

The concepts (trunk ports, access ports, VLAN IDs) are the same — only the interface differs. Check `System > RouterOS` in WebFig or run `/system resource print` via SSH. If it returns output, it's RouterOS. SwOS devices have no SSH.
