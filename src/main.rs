#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr::{read_volatile, write_volatile};
use core::str;

global_asm!(include_str!("boot.S"));

const UART2_BASE: usize = 0xfe66_0000;
const UART_RBR_THR: usize = 0x00;
const UART_LSR: usize = 0x14;
const UART_LSR_DR: u32 = 1 << 0;
const UART_LSR_THRE: u32 = 1 << 5;

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

struct MmioRegion {
    name: &'static str,
    start: u64,
    end: u64,
    description: &'static str,
}

const MMIO_REGIONS: &[MmioRegion] = &[MmioRegion {
    name: "uart2",
    start: UART2_BASE as u64,
    end: UART2_BASE as u64 + 0x100,
    description: "RK3568 debug UART2",
}];

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

    uart.put_str("\n");
    uart.put_str("========================================\n");
    uart.put_str("Hello from Xiaojin OS.\n");
    uart.put_str("AArch64 bare-metal Rust control shell is running.\n");
    uart.put_str("Board profile: nanopi-r5c / rk3568\n");
    uart.put_str("UART2 MMIO base: 0xfe660000\n");
    uart.put_str("Loaded at: 0x04080000\n");
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
    uart.put_str("  about                show kernel and board identity\n");
    uart.put_str("  mem                  show current memory map facts\n");
    uart.put_str("  regs                 show selected CPU registers\n");
    uart.put_str("  time                 show generic timer registers and uptime\n");
    uart.put_str("  uptime               show elapsed kernel time\n");
    uart.put_str("  sleep <ms>           busy-wait with generic timer, max 10000\n");
    uart.put_str("  bench [iters]        run timer-backed CPU microbenchmarks\n");
    uart.put_str("  hexdump <addr> [n]   dump DRAM bytes, max 256\n");
    uart.put_str("  peek32 <addr>        read one safe 32-bit word\n");
    uart.put_str("  mmio list            list whitelisted MMIO regions\n");
    uart.put_str("  mmio read32 <r> <o>  read one region-relative 32-bit MMIO word\n");
    uart.put_str("  last-exception       show the latest captured exception\n");
    uart.put_str("  fault                trigger BRK to test exception vectors\n");
    uart.put_str("  abort confirm        trigger fatal data abort test\n");
    uart.put_str("  reboot               request PSCI system reset\n");
}

fn print_about(uart: &mut Uart) {
    uart.put_str("Xiaojin OS 0.1.0\n");
    uart.put_str("board: nanopi-r5c\n");
    uart.put_str("soc: rk3568\n");
    uart.put_str("kernel: AArch64 no_std Rust\n");
    uart.put_str("control: UART2 shell\n");
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

    let value = unsafe { read_volatile(addr as usize as *const u32) };
    let _ = writeln!(uart, "0x{addr:016x}: 0x{value:08x}");
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
            "  {:<8} 0x{:016x}..0x{:016x}  {}",
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

    let value = unsafe { read_volatile(addr as usize as *const u32) };
    let _ = writeln!(
        uart,
        "{}+0x{offset:x} @ 0x{addr:016x}: 0x{value:08x}",
        region.name
    );
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
