# Xiaojin OS

Minimal bare-metal operating system. NanoPi R5C / RK3568 is the first board
profile, not the whole project boundary.

## Architecture Boundary

R5C currently owns the first concrete boot path, but shared Xiaojin OS features
must stay above the platform layer when possible:

- `XJNL` image loading, `XJQ1` queries, `XJEV` events, DHCP control, A/B OTA
  state, and XiaojinDB should remain architecture-neutral.
- RK3568 code owns ARM64 exception vectors, ARM generic timer, GIC/PSCI facts,
  DWMMC, PCIe bring-up, and RTL8125 details.
- The planned AMD 8845HS-class x86_64 target should use UEFI, APIC/x2APIC,
  TSC/HPET/APIC timer, PCIe NIC discovery, and NVMe/SATA/USB block drivers.
- Local LAN hot update remains the fastest development path on every target.
  A GCP maintenance service is a remote command/artifact plane, not a replacement
  for direct `xjctl net-dev` and `xjctl net-ota`.

## Current Runtime

The kernel boots into a UART control shell:

```text
xj> 
```

Available commands:

- `help`
- `caps`
- `about`
- `mem`
- `regs`
- `time`
- `uptime`
- `sleep <ms>`
- `bench [iters]`
- `rng status`
- `rng sample <n> confirm`
- `net facts`
- `net ips`
- `net ip <if> <ip> confirm`
- `net dhcp <if> confirm`
- `net dns <if> <name> confirm`
- `net tcp <if> <host|ip> <port> confirm`
- `net http <if> <host|ip> <port> <path> confirm`
- `net tls <if> <host|ip> <port> [sni] confirm`
- `net tls-pin <if> <host|ip> <port> [sni|-] <sha256> confirm`
- `net loop confirm`
- `net listen confirm`
- `net sniff confirm`
- `pcie facts`
- `pcie reset-pins`
- `pcie status`
- `pcie phy-status`
- `pcie phy-init confirm`
- `pcie apb-status confirm`
- `pcie cfg-probe confirm`
- `pcie iatu-probe confirm`
- `pcie cfg-scan confirm`
- `pcie bar-scan confirm`
- `pcie bar-map confirm`
- `pcie rtl-probe confirm`
- `pcie rtl-facts confirm`
- `pcie rtl-init confirm`
- `pcie rtl-tx-test confirm`
- `pcie rtl-rx-poll confirm`
- `pcie cru-status`
- `pcie cru-enable confirm`
- `pcie rc-init confirm`
- `pcie ltssm-enable confirm`
- `pcie ltssm-disable confirm`
- `pcie link-start confirm`
- `pcie reset-deassert confirm`
- `pcie reset-assert confirm`
- `xload <addr|auto> <size> <crc32> [go]`
- `hexdump <addr> [n]`
- `peek32 <addr>`
- `mmio list`
- `mmio read32 <region> <offset>`
- `db facts`
- `db status confirm`
- `db init confirm`
- `db catalog confirm`
- `db catalog-init confirm`
- `db page <n> confirm`
- `db wal-append confirm`
- `db table confirm`
- `db table-init confirm`
- `db put <key> <value> confirm`
- `db get <key> confirm`
- `db btree confirm`
- `db btree-init confirm`
- `db bput <key> <value> confirm`
- `db bget <key> confirm`
- `db bscan <start> <limit> confirm`
- `db nskey <table> <row> confirm`
- `db tput <table> <row> <value> confirm`
- `db tget <table> <row> confirm`
- `db bleaf <page> confirm`
- `db event-init confirm`
- `db event-status confirm`
- `db events <start-seq> <limit> confirm`
- `db event <seq> confirm`
- `last-exception`
- `fault`
- `abort confirm`
- `reboot`

`xiaojin-os.lisp` is the current thin authoring SSOT for board facts and shell
commands. The kernel does not parse Lisp at runtime; those facts are mirrored in
Rust constants until a generated ABI exists.

