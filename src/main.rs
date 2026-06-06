#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use core::str;
use core::sync::atomic::{AtomicBool, Ordering};

global_asm!(include_str!("boot.S"));

const UART2_BASE: usize = 0xfe66_0000;
const UART_RBR_THR: usize = 0x00;
const UART_LSR: usize = 0x14;
const UART_LSR_DR: u32 = 1 << 0;
const UART_LSR_THRE: u32 = 1 << 5;
const PCIE30_PHY_GRF_BASE: u64 = 0xfdcb_8000;
const PCIE3X1_APB_BASE: u64 = 0xfe27_0000;
const PCIE3X2_APB_BASE: u64 = 0xfe28_0000;
const PCIE30_PHY_BASE: u64 = 0xfe8c_0000;
const PCIE3X1_CFG_BASE: u64 = 0xf200_0000;
const PCIE3X1_IO_BASE: u64 = 0xf210_0000;
const PCIE3X1_MEM32_BASE: u64 = 0xf220_0000;
const PCIE3X2_CFG_BASE: u64 = 0xf000_0000;
const PCIE3X2_IO_BASE: u64 = 0xf010_0000;
const PCIE3X2_MEM32_BASE: u64 = 0xf020_0000;
const PCIE3X1_DBI_BASE: u64 = 0x0000_0003_c040_0000;
const PCIE3X2_DBI_BASE: u64 = 0x0000_0003_c080_0000;
const PCIE3X1_MEM64_BASE: u64 = 0x0000_0003_4000_0000;
const PCIE3X2_MEM64_BASE: u64 = 0x0000_0003_8000_0000;
const GPIO0_BASE: u64 = 0xfdd6_0000;
const GPIO3_BASE: u64 = 0xfe76_0000;
const GPIO_V2_PORT_DR: u64 = 0x00;
const GPIO_V2_PORT_DDR: u64 = 0x08;
const GPIO_V2_EXT_PORT: u64 = 0x70;
const GPIO_V2_VERSION_ID: u64 = 0x78;
const GPIO_TYPE_V2: u32 = 0x0100_0c2b;
const GPIO_TYPE_V2_1: u32 = 0x0101_157c;
const PCIE_ETH_RESET_COUNT: usize = 2;

const LOAD_ADDR: u64 = 0x0408_0000;
const DRAM_BANK0_START: u64 = 0x0020_0000;
const DRAM_BANK0_END: u64 = 0x0840_0000;
const DRAM_BANK1_START: u64 = 0x0940_0000;
const DRAM_BANK1_END: u64 = 0xf000_0000;
const MAX_LINE_LEN: usize = 128;
const MAX_HEXDUMP_LEN: usize = 256;
const MAX_SLEEP_MS: u64 = 10_000;
const DEFAULT_BENCH_ITERS: u64 = 1_000_000;
const MAX_BENCH_ITERS: u64 = 20_000_000;
const XLOAD_SLOT0_ADDR: u64 = 0x0408_0000;
const XLOAD_SLOT1_ADDR: u64 = 0x0508_0000;
const XLOAD_SLOT_SIZE: u64 = 0x0080_0000;
const XLOAD_MAX_IMAGE_SIZE: u64 = XLOAD_SLOT_SIZE / 2;
const XLOAD_TRAMPOLINE_ADDR: u64 = XLOAD_SLOT1_ADDR + XLOAD_SLOT_SIZE - 0x1000;
const XLOAD_BYTE_TIMEOUT_MS: u64 = 5_000;
const PSCI_SYSTEM_RESET: u64 = 0x8400_0009;
const ESR_EC_SHIFT: u64 = 26;
const ESR_IL_SHIFT: u64 = 25;
const ESR_ISS_MASK: u64 = 0x01ff_ffff;
const ESR_EC_INSN_ABORT_CURRENT: u64 = 0x21;
const ESR_EC_DATA_ABORT_CURRENT: u64 = 0x25;
const ESR_EC_BRK_A64: u64 = 0x3c;
const ABORT_TEST_ADDR: u64 = 0xffff_ffff_ffff_f000;

#[repr(C)]
pub struct ExceptionFrame {
    regs: [u64; 31],
}

#[derive(Clone, Copy)]
struct ExceptionSnapshot {
    valid: bool,
    vector: u64,
    esr: u64,
    elr: u64,
    far: u64,
    spsr: u64,
    vbar: u64,
    x0: u64,
    x1: u64,
    x29: u64,
    x30: u64,
}

impl ExceptionSnapshot {
    const fn empty() -> Self {
        Self {
            valid: false,
            vector: 0,
            esr: 0,
            elr: 0,
            far: 0,
            spsr: 0,
            vbar: 0,
            x0: 0,
            x1: 0,
            x29: 0,
            x30: 0,
        }
    }
}

static mut LAST_EXCEPTION: ExceptionSnapshot = ExceptionSnapshot::empty();
static mut BOOT_CNTPCT: u64 = 0;
static PROBE_ACTIVE: AtomicBool = AtomicBool::new(false);
static PROBE_FAULTED: AtomicBool = AtomicBool::new(false);

struct MmioRegion {
    name: &'static str,
    start: u64,
    end: u64,
    description: &'static str,
}

#[derive(Clone, Copy)]
struct GpioBank {
    name: &'static str,
    base: u64,
}

struct PcieResetPin {
    controller: &'static str,
    bank: GpioBank,
    pin: u8,
    label: &'static str,
    active_high: bool,
    source: &'static str,
}

const MMIO_REGIONS: &[MmioRegion] = &[
    MmioRegion {
        name: "uart2",
        start: UART2_BASE as u64,
        end: UART2_BASE as u64 + 0x100,
        description: "RK3568 debug UART2",
    },
    MmioRegion {
        name: "gpio0",
        start: GPIO0_BASE,
        end: GPIO0_BASE + 0x100,
        description: "RK3568 GPIO bank 0",
    },
    MmioRegion {
        name: "gpio3",
        start: GPIO3_BASE,
        end: GPIO3_BASE + 0x100,
        description: "RK3568 GPIO bank 3",
    },
];

const GPIO0: GpioBank = GpioBank {
    name: "gpio0",
    base: GPIO0_BASE,
};
const GPIO3: GpioBank = GpioBank {
    name: "gpio3",
    base: GPIO3_BASE,
};

const PCIE_RESET_PINS: &[PcieResetPin] = &[
    PcieResetPin {
        controller: "pcie3x1",
        bank: GPIO0,
        pin: 0,
        label: "PA0",
        active_high: true,
        source: "reset-gpios",
    },
    PcieResetPin {
        controller: "pcie3x2",
        bank: GPIO0,
        pin: 14,
        label: "PB6",
        active_high: true,
        source: "reset-gpios",
    },
    PcieResetPin {
        controller: "pcie2x1",
        bank: GPIO3,
        pin: 17,
        label: "PC1",
        active_high: true,
        source: "reset-gpios",
    },
];

struct Uart {
    base: usize,
}

impl Uart {
    const fn new(base: usize) -> Self {
        Self { base }
    }

    fn put_byte(&mut self, byte: u8) {
        unsafe {
            while read_volatile((self.base + UART_LSR) as *const u32) & UART_LSR_THRE == 0 {}
            write_volatile((self.base + UART_RBR_THR) as *mut u32, byte as u32);
        }
    }

    fn get_byte(&mut self) -> Option<u8> {
        unsafe {
            if read_volatile((self.base + UART_LSR) as *const u32) & UART_LSR_DR == 0 {
                return None;
            }
            Some(read_volatile((self.base + UART_RBR_THR) as *const u32) as u8)
        }
    }

