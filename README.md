# Xiaojin OS

Minimal bare-metal operating system. NanoPi R5C / RK3568 is the first board
profile, not the whole project boundary.

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
- `net facts`
- `pcie facts`
- `pcie reset-pins`
- `pcie status`
- `pcie phy-status`
- `pcie apb-status confirm`
- `pcie reset-deassert confirm`
- `pcie reset-assert confirm`
- `xload <addr|auto> <size> <crc32> [go]`
- `hexdump <addr> [n]`
- `peek32 <addr>`
- `mmio list`
- `mmio read32 <region> <offset>`
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

`xload` is the first resident serial loader. A kernel that was started from SD
can receive another raw image over UART, validate CRC32, flush caches, and jump
to it. The `auto` target uses two ping-pong slots, `0x04080000` and
`0x05080000`, so later development loops can avoid repeated SD-card copies.
Because the current image is statically linked at `0x04080000`, `auto ... go`
does not run directly from the alternate slot. Instead, Xiaojin OS copies a
small reload trampoline to `0x0587f000`; the trampoline copies the newly received
image back down to `0x04080000`, flushes caches, and branches to the link
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
`pcie init` is still a read-only preflight; PCIe DBI/config reads stay disabled
until reset, clocks, and PHY state are verified on hardware.

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
4. Minimal network stack: ARP, IPv4, UDP, and `xjctl net-dev` image loading.

Useful first probes after booting the latest kernel:

```text
net facts
pcie facts
pcie reset-pins
pcie status
pcie phy-status
pcie apb-status confirm
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
- Chosen load address: `0x04080000`
- Serial xload slots:
  - `0x04080000..0x04880000`
  - `0x05080000..0x05880000`
- Serial xload trampoline:
  - `0x0587f000..0x05880000`
- R5C wired Ethernet path:
  - PCIe 3.0 x1 domain 1: DBI `0x00000003c0400000`, APB `0xfe270000`
  - PCIe 3.0 x2 domain 2: DBI `0x00000003c0800000`, APB `0xfe280000`
  - PCIe 3.0 PHY: `0xfe8c0000`
- R5C PCIe reset GPIOs:
  - `pcie3x1`: `gpio0 PA0`, active high
  - `pcie3x2`: `gpio0 PB6`, active high
  - `pcie2x1`: `gpio3 PC1`, active high
- U-Boot default kernel address: `0x00280000`
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
fatload mmc 1:1 0x04080000 xiaojin-os.bin
go 0x04080000
```

If SD is `mmc 0`, use `mmc dev 0` and `fatload mmc 0:1 ...` instead.

With `xjctl`, the usual path is:

```bash
xjctl build-os
xjctl sd-copy --volume /Volumes/R5COS
xjctl run-sd --arm-after-enter --interactive
```