`time`, `uptime`, `sleep`, and `bench` use the ARM generic timer registers
`CNTFRQ_EL0` and `CNTPCT_EL0`. This is the first timebase for the kernel:
interrupt-driven timers and scheduling can build on the same counter path later.

`rng status` and `rng sample <n> confirm` expose the first kernel nonce source:
an HMAC-SHA256 counter DRBG seeded from the generic timer, device identity,
board constants, and a stack-address mix. This is explicitly a development
source (`hardware-trng=no`) so TLS and signed-session work can share one API
while a real RK3568 or x86_64 hardware entropy source is added.

`xload` is the first resident serial loader. A kernel that was started from SD
can receive another raw image over UART, validate CRC32, flush caches, and jump
to it. The `auto` target uses two ping-pong slots, `0x02080000` and
`0x05080000`, so later development loops can avoid repeated SD-card copies.
Because the current image is statically linked at `0x02080000`, `auto ... go`
does not run directly from the alternate slot. Instead, Xiaojin OS copies a
small reload trampoline to `0x0587f000`; the trampoline copies the newly received
image back down to `0x02080000`, flushes caches, and branches to the link
address.

`caps` prints machine-readable feature flags for host tools. `xjctl dev` uses
`xload auto-reload yes` to confirm that this copy-down reload path exists before
using hot reload.

The kernel installs an AArch64 exception vector table at boot. The `fault`
command executes `BRK #0x123`, prints decoded `ESR_EL2` fields, `ELR_EL2`,
`FAR_EL2`, `SPSR_EL2`, selected saved registers, advances `ELR_EL2`, and
returns to the shell. `last-exception` prints the latest captured exception
snapshot after recoverable exceptions.

`abort confirm` intentionally reads an invalid address to verify the fatal
exception path. It should print the same exception frame and halt; reset or
power-cycle the board afterward.

`peek32` is intentionally conservative: it reads only known DRAM or whitelisted
MMIO. Use `mmio list` and `mmio read32 uart2 0x14` for region-scoped device
register reads. MMIO reads now use a fault-reporting probe path: a data abort
during the read is captured, reported, and returned to the shell instead of
halting the system. This does not make every device window safe to touch:
uninitialized PCIe DBI/config reads can stall the bus instead of raising a
recoverable exception, so PCIe windows are listed as facts but are not exposed
through the generic `mmio read32` whitelist yet.

`net facts` and `pcie facts` are the first wired-network bring-up commands. The
NanoPi R5C Ethernet path is PCIe first: the board exposes its 2.5G RJ45 ports
through Realtek RTL8125-family PCIe NICs, so the kernel must enumerate PCIe
before it can write a native Ethernet driver.