    fn get_byte_timeout(&mut self, timeout_ms: u64) -> Option<u8> {
        let freq = read_cntfrq();
        let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
        let start = read_cntpct();
        loop {
            if let Some(byte) = self.get_byte() {
                return Some(byte);
            }
            if read_cntpct().wrapping_sub(start) >= timeout_ticks {
                return None;
            }
            core::hint::spin_loop();
        }
    }

    fn put_str(&mut self, text: &str) {
        for byte in text.bytes() {
            if byte == b'\n' {
                self.put_byte(b'\r');
            }
            self.put_byte(byte);
        }
    }
}

impl Write for Uart {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.put_str(s);
        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    let mut uart = Uart::new(UART2_BASE);
    init_timebase();
    let (runtime_start, runtime_end) = resident_memory_range();

    uart.put_str("\n");
    uart.put_str("========================================\n");
    uart.put_str("Hello from Xiaojin OS.\n");
    uart.put_str("AArch64 bare-metal Rust control shell is running.\n");
    uart.put_str("Board profile: nanopi-r5c / rk3568\n");
    uart.put_str("UART2 MMIO base: 0xfe660000\n");
    let _ = writeln!(uart, "Link address: 0x{LOAD_ADDR:08x}");
    let _ = writeln!(
        uart,
        "Runtime image: 0x{runtime_start:016x}..0x{runtime_end:016x}"
    );
    uart.put_str("========================================\n");
    uart.put_str("Type `help` for commands.\n");

    shell(&mut uart)
}

fn shell(uart: &mut Uart) -> ! {
    let mut line = [0u8; MAX_LINE_LEN];
    let mut len = 0usize;

    print_prompt(uart);
    loop {
        if let Some(byte) = uart.get_byte() {
            match byte {
                b'\r' | b'\n' => {
                    uart.put_str("\n");
                    let command = str::from_utf8(&line[..len]).unwrap_or("");
                    handle_command(uart, command);
                    len = 0;
                    print_prompt(uart);
                }
                0x03 => {
                    len = 0;
                    uart.put_str("^C\n");
                    print_prompt(uart);
                }
                0x08 | 0x7f => {
                    if len > 0 {
                        len -= 1;
                        uart.put_str("\x08 \x08");
                    }
                }
                b if b.is_ascii_graphic() || b == b' ' => {
                    if len < line.len() {
                        line[len] = b;
                        len += 1;
                        uart.put_byte(b);
                    } else {
                        uart.put_byte(0x07);
                    }
                }
                _ => {}
            }
        }
        core::hint::spin_loop();
    }
}

fn print_prompt(uart: &mut Uart) {
    uart.put_str("xj> ");
}

fn handle_command(uart: &mut Uart, input: &str) {
    let mut parts = input.split_whitespace();
    let Some(command) = parts.next() else {
        return;
    };

    if command.eq_ignore_ascii_case("help") || command == "?" {
        print_help(uart);
    } else if command.eq_ignore_ascii_case("caps") {
        print_caps(uart);
    } else if command.eq_ignore_ascii_case("about") {
        print_about(uart);
    } else if command.eq_ignore_ascii_case("mem") {
        print_memory_map(uart);
    } else if command.eq_ignore_ascii_case("regs") {
        print_registers(uart);
    } else if command.eq_ignore_ascii_case("time") {
        command_time(uart);
    } else if command.eq_ignore_ascii_case("uptime") {
        command_uptime(uart);
    } else if command.eq_ignore_ascii_case("sleep") {
        command_sleep(uart, parts.next());
    } else if command.eq_ignore_ascii_case("bench") {
        command_bench(uart, parts.next());
    } else if command.eq_ignore_ascii_case("net") {
        command_net(uart, parts.next());
    } else if command.eq_ignore_ascii_case("pcie") {
        command_pcie(uart, parts.next(), parts.next());
    } else if command.eq_ignore_ascii_case("xload") {
        command_xload(uart, parts);
    } else if command.eq_ignore_ascii_case("hexdump") || command.eq_ignore_ascii_case("hd") {
        command_hexdump(uart, parts.next(), parts.next());
    } else if command.eq_ignore_ascii_case("peek32") {
        command_peek32(uart, parts.next());
    } else if command.eq_ignore_ascii_case("mmio") {
        command_mmio(uart, parts.next(), parts.next(), parts.next());
    } else if command.eq_ignore_ascii_case("last-exception")
        || command.eq_ignore_ascii_case("lastx")
    {
        command_last_exception(uart);
    } else if command.eq_ignore_ascii_case("fault") {
        command_fault(uart);
    } else if command.eq_ignore_ascii_case("abort") {
        command_abort(uart, parts.next());
    } else if command.eq_ignore_ascii_case("reboot") {
        command_reboot(uart);
    } else {
        let _ = writeln!(uart, "unknown command: {command}");
        uart.put_str("try `help`\n");
    }
}

fn print_help(uart: &mut Uart) {
    uart.put_str("commands:\n");
    uart.put_str("  help                 show this command list\n");
    uart.put_str("  caps                 show machine-readable feature flags\n");
    uart.put_str("  about                show kernel and board identity\n");
    uart.put_str("  mem                  show current memory map facts\n");
    uart.put_str("  regs                 show selected CPU registers\n");
    uart.put_str("  time                 show generic timer registers and uptime\n");
    uart.put_str("  uptime               show elapsed kernel time\n");
    uart.put_str("  sleep <ms>           busy-wait with generic timer, max 10000\n");
    uart.put_str("  bench [iters]        run timer-backed CPU microbenchmarks\n");
    uart.put_str("  net facts            show wired-network bring-up plan\n");
    uart.put_str("  pcie facts           show R5C PCIe controller facts\n");
    uart.put_str("  pcie reset-pins      read PCIe reset GPIO levels\n");
    uart.put_str("  pcie status          read PCIe bring-up status, no writes\n");
    uart.put_str("  pcie reset-deassert confirm\n");
    uart.put_str("                       release RJ45 PCIe reset GPIOs\n");
    uart.put_str("  xload <addr|auto> <size> <crc32> [go]\n");
    uart.put_str("                       receive a serial-loaded image\n");
    uart.put_str("  hexdump <addr> [n]   dump DRAM bytes, max 256\n");
    uart.put_str("  peek32 <addr>        read one safe 32-bit word\n");
    uart.put_str("  mmio list            list whitelisted MMIO regions\n");
    uart.put_str("  mmio read32 <r> <o>  read one region-relative 32-bit MMIO word\n");
    uart.put_str("  last-exception       show the latest captured exception\n");
    uart.put_str("  fault                trigger BRK to test exception vectors\n");
    uart.put_str("  abort confirm        trigger fatal data abort test\n");
    uart.put_str("  reboot               request PSCI system reset\n");
}

fn print_caps(uart: &mut Uart) {
    uart.put_str("caps:\n");
    uart.put_str("  xload auto-reload yes\n");
    uart.put_str("  xload reload-mode copy-down-trampoline\n");
    let _ = writeln!(uart, "  xload link-address 0x{LOAD_ADDR:016x}");
    let _ = writeln!(
        uart,
        "  xload trampoline-address 0x{XLOAD_TRAMPOLINE_ADDR:016x}"
    );
}

fn print_about(uart: &mut Uart) {
    uart.put_str("Xiaojin OS 0.1.0\n");
    uart.put_str("board: nanopi-r5c\n");
    uart.put_str("soc: rk3568\n");
    uart.put_str("kernel: AArch64 no_std Rust\n");
    uart.put_str("control: UART2 shell\n");
    uart.put_str("network target: PCIe RTL8125 wired loader\n");
}

fn command_net(uart: &mut Uart, action: Option<&str>) {
    match action {
        None | Some("facts") => print_net_facts(uart),
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  net facts\n");
        }
    }
}

