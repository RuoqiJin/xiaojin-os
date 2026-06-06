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
- `about`
- `mem`
- `regs`
- `time`
- `uptime`
- `sleep <ms>`
- `bench [iters]`
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
register reads.

## Facts Discovered From U-Boot

- UART baud: `1500000`
- UART2 MMIO base: `0xfe660000`
- Chosen load address: `0x04080000`
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