`pcie reset-pins` and `pcie status` read the RK3568 GPIO bank version, GPIO
external port values, and the R5C PCIe reset GPIO levels. `pcie reset-deassert
confirm` and `pcie reset-assert confirm` are the first controlled write commands:
they only drive the RJ45-related `pcie3x1` and `pcie3x2` reset GPIOs. `pcie
phy-status` reads the PCIe3 PHY GRF configuration/status words and decodes the
PHY SRAM init-done bit. `pcie apb-status confirm` reads only the two RJ45-related
PCIe controller APB status windows and decodes coarse RC/LTSSM/link state.
`pcie phy-init confirm` writes the shared PCIe3 PHY GRF lane/bifurcation bits and
polls the PHY SRAM init-done bit. `pcie rc-init confirm` writes the two
RJ45-related PCIe controller APB windows into RC pre-LTSSM mode. `pcie
cru-status` reads the PCIe-related CRU/PMUCRU clock gate and reset bits. `pcie
cru-enable confirm` enables the PCIe3 clock gates and deasserts the PCIe3 PHY and
controller resets. `pcie init` is still a read-only preflight. `pcie
ltssm-enable confirm` enables link training on both RJ45-related PCIe controllers
and polls for link-up; `pcie ltssm-disable confirm` clears the same bit. PCIe
DBI/config reads stay disabled until reset, clocks, PHY, and link state are
verified on hardware. `pcie link-start confirm` runs the validated bring-up
sequence: hold the RJ45 PCIe devices, enable CRU clocks/resets, initialize PHY,
set RC/LTSSM, release the devices, then poll for link-up.
`pcie cfg-probe confirm` reads only a few PCIe DBI/config probe words after the
link is up; if the config window is not mapped yet, the next step is DWC iATU
setup. `pcie iatu-probe confirm` reads the DWC iATU mode/status registers.
`pcie cfg-scan confirm` programs outbound iATU region 0 as CFG0/CFG1 and scans
the first few bus/device/function slots for endpoint vendor IDs. `pcie bar-scan
confirm` sizes the visible endpoint BARs and restores the original values.
`pcie bar-map confirm` assigns the RTL8125 BAR2/BAR4 MMIO windows inside each
controller's 32-bit outbound memory aperture and enables PCI Memory + BusMaster
in the endpoint command register. `pcie rtl-probe confirm` reads a small set of
mapped RTL8125 BAR2 MMIO probe words. `pcie rtl-facts confirm` decodes the
RTL8125 backup MAC address, chip XID/family, command bits, interrupt status, and
basic PHY link flags. `pcie rtl-init confirm` resets the RTL8125 MACs, assigns
local-admin test MAC addresses, initializes static TX/RX descriptor rings, and
enables polling-mode RX/TX without interrupts. `pcie rtl-tx-test confirm` sends
one raw broadcast Ethernet test frame per initialized controller; on current R5C
hardware, `pcie3x2` has been observed to complete the descriptor. `pcie
rtl-rx-poll confirm` polls RX descriptors once and re-arms any consumed buffers.
`net ips` prints the runtime IP, subnet mask, gateway, and DNS configuration.
`net ip <if> <ip> confirm` updates a PCIe NIC address and preserves it across
`net-dev` hot reloads. `net dhcp <if> confirm` runs a manual DHCP client on one
RTL8125 port: it sends DHCPDISCOVER, waits for DHCPOFFER, sends DHCPREQUEST,
waits for DHCPACK, and applies the runtime IP/mask/gateway/DNS lease. DHCP is
also attempted automatically when `net listen` starts, and then retried while
gateway/DNS are still missing. The known static `net-dev` address remains the
early recovery address until DHCP succeeds.
The same DHCP path is also exposed through the UDP `XJDH` control packet, used
by `xjctl net-dhcp`, so DHCP can now be triggered over Ethernet without opening
the UART shell.
Short read-only board queries are exposed through the UDP `XJQ1` control
packet, used by `xjctl net-command`. Current queries include `caps`, `net ips`,
`sd facts`, `sd regs confirm`, `sd read0 confirm`, `sd bpb confirm`, `sd root
confirm`, `sd slots confirm`, and the XiaojinDB `db ... confirm` probes, which
lets storage/OTA and DB bring-up continue over Ethernet instead of requiring the
UART shell for every probe.