fn print_net_facts(uart: &mut Uart) {
    uart.put_str("wired network bring-up facts:\n");
    uart.put_str("  board path       NanoPi R5C RJ45 ports sit behind PCIe NICs\n");
    uart.put_str("  expected NIC     Realtek RTL8125B family 2.5G Ethernet\n");
    uart.put_str("  not the path     RK3568 GMAC direct PHY driver is not enough here\n");
    uart.put_str("  stage 1          PCIe facts + fault-reporting MMIO probe\n");
    uart.put_str("  stage 2          PCIe reset, clock, PHY, link, config enumeration\n");
    uart.put_str("  stage 3          RTL8125 MMIO BAR, RX/TX rings, interrupts/polling\n");
    uart.put_str("  stage 4          ARP + IPv4 + UDP xload, then host xjctl net-dev\n");
    uart.put_str("  rescue path      UART xload remains the fallback until UDP loader is stable\n");
}

fn command_pcie(uart: &mut Uart, action: Option<&str>, arg1: Option<&str>) {
    match action {
        None | Some("facts") => print_pcie_facts(uart),
        Some("reset-pins") => print_pcie_reset_pins(uart),
        Some("status") => print_pcie_status(uart),
        Some("init") => command_pcie_init(uart),
        Some("reset-deassert") => command_pcie_reset_write(uart, false, arg1),
        Some("reset-assert") => command_pcie_reset_write(uart, true, arg1),
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  pcie facts\n");
            uart.put_str("  pcie reset-pins\n");
            uart.put_str("  pcie status\n");
            uart.put_str("  pcie init\n");
            uart.put_str("  pcie reset-deassert confirm\n");
            uart.put_str("  pcie reset-assert confirm\n");
        }
    }
}

fn print_pcie_facts(uart: &mut Uart) {
    uart.put_str("R5C PCIe facts from board profile:\n");
    uart.put_str("  pcie3x1          enabled, domain=1, bus=0x10..0x1f, lanes=1\n");
    uart.put_str("  pcie3x1 reset    gpio0 A0, active high\n");
    let _ = writeln!(
        uart,
        "  pcie3x1 dbi      0x{PCIE3X1_DBI_BASE:016x}..0x{:016x}",
        PCIE3X1_DBI_BASE + 0x0040_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x1 apb      0x{PCIE3X1_APB_BASE:016x}..0x{:016x}",
        PCIE3X1_APB_BASE + 0x0001_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x1 cfg      0x{PCIE3X1_CFG_BASE:016x}..0x{:016x}",
        PCIE3X1_CFG_BASE + 0x0010_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x1 io       0x{PCIE3X1_IO_BASE:016x}..0x{:016x}",
        PCIE3X1_IO_BASE + 0x0010_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x1 mem32    0x{PCIE3X1_MEM32_BASE:016x}..0x{:016x}",
        PCIE3X1_MEM32_BASE + 0x01e0_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x1 mem64    0x{PCIE3X1_MEM64_BASE:016x}..0x{:016x}",
        PCIE3X1_MEM64_BASE + 0x4000_0000
    );
    uart.put_str("  pcie3x2          enabled, domain=2, bus=0x20..0x2f, lanes=1 on R5C\n");
    uart.put_str("  pcie3x2 reset    gpio0 B6, active high\n");
    let _ = writeln!(
        uart,
        "  pcie3x2 dbi      0x{PCIE3X2_DBI_BASE:016x}..0x{:016x}",
        PCIE3X2_DBI_BASE + 0x0040_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x2 apb      0x{PCIE3X2_APB_BASE:016x}..0x{:016x}",
        PCIE3X2_APB_BASE + 0x0001_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x2 cfg      0x{PCIE3X2_CFG_BASE:016x}..0x{:016x}",
        PCIE3X2_CFG_BASE + 0x0010_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x2 io       0x{PCIE3X2_IO_BASE:016x}..0x{:016x}",
        PCIE3X2_IO_BASE + 0x0010_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x2 mem32    0x{PCIE3X2_MEM32_BASE:016x}..0x{:016x}",
        PCIE3X2_MEM32_BASE + 0x01e0_0000
    );
    let _ = writeln!(
        uart,
        "  pcie3x2 mem64    0x{PCIE3X2_MEM64_BASE:016x}..0x{:016x}",
        PCIE3X2_MEM64_BASE + 0x4000_0000
    );
    uart.put_str("  pcie2x1          enabled on R5C, reset gpio3 C1, likely not RJ45 path\n");
    let _ = writeln!(
        uart,
        "  pcie30phy        0x{PCIE30_PHY_BASE:016x}..0x{:016x}, data-lanes=<1 2>",
        PCIE30_PHY_BASE + 0x0002_0000
    );
    let _ = writeln!(
        uart,
        "  pcie30phy-grf    0x{PCIE30_PHY_GRF_BASE:016x}..0x{:016x}",
        PCIE30_PHY_GRF_BASE + 0x0001_0000
    );
    uart.put_str("  raw read rule    do not read PCIe windows before controller init\n");
    uart.put_str("  observed limit   uninitialized DBI reads can stall instead of faulting\n");
    uart.put_str("  next driver step reset, clock, PHY, then config-space scan\n");
}

fn command_pcie_init(uart: &mut Uart) {
    uart.put_str("pcie init stage: read-only preflight\n");
    uart.put_str("  writes           disabled in this build\n");
    uart.put_str("  reason           reset/clock/PHY writes need one more verified map pass\n");
    uart.put_str("  next             validate GPIO levels, then add explicit reset control\n");
    print_pcie_status(uart);
}

fn command_pcie_reset_write(uart: &mut Uart, assert_reset: bool, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage:\n");
        uart.put_str("  pcie reset-deassert confirm\n");
        uart.put_str("  pcie reset-assert confirm\n");
        uart.put_str("scope: pcie3x1 and pcie3x2 RJ45 reset GPIOs only\n");
        return;
    }

    for pin in &PCIE_RESET_PINS[..PCIE_ETH_RESET_COUNT] {
        let level = if pin.active_high {
            assert_reset
        } else {
            !assert_reset
        };
        gpio_v2_write_bit(pin.bank, GPIO_V2_PORT_DR, pin.pin, level);
        gpio_v2_write_bit(pin.bank, GPIO_V2_PORT_DDR, pin.pin, true);
    }

    if assert_reset {
        uart.put_str("pcie reset assert: pcie3x1/pcie3x2 driven high\n");
    } else {
        uart.put_str("pcie reset deassert: pcie3x1/pcie3x2 driven low\n");
    }
    print_pcie_reset_pins(uart);
}

fn print_pcie_status(uart: &mut Uart) {
    uart.put_str("pcie status:\n");
    uart.put_str("  mode            read-only\n");
    print_gpio_bank_status(uart, GPIO0);
    print_gpio_bank_status(uart, GPIO3);
    print_pcie_reset_pins(uart);
}

fn print_gpio_bank_status(uart: &mut Uart, bank: GpioBank) {
    let version = probe_read32(bank.base + GPIO_V2_VERSION_ID);
    let ext = probe_read32(bank.base + GPIO_V2_EXT_PORT);

    match (version, ext) {
        (Ok(version), Ok(ext)) => {
            let _ = writeln!(
                uart,
                "  {:<14} base=0x{:016x} version=0x{version:08x} ({}) ext=0x{ext:08x}",
                bank.name,
                bank.base,
                gpio_version_name(version)
            );
        }
        (version, ext) => {
            let _ = writeln!(uart, "  {:<14} base=0x{:016x}", bank.name, bank.base);
            print_probe_result(uart, "    version", version);
            print_probe_result(uart, "    ext", ext);
        }
    }
}

fn print_pcie_reset_pins(uart: &mut Uart) {
    uart.put_str("pcie reset pins:\n");
    for pin in PCIE_RESET_PINS {
        let direction = read_gpio_v2_direction(pin.bank, pin.pin);
        let level = read_gpio_v2_level(pin.bank, pin.pin);
        match (direction, level) {
            (Ok(direction), Ok(level)) => {
                let asserted = if pin.active_high { level } else { !level };
                let _ = writeln!(
                    uart,
                    "  {:<8} {:<5} {:<3} bit={:<2} dir={:<3} level={} reset={} source={}",
                    pin.controller,
                    pin.bank.name,
                    pin.label,
                    pin.pin,
                    gpio_direction_name(direction),
                    bit_name(level),
                    if asserted { "asserted" } else { "deasserted" },
                    pin.source
                );
            }
            (direction, level) => {
                let _ = writeln!(
                    uart,
                    "  {:<8} {:<5} {:<3} bit={:<2}",
                    pin.controller, pin.bank.name, pin.label, pin.pin
                );
                print_probe_result(uart, "    direction", direction.map(bool_to_u32));
                print_probe_result(uart, "    level", level.map(bool_to_u32));
            }
        }
    }
    uart.put_str("  note            reset-gpios are active high; asserted means held in reset\n");
}

fn read_gpio_v2_direction(bank: GpioBank, pin: u8) -> Result<bool, ExceptionSnapshot> {
    let reg = bank.base + GPIO_V2_PORT_DDR + if pin >= 16 { 4 } else { 0 };
    let bit = pin % 16;
    probe_read32(reg).map(|value| value & (1u32 << bit) != 0)
}

fn read_gpio_v2_level(bank: GpioBank, pin: u8) -> Result<bool, ExceptionSnapshot> {
    probe_read32(bank.base + GPIO_V2_EXT_PORT).map(|value| value & (1u32 << pin) != 0)
}

fn gpio_v2_write_bit(bank: GpioBank, offset: u64, pin: u8, value: bool) {
    let reg = bank.base + offset + if pin >= 16 { 4 } else { 0 };
    let bit = pin % 16;
    let mask = 1u32 << (bit + 16);
    let data = if value { 1u32 << bit } else { 0 };
    unsafe {
        write_volatile(reg as usize as *mut u32, mask | data);
    }
}

fn print_probe_result(uart: &mut Uart, label: &str, result: Result<u32, ExceptionSnapshot>) {
    match result {
        Ok(value) => {
            let _ = writeln!(uart, "{label:<16}0x{value:08x}");
        }
        Err(snapshot) => {
            let _ = writeln!(
                uart,
                "{label:<16}fault ec=0x{:02x} far=0x{:016x}",
                snapshot.esr >> ESR_EC_SHIFT,
                snapshot.far
            );
        }
    }
}

fn gpio_version_name(version: u32) -> &'static str {
    match version {
        GPIO_TYPE_V2 => "v2",
        GPIO_TYPE_V2_1 => "v2.1",
        _ => "unknown",
    }
}

fn gpio_direction_name(output: bool) -> &'static str {
    if output {
        "out"
    } else {
        "in"
    }
}

fn bit_name(value: bool) -> &'static str {
    if value {
        "high"
    } else {
        "low"
    }
}

fn bool_to_u32(value: bool) -> u32 {
    if value {
        1
    } else {
        0
    }
}

fn print_memory_map(uart: &mut Uart) {
    extern "C" {
        static __bss_start: u8;
        static __bss_end: u8;
        static _boot_stack_bottom: u8;
        static _boot_stack_top: u8;
        static __exception_vectors: u8;
    }

    let bss_start = unsafe { &__bss_start as *const u8 as u64 };
    let bss_end = unsafe { &__bss_end as *const u8 as u64 };
    let stack_bottom = unsafe { &_boot_stack_bottom as *const u8 as u64 };
    let stack_top = unsafe { &_boot_stack_top as *const u8 as u64 };
    let vectors = unsafe { &__exception_vectors as *const u8 as u64 };

    let _ = writeln!(uart, "load_addr        0x{LOAD_ADDR:016x}");
    let _ = writeln!(
        uart,
        "dram bank0       0x{DRAM_BANK0_START:016x}..0x{DRAM_BANK0_END:016x}"
    );
    let _ = writeln!(
        uart,
        "dram bank1       0x{DRAM_BANK1_START:016x}..0x{DRAM_BANK1_END:016x}"
    );
    let _ = writeln!(uart, "uart2 mmio       0x{UART2_BASE:016x}");
    let _ = writeln!(
        uart,
        "xload slot0      0x{XLOAD_SLOT0_ADDR:016x}..0x{:016x}",
        XLOAD_SLOT0_ADDR + XLOAD_SLOT_SIZE
    );
    let _ = writeln!(
        uart,
        "xload slot1      0x{XLOAD_SLOT1_ADDR:016x}..0x{:016x}",
        XLOAD_SLOT1_ADDR + XLOAD_SLOT_SIZE
    );
    let _ = writeln!(uart, "exception table  0x{vectors:016x}");
    let _ = writeln!(
        uart,
        "bss              0x{bss_start:016x}..0x{bss_end:016x}"
    );
    let _ = writeln!(
        uart,
        "boot stack       0x{stack_bottom:016x}..0x{stack_top:016x}"
    );
}

fn print_registers(uart: &mut Uart) {
    let current_el = read_current_el();
    let daif = read_daif();
    let sp = read_sp();
    let vbar = read_vbar_el2();
    let cntfrq = read_cntfrq();
    let cntpct = read_cntpct();

    let _ = writeln!(
        uart,
        "CurrentEL        0x{current_el:016x} (EL{})",
        current_el >> 2
    );
    let _ = writeln!(uart, "DAIF             0x{daif:016x}");
    let _ = writeln!(uart, "SP               0x{sp:016x}");
    let _ = writeln!(uart, "VBAR_EL2         0x{vbar:016x}");
    let _ = writeln!(uart, "CNTFRQ_EL0       0x{cntfrq:016x} ({cntfrq} Hz)");
    let _ = writeln!(uart, "CNTPCT_EL0       0x{cntpct:016x}");
}

fn command_time(uart: &mut Uart) {
    let freq = read_cntfrq();
    let now = read_cntpct();
    let boot = boot_cntpct();
    let elapsed = now.wrapping_sub(boot);

    let _ = writeln!(uart, "CNTFRQ_EL0       0x{freq:016x} ({freq} Hz)");
    let _ = writeln!(uart, "CNTPCT_EL0       0x{now:016x}");
    let _ = writeln!(uart, "boot CNTPCT      0x{boot:016x}");
    let _ = writeln!(uart, "elapsed ticks    0x{elapsed:016x} ({elapsed})");
    print_duration_us(uart, "uptime", ticks_to_us(elapsed, freq));
}

fn command_uptime(uart: &mut Uart) {
    let freq = read_cntfrq();
    let elapsed = read_cntpct().wrapping_sub(boot_cntpct());

    print_duration_us(uart, "uptime", ticks_to_us(elapsed, freq));
    let _ = writeln!(uart, "ticks            {elapsed}");
}

fn command_sleep(uart: &mut Uart, ms_token: Option<&str>) {
    let Some(ms) = ms_token.and_then(parse_u64) else {
        uart.put_str("usage: sleep <ms>\n");
        return;
    };
    if ms > MAX_SLEEP_MS {
        let _ = writeln!(uart, "sleep is capped at {MAX_SLEEP_MS} ms");
        return;
    }

    let freq = read_cntfrq();
    let Some(target_ticks) = ms_to_ticks(ms, freq) else {
        uart.put_str("timer frequency is invalid\n");
        return;
    };

    let _ = writeln!(uart, "sleeping for {ms} ms ({target_ticks} timer ticks)...");
    let start = read_cntpct();
    while read_cntpct().wrapping_sub(start) < target_ticks {
        core::hint::spin_loop();
    }
    let elapsed_ticks = read_cntpct().wrapping_sub(start);
    let elapsed_us = ticks_to_us(elapsed_ticks, freq);

    let _ = writeln!(uart, "slept ticks      {elapsed_ticks}");
    print_duration_us(uart, "slept", elapsed_us);
}