XiaojinDB is a preallocated FAT32 page-file database in `/xiaojin-db.dat`.
Current storage primitives include a superblock, catalog, circular WAL pages, a
fixed-size key/value table, an experimental B+tree over 512-byte pages, a stable
logical table namespace (`table + row` mapped to the current `u32` B+tree key
space), and an append-only event log. Host tools can verify the full path with
`xjctl net-db-smoke`, or append one metadata-only event with `xjctl net-event`.
The GCP maintenance bridge can now schedule these checks through
`xjctl gcp-bridge` and the `xiaojin-maintainer` service. Xiaojin OS can also run
a bounded device-side maintainer loop with
`maint loop <if> <host|ip> <port> <poll-path> <event-path> <count> <interval-ms> confirm`;
the same bounded loop is exposed over Ethernet as
`xjctl net-command "maint loop <if> <host|ip> <port> <poll-path> <event-path> <count> <interval-ms> confirm"`.
The UDP query form returns a compact per-iteration status and is limited to 4
iterations; the UART shell form remains limited to 16. Both reuse the signed
`maint pull-exec` path, so a local maintainer HMAC key must already be
provisioned and every executed query is audited in XiaojinDB. This is
manual/bounded today. The board can persist the maintainer endpoint in
XiaojinDB with
`xjctl net-command "maint config-set <if> <host|ip> <port> <poll-path> <event-path> <count> <interval-ms> confirm"`,
add or clear a future HTTPS maintainer pin with `maint config-pin <sha256|none>
confirm`, inspect it with `maint config-status`, and execute the saved bounded
loop with `maint config-run`. The current v3 persistent format stores an IPv4
maintainer host, count, interval, and pin-present flag in the fixed config row,
stores custom poll/event paths as checked chunks in the XiaojinDB B-tree, and
stores the optional first-certificate SHA-256 pin as separate B-tree chunks; old
v1/v2 rows still load. This lets one SD image persist a device-specific GCP VM
queue path and endpoint identity material without rebuilding the OS. The
runtime poll itself is still the signed cleartext maintainer path until HTTPS
polling lands. When the network listener starts
without a serial escape, it performs one bounded `maint config-run` tick if a
saved config exists, then continues serving local LAN control. The PostgreSQL
backup lane now has a host-side backup-set path:
`xjctl` can run VM-side `pg_dump`, pack multiple dumps plus restore lists into a
tarball, upload it into the XiaojinDB blob store, download it back, restore-smoke
it, and register restore-smoked evidence with the GCP maintainer. The serving
lane has a host-side PostgreSQL-compatible bridge with password auth, system
`psql` smoke, read-only health checks, and a VM-local systemd template. Native
row-level import, logical replay, query planning, transactions, and kernel-side
PostgreSQL serving remain follow-on stages.
Persistent inactive-slot image staging is exposed through the UDP `XJPS` control
packet, used by `xjctl net-dev --persist --slot b`. This writes the last
uploaded image into the selected preallocated A/B slot and verifies it by
readback CRC, but it does not yet change the boot target.
`net loop confirm` runs a bounded end-to-end network smoke test. `net listen
confirm` runs the same ARP/IPv4/UDP responder without a time limit until serial
Ctrl-C. The listener initializes the PCIe/RTL8125 path, answers ARP for the
configured NIC addresses, periodically sends gratuitous ARP plus UDP
`xjhello ...` to port `5511`, replies to UDP `xjping` probes on port `5510` with
`xjpong ...`, accepts `XJIP` UDP IP-configuration packets, and accepts the
`XJNL` UDP image-loading protocol used by `xjctl net-dev`.

`net tls <if> <host|ip> <port> [sni] confirm` is the first TLS first-flight
probe. It builds a small TLS ClientHello with SNI, SHA-256-based TLS 1.3 cipher
suites, a development RNG random, supported versions/groups/signature
extensions, and an ephemeral X25519 key share. It then parses the first server
TLS record, extracts ServerHello selected-version/key-share metadata when
present, and reports a SHA-256 fingerprint of the derived shared secret plus the
TLS 1.3 transcript and handshake traffic secret/key/IV fingerprints. This proves
DNS/route/ARP/TCP plus TLS key-agreement and record-key schedule reachability,
and, when the server selects `TLS_AES_128_GCM_SHA256` or
`TLS_CHACHA20_POLY1305_SHA256`, attempts to decrypt the first encrypted server
handshake record. The decrypted payload is then parsed as a TLS handshake
message stream and reports whether EncryptedExtensions, Certificate,
CertificateVerify, and Finished are present in that first record. When Finished
is present in that first decrypted record, the probe verifies the TLS 1.3 server
Finished `verify_data` against the transcript and reports
`server-finished-verified=yes`. When a
Certificate message is present, the probe also parses the TLS 1.3 certificate
list framing and reports the certificate-entry count, certificate-list length,
first-certificate length, list completeness, and the SHA-256 fingerprint of the
first certificate DER bytes. `net tls-pin <if> <host|ip> <port> [sni|-] <sha256>
confirm` runs the same first-flight probe and fails the pin check unless that
first certificate DER SHA-256 matches the supplied 64-character hex digest. It is not yet a complete TLS stack,
certificate-chain validator, or HTTPS client.