fn command_bench(uart: &mut Uart, iters_token: Option<&str>) {
    let iters = iters_token
        .and_then(parse_u64)
        .unwrap_or(DEFAULT_BENCH_ITERS);
    if iters == 0 || iters > MAX_BENCH_ITERS {
        let _ = writeln!(uart, "usage: bench [iters], range: 1..={MAX_BENCH_ITERS}");
        return;
    }

    let freq = read_cntfrq();
    if freq == 0 {
        uart.put_str("timer frequency is invalid\n");
        return;
    }

    let _ = writeln!(uart, "bench iters      {iters}");
    let counter_iters = if iters > 200_000 { 200_000 } else { iters };
    let mut timer_mix = 0u64;
    let timer_start = read_cntpct();
    for _ in 0..counter_iters {
        timer_mix ^= read_cntpct();
    }
    let timer_ticks = read_cntpct().wrapping_sub(timer_start);
    core::hint::black_box(timer_mix);
    print_bench_result(uart, "cntpct read", counter_iters, timer_ticks, freq);

    let mut x = 0x243f_6a88_85a3_08d3u64;
    let int_start = read_cntpct();
    for i in 0..iters {
        x = x.rotate_left(9);
        x = x.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        x = x.wrapping_add(i ^ 0xbf58_476d_1ce4_e5b9);
    }
    let int_ticks = read_cntpct().wrapping_sub(int_start);
    core::hint::black_box(x);
    print_bench_result(uart, "integer mix", iters, int_ticks, freq);
    let _ = writeln!(uart, "bench checksum   0x{x:016x}");
}

fn command_xload<'a, I>(uart: &mut Uart, mut parts: I)
where
    I: Iterator<Item = &'a str>,
{
    let Some(addr_token) = parts.next() else {
        uart.put_str("usage: xload <addr|auto> <size> <crc32> [go]\n");
        uart.put_str("example: xload auto 39168 0x12345678 go\n");
        return;
    };
    let Some(size) = parts.next().and_then(parse_u64) else {
        uart.put_str("usage: xload <addr|auto> <size> <crc32> [go]\n");
        return;
    };
    let Some(expected_crc) = parts.next().and_then(parse_u64) else {
        uart.put_str("usage: xload <addr|auto> <size> <crc32> [go]\n");
        return;
    };
    let go = match parts.next() {
        None => false,
        Some(flag) if flag.eq_ignore_ascii_case("go") || flag.eq_ignore_ascii_case("run") => true,
        Some(_) => {
            uart.put_str("usage: xload <addr|auto> <size> <crc32> [go]\n");
            return;
        }
    };
    if parts.next().is_some() {
        uart.put_str("usage: xload <addr|auto> <size> <crc32> [go]\n");
        return;
    }

    if size == 0 || size > XLOAD_MAX_IMAGE_SIZE {
        let _ = writeln!(
            uart,
            "xload size must be in 1..={XLOAD_MAX_IMAGE_SIZE} bytes"
        );
        return;
    }
    if expected_crc > u32::MAX as u64 {
        uart.put_str("xload crc32 must fit in 32 bits\n");
        return;
    }

    let addr = if addr_token.eq_ignore_ascii_case("auto") {
        let Some(addr) = choose_xload_auto_addr(size) else {
            uart.put_str("xload refused: no safe auto slot is available\n");
            return;
        };
        addr
    } else {
        let Some(addr) = parse_u64(addr_token) else {
            uart.put_str("invalid xload address\n");
            return;
        };
        addr
    };

    if let Err(reason) = validate_xload_target(addr, size) {
        let _ = writeln!(uart, "xload refused: {reason}");
        return;
    }

    let expected_crc = expected_crc as u32;
    let _ = writeln!(
        uart,
        "XLOAD READY addr=0x{addr:016x} size={size} crc32=0x{expected_crc:08x} go={}",
        yes_no(go)
    );

    let mut crc = 0xffff_ffffu32;
    for offset in 0..size {
        let Some(byte) = uart.get_byte_timeout(XLOAD_BYTE_TIMEOUT_MS) else {
            let _ = writeln!(
                uart,
                "\nXLOAD ERR timeout offset={offset} expected_size={size}"
            );
            return;
        };
        unsafe {
            write_volatile((addr + offset) as usize as *mut u8, byte);
        }
        crc = crc32_update(crc, byte);
    }
    let actual_crc = !crc;
    if actual_crc != expected_crc {
        let _ = writeln!(
            uart,
            "\nXLOAD ERR crc32 expected=0x{expected_crc:08x} actual=0x{actual_crc:08x}"
        );
        return;
    }

    flush_loaded_image(addr, size);
    let _ = writeln!(
        uart,
        "\nXLOAD OK addr=0x{addr:016x} size={size} crc32=0x{actual_crc:08x}"
    );
    if go {
        if addr == LOAD_ADDR {
            let _ = writeln!(uart, "XLOAD GO 0x{addr:016x}");
            jump_to(addr);
        } else {
            let _ = writeln!(
                uart,
                "XLOAD RELOAD src=0x{addr:016x} dst=0x{LOAD_ADDR:016x} size={size}"
            );
            reload_to_link_address(addr, size);
        }
    }
}

fn command_hexdump(uart: &mut Uart, addr_token: Option<&str>, len_token: Option<&str>) {
    let Some(addr) = addr_token.and_then(parse_u64) else {
        uart.put_str("usage: hexdump <addr> [n]\n");
        return;
    };
    let len = len_token.and_then(parse_u64).unwrap_or(64) as usize;
    let len = if len > MAX_HEXDUMP_LEN {
        MAX_HEXDUMP_LEN
    } else {
        len
    };

    if len == 0 {
        uart.put_str("length must be greater than zero\n");
        return;
    }
    if !is_dram_range(addr, len as u64) {
        uart.put_str("refusing hexdump outside known DRAM range\n");
        uart.put_str("use `peek32 <addr>` for one MMIO/physical word\n");
        return;
    }

    let mut offset = 0usize;
    while offset < len {
        let line_addr = addr + offset as u64;
        let _ = write!(uart, "0x{line_addr:016x}:");
        for i in 0..16 {
            if offset + i < len {
                let byte = unsafe { read_volatile((addr as usize + offset + i) as *const u8) };
                let _ = write!(uart, " {byte:02x}");
            } else {
                uart.put_str("   ");
            }
        }
        uart.put_str("  ");
        for i in 0..16 {
            if offset + i < len {
                let byte = unsafe { read_volatile((addr as usize + offset + i) as *const u8) };
                let ch = if byte.is_ascii_graphic() || byte == b' ' {
                    byte
                } else {
                    b'.'
                };
                uart.put_byte(ch);
            }
        }
        uart.put_str("\n");
        offset += 16;
    }
}