`wifi facts`, `wifi status`, `wifi pcie`, `wifi cru`, `wifi apb`, `wifi phy`,
`wifi firmware`, `wifi enum confirm`, and `wifi roadmap` expose the R5C
wireless bring-up path before the full WiFi driver exists. The current hardware assumption
is an RTL8822CE-family M.2 E-Key WiFi/BT module. The OpenWrt R5C device tree
places the candidate WiFi PCIe controller at `/pcie@fe260000`, with
DBI `0x3c0000000`, APB `0xfe260000`, config `0xf4000000`, bus `0x00..0x0f`,
one lane, and reset GPIO `gpio3 PC1` active-high. `wifi cru` reads the PCIe2
clock gate, soft reset, and reset-GPIO state without writing any WiFi/PCIe2
registers. `wifi apb` reads controller-local APB status without DBI/config
access. `wifi phy` reads RK3568 combphy2 and pipe-phy-grf status without writes.
`wifi power` reads the `vcc3v3_pcie` regulator GPIO state. These facts are also
available over Ethernet through `xjctl wifi-facts`, `xjctl wifi-status`, and
`xjctl wifi-pcie`/`xjctl wifi-cru`/`xjctl wifi-apb`/`xjctl wifi-phy`/
`xjctl wifi-power`. `wifi firmware` reports conservative Realtek Linux driver
and firmware-blob candidates for RTL8822/RTL8852 PCIe modules, and `wifi enum
confirm` includes the detected endpoint's driver/firmware candidate beside
vendor/device/class/BAR data. `wifi firmware-sd confirm` checks the SD FAT32
boot volume for firmware blobs at standard Linux firmware paths:
`rtw88/rtw8822c_fw.bin`, `rtw88/rtw8822b_fw.bin`, `rtw89/rtw8852b_fw.bin`,
and `rtw89/rtw8852c_fw.bin`; for early lab use it also accepts the same file
names directly in the FAT32 root. This is a metadata presence probe only and
does not touch WiFi MMIO. `wifi firmware-crc confirm` then walks the FAT chain
for each staged blob and reports size/cluster/CRC32, proving the OS can read
firmware bytes from SD before the loader exists. `wifi firmware-load
<chip|auto> confirm` copies one staged blob into the reserved RAM staging range
`0x06080000..0x06480000` and verifies RAM CRC32. It still does not write WiFi
MMIO or start the chip; it prepares the blob for the next register-level loader
step. `wifi mmio-snapshot confirm` then maps the Realtek WiFi endpoint BAR2 into
the PCIe2 mem32 window, enables PCI Memory/BusMaster in config space, and reads
a small fixed MMIO snapshot without writing WiFi registers. `wifi
mmio-fingerprint confirm` keeps the same safety boundary and probe-reads the
first 4KiB of BAR2 as 32-bit words, returning readable/fault/zero/all-ones
counts, transitions, first sample offsets, and a CRC32. This is the next
evidence gate before Realtek firmware/register writes. `wifi driver-gate
<chip|auto> confirm` combines firmware RAM staging, PCIe2 endpoint identity,
BAR2 mapping, PCI Memory/BusMaster state, and the MMIO fingerprint into a
machine-readable `ready=yes|no` gate for the first RTL register-loader
experiment; it still does not write WiFi MMIO registers. `wifi rtw88-profile
confirm` maps BAR2 and reads a curated, read-only Linux `rtw88` PCI/MMIO
register profile using 8/16/32-bit probe reads as appropriate, so non-aligned
Realtek status registers can be captured without provoking a system halt.
Station mode and AP mode are not implemented yet. `wifi pcie-init confirm` /
`xjctl wifi-init` is the first write-stage probe: it
initializes only the PCIe2/combphy2 WiFi path, waits for LTSSM, and reads config
word 0 when the link comes up. `wifi pcie-init-dt confirm` / `xjctl
wifi-init-dt` is the narrower device-tree-aligned probe: it only drives the
declared PCIe2 pipe reset and active-high PC1 PERST before starting LTSSM. WiFi
DBI/config reads are skipped until PCIe2 reports link-up, because unlinked DBI
windows can stall the board. `wifi enum confirm` / `xjctl wifi-enum` runs the
DT-aligned PCIe2 bring-up path and reads endpoint vendor/device/class/BAR and
subsystem config-space words without sizing BARs or touching WiFi firmware/MMIO.
The already-working RTL8125 wired OTA path remains the rescue channel.

## Network Bring-Up Plan

The goal is to replace UART development with an Ethernet loader, while keeping
UART `xload` as the recovery path until the network path is stable.

1. PCIe facts and fault-reporting MMIO probe. This is implemented by
   `net facts`, `pcie facts`, `mmio list`, and `mmio read32` for whitelisted
   windows.
2. PCIe controller bring-up: reset GPIO, PCIe 3.0 PHY, link training, and config
   space enumeration.
3. RTL8125 driver: discover BARs, map MMIO, set RX/TX rings, and use polling
   before interrupts.
4. Minimal network stack: ARP, IPv4, UDP board-to-host hello, and UDP ping
   response in the kernel.
5. Host transport: macOS BPF raw Ethernet is available for direct-cable recovery;
   normal UDP transport is available for routed LAN development.
6. UDP image loading: `xjctl net-dev` sends a new Xiaojin OS image over
   Ethernet, receives `XJNL READY/ACK/OK`, and reloads through the same
   copy-down trampoline used by UART `xload`.
7. Runtime IP config: `net ip ... confirm` and `XJIP` packets can move a NIC
   from the direct `10.77.0.0/24` development subnet to a routed LAN subnet.
8. DHCP client: `net dhcp ... confirm` can learn a runtime lease, gateway, and
   DNS server from a LAN router.
9. UDP network control: `XJDH` lets `xjctl net-dhcp` trigger the DHCP client
   remotely; `XJQ1` lets `xjctl net-command` ask short board-control questions;
   `xjctl net-discover` listens for `xjhello` to find the post-DHCP address.
   The polling listener also includes a conservative RTL8125 DMA refresh
   watchdog for the development phase: if no UDP control reply is completed for
   a watchdog window, it rearms the local descriptor rings without changing the
   current IP configuration.
10. Outbound DNS/TCP/HTTP probes: `net dns ... confirm` resolves one A record through
    runtime DNS/gateway state, and `net tcp ... confirm` proves an outbound
    SYN/SYN-ACK/ACK TCP path. `net http ... confirm` performs one HTTP/1.0 GET
    and parses the response status while ACKing sequential response payload
    segments up to the built-in sample cap. `net tls ... confirm` performs TLS
    first-flight key schedule, first encrypted record decrypt, certificate list
    framing, and server Finished probes; `net tls-pin ... confirm` adds
    first-certificate SHA-256 pin verification for early secure-control
    experiments. Full TCP stream reassembly, certificate validation, complete
    TLS/HTTPS, and kernel-native cloud polling remain follow-on stages.
11. OTA storage: `sd facts`, `sd regs confirm`, `sd read0 confirm`, `sd bpb
    confirm`, `sd root confirm`, and `sd slots confirm` verify the SD
    controller, FAT32 BPB, root directory, and A/B slot FAT chains.
12. Persistent staging: `xjctl net-dev --persist --slot b` uploads an image over
    Ethernet, writes it into the inactive preallocated slot, and readback-checks
    CRC. Boot target switching, bootstate updates, automatic confirmation, and
    rollback are still follow-on stages.

Useful first probes after booting the latest kernel:

```text
net facts
net loop confirm
pcie facts
pcie reset-pins
pcie status
pcie phy-status
pcie phy-init confirm
pcie apb-status confirm
pcie cfg-probe confirm
pcie iatu-probe confirm
pcie cfg-scan confirm
pcie bar-scan confirm
pcie bar-map confirm
pcie rtl-probe confirm
pcie rtl-facts confirm
pcie rtl-init confirm
pcie rtl-tx-test confirm
pcie rtl-rx-poll confirm
pcie cru-status
pcie cru-enable confirm
pcie rc-init confirm
pcie ltssm-enable confirm
pcie link-start confirm
pcie reset-deassert confirm
pcie status
mmio list
mmio read32 uart2 0x14
```

Do not raw-read PCIe DBI/config/outbound windows until a dedicated `pcie init`
path exists. On the R5C, a direct DBI read before controller bring-up has been
observed to stall instead of producing a recoverable data abort.

## Facts Discovered From U-Boot

- UART baud: `1500000`
- UART2 MMIO base: `0xfe660000`
- Chosen load address: `0x02080000`
- Serial xload slots:
  - `0x02080000..0x02880000`
  - `0x05080000..0x05880000`
- Serial xload trampoline:
  - `0x0587f000..0x05880000`
- R5C wired Ethernet path:
  - PCIe 3.0 x1 domain 1: DBI `0x00000003c0400000`, APB `0xfe270000`
  - PCIe 3.0 x2 domain 2: DBI `0x00000003c0800000`, APB `0xfe280000`
  - PCIe 3.0 PHY: `0xfe8c0000`
- R5C PCIe reset GPIOs:
  - `pcie3x1`: `gpio0 PA0`, release-high behavior validated by link-up
  - `pcie3x2`: `gpio0 PB6`, release-high behavior validated by link-up
  - `pcie2x1`: `gpio3 PC1`, active high
- R5C RTL8125 facts:
  - `pcie3x1` endpoint: PCI vendor/device `0x812510ec`, chip XID `0x641`
    (`RTL8125B`)
  - `pcie3x2` endpoint: PCI vendor/device `0x812510ec`, chip XID `0x641`
    (`RTL8125B`)
  - local-admin test MACs assigned by Xiaojin OS:
    - `pcie3x1`: `02:58:4a:52:35:c0`
    - `pcie3x2`: `02:58:4a:52:35:c1`
  - default test IPv4 addresses assigned by Xiaojin OS:
    - `pcie3x1`: `192.168.80.251`
    - `pcie3x2`: `192.168.80.250`
  - default subnet mask: `255.255.255.0`; gateway/DNS are unset until DHCP or a
    future manual route command provides them
  - cold boot waits 2 seconds for a serial escape key; if none arrives, it
    enters the network listener on the default LAN addresses
  - runtime IPv4 addresses can be changed with `net ip ... confirm`, `XJIP`
    host packets, or `net dhcp ... confirm`; these runtime facts survive
    net-dev reloads through RAM handoff, while power-cycle defaults are compiled
    into the SD image
  - UDP smoke-test port: `5510`
  - `pcie3x2` raw TX descriptor completion has been observed after
    `pcie rtl-init confirm` and `pcie rtl-tx-test confirm`
- OpenWrt U-Boot Image load address: `0x02000000`
- FDT address: `0x08300000`
- Usable DRAM banks:
  - `0x00200000..0x08400000`
  - `0x09400000..0xf0000000`

## Build

```bash
./build.sh
```

Outputs:

- `target/aarch64-unknown-none/release/xiaojin-os`
- `target/aarch64-unknown-none/release/xiaojin-os.bin`

## Run From U-Boot

The first safe path is to copy `xiaojin-os.bin` to a FAT32 SD card, stop in U-Boot,
then load and jump:

```text
mmc list
mmc dev 1
fatload mmc 1:1 0x02000000 xiaojin-os.bin
booti 0x02000000 - 0x08300000
```

If SD is `mmc 0`, use `mmc dev 0` and `fatload mmc 0:1 ...` instead.

With `xjctl`, the usual path is:

```bash
xjctl build-os
xjctl sd-copy --volume /Volumes/R5COS
xjctl run-sd --arm-after-enter --interactive
```