fn command_peek32(uart: &mut Uart, addr_token: Option<&str>) {
    let Some(addr) = addr_token.and_then(parse_u64) else {
        uart.put_str("usage: peek32 <addr>\n");
        return;
    };
    if addr & 0x3 != 0 {
        uart.put_str("address must be 4-byte aligned\n");
        return;
    }
    if !is_safe_read32_address(addr) {
        uart.put_str("refusing unsafe 32-bit physical read\n");
        uart.put_str("allowed: known DRAM or whitelisted MMIO from `mmio list`\n");
        return;
    }

    match probe_read32(addr) {
        Ok(value) => {
            let _ = writeln!(uart, "0x{addr:016x}: 0x{value:08x}");
        }
        Err(snapshot) => {
            let _ = writeln!(uart, "0x{addr:016x}: read faulted");
            print_exception_snapshot(uart, &snapshot);
        }
    }
}

fn command_mmio(
    uart: &mut Uart,
    action: Option<&str>,
    region_token: Option<&str>,
    offset_token: Option<&str>,
) {
    match action {
        None | Some("list") => list_mmio_regions(uart),
        Some("read32") => command_mmio_read32(uart, region_token, offset_token),
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  mmio list\n");
            uart.put_str("  mmio read32 <region> <offset>\n");
        }
    }
}

fn list_mmio_regions(uart: &mut Uart) {
    uart.put_str("mmio regions:\n");
    for region in MMIO_REGIONS {
        let _ = writeln!(
            uart,
            "  {:<14} 0x{:016x}..0x{:016x}  {}",
            region.name, region.start, region.end, region.description
        );
    }
}

fn command_mmio_read32(uart: &mut Uart, region_token: Option<&str>, offset_token: Option<&str>) {
    let Some(region_name) = region_token else {
        uart.put_str("usage: mmio read32 <region> <offset>\n");
        return;
    };
    let Some(offset) = offset_token.and_then(parse_u64) else {
        uart.put_str("usage: mmio read32 <region> <offset>\n");
        return;
    };
    let Some(region) = find_mmio_region(region_name) else {
        let _ = writeln!(uart, "unknown MMIO region: {region_name}");
        uart.put_str("try `mmio list`\n");
        return;
    };
    if offset & 0x3 != 0 {
        uart.put_str("offset must be 4-byte aligned\n");
        return;
    }

    let addr = region.start.saturating_add(offset);
    if addr < region.start || addr.checked_add(4).is_none_or(|end| end > region.end) {
        uart.put_str("offset is outside the MMIO region\n");
        return;
    }

    match probe_read32(addr) {
        Ok(value) => {
            let _ = writeln!(
                uart,
                "{}+0x{offset:x} @ 0x{addr:016x}: 0x{value:08x}",
                region.name
            );
        }
        Err(snapshot) => {
            let _ = writeln!(
                uart,
                "{}+0x{offset:x} @ 0x{addr:016x}: read faulted",
                region.name
            );
            print_exception_snapshot(uart, &snapshot);
        }
    }
}

fn probe_read32(addr: u64) -> Result<u32, ExceptionSnapshot> {
    let mut value: u32 = 0;
    PROBE_FAULTED.store(false, Ordering::SeqCst);
    PROBE_ACTIVE.store(true, Ordering::SeqCst);
    unsafe {
        asm!(
            "ldr {value:w}, [{addr}]",
            addr = in(reg) addr as usize,
            value = inout(reg) value,
            options(nostack, preserves_flags)
        );
    }
    PROBE_ACTIVE.store(false, Ordering::SeqCst);

    if PROBE_FAULTED.swap(false, Ordering::SeqCst) {
        Err(unsafe { LAST_EXCEPTION })
    } else {
        Ok(value)
    }
}

fn command_last_exception(uart: &mut Uart) {
    let snapshot = unsafe { LAST_EXCEPTION };
    if !snapshot.valid {
        uart.put_str("no exception recorded\n");
        return;
    }

    uart.put_str("last exception:\n");
    print_exception_snapshot(uart, &snapshot);
}

fn command_fault(uart: &mut Uart) {
    uart.put_str("triggering BRK #0x123...\n");
    unsafe {
        asm!("brk #0x123");
    }
    uart.put_str("returned from handled BRK exception\n");
}

fn command_abort(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: abort confirm\n");
        uart.put_str("This intentionally reads an invalid address and should halt.\n");
        return;
    }

    let _ = writeln!(
        uart,
        "triggering fatal data abort by reading 0x{ABORT_TEST_ADDR:016x}..."
    );
    let value = unsafe { read_volatile(ABORT_TEST_ADDR as usize as *const u64) };
    let _ = writeln!(
        uart,
        "unexpectedly read 0x{value:016x}; abort test did not fault"
    );
}

fn command_reboot(uart: &mut Uart) -> ! {
    uart.put_str("requesting PSCI SYSTEM_RESET...\n");
    let mut fid = PSCI_SYSTEM_RESET;
    unsafe {
        asm!(
            "smc #0",
            inout("x0") fid,
            out("x1") _,
            out("x2") _,
            out("x3") _,
            out("x4") _,
            out("x5") _,
            out("x6") _,
            out("x7") _,
        );
    }
    let _ = writeln!(uart, "PSCI reset returned unexpectedly: 0x{fid:016x}");
    loop {
        core::hint::spin_loop();
    }
}

fn parse_u64(token: &str) -> Option<u64> {
    if let Some(hex) = token
        .strip_prefix("0x")
        .or_else(|| token.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok()
    } else {
        token.parse::<u64>().ok()
    }
}

fn choose_xload_auto_addr(size: u64) -> Option<u64> {
    let (resident_start, _) = resident_memory_range();
    let current_in_slot0 =
        resident_start >= XLOAD_SLOT0_ADDR && resident_start < XLOAD_SLOT0_ADDR + XLOAD_SLOT_SIZE;
    let candidates = if current_in_slot0 {
        [XLOAD_SLOT1_ADDR, XLOAD_SLOT0_ADDR]
    } else {
        [XLOAD_SLOT0_ADDR, XLOAD_SLOT1_ADDR]
    };

    candidates
        .into_iter()
        .find(|addr| validate_xload_target(*addr, size).is_ok())
}

fn validate_xload_target(addr: u64, size: u64) -> Result<(), &'static str> {
    if addr & 0xfff != 0 {
        return Err("target address must be 4K aligned");
    }
    if !is_dram_range(addr, size) {
        return Err("target range is outside known DRAM");
    }

    let (resident_start, resident_end) = resident_memory_range();
    let guard_len = if size < 0x0008_0000 {
        0x0008_0000
    } else {
        size
    };
    if ranges_overlap(addr, guard_len, resident_start, resident_end) {
        return Err("target range overlaps the running loader");
    }

    Ok(())
}

fn resident_memory_range() -> (u64, u64) {
    extern "C" {
        static _start: u8;
        static _boot_stack_top: u8;
    }

    let start = unsafe { &_start as *const u8 as u64 };
    let stack_top = unsafe { &_boot_stack_top as *const u8 as u64 };
    (start, stack_top.saturating_add(0x1000))
}

fn ranges_overlap(a_start: u64, a_len: u64, b_start: u64, b_end: u64) -> bool {
    let Some(a_end) = a_start.checked_add(a_len) else {
        return true;
    };
    a_start < b_end && b_start < a_end
}

fn crc32_update(mut crc: u32, byte: u8) -> u32 {
    crc ^= byte as u32;
    for _ in 0..8 {
        if crc & 1 != 0 {
            crc = (crc >> 1) ^ 0xedb8_8320;
        } else {
            crc >>= 1;
        }
    }
    crc
}

fn flush_loaded_image(addr: u64, len: u64) {
    let ctr = read_ctr_el0();
    let dcache_line = cache_line_size(ctr, 16);
    let icache_line = cache_line_size(ctr, 0);
    let end = addr.saturating_add(len);

    let mut current = align_down(addr, dcache_line);
    while current < end {
        unsafe {
            asm!("dc cvau, {0}", in(reg) current);
        }
        current = current.saturating_add(dcache_line);
    }
    unsafe {
        asm!("dsb ish");
    }

    current = align_down(addr, icache_line);
    while current < end {
        unsafe {
            asm!("ic ivau, {0}", in(reg) current);
        }
        current = current.saturating_add(icache_line);
    }
    unsafe {
        asm!("dsb ish");
        asm!("isb");
    }
}

fn reload_to_link_address(src: u64, size: u64) -> ! {
    extern "C" {
        static reload_trampoline_start: u8;
        static reload_trampoline_end: u8;
    }

    let trampoline_start = unsafe { &reload_trampoline_start as *const u8 as usize };
    let trampoline_end = unsafe { &reload_trampoline_end as *const u8 as usize };
    let trampoline_len = trampoline_end.saturating_sub(trampoline_start);
    if trampoline_len == 0 || trampoline_len > 0x1000 {
        loop {
            core::hint::spin_loop();
        }
    }

    for offset in 0..trampoline_len {
        let byte = unsafe { read_volatile((trampoline_start + offset) as *const u8) };
        unsafe {
            write_volatile((XLOAD_TRAMPOLINE_ADDR as usize + offset) as *mut u8, byte);
        }
    }
    flush_loaded_image(XLOAD_TRAMPOLINE_ADDR, trampoline_len as u64);

    unsafe {
        asm!("dsb sy");
        asm!("isb");
        let trampoline: extern "C" fn(u64, u64, u64, u64) -> ! =
            core::mem::transmute(XLOAD_TRAMPOLINE_ADDR as usize);
        trampoline(LOAD_ADDR, src, size, LOAD_ADDR);
    }
}

fn cache_line_size(ctr_el0: u64, shift: u64) -> u64 {
    let encoded = (ctr_el0 >> shift) & 0xf;
    4u64 << encoded
}

fn align_down(value: u64, align: u64) -> u64 {
    if align == 0 {
        value
    } else {
        value & !(align - 1)
    }
}

fn jump_to(addr: u64) -> ! {
    unsafe {
        asm!("dsb sy");
        asm!("isb");
        let entry: extern "C" fn() -> ! = core::mem::transmute(addr as usize);
        entry();
    }
}

fn init_timebase() {
    unsafe {
        BOOT_CNTPCT = read_cntpct();
    }
}

fn boot_cntpct() -> u64 {
    unsafe { BOOT_CNTPCT }
}

fn ms_to_ticks(ms: u64, freq: u64) -> Option<u64> {
    if freq == 0 {
        return None;
    }
    let ticks = (ms as u128).checked_mul(freq as u128)?.checked_add(999)? / 1000;
    if ticks > u64::MAX as u128 {
        None
    } else {
        Some(ticks as u64)
    }
}

fn ticks_to_us(ticks: u64, freq: u64) -> u64 {
    if freq == 0 {
        return 0;
    }
    let micros = (ticks as u128).saturating_mul(1_000_000) / freq as u128;
    if micros > u64::MAX as u128 {
        u64::MAX
    } else {
        micros as u64
    }
}

fn ticks_to_ns_per_iter(ticks: u64, freq: u64, iters: u64) -> u64 {
    if freq == 0 || iters == 0 {
        return 0;
    }
    let nanos = (ticks as u128).saturating_mul(1_000_000_000);
    let per_iter = nanos / freq as u128 / iters as u128;
    if per_iter > u64::MAX as u128 {
        u64::MAX
    } else {
        per_iter as u64
    }
}

fn iters_per_second(iters: u64, ticks: u64, freq: u64) -> u64 {
    if ticks == 0 {
        return u64::MAX;
    }
    let rate = (iters as u128).saturating_mul(freq as u128) / ticks as u128;
    if rate > u64::MAX as u128 {
        u64::MAX
    } else {
        rate as u64
    }
}

fn print_duration_us(uart: &mut Uart, label: &str, us: u64) {
    let seconds = us / 1_000_000;
    let micros = us % 1_000_000;
    let _ = writeln!(uart, "{label:<16}{seconds}.{micros:06} s");
}

fn print_bench_result(uart: &mut Uart, name: &str, iters: u64, ticks: u64, freq: u64) {
    let elapsed_us = ticks_to_us(ticks, freq);
    let ns_iter = ticks_to_ns_per_iter(ticks, freq, iters);
    let rate = iters_per_second(iters, ticks, freq);

    let _ = writeln!(uart, "{name}:");
    let _ = writeln!(uart, "  ticks          {ticks}");
    print_duration_us(uart, "  elapsed", elapsed_us);
    let _ = writeln!(uart, "  ns/iter        {ns_iter}");
    let _ = writeln!(uart, "  iter/s         {rate}");
}

fn is_dram_range(addr: u64, len: u64) -> bool {
    let Some(end) = addr.checked_add(len) else {
        return false;
    };

    (addr >= DRAM_BANK0_START && end <= DRAM_BANK0_END)
        || (addr >= DRAM_BANK1_START && end <= DRAM_BANK1_END)
}

fn is_safe_read32_address(addr: u64) -> bool {
    is_dram_range(addr, 4) || find_mmio_region_by_addr(addr, 4).is_some()
}

fn find_mmio_region(name: &str) -> Option<&'static MmioRegion> {
    MMIO_REGIONS
        .iter()
        .find(|region| region.name.eq_ignore_ascii_case(name))
}

fn find_mmio_region_by_addr(addr: u64, len: u64) -> Option<&'static MmioRegion> {
    let end = addr.checked_add(len)?;
    MMIO_REGIONS
        .iter()
        .find(|region| addr >= region.start && end <= region.end)
}

fn read_current_el() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, CurrentEL", out(reg) value);
    }
    value
}

fn read_daif() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, DAIF", out(reg) value);
    }
    value
}

fn read_sp() -> u64 {
    let value;
    unsafe {
        asm!("mov {0}, sp", out(reg) value);
    }
    value
}

fn read_vbar_el2() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, VBAR_EL2", out(reg) value);
    }
    value
}

fn read_cntfrq() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, CNTFRQ_EL0", out(reg) value);
    }
    value
}

fn read_ctr_el0() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, CTR_EL0", out(reg) value);
    }
    value
}

fn read_cntpct() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, CNTPCT_EL0", out(reg) value);
    }
    value
}

fn read_esr_el2() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, ESR_EL2", out(reg) value);
    }
    value
}

fn read_elr_el2() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, ELR_EL2", out(reg) value);
    }
    value
}

fn write_elr_el2(value: u64) {
    unsafe {
        asm!("msr ELR_EL2, {0}", in(reg) value);
    }
}

fn read_far_el2() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, FAR_EL2", out(reg) value);
    }
    value
}

fn read_spsr_el2() -> u64 {
    let value;
    unsafe {
        asm!("mrs {0}, SPSR_EL2", out(reg) value);
    }
    value
}

fn read_vbar_el2_for_exception() -> u64 {
    read_vbar_el2()
}

#[no_mangle]
pub extern "C" fn exception_handler(vector: u64, frame: *const ExceptionFrame) {
    let mut uart = Uart::new(UART2_BASE);
    let esr = read_esr_el2();
    let elr = read_elr_el2();
    let far = read_far_el2();
    let spsr = read_spsr_el2();
    let vbar = read_vbar_el2_for_exception();
    let ec = esr >> ESR_EC_SHIFT;
    let frame_ref = unsafe { &*frame };
    let snapshot = ExceptionSnapshot {
        valid: true,
        vector,
        esr,
        elr,
        far,
        spsr,
        vbar,
        x0: frame_ref.regs[0],
        x1: frame_ref.regs[1],
        x29: frame_ref.regs[29],
        x30: frame_ref.regs[30],
    };
    unsafe {
        LAST_EXCEPTION = snapshot;
    }

    if ec == ESR_EC_DATA_ABORT_CURRENT && PROBE_ACTIVE.load(Ordering::SeqCst) {
        PROBE_FAULTED.store(true, Ordering::SeqCst);
        write_elr_el2(elr.wrapping_add(4));
        return;
    }

    uart.put_str("\n");
    uart.put_str("========================================\n");
    uart.put_str("Xiaojin OS exception\n");
    uart.put_str("========================================\n");
    print_exception_snapshot(&mut uart, &snapshot);

    if ec == ESR_EC_BRK_A64 {
        write_elr_el2(elr.wrapping_add(4));
        uart.put_str("BRK handled: advancing ELR by 4 and returning.\n");
        uart.put_str("========================================\n");
        return;
    }

    uart.put_str("fatal exception: system halted.\n");
    uart.put_str("========================================\n");
    loop {
        core::hint::spin_loop();
    }
}

fn print_exception_snapshot(uart: &mut Uart, snapshot: &ExceptionSnapshot) {
    let ec = snapshot.esr >> ESR_EC_SHIFT;
    let il = (snapshot.esr >> ESR_IL_SHIFT) & 1;
    let iss = snapshot.esr & ESR_ISS_MASK;

    let _ = writeln!(
        uart,
        "vector           {} ({})",
        snapshot.vector,
        vector_name(snapshot.vector)
    );
    let _ = writeln!(
        uart,
        "ESR_EL2          0x{:016x} ec=0x{ec:02x} ({})",
        snapshot.esr,
        exception_class_name(ec)
    );
    let _ = writeln!(uart, "IL/ISS           il={il} iss=0x{iss:08x}");
    match ec {
        ESR_EC_BRK_A64 => {
            let _ = writeln!(uart, "BRK immediate    0x{:04x}", iss & 0xffff);
        }
        ESR_EC_DATA_ABORT_CURRENT => print_data_abort_decode(uart, iss),
        ESR_EC_INSN_ABORT_CURRENT => print_instruction_abort_decode(uart, iss),
        _ => {}
    }
    let _ = writeln!(uart, "ELR_EL2          0x{:016x}", snapshot.elr);
    let _ = writeln!(uart, "FAR_EL2          0x{:016x}", snapshot.far);
    let _ = writeln!(uart, "SPSR_EL2         0x{:016x}", snapshot.spsr);
    let _ = writeln!(uart, "VBAR_EL2         0x{:016x}", snapshot.vbar);
    let _ = writeln!(uart, "x0               0x{:016x}", snapshot.x0);
    let _ = writeln!(uart, "x1               0x{:016x}", snapshot.x1);
    let _ = writeln!(uart, "x29              0x{:016x}", snapshot.x29);
    let _ = writeln!(uart, "x30              0x{:016x}", snapshot.x30);
}

fn print_data_abort_decode(uart: &mut Uart, iss: u64) {
    let fsc = iss & 0x3f;
    let write = (iss >> 6) & 1 != 0;
    let s1ptw = (iss >> 7) & 1 != 0;
    let cm = (iss >> 8) & 1 != 0;
    let ea = (iss >> 9) & 1 != 0;
    let fnv = (iss >> 10) & 1 != 0;
    let isv = (iss >> 24) & 1 != 0;

    let _ = writeln!(
        uart,
        "abort fsc        0x{fsc:02x} ({})",
        abort_fsc_name(fsc)
    );
    let _ = writeln!(
        uart,
        "abort access     {} far={} s1ptw={} ea={} cm={}",
        if write { "write" } else { "read" },
        if fnv { "invalid" } else { "valid" },
        yes_no(s1ptw),
        yes_no(ea),
        yes_no(cm)
    );

    if isv {
        let sas = (iss >> 22) & 0x3;
        let srt = (iss >> 16) & 0x1f;
        let sf = (iss >> 15) & 1 != 0;
        let ar = (iss >> 14) & 1 != 0;
        let _ = writeln!(
            uart,
            "data detail      size={} reg=x{srt} sf={} ar={}",
            data_access_size_name(sas),
            yes_no(sf),
            yes_no(ar)
        );
    } else {
        uart.put_str("data detail      isv=0 (size/register unavailable)\n");
    }
}

fn print_instruction_abort_decode(uart: &mut Uart, iss: u64) {
    let fsc = iss & 0x3f;
    let s1ptw = (iss >> 7) & 1 != 0;
    let ea = (iss >> 9) & 1 != 0;
    let fnv = (iss >> 10) & 1 != 0;

    let _ = writeln!(
        uart,
        "abort fsc        0x{fsc:02x} ({})",
        abort_fsc_name(fsc)
    );
    let _ = writeln!(
        uart,
        "abort access     instruction far={} s1ptw={} ea={}",
        if fnv { "invalid" } else { "valid" },
        yes_no(s1ptw),
        yes_no(ea)
    );
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn data_access_size_name(sas: u64) -> &'static str {
    match sas {
        0 => "byte",
        1 => "halfword",
        2 => "word",
        3 => "doubleword",
        _ => "unknown",
    }
}

fn abort_fsc_name(fsc: u64) -> &'static str {
    match fsc {
        0x00 => "address size fault, level 0",
        0x01 => "address size fault, level 1",
        0x02 => "address size fault, level 2",
        0x03 => "address size fault, level 3",
        0x04 => "translation fault, level 0",
        0x05 => "translation fault, level 1",
        0x06 => "translation fault, level 2",
        0x07 => "translation fault, level 3",
        0x09 => "access flag fault, level 1",
        0x0a => "access flag fault, level 2",
        0x0b => "access flag fault, level 3",
        0x0d => "permission fault, level 1",
        0x0e => "permission fault, level 2",
        0x0f => "permission fault, level 3",
        0x10 => "synchronous external abort",
        0x14 => "external abort on table walk, level 0",
        0x15 => "external abort on table walk, level 1",
        0x16 => "external abort on table walk, level 2",
        0x17 => "external abort on table walk, level 3",
        0x18 => "synchronous parity or ECC error",
        0x1c => "parity or ECC error on table walk, level 0",
        0x1d => "parity or ECC error on table walk, level 1",
        0x1e => "parity or ECC error on table walk, level 2",
        0x1f => "parity or ECC error on table walk, level 3",
        0x21 => "alignment fault",
        0x22 => "debug event",
        0x30 => "TLB conflict abort",
        _ => "unclassified or implementation-defined",
    }
}

fn vector_name(vector: u64) -> &'static str {
    match vector {
        0 => "current EL SP0 sync",
        1 => "current EL SP0 irq",
        2 => "current EL SP0 fiq",
        3 => "current EL SP0 serror",
        4 => "current EL SPx sync",
        5 => "current EL SPx irq",
        6 => "current EL SPx fiq",
        7 => "current EL SPx serror",
        8 => "lower EL AArch64 sync",
        9 => "lower EL AArch64 irq",
        10 => "lower EL AArch64 fiq",
        11 => "lower EL AArch64 serror",
        12 => "lower EL AArch32 sync",
        13 => "lower EL AArch32 irq",
        14 => "lower EL AArch32 fiq",
        15 => "lower EL AArch32 serror",
        _ => "unknown",
    }
}

fn exception_class_name(ec: u64) -> &'static str {
    match ec {
        ESR_EC_INSN_ABORT_CURRENT => "instruction abort, current EL",
        ESR_EC_DATA_ABORT_CURRENT => "data abort, current EL",
        ESR_EC_BRK_A64 => "BRK, AArch64",
        _ => "unknown",
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = Uart::new(UART2_BASE);
    let _ = writeln!(uart, "\nKERNEL PANIC: {}", info);
    loop {
        core::hint::spin_loop();
    }
}
