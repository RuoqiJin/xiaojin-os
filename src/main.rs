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
const PMUCRU_BASE: u64 = 0xfdd0_0000;
const CRU_BASE: u64 = 0xfdd2_0000;
const RK3568_CLKGATE_BASE: u64 = 0x300;
const RK3568_SOFTRST_BASE: u64 = 0x400;
const RK3568_PMU_CLKGATE_BASE: u64 = 0x180;
const PCIE30_PHY_GRF_CON1: u64 = 0x04;
const PCIE30_PHY_GRF_CON6: u64 = 0x18;
const PCIE30_PHY_GRF_CON9: u64 = 0x24;
const PCIE30_PHY_GRF_STATUS0: u64 = 0x80;
const PCIE30_PHY_SRAM_INIT_DONE: u32 = 1 << 14;
const PCIE30_PHY_DA_OCM: u32 = (1 << 31) | (1 << 15);
const PCIE30_PHY_WR_EN: u32 = 0x000f_0000;
const PCIE30_PHY_BIFURCATION_LANE_0_1: u32 = 1 << 0;
const PCIE_CLIENT_GENERAL_CON: u64 = 0x000;
const PCIE_CLIENT_INTR_STATUS_MISC: u64 = 0x010;
const PCIE_CLIENT_POWER_CON: u64 = 0x02c;
const PCIE_CLIENT_HOT_RESET_CTRL: u64 = 0x180;
const PCIE_CLIENT_LTSSM_STATUS: u64 = 0x300;
const PCIE_CLIENT_LTSSM_ENABLE_BIT: u32 = 1 << 2;
const PCIE_CLIENT_RC_MODE_BIT: u32 = 1 << 6;
const PCIE_CLIENT_SET_RC_MODE: u32 = 0x00f0_0040;
const PCIE_CLIENT_ENABLE_LTSSM_VALUE: u32 = 0x0004_0004;
const PCIE_CLIENT_DISABLE_LTSSM_VALUE: u32 = 0x0004_0000;
const PCIE_CLIENT_HOT_RESET_ENHANCE: u32 = 0x0010_0010;
const PCIE_CLIENT_CLKREQ_PULL_DOWN_NOT_READY: u32 = 0x3001_1000;
const PCIE_CLIENT_LINKUP_MASK: u32 = 0x3 << 16;
const PCIE_CLIENT_LINKUP: u32 = 0x3 << 16;
const PCIE_CLIENT_LTSSM_STATUS_MASK: u32 = 0x3f;
const PCIE_CLIENT_LTSSM_L0: u32 = 0x11;
const PCIE_PHY_INIT_TIMEOUT_MS: u64 = 100;
const PCIE_LINK_TRAIN_TIMEOUT_MS: u64 = 1_000;
const PCIE_ATU_VIEWPORT: u64 = 0x900;
const PCIE_ATU_VIEWPORT_BASE: u64 = 0x904;
const PCIE_ATU_UNROLL_BASE: u64 = 0x0030_0000;
const PCIE_ATU_UNROLL_REGION_SIZE: u64 = 0x200;
const PCIE_ATU_REGION_CTRL1: u64 = 0x000;
const PCIE_ATU_REGION_CTRL2: u64 = 0x004;
const PCIE_ATU_LOWER_BASE: u64 = 0x008;
const PCIE_ATU_UPPER_BASE: u64 = 0x00c;
const PCIE_ATU_LIMIT: u64 = 0x010;
const PCIE_ATU_LOWER_TARGET: u64 = 0x014;
const PCIE_ATU_UPPER_TARGET: u64 = 0x018;
const PCIE_ATU_TYPE_CFG0: u32 = 0x4;
const PCIE_ATU_TYPE_CFG1: u32 = 0x5;
const PCIE_ATU_TYPE_MEM: u32 = 0x0;
const PCIE_ATU_ENABLE: u32 = 1 << 31;
const PCIE_CFG_WINDOW_SIZE: u64 = 0x0010_0000;
const PCIE_ATU_PROBE_UNROLLED: u32 = 0xffff_ffff;
const RTL8125_BAR2_SIZE: u64 = 0x0001_0000;
const RTL8125_BAR4_SIZE: u64 = 0x0000_4000;
const RTL8125_MEM_WINDOW_SIZE: u64 = RTL8125_BAR2_SIZE + RTL8125_BAR4_SIZE;
const RTL_REG_MAC0: u64 = 0x0000;
const RTL_REG_TX_DESC_ADDR_LOW: u64 = 0x0020;
const RTL_REG_TX_DESC_ADDR_HIGH: u64 = 0x0024;
const RTL_REG_CHIP_CMD_WORD: u64 = 0x0034;
const RTL_REG_INTR_STATUS_8125: u64 = 0x003c;
const RTL_REG_TX_CONFIG: u64 = 0x0040;
const RTL_REG_RX_CONFIG: u64 = 0x0044;
const RTL_REG_CFG9346: u64 = 0x0050;
const RTL_REG_PHY_STATUS: u64 = 0x006c;
const RTL_REG_TX_POLL_8125: u64 = 0x0090;
const RTL_REG_RX_MAX_SIZE: u64 = 0x00da;
const RTL_REG_CPLUS_CMD: u64 = 0x00e0;
const RTL_REG_RX_DESC_ADDR_LOW: u64 = 0x00e4;
const RTL_REG_RX_DESC_ADDR_HIGH: u64 = 0x00e8;
const RTL_REG_MAC0_BKP: u64 = 0x19e0;
const RTL_CHIP_CMD_BYTE_IN_WORD: u32 = 24;
const RTL_CHIP_CMD_TX_ENABLE: u8 = 0x04;
const RTL_CHIP_CMD_RX_ENABLE: u8 = 0x08;
const RTL_CHIP_CMD_RESET: u8 = 0x10;
const RTL_PHY_LINK_STATUS: u32 = 0x02;
const RTL_PHY_FULL_DUPLEX: u32 = 0x01;
const RTL_PHY_1000_FULL: u32 = 0x10;
const RTL_PHY_100M: u32 = 0x08;
const RTL_PHY_10M: u32 = 0x04;
const RTL_CFG9346_UNLOCK: u8 = 0xc0;
const RTL_CFG9346_LOCK: u8 = 0x00;
const RTL_DESC_OWN: u32 = 1 << 31;
const RTL_DESC_RING_END: u32 = 1 << 30;
const RTL_DESC_FIRST_FRAG: u32 = 1 << 29;
const RTL_DESC_LAST_FRAG: u32 = 1 << 28;
const RTL_RX_FETCH_DFLT_8125: u32 = 8 << 27;
const RTL_RX_PAUSE_SLOT_ON: u32 = 1 << 11;
const RTL_RX_DMA_BURST: u32 = 7 << 8;
const RTL_RX_ACCEPT_BROADCAST: u32 = 0x08;
const RTL_RX_ACCEPT_MULTICAST: u32 = 0x04;
const RTL_RX_ACCEPT_MY_PHYS: u32 = 0x02;
const RTL_TX_DMA_BURST: u32 = 7 << 8;
const RTL_TX_INTER_FRAME_GAP: u32 = 3 << 24;
const RTL_RX_BUFFER_SIZE: usize = 2048;
const RTL_TX_BUFFER_SIZE: usize = 1536;
const RTL_TX_DESC_COUNT: usize = 8;
const RTL_RX_DESC_COUNT: usize = 8;
const RTL_DEVICE_COUNT: usize = 2;
const RTL_INIT_TIMEOUT_MS: u64 = 100;
const PCI_COMMAND_MEMORY: u32 = 1 << 1;
const PCI_COMMAND_MASTER: u32 = 1 << 2;
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
static mut RTL_TX_RINGS: RtlTxRings =
    RtlTxRings([[RtlDesc::zero(); RTL_TX_DESC_COUNT]; RTL_DEVICE_COUNT]);
static mut RTL_RX_RINGS: RtlRxRings =
    RtlRxRings([[RtlDesc::zero(); RTL_RX_DESC_COUNT]; RTL_DEVICE_COUNT]);
static mut RTL_TX_BUFFERS: RtlTxBuffers = RtlTxBuffers([[0; RTL_TX_BUFFER_SIZE]; RTL_DEVICE_COUNT]);
static mut RTL_RX_BUFFERS: RtlRxBuffers =
    RtlRxBuffers([[[0; RTL_RX_BUFFER_SIZE]; RTL_RX_DESC_COUNT]; RTL_DEVICE_COUNT]);
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

#[derive(Clone, Copy)]
struct PcieController {
    name: &'static str,
    apb_base: u64,
    dbi_base: u64,
    cfg_base: u64,
    mem32_base: u64,
    root_bus: u8,
    reset_pin: &'static str,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RtlDesc {
    opts1: u32,
    opts2: u32,
    addr: u64,
}

impl RtlDesc {
    const fn zero() -> Self {
        Self {
            opts1: 0,
            opts2: 0,
            addr: 0,
        }
    }
}

#[repr(C, align(256))]
struct RtlTxRings([[RtlDesc; RTL_TX_DESC_COUNT]; RTL_DEVICE_COUNT]);

#[repr(C, align(256))]
struct RtlRxRings([[RtlDesc; RTL_RX_DESC_COUNT]; RTL_DEVICE_COUNT]);

#[repr(C, align(64))]
struct RtlTxBuffers([[u8; RTL_TX_BUFFER_SIZE]; RTL_DEVICE_COUNT]);

#[repr(C, align(64))]
struct RtlRxBuffers([[[u8; RTL_RX_BUFFER_SIZE]; RTL_RX_DESC_COUNT]; RTL_DEVICE_COUNT]);

struct RtlChipInfo {
    mask: u32,
    value: u32,
    mac_version: &'static str,
    name: &'static str,
}

#[derive(Clone, Copy)]
struct CruBit {
    name: &'static str,
    base: u64,
    offset: u64,
    bit: u8,
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
        active_high: false,
        source: "reset-gpios, release-high observed",
    },
    PcieResetPin {
        controller: "pcie3x2",
        bank: GPIO0,
        pin: 14,
        label: "PB6",
        active_high: false,
        source: "reset-gpios, release-high observed",
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

const PCIE_CONTROLLERS: &[PcieController] = &[
    PcieController {
        name: "pcie3x1",
        apb_base: PCIE3X1_APB_BASE,
        dbi_base: PCIE3X1_DBI_BASE,
        cfg_base: PCIE3X1_CFG_BASE,
        mem32_base: PCIE3X1_MEM32_BASE,
        root_bus: 0x10,
        reset_pin: "gpio0 PA0",
    },
    PcieController {
        name: "pcie3x2",
        apb_base: PCIE3X2_APB_BASE,
        dbi_base: PCIE3X2_DBI_BASE,
        cfg_base: PCIE3X2_CFG_BASE,
        mem32_base: PCIE3X2_MEM32_BASE,
        root_bus: 0x20,
        reset_pin: "gpio0 PB6",
    },
];

const RTL_CHIP_INFOS_8125: &[RtlChipInfo] = &[
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x681,
        mac_version: "RTL_GIGA_MAC_VER_66",
        name: "RTL8125BP",
    },
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x708,
        mac_version: "RTL_GIGA_MAC_VER_65",
        name: "RTL8125CP",
    },
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x68b,
        mac_version: "RTL_GIGA_MAC_VER_64",
        name: "RTL9151A",
    },
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x68a,
        mac_version: "RTL_GIGA_MAC_VER_64",
        name: "RTL8125K",
    },
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x689,
        mac_version: "RTL_GIGA_MAC_VER_64",
        name: "RTL8125D",
    },
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x688,
        mac_version: "RTL_GIGA_MAC_VER_64",
        name: "RTL8125D",
    },
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x641,
        mac_version: "RTL_GIGA_MAC_VER_63",
        name: "RTL8125B",
    },
    RtlChipInfo {
        mask: 0x7cf,
        value: 0x609,
        mac_version: "RTL_GIGA_MAC_VER_61",
        name: "RTL8125A",
    },
];

const PCIE_CLOCK_GATES: &[CruBit] = &[
    CruBit {
        name: "aclk_pipe",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 10 * 4,
        bit: 0,
    },
    CruBit {
        name: "pclk_pipe",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 10 * 4,
        bit: 1,
    },
    CruBit {
        name: "aclk_pcie30x1_mst",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 8,
    },
    CruBit {
        name: "aclk_pcie30x1_slv",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 9,
    },
    CruBit {
        name: "aclk_pcie30x1_dbi",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 10,
    },
    CruBit {
        name: "pclk_pcie30x1",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 11,
    },
    CruBit {
        name: "clk_pcie30x1_aux",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 12,
    },
    CruBit {
        name: "clk_pcie30x1_pipe",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 13,
    },
    CruBit {
        name: "aclk_pcie30x2_mst",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 13 * 4,
        bit: 0,
    },
    CruBit {
        name: "aclk_pcie30x2_slv",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 13 * 4,
        bit: 1,
    },
    CruBit {
        name: "aclk_pcie30x2_dbi",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 13 * 4,
        bit: 2,
    },
    CruBit {
        name: "pclk_pcie30x2",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 13 * 4,
        bit: 3,
    },
    CruBit {
        name: "clk_pcie30x2_aux",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 13 * 4,
        bit: 4,
    },
    CruBit {
        name: "clk_pcie30x2_pipe",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 13 * 4,
        bit: 5,
    },
    CruBit {
        name: "pclk_pcie30phy",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 33 * 4,
        bit: 8,
    },
    CruBit {
        name: "clk_pcie30phy_ref_m",
        base: PMUCRU_BASE,
        offset: RK3568_PMU_CLKGATE_BASE + 2 * 4,
        bit: 13,
    },
    CruBit {
        name: "clk_pcie30phy_ref_n",
        base: PMUCRU_BASE,
        offset: RK3568_PMU_CLKGATE_BASE + 2 * 4,
        bit: 14,
    },
];

const PCIE_RESETS: &[CruBit] = &[
    CruBit {
        name: "srst_pcie30x1_powerup",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 11 * 4,
        bit: 1,
    },
    CruBit {
        name: "srst_pcie30x2_powerup",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 12 * 4,
        bit: 1,
    },
    CruBit {
        name: "srst_p_pcie30phy",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 27 * 4,
        bit: 13,
    },
    CruBit {
        name: "srst_pcie30phy",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 27 * 4,
        bit: 14,
    },
    CruBit {
        name: "srst_p_pcie30phy_grf",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 27 * 4,
        bit: 15,
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
    uart.put_str("  pcie phy-status      read PCIe3 PHY GRF status\n");
    uart.put_str("  pcie phy-init confirm\n");
    uart.put_str("                       initialize shared PCIe3 PHY GRF state\n");
    uart.put_str("  pcie apb-status confirm\n");
    uart.put_str("                       read PCIe controller APB status registers\n");
    uart.put_str("  pcie cfg-probe confirm\n");
    uart.put_str("                       read PCIe DBI/config probe words\n");
    uart.put_str("  pcie iatu-probe confirm\n");
    uart.put_str("                       read DWC iATU mode/status words\n");
    uart.put_str("  pcie cfg-scan confirm\n");
    uart.put_str("                       configure iATU CFG windows and scan endpoints\n");
    uart.put_str("  pcie bar-scan confirm\n");
    uart.put_str("                       size endpoint PCI BARs and restore them\n");
    uart.put_str("  pcie bar-map confirm\n");
    uart.put_str("                       assign RTL8125 BAR2/BAR4 MMIO windows\n");
    uart.put_str("  pcie rtl-probe confirm\n");
    uart.put_str("                       read mapped RTL8125 MMIO probe words\n");
    uart.put_str("  pcie rtl-facts confirm\n");
    uart.put_str("                       decode mapped RTL8125 MAC/chip/link facts\n");
    uart.put_str("  pcie rtl-init confirm\n");
    uart.put_str("                       initialize RTL8125 descriptor rings, no IRQs\n");
    uart.put_str("  pcie rtl-tx-test confirm\n");
    uart.put_str("                       send one raw broadcast Ethernet test frame\n");
    uart.put_str("  pcie rtl-rx-poll confirm\n");
    uart.put_str("                       poll RTL8125 RX descriptors and print frame heads\n");
    uart.put_str("  pcie cru-status      read PCIe CRU clock/reset bits\n");
    uart.put_str("  pcie cru-enable confirm\n");
    uart.put_str("                       enable PCIe CRU clocks and deassert resets\n");
    uart.put_str("  pcie rc-init confirm\n");
    uart.put_str("                       set PCIe3 controllers to RC pre-LTSSM mode\n");
    uart.put_str("  pcie ltssm-enable confirm\n");
    uart.put_str("                       enable PCIe link training and poll link state\n");
    uart.put_str("  pcie ltssm-disable confirm\n");
    uart.put_str("                       disable PCIe link training\n");
    uart.put_str("  pcie link-start confirm\n");
    uart.put_str("                       run PCIe3 PHY/RC/LTSSM/start-link sequence\n");
    uart.put_str("  pcie reset-deassert confirm\n");
    uart.put_str("                       release RJ45 PCIe reset GPIOs\n");
    uart.put_str("  pcie reset-assert confirm\n");
    uart.put_str("                       hold RJ45 PCIe reset GPIOs\n");
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
        Some("phy-status") => print_pcie_phy_status(uart),
        Some("phy-init") => command_pcie_phy_init(uart, arg1),
        Some("apb-status") => command_pcie_apb_status(uart, arg1),
        Some("cfg-probe") => command_pcie_cfg_probe(uart, arg1),
        Some("iatu-probe") => command_pcie_iatu_probe(uart, arg1),
        Some("cfg-scan") => command_pcie_cfg_scan(uart, arg1),
        Some("bar-scan") => command_pcie_bar_scan(uart, arg1),
        Some("bar-map") => command_pcie_bar_map(uart, arg1),
        Some("rtl-probe") => command_pcie_rtl_probe(uart, arg1),
        Some("rtl-facts") => command_pcie_rtl_facts(uart, arg1),
        Some("rtl-init") => command_pcie_rtl_init(uart, arg1),
        Some("rtl-tx-test") => command_pcie_rtl_tx_test(uart, arg1),
        Some("rtl-rx-poll") => command_pcie_rtl_rx_poll(uart, arg1),
        Some("cru-status") => print_pcie_cru_status(uart),
        Some("cru-enable") => command_pcie_cru_enable(uart, arg1),
        Some("rc-init") => command_pcie_rc_init(uart, arg1),
        Some("ltssm-enable") => command_pcie_ltssm(uart, true, arg1),
        Some("ltssm-disable") => command_pcie_ltssm(uart, false, arg1),
        Some("link-start") => command_pcie_link_start(uart, arg1),
        Some("init") => command_pcie_init(uart),
        Some("reset-deassert") => command_pcie_reset_write(uart, false, arg1),
        Some("reset-assert") => command_pcie_reset_write(uart, true, arg1),
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  pcie facts\n");
            uart.put_str("  pcie reset-pins\n");
            uart.put_str("  pcie status\n");
            uart.put_str("  pcie phy-status\n");
            uart.put_str("  pcie phy-init confirm\n");
            uart.put_str("  pcie apb-status confirm\n");
            uart.put_str("  pcie cfg-probe confirm\n");
            uart.put_str("  pcie iatu-probe confirm\n");
            uart.put_str("  pcie cfg-scan confirm\n");
            uart.put_str("  pcie bar-scan confirm\n");
            uart.put_str("  pcie bar-map confirm\n");
            uart.put_str("  pcie rtl-probe confirm\n");
            uart.put_str("  pcie rtl-facts confirm\n");
            uart.put_str("  pcie rtl-init confirm\n");
            uart.put_str("  pcie rtl-tx-test confirm\n");
            uart.put_str("  pcie rtl-rx-poll confirm\n");
            uart.put_str("  pcie cru-status\n");
            uart.put_str("  pcie cru-enable confirm\n");
            uart.put_str("  pcie rc-init confirm\n");
            uart.put_str("  pcie ltssm-enable confirm\n");
            uart.put_str("  pcie ltssm-disable confirm\n");
            uart.put_str("  pcie link-start confirm\n");
            uart.put_str("  pcie init\n");
            uart.put_str("  pcie reset-deassert confirm\n");
            uart.put_str("  pcie reset-assert confirm\n");
        }
    }
}

fn print_pcie_facts(uart: &mut Uart) {
    uart.put_str("R5C PCIe facts from board profile:\n");
    uart.put_str("  pcie3x1          enabled, domain=1, bus=0x10..0x1f, lanes=1\n");
    uart.put_str("  pcie3x1 reset    gpio0 A0, release-high observed on hardware\n");
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
    uart.put_str("  pcie3x2 reset    gpio0 B6, release-high observed on hardware\n");
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
    uart.put_str("  reason           clock/PHY/LTSSM writes need one more verified map pass\n");
    uart.put_str("  next             inspect PHY GRF and APB LTSSM status\n");
    print_pcie_status(uart);
    print_pcie_phy_status(uart);
}

fn command_pcie_apb_status(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie apb-status confirm\n");
        uart.put_str("scope: read controller-local APB status registers only\n");
        uart.put_str("note: DBI/config/BAR windows remain disabled in generic MMIO reads\n");
        return;
    }

    uart.put_str("pcie apb status:\n");
    for controller in PCIE_CONTROLLERS {
        print_pcie_controller_apb_status(uart, *controller);
    }
}

fn command_pcie_cfg_probe(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie cfg-probe confirm\n");
        uart.put_str("scope: read DBI header and config-window probe words only\n");
        uart.put_str("note: run `pcie link-start confirm` first\n");
        return;
    }

    uart.put_str("pcie cfg probe:\n");
    for controller in PCIE_CONTROLLERS {
        let _ = writeln!(
            uart,
            "  {} dbi=0x{:016x} cfg=0x{:016x}",
            controller.name, controller.dbi_base, controller.cfg_base
        );
        print_pcie_reg(
            uart,
            "    dbi.vendor",
            controller.dbi_base,
            probe_read32(controller.dbi_base),
        );
        print_pcie_reg(
            uart,
            "    dbi.command",
            controller.dbi_base + 0x04,
            probe_read32(controller.dbi_base + 0x04),
        );
        print_pcie_reg(
            uart,
            "    dbi.class",
            controller.dbi_base + 0x08,
            probe_read32(controller.dbi_base + 0x08),
        );
        print_pcie_reg(
            uart,
            "    cfg.word0",
            controller.cfg_base,
            probe_read32(controller.cfg_base),
        );
    }
}

fn command_pcie_iatu_probe(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie iatu-probe confirm\n");
        uart.put_str("scope: read DWC iATU viewport/unroll status words only\n");
        return;
    }

    uart.put_str("pcie iatu probe:\n");
    for controller in PCIE_CONTROLLERS {
        let viewport = probe_read32(controller.dbi_base + PCIE_ATU_VIEWPORT);
        let mode = if iatu_is_unrolled(*controller) {
            "unrolled"
        } else {
            "viewport"
        };
        let _ = writeln!(
            uart,
            "  {} mode={} dbi=0x{:016x}",
            controller.name, mode, controller.dbi_base
        );
        print_pcie_reg(
            uart,
            "    viewport",
            controller.dbi_base + PCIE_ATU_VIEWPORT,
            viewport,
        );
        print_pcie_reg(
            uart,
            "    ob0.ctrl1",
            pcie_atu_addr(*controller, 0, PCIE_ATU_REGION_CTRL1),
            probe_read32(pcie_atu_addr(*controller, 0, PCIE_ATU_REGION_CTRL1)),
        );
        print_pcie_reg(
            uart,
            "    ob0.ctrl2",
            pcie_atu_addr(*controller, 0, PCIE_ATU_REGION_CTRL2),
            probe_read32(pcie_atu_addr(*controller, 0, PCIE_ATU_REGION_CTRL2)),
        );
        print_pcie_reg(
            uart,
            "    ob0.target",
            pcie_atu_addr(*controller, 0, PCIE_ATU_LOWER_TARGET),
            probe_read32(pcie_atu_addr(*controller, 0, PCIE_ATU_LOWER_TARGET)),
        );
    }
}

fn command_pcie_cfg_scan(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie cfg-scan confirm\n");
        uart.put_str("scope: program outbound iATU region 0 for config reads only\n");
        uart.put_str("note: run `pcie link-start confirm` first\n");
        return;
    }

    uart.put_str("pcie cfg scan:\n");
    for controller in PCIE_CONTROLLERS {
        let mut found = false;
        let _ = writeln!(
            uart,
            "  {} root-bus=0x{:02x} cfg=0x{:016x}",
            controller.name, controller.root_bus, controller.cfg_base
        );

        let bus_candidates = [
            0,
            1,
            controller.root_bus,
            controller.root_bus.wrapping_add(1),
            controller.root_bus.wrapping_add(2),
            controller.root_bus.wrapping_add(3),
        ];
        let type_candidates = [PCIE_ATU_TYPE_CFG0, PCIE_ATU_TYPE_CFG1];
        for bus in bus_candidates {
            for cfg_type in type_candidates {
                for dev in 0..=3u8 {
                    pcie_program_cfg_atu(*controller, bus, dev, 0, cfg_type);
                    delay_ms(1);
                    let vendor = probe_read32(controller.cfg_base);
                    match vendor {
                        Ok(value) if value != 0xffff_ffff && value != 0 => {
                            found = true;
                            let command = probe_read32(controller.cfg_base + 0x04).unwrap_or(0);
                            let class = probe_read32(controller.cfg_base + 0x08).unwrap_or(0);
                            let bar0 = probe_read32(controller.cfg_base + 0x10).unwrap_or(0);
                            let bar1 = probe_read32(controller.cfg_base + 0x14).unwrap_or(0);
                            let _ = writeln!(
                                uart,
                                "    bus=0x{bus:02x} dev={dev} fn=0 type={} vendor-device=0x{value:08x} command=0x{command:08x} class=0x{class:08x} bar0=0x{bar0:08x} bar1=0x{bar1:08x}",
                                cfg_type_name(cfg_type)
                            );
                        }
                        Ok(_) => {}
                        Err(snapshot) => {
                            let _ = writeln!(
                                uart,
                                "    bus=0x{bus:02x} dev={dev} fn=0 type={} config read fault ec=0x{:02x} far=0x{:016x}",
                                cfg_type_name(cfg_type),
                                snapshot.esr >> ESR_EC_SHIFT,
                                snapshot.far
                            );
                        }
                    }
                }
            }
        }

        if !found {
            uart.put_str("    no endpoint found in scanned range\n");
        }
    }
}

fn command_pcie_bar_scan(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie bar-scan confirm\n");
        uart.put_str("scope: write 0xffffffff to endpoint BARs, read size mask, restore\n");
        uart.put_str("note: run `pcie link-start confirm` first\n");
        return;
    }

    uart.put_str("pcie bar scan:\n");
    for controller in PCIE_CONTROLLERS {
        pcie_program_cfg_atu(
            *controller,
            controller.root_bus.wrapping_add(1),
            0,
            0,
            PCIE_ATU_TYPE_CFG0,
        );
        let vendor = probe_read32(controller.cfg_base).unwrap_or(0xffff_ffff);
        let _ = writeln!(
            uart,
            "  {} endpoint vendor-device=0x{vendor:08x}",
            controller.name
        );
        if vendor == 0xffff_ffff || vendor == 0 {
            uart.put_str("    endpoint not visible; run link-start first\n");
            continue;
        }

        let mut bar = 0u8;
        while bar < 6 {
            let offset = 0x10 + (bar as u64) * 4;
            let original = probe_read32(controller.cfg_base + offset).unwrap_or(0);
            write_mmio32(controller.cfg_base + offset, 0xffff_ffff);
            let mask = probe_read32(controller.cfg_base + offset).unwrap_or(0);
            write_mmio32(controller.cfg_base + offset, original);

            if mask == 0 || mask == 0xffff_ffff {
                let _ = writeln!(
                    uart,
                    "    bar{bar} offset=0x{offset:02x} original=0x{original:08x} mask=0x{mask:08x} unused-or-unavailable"
                );
                bar += 1;
                continue;
            }

            if mask & 0x1 != 0 {
                let size = (!(mask & 0xffff_fffc)).wrapping_add(1);
                let _ = writeln!(
                    uart,
                    "    bar{bar} io  original=0x{original:08x} mask=0x{mask:08x} size=0x{size:08x}"
                );
                bar += 1;
                continue;
            }

            let mem_type = (mask >> 1) & 0x3;
            let size = (!(mask & 0xffff_fff0)).wrapping_add(1);
            let _ = writeln!(
                uart,
                "    bar{bar} mem type={} original=0x{original:08x} mask=0x{mask:08x} size=0x{size:08x}",
                pci_bar_mem_type_name(mem_type)
            );
            if mem_type == 0x2 {
                bar += 2;
            } else {
                bar += 1;
            }
        }
    }
}

fn command_pcie_bar_map(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie bar-map confirm\n");
        uart.put_str("scope: assign BAR2/BAR4 MMIO windows and enable Memory+BusMaster\n");
        uart.put_str("note: run `pcie link-start confirm` first\n");
        return;
    }

    uart.put_str("pcie bar map:\n");
    for controller in PCIE_CONTROLLERS {
        pcie_select_rtl_endpoint(*controller);
        let vendor = probe_read32(controller.cfg_base).unwrap_or(0xffff_ffff);
        let bar2 = controller.mem32_base;
        let bar4 = controller.mem32_base + RTL8125_BAR2_SIZE;

        let _ = writeln!(
            uart,
            "  {} endpoint vendor-device=0x{vendor:08x}",
            controller.name
        );
        if vendor == 0xffff_ffff || vendor == 0 {
            uart.put_str("    endpoint not visible; run link-start first\n");
            continue;
        }

        pcie_map_rtl_bars(*controller);

        let command = probe_read32(controller.cfg_base + 0x04).unwrap_or(0);
        let read_bar2 = probe_read32(controller.cfg_base + 0x18).unwrap_or(0);
        let read_bar3 = probe_read32(controller.cfg_base + 0x1c).unwrap_or(0);
        let read_bar4 = probe_read32(controller.cfg_base + 0x20).unwrap_or(0);
        let read_bar5 = probe_read32(controller.cfg_base + 0x24).unwrap_or(0);
        let _ = writeln!(
            uart,
            "    command=0x{command:08x} bar2=0x{read_bar2:08x}:{read_bar3:08x} bar4=0x{read_bar4:08x}:{read_bar5:08x}"
        );
        let _ = writeln!(
            uart,
            "    cpu-mmio bar2=0x{bar2:016x} size=0x{RTL8125_BAR2_SIZE:05x} bar4=0x{bar4:016x} size=0x{RTL8125_BAR4_SIZE:05x}"
        );
    }
}

fn command_pcie_rtl_probe(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie rtl-probe confirm\n");
        uart.put_str("scope: read mapped RTL8125 BAR2 MMIO words only\n");
        uart.put_str("note: run `pcie link-start confirm` and `pcie bar-map confirm` first\n");
        return;
    }

    uart.put_str("pcie rtl probe:\n");
    for controller in PCIE_CONTROLLERS {
        let base = controller.mem32_base;
        let _ = writeln!(uart, "  {} bar2=0x{base:016x}", controller.name);
        for offset in [0x00u64, 0x04, 0x08, 0x0c, 0x20, 0x3c, 0x40, 0x44, 0x50] {
            print_pcie_reg(uart, "    rtl", base + offset, probe_read32(base + offset));
        }
    }
}

fn command_pcie_rtl_facts(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie rtl-facts confirm\n");
        uart.put_str("scope: decode mapped RTL8125 MAC/chip/link facts only\n");
        uart.put_str("note: run `pcie link-start confirm` and `pcie bar-map confirm` first\n");
        return;
    }

    uart.put_str("pcie rtl facts:\n");
    for controller in PCIE_CONTROLLERS {
        let base = controller.mem32_base;
        let _ = writeln!(uart, "  {} bar2=0x{base:016x}", controller.name);

        let mac_bkp = rtl_read_mac(base, RTL_REG_MAC0_BKP);
        let mac0 = rtl_read_mac(base, RTL_REG_MAC0);
        print_rtl_mac(uart, "    mac-bkp", mac_bkp);
        print_rtl_mac(uart, "    mac0", mac0);

        let tx_config = probe_read32(base + RTL_REG_TX_CONFIG);
        print_pcie_reg(uart, "    tx-config", base + RTL_REG_TX_CONFIG, tx_config);
        if let Ok(tx_config) = tx_config {
            let xid = (tx_config >> 20) & 0xfcf;
            let chip = rtl_chip_info_from_xid(xid);
            match chip {
                Some(chip) => {
                    let _ = writeln!(
                        uart,
                        "    chip          xid=0x{xid:03x} {} {}",
                        chip.name, chip.mac_version
                    );
                }
                None => {
                    let _ = writeln!(uart, "    chip          xid=0x{xid:03x} unknown");
                }
            }
        }

        let rx_config = probe_read32(base + RTL_REG_RX_CONFIG);
        let intr_status = probe_read32(base + RTL_REG_INTR_STATUS_8125);
        let chip_cmd_word = probe_read32(base + RTL_REG_CHIP_CMD_WORD);
        let phy_status = probe_read32(base + RTL_REG_PHY_STATUS);
        let cplus_cmd = probe_read32(base + RTL_REG_CPLUS_CMD);

        print_pcie_reg(uart, "    rx-config", base + RTL_REG_RX_CONFIG, rx_config);
        print_pcie_reg(
            uart,
            "    intr-status",
            base + RTL_REG_INTR_STATUS_8125,
            intr_status,
        );
        print_pcie_reg(
            uart,
            "    chip-cmd",
            base + RTL_REG_CHIP_CMD_WORD,
            chip_cmd_word,
        );
        if let Ok(word) = chip_cmd_word {
            let chip_cmd = (word >> RTL_CHIP_CMD_BYTE_IN_WORD) & 0xff;
            let _ = writeln!(
                uart,
                "    chip-cmd.byte 0x{chip_cmd:02x} tx={} rx={} reset={}",
                enabled_name(chip_cmd & 0x04 != 0),
                enabled_name(chip_cmd & 0x08 != 0),
                enabled_name(chip_cmd & 0x10 != 0)
            );
        }
        print_pcie_reg(
            uart,
            "    phy-status",
            base + RTL_REG_PHY_STATUS,
            phy_status,
        );
        if let Ok(phy_status) = phy_status {
            print_rtl_link_status(uart, phy_status);
        }
        print_pcie_reg(uart, "    cplus-cmd", base + RTL_REG_CPLUS_CMD, cplus_cmd);
    }
}

fn command_pcie_rtl_init(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie rtl-init confirm\n");
        uart.put_str("scope: reset RTL8125 MAC, assign local MACs, initialize TX/RX rings\n");
        uart.put_str("note: run `pcie link-start confirm` first\n");
        return;
    }

    uart.put_str("pcie rtl init:\n");
    for (index, controller) in PCIE_CONTROLLERS.iter().enumerate() {
        let base = controller.mem32_base;
        let mac = rtl_local_mac(index);
        pcie_select_rtl_endpoint(*controller);
        let vendor = probe_read32(controller.cfg_base).unwrap_or(0xffff_ffff);
        let _ = writeln!(
            uart,
            "  {} endpoint vendor-device=0x{vendor:08x}",
            controller.name
        );
        if vendor != 0x8125_10ec {
            uart.put_str("    skipped: expected Realtek RTL8125 endpoint\n");
            continue;
        }

        pcie_map_rtl_bars(*controller);
        rtl_irq_mask_and_ack(base);
        match rtl_mac_reset(base, RTL_INIT_TIMEOUT_MS) {
            Some(chip_cmd) => {
                let _ = writeln!(uart, "    reset         done chip-cmd=0x{chip_cmd:02x}");
            }
            None => {
                uart.put_str("    reset         timeout; continuing with conservative init\n");
            }
        }
        rtl_irq_mask_and_ack(base);
        rtl_write_mac(base, mac);
        rtl_prepare_rings(index);
        rtl_program_ring_registers(base, index);

        write_mmio16(base + RTL_REG_RX_MAX_SIZE, RTL_RX_BUFFER_SIZE as u16);
        write_mmio32(
            base + RTL_REG_RX_CONFIG,
            RTL_RX_FETCH_DFLT_8125
                | RTL_RX_DMA_BURST
                | RTL_RX_PAUSE_SLOT_ON
                | RTL_RX_ACCEPT_BROADCAST
                | RTL_RX_ACCEPT_MULTICAST
                | RTL_RX_ACCEPT_MY_PHYS,
        );
        write_mmio32(
            base + RTL_REG_TX_CONFIG,
            RTL_TX_INTER_FRAME_GAP | RTL_TX_DMA_BURST,
        );
        write_mmio8(
            base + RTL_REG_CHIP_CMD_WORD + 3,
            RTL_CHIP_CMD_TX_ENABLE | RTL_CHIP_CMD_RX_ENABLE,
        );
        mmio_write_barrier();

        let tx_ring = rtl_tx_ring_addr(index);
        let rx_ring = rtl_rx_ring_addr(index);
        let _ = writeln!(
            uart,
            "    mac           {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
        let _ = writeln!(
            uart,
            "    tx-ring       0x{tx_ring:016x} descs={} rx-ring=0x{rx_ring:016x} descs={}",
            RTL_TX_DESC_COUNT, RTL_RX_DESC_COUNT
        );
        print_pcie_reg(
            uart,
            "    chip-cmd",
            base + RTL_REG_CHIP_CMD_WORD,
            probe_read32(base + RTL_REG_CHIP_CMD_WORD),
        );
        print_pcie_reg(
            uart,
            "    tx-config",
            base + RTL_REG_TX_CONFIG,
            probe_read32(base + RTL_REG_TX_CONFIG),
        );
        print_pcie_reg(
            uart,
            "    rx-config",
            base + RTL_REG_RX_CONFIG,
            probe_read32(base + RTL_REG_RX_CONFIG),
        );
        print_pcie_reg(
            uart,
            "    rx-desc-low",
            base + RTL_REG_RX_DESC_ADDR_LOW,
            probe_read32(base + RTL_REG_RX_DESC_ADDR_LOW),
        );
        print_pcie_reg(
            uart,
            "    tx-desc-low",
            base + RTL_REG_TX_DESC_ADDR_LOW,
            probe_read32(base + RTL_REG_TX_DESC_ADDR_LOW),
        );
    }
    uart.put_str("  note            DMA packet I/O is still polling-only and comes next\n");
}

fn command_pcie_rtl_tx_test(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie rtl-tx-test confirm\n");
        uart.put_str("scope: send one raw broadcast Ethernet frame from each RTL8125\n");
        uart.put_str("note: run `pcie link-start confirm` and `pcie rtl-init confirm` first\n");
        return;
    }

    uart.put_str("pcie rtl tx test:\n");
    for (index, controller) in PCIE_CONTROLLERS.iter().enumerate() {
        let base = controller.mem32_base;
        let mac = rtl_local_mac(index);
        let len = rtl_build_tx_test_packet(index, mac);
        let buffer_addr = rtl_tx_buffer_addr(index);
        let desc_addr = rtl_tx_desc_addr(index, 0);

        unsafe {
            let desc = &mut RTL_TX_RINGS.0[index][0];
            desc.addr = buffer_addr;
            desc.opts2 = 0;
            desc.opts1 = len as u32 | RTL_DESC_FIRST_FRAG | RTL_DESC_LAST_FRAG;
            clean_dcache_range(buffer_addr, len as u64);
            clean_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
            desc.opts1 |= RTL_DESC_OWN;
            clean_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        }

        write_mmio16(base + RTL_REG_TX_POLL_8125, 1);
        mmio_write_barrier();
        let completed = rtl_wait_tx_complete(index, 250);
        let intr = probe_read32(base + RTL_REG_INTR_STATUS_8125).unwrap_or(0);
        let status = unsafe { RTL_TX_RINGS.0[index][0].opts1 };
        let _ = writeln!(
            uart,
            "  {} len={} buffer=0x{buffer_addr:016x} desc=0x{desc_addr:016x} complete={} opts1=0x{status:08x} intr=0x{intr:08x}",
            controller.name,
            len,
            yes_no(completed)
        );
    }
}

fn command_pcie_rtl_rx_poll(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie rtl-rx-poll confirm\n");
        uart.put_str("scope: poll RX descriptors once and re-arm consumed descriptors\n");
        uart.put_str("note: run `pcie link-start confirm` and `pcie rtl-init confirm` first\n");
        return;
    }

    uart.put_str("pcie rtl rx poll:\n");
    for (index, controller) in PCIE_CONTROLLERS.iter().enumerate() {
        let mut packets = 0usize;
        let _ = writeln!(uart, "  {}", controller.name);
        for entry in 0..RTL_RX_DESC_COUNT {
            let desc_addr = rtl_rx_desc_addr(index, entry);
            invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
            let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
            if status & RTL_DESC_OWN != 0 {
                continue;
            }

            packets += 1;
            let len = (status & 0x3fff) as usize;
            let buffer_addr = rtl_rx_buffer_addr(index, entry);
            let sample_len = if len > 32 { 32 } else { len };
            invalidate_dcache_range(buffer_addr, len as u64);
            let _ = write!(
                uart,
                "    rx desc={} len={} opts1=0x{status:08x} head=",
                entry, len
            );
            unsafe {
                let buffer = &RTL_RX_BUFFERS.0[index][entry];
                for byte in &buffer[..sample_len] {
                    let _ = write!(uart, "{byte:02x}");
                }
            }
            uart.put_str("\n");
            rtl_rearm_rx_desc(index, entry);
        }
        let _ = writeln!(uart, "    packets={packets}");
    }
}

fn command_pcie_phy_init(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie phy-init confirm\n");
        uart.put_str("scope: write shared PCIe3 PHY GRF lane/bifurcation bits only\n");
        return;
    }

    uart.put_str("pcie phy init:\n");
    write_mmio32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON9, PCIE30_PHY_DA_OCM);
    write_mmio32(
        PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON6,
        PCIE30_PHY_WR_EN | PCIE30_PHY_BIFURCATION_LANE_0_1,
    );
    write_mmio32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON1, PCIE30_PHY_DA_OCM);
    uart.put_str("  wrote           con9 DA_OCM, con6 bifurcation lane0/1, con1 DA_OCM\n");

    match wait_pcie_phy_sram_ready(PCIE_PHY_INIT_TIMEOUT_MS) {
        Some(status0) => {
            let _ = writeln!(uart, "  phy sram       init-done status0=0x{status0:08x}");
        }
        None => {
            uart.put_str("  phy sram       timeout; CRU reset/clock may still be gated\n");
        }
    }
    print_pcie_phy_status(uart);
}

fn command_pcie_cru_enable(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie cru-enable confirm\n");
        uart.put_str("scope: enable PCIe3 clock gates and deassert PCIe3 resets only\n");
        return;
    }

    uart.put_str("pcie cru enable:\n");
    for gate in PCIE_CLOCK_GATES {
        cru_gate_enable(*gate);
    }
    uart.put_str("  clocks          enabled by hiword gate clear\n");

    for reset in PCIE_RESETS {
        cru_reset_deassert(*reset);
    }
    uart.put_str("  resets          deasserted by hiword reset clear\n");
    print_pcie_cru_status(uart);
}

fn command_pcie_rc_init(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie rc-init confirm\n");
        uart.put_str("scope: write controller-local APB RC mode and LTSSM enhance bits only\n");
        return;
    }

    uart.put_str("pcie rc init:\n");
    for controller in PCIE_CONTROLLERS {
        write_mmio32(
            controller.apb_base + PCIE_CLIENT_HOT_RESET_CTRL,
            PCIE_CLIENT_HOT_RESET_ENHANCE,
        );
        write_mmio32(
            controller.apb_base + PCIE_CLIENT_POWER_CON,
            PCIE_CLIENT_CLKREQ_PULL_DOWN_NOT_READY,
        );
        write_mmio32(
            controller.apb_base + PCIE_CLIENT_GENERAL_CON,
            PCIE_CLIENT_SET_RC_MODE,
        );
        let _ = writeln!(
            uart,
            "  {}           wrote hot-reset-enhance, clkreq policy, rc mode",
            controller.name
        );
        print_pcie_controller_apb_status(uart, *controller);
    }
}

fn command_pcie_ltssm(uart: &mut Uart, enable: bool, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        if enable {
            uart.put_str("usage: pcie ltssm-enable confirm\n");
        } else {
            uart.put_str("usage: pcie ltssm-disable confirm\n");
        }
        uart.put_str("scope: write controller-local APB LTSSM enable bit only\n");
        return;
    }

    if enable {
        uart.put_str("pcie ltssm enable:\n");
    } else {
        uart.put_str("pcie ltssm disable:\n");
    }

    for controller in PCIE_CONTROLLERS {
        set_pcie_ltssm(*controller, enable);
        if enable {
            match wait_pcie_link(*controller, PCIE_LINK_TRAIN_TIMEOUT_MS) {
                Some(ltssm) => {
                    let state = ltssm & PCIE_CLIENT_LTSSM_STATUS_MASK;
                    let _ = writeln!(
                        uart,
                        "  {}           link up ltssm=0x{ltssm:08x} state=0x{state:02x} ({})",
                        controller.name,
                        pcie_ltssm_state_name(state)
                    );
                }
                None => {
                    let _ = writeln!(
                        uart,
                        "  {}           link timeout after {PCIE_LINK_TRAIN_TIMEOUT_MS}ms",
                        controller.name
                    );
                }
            }
        } else {
            let _ = writeln!(uart, "  {}           ltssm disabled", controller.name);
        }
        print_pcie_controller_apb_status(uart, *controller);
    }
}

fn command_pcie_link_start(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: pcie link-start confirm\n");
        uart.put_str(
            "scope: enable clocks, init PHY, set RC mode, enable LTSSM, release RJ45 reset\n",
        );
        return;
    }

    uart.put_str("pcie link start:\n");
    set_pcie_reset_asserted(true);
    uart.put_str("  reset           RJ45 PCIe devices held\n");

    for gate in PCIE_CLOCK_GATES {
        cru_gate_enable(*gate);
    }
    for reset in PCIE_RESETS {
        cru_reset_deassert(*reset);
    }
    uart.put_str("  cru             clocks enabled, resets deasserted\n");

    write_mmio32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON9, PCIE30_PHY_DA_OCM);
    write_mmio32(
        PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON6,
        PCIE30_PHY_WR_EN | PCIE30_PHY_BIFURCATION_LANE_0_1,
    );
    write_mmio32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON1, PCIE30_PHY_DA_OCM);
    match wait_pcie_phy_sram_ready(PCIE_PHY_INIT_TIMEOUT_MS) {
        Some(status0) => {
            let _ = writeln!(uart, "  phy             init-done status0=0x{status0:08x}");
        }
        None => {
            uart.put_str("  phy             timeout; aborting before LTSSM\n");
            print_pcie_phy_status(uart);
            return;
        }
    }

    for controller in PCIE_CONTROLLERS {
        write_mmio32(
            controller.apb_base + PCIE_CLIENT_HOT_RESET_CTRL,
            PCIE_CLIENT_HOT_RESET_ENHANCE,
        );
        write_mmio32(
            controller.apb_base + PCIE_CLIENT_POWER_CON,
            PCIE_CLIENT_CLKREQ_PULL_DOWN_NOT_READY,
        );
        write_mmio32(
            controller.apb_base + PCIE_CLIENT_GENERAL_CON,
            PCIE_CLIENT_SET_RC_MODE,
        );
        set_pcie_ltssm(*controller, true);
    }
    uart.put_str("  rc/ltssm        RC mode set, LTSSM enabled\n");

    delay_ms(100);
    set_pcie_reset_asserted(false);
    uart.put_str("  reset           RJ45 PCIe devices released\n");

    for controller in PCIE_CONTROLLERS {
        match wait_pcie_link(*controller, PCIE_LINK_TRAIN_TIMEOUT_MS) {
            Some(ltssm) => {
                let state = ltssm & PCIE_CLIENT_LTSSM_STATUS_MASK;
                let _ = writeln!(
                    uart,
                    "  {}           link up ltssm=0x{ltssm:08x} state=0x{state:02x} ({})",
                    controller.name,
                    pcie_ltssm_state_name(state)
                );
            }
            None => {
                let _ = writeln!(
                    uart,
                    "  {}           link timeout after {PCIE_LINK_TRAIN_TIMEOUT_MS}ms",
                    controller.name
                );
            }
        }
    }
    print_pcie_reset_pins(uart);
    command_pcie_apb_status(uart, Some("confirm"));
}

fn command_pcie_reset_write(uart: &mut Uart, assert_reset: bool, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage:\n");
        uart.put_str("  pcie reset-deassert confirm\n");
        uart.put_str("  pcie reset-assert confirm\n");
        uart.put_str("scope: pcie3x1 and pcie3x2 RJ45 reset GPIOs only\n");
        return;
    }

    set_pcie_reset_asserted(assert_reset);

    if assert_reset {
        uart.put_str("pcie reset assert: RJ45 PCIe reset GPIOs driven to hold level\n");
    } else {
        uart.put_str("pcie reset deassert: RJ45 PCIe reset GPIOs driven to release level\n");
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

fn print_pcie_phy_status(uart: &mut Uart) {
    uart.put_str("pcie phy status:\n");
    let con1 = probe_read32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON1);
    let con6 = probe_read32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON6);
    let con9 = probe_read32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON9);
    let status0 = probe_read32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_STATUS0);

    print_pcie_reg(
        uart,
        "  phy-grf.con1",
        PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON1,
        con1,
    );
    print_pcie_reg(
        uart,
        "  phy-grf.con6",
        PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON6,
        con6,
    );
    print_pcie_reg(
        uart,
        "  phy-grf.con9",
        PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_CON9,
        con9,
    );
    print_pcie_reg(
        uart,
        "  phy-grf.status0",
        PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_STATUS0,
        status0,
    );

    if let Ok(status0) = status0 {
        let sram_ready = status0 & PCIE30_PHY_SRAM_INIT_DONE != 0;
        let _ = writeln!(
            uart,
            "  phy sram       {}",
            if sram_ready { "init-done" } else { "not-ready" }
        );
    }
}

fn print_pcie_cru_status(uart: &mut Uart) {
    uart.put_str("pcie cru status:\n");
    uart.put_str("  clocks          gate bit set means disabled\n");
    for gate in PCIE_CLOCK_GATES {
        print_cru_bit(uart, *gate, false);
    }
    uart.put_str("  resets          reset bit set means asserted\n");
    for reset in PCIE_RESETS {
        print_cru_bit(uart, *reset, true);
    }
}

fn print_pcie_controller_apb_status(uart: &mut Uart, controller: PcieController) {
    let general = probe_read32(controller.apb_base + PCIE_CLIENT_GENERAL_CON);
    let misc = probe_read32(controller.apb_base + PCIE_CLIENT_INTR_STATUS_MISC);
    let power = probe_read32(controller.apb_base + PCIE_CLIENT_POWER_CON);
    let hot_reset = probe_read32(controller.apb_base + PCIE_CLIENT_HOT_RESET_CTRL);
    let ltssm = probe_read32(controller.apb_base + PCIE_CLIENT_LTSSM_STATUS);

    let _ = writeln!(
        uart,
        "  {} apb=0x{:016x} reset={}",
        controller.name, controller.apb_base, controller.reset_pin
    );
    print_pcie_reg(
        uart,
        "    general",
        controller.apb_base + PCIE_CLIENT_GENERAL_CON,
        general,
    );
    if let Ok(general) = general {
        let _ = writeln!(
            uart,
            "    mode         rc={} ltssm-enable={}",
            enabled_name(general & PCIE_CLIENT_RC_MODE_BIT != 0),
            enabled_name(general & PCIE_CLIENT_LTSSM_ENABLE_BIT != 0)
        );
    }
    print_pcie_reg(
        uart,
        "    misc",
        controller.apb_base + PCIE_CLIENT_INTR_STATUS_MISC,
        misc,
    );
    print_pcie_reg(
        uart,
        "    power",
        controller.apb_base + PCIE_CLIENT_POWER_CON,
        power,
    );
    print_pcie_reg(
        uart,
        "    hot-reset",
        controller.apb_base + PCIE_CLIENT_HOT_RESET_CTRL,
        hot_reset,
    );
    print_pcie_reg(
        uart,
        "    ltssm",
        controller.apb_base + PCIE_CLIENT_LTSSM_STATUS,
        ltssm,
    );
    if let Ok(ltssm) = ltssm {
        let link_up = ltssm & PCIE_CLIENT_LINKUP_MASK == PCIE_CLIENT_LINKUP;
        let state = ltssm & PCIE_CLIENT_LTSSM_STATUS_MASK;
        let _ = writeln!(
            uart,
            "    link         {} ltssm-state=0x{state:02x} ({})",
            if link_up { "up" } else { "down" },
            pcie_ltssm_state_name(state)
        );
    }
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
    uart.put_str("  note            RJ45 release-high behavior was validated by PCIe link-up\n");
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
    write_mmio32(reg, mask | data);
}

fn set_pcie_reset_asserted(asserted: bool) {
    for pin in &PCIE_RESET_PINS[..PCIE_ETH_RESET_COUNT] {
        let level = if pin.active_high { asserted } else { !asserted };
        gpio_v2_write_bit(pin.bank, GPIO_V2_PORT_DR, pin.pin, level);
        gpio_v2_write_bit(pin.bank, GPIO_V2_PORT_DDR, pin.pin, true);
    }
}

fn write_mmio32(addr: u64, value: u32) {
    unsafe {
        write_volatile(addr as usize as *mut u32, value);
    }
}

fn write_mmio16(addr: u64, value: u16) {
    unsafe {
        write_volatile(addr as usize as *mut u16, value);
    }
}

fn write_mmio8(addr: u64, value: u8) {
    unsafe {
        write_volatile(addr as usize as *mut u8, value);
    }
}

fn mmio_write_barrier() {
    unsafe {
        asm!("dsb sy");
    }
}

fn cru_gate_enable(bit: CruBit) {
    write_mmio32(bit.base + bit.offset, hiword_update(bit.bit, false));
}

fn cru_reset_deassert(bit: CruBit) {
    write_mmio32(bit.base + bit.offset, hiword_update(bit.bit, false));
}

fn hiword_update(bit: u8, value: bool) -> u32 {
    let mask = 1u32 << (bit + 16);
    let data = if value { 1u32 << bit } else { 0 };
    mask | data
}

fn print_cru_bit(uart: &mut Uart, bit: CruBit, reset_semantics: bool) {
    let addr = bit.base + bit.offset;
    match probe_read32(addr) {
        Ok(value) => {
            let set = value & (1u32 << bit.bit) != 0;
            let state = if reset_semantics {
                if set {
                    "asserted"
                } else {
                    "deasserted"
                }
            } else if set {
                "disabled"
            } else {
                "enabled"
            };
            let _ = writeln!(
                uart,
                "  {:<24} @ 0x{addr:016x} bit={:<2} {} raw=0x{value:08x}",
                bit.name, bit.bit, state
            );
        }
        Err(snapshot) => {
            let _ = writeln!(
                uart,
                "  {:<24} @ 0x{addr:016x} bit={:<2} fault ec=0x{:02x} far=0x{:016x}",
                bit.name,
                bit.bit,
                snapshot.esr >> ESR_EC_SHIFT,
                snapshot.far
            );
        }
    }
}

fn wait_pcie_phy_sram_ready(timeout_ms: u64) -> Option<u32> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Ok(status0) = probe_read32(PCIE30_PHY_GRF_BASE + PCIE30_PHY_GRF_STATUS0) {
            if status0 & PCIE30_PHY_SRAM_INIT_DONE != 0 {
                return Some(status0);
            }
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn delay_ms(ms: u64) {
    let freq = read_cntfrq();
    let Some(ticks) = ms_to_ticks(ms, freq) else {
        return;
    };
    let start = read_cntpct();
    while read_cntpct().wrapping_sub(start) < ticks {
        core::hint::spin_loop();
    }
}

fn set_pcie_ltssm(controller: PcieController, enable: bool) {
    let value = if enable {
        PCIE_CLIENT_ENABLE_LTSSM_VALUE
    } else {
        PCIE_CLIENT_DISABLE_LTSSM_VALUE
    };
    write_mmio32(controller.apb_base + PCIE_CLIENT_GENERAL_CON, value);
}

fn wait_pcie_link(controller: PcieController, timeout_ms: u64) -> Option<u32> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Ok(ltssm) = probe_read32(controller.apb_base + PCIE_CLIENT_LTSSM_STATUS) {
            if ltssm & PCIE_CLIENT_LINKUP_MASK == PCIE_CLIENT_LINKUP {
                return Some(ltssm);
            }
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn pcie_program_cfg_atu(controller: PcieController, bus: u8, dev: u8, func: u8, cfg_type: u32) {
    let base = controller.cfg_base;
    let limit = base + PCIE_CFG_WINDOW_SIZE - 1;
    let target = ((bus as u32) << 24) | ((dev as u32) << 19) | ((func as u32) << 16);

    pcie_atu_write(controller, 0, PCIE_ATU_REGION_CTRL2, 0);
    pcie_atu_write(controller, 0, PCIE_ATU_LOWER_BASE, base as u32);
    pcie_atu_write(controller, 0, PCIE_ATU_UPPER_BASE, (base >> 32) as u32);
    pcie_atu_write(controller, 0, PCIE_ATU_LIMIT, limit as u32);
    pcie_atu_write(controller, 0, PCIE_ATU_LOWER_TARGET, target);
    pcie_atu_write(controller, 0, PCIE_ATU_UPPER_TARGET, 0);
    pcie_atu_write(controller, 0, PCIE_ATU_REGION_CTRL1, cfg_type);
    pcie_atu_write(controller, 0, PCIE_ATU_REGION_CTRL2, PCIE_ATU_ENABLE);
}

fn pcie_select_rtl_endpoint(controller: PcieController) {
    pcie_program_cfg_atu(
        controller,
        controller.root_bus.wrapping_add(1),
        0,
        0,
        PCIE_ATU_TYPE_CFG0,
    );
}

fn pcie_map_rtl_bars(controller: PcieController) {
    let bar2 = controller.mem32_base;
    let bar4 = controller.mem32_base + RTL8125_BAR2_SIZE;

    pcie_select_rtl_endpoint(controller);
    pcie_program_mem_atu(
        controller,
        1,
        controller.mem32_base,
        controller.mem32_base,
        RTL8125_MEM_WINDOW_SIZE,
    );
    write_mmio32(controller.cfg_base + 0x18, bar2 as u32);
    write_mmio32(controller.cfg_base + 0x1c, (bar2 >> 32) as u32);
    write_mmio32(controller.cfg_base + 0x20, bar4 as u32);
    write_mmio32(controller.cfg_base + 0x24, (bar4 >> 32) as u32);
    write_mmio32(
        controller.cfg_base + 0x04,
        PCI_COMMAND_MEMORY | PCI_COMMAND_MASTER,
    );
    mmio_write_barrier();
}

fn pcie_program_mem_atu(
    controller: PcieController,
    index: u32,
    cpu_base: u64,
    pci_base: u64,
    size: u64,
) {
    let limit = cpu_base + size - 1;

    pcie_atu_write(controller, index, PCIE_ATU_REGION_CTRL2, 0);
    pcie_atu_write(controller, index, PCIE_ATU_LOWER_BASE, cpu_base as u32);
    pcie_atu_write(
        controller,
        index,
        PCIE_ATU_UPPER_BASE,
        (cpu_base >> 32) as u32,
    );
    pcie_atu_write(controller, index, PCIE_ATU_LIMIT, limit as u32);
    pcie_atu_write(controller, index, PCIE_ATU_LOWER_TARGET, pci_base as u32);
    pcie_atu_write(
        controller,
        index,
        PCIE_ATU_UPPER_TARGET,
        (pci_base >> 32) as u32,
    );
    pcie_atu_write(controller, index, PCIE_ATU_REGION_CTRL1, PCIE_ATU_TYPE_MEM);
    pcie_atu_write(controller, index, PCIE_ATU_REGION_CTRL2, PCIE_ATU_ENABLE);
}

fn pcie_atu_write(controller: PcieController, index: u32, reg: u64, value: u32) {
    write_mmio32(pcie_atu_addr(controller, index, reg), value);
}

fn pcie_atu_addr(controller: PcieController, index: u32, reg: u64) -> u64 {
    if iatu_is_unrolled(controller) {
        controller.dbi_base
            + PCIE_ATU_UNROLL_BASE
            + (index as u64) * PCIE_ATU_UNROLL_REGION_SIZE
            + reg
    } else {
        write_mmio32(controller.dbi_base + PCIE_ATU_VIEWPORT, index & 0xff);
        controller.dbi_base + PCIE_ATU_VIEWPORT_BASE + reg
    }
}

fn iatu_is_unrolled(controller: PcieController) -> bool {
    matches!(
        probe_read32(controller.dbi_base + PCIE_ATU_VIEWPORT),
        Ok(PCIE_ATU_PROBE_UNROLLED)
    )
}

fn cfg_type_name(cfg_type: u32) -> &'static str {
    match cfg_type {
        PCIE_ATU_TYPE_CFG0 => "cfg0",
        PCIE_ATU_TYPE_CFG1 => "cfg1",
        _ => "unknown",
    }
}

fn pci_bar_mem_type_name(value: u32) -> &'static str {
    match value {
        0x0 => "32",
        0x1 => "below-1m",
        0x2 => "64",
        _ => "reserved",
    }
}

fn rtl_chip_info_from_xid(xid: u32) -> Option<&'static RtlChipInfo> {
    RTL_CHIP_INFOS_8125
        .iter()
        .find(|chip| xid & chip.mask == chip.value)
}

fn rtl_read_mac(base: u64, offset: u64) -> Result<[u8; 6], ExceptionSnapshot> {
    let low = probe_read32(base + offset)?;
    let high = probe_read32(base + offset + 4)?;

    Ok([
        (low & 0xff) as u8,
        ((low >> 8) & 0xff) as u8,
        ((low >> 16) & 0xff) as u8,
        ((low >> 24) & 0xff) as u8,
        (high & 0xff) as u8,
        ((high >> 8) & 0xff) as u8,
    ])
}

fn rtl_local_mac(index: usize) -> [u8; 6] {
    [0x02, 0x58, 0x4a, 0x52, 0x35, 0xc0 + index as u8]
}

fn rtl_write_mac(base: u64, mac: [u8; 6]) {
    let low = (mac[0] as u32)
        | ((mac[1] as u32) << 8)
        | ((mac[2] as u32) << 16)
        | ((mac[3] as u32) << 24);
    let high = (mac[4] as u16) | ((mac[5] as u16) << 8);

    write_mmio8(base + RTL_REG_CFG9346, RTL_CFG9346_UNLOCK);
    write_mmio32(base + RTL_REG_MAC0, low);
    write_mmio16(base + RTL_REG_MAC0 + 4, high);
    write_mmio32(base + RTL_REG_MAC0_BKP, low);
    write_mmio16(base + RTL_REG_MAC0_BKP + 4, high);
    write_mmio8(base + RTL_REG_CFG9346, RTL_CFG9346_LOCK);
    mmio_write_barrier();
}

fn rtl_irq_mask_and_ack(base: u64) {
    write_mmio32(base + 0x38, 0);
    write_mmio32(base + RTL_REG_INTR_STATUS_8125, 0xffff_ffff);
    mmio_write_barrier();
}

fn rtl_read_chip_cmd(base: u64) -> Option<u8> {
    probe_read32(base + RTL_REG_CHIP_CMD_WORD)
        .ok()
        .map(|word| ((word >> RTL_CHIP_CMD_BYTE_IN_WORD) & 0xff) as u8)
}

fn rtl_mac_reset(base: u64, timeout_ms: u64) -> Option<u8> {
    write_mmio8(base + RTL_REG_CHIP_CMD_WORD + 3, RTL_CHIP_CMD_RESET);
    mmio_write_barrier();

    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Some(chip_cmd) = rtl_read_chip_cmd(base) {
            if chip_cmd & RTL_CHIP_CMD_RESET == 0 {
                return Some(chip_cmd);
            }
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn rtl_prepare_rings(index: usize) {
    unsafe {
        let tx_ring = &mut RTL_TX_RINGS.0[index];
        for desc in tx_ring.iter_mut() {
            *desc = RtlDesc::zero();
        }
        tx_ring[RTL_TX_DESC_COUNT - 1].opts1 = RTL_DESC_RING_END;

        let rx_ring = &mut RTL_RX_RINGS.0[index];
        for i in 0..RTL_RX_DESC_COUNT {
            let buffer = &mut RTL_RX_BUFFERS.0[index][i];
            for byte in buffer.iter_mut() {
                *byte = 0;
            }
            let mut opts1 = RTL_DESC_OWN | RTL_RX_BUFFER_SIZE as u32;
            if i == RTL_RX_DESC_COUNT - 1 {
                opts1 |= RTL_DESC_RING_END;
            }
            rx_ring[i] = RtlDesc {
                opts1,
                opts2: 0,
                addr: buffer.as_mut_ptr() as u64,
            };
            clean_dcache_range(buffer.as_ptr() as u64, RTL_RX_BUFFER_SIZE as u64);
        }

        clean_dcache_range(
            tx_ring.as_ptr() as u64,
            (RTL_TX_DESC_COUNT * core::mem::size_of::<RtlDesc>()) as u64,
        );
        clean_dcache_range(
            rx_ring.as_ptr() as u64,
            (RTL_RX_DESC_COUNT * core::mem::size_of::<RtlDesc>()) as u64,
        );
    }
}

fn rtl_program_ring_registers(base: u64, index: usize) {
    let tx = rtl_tx_ring_addr(index);
    let rx = rtl_rx_ring_addr(index);

    write_mmio32(base + RTL_REG_TX_DESC_ADDR_HIGH, (tx >> 32) as u32);
    write_mmio32(base + RTL_REG_TX_DESC_ADDR_LOW, tx as u32);
    write_mmio32(base + RTL_REG_RX_DESC_ADDR_HIGH, (rx >> 32) as u32);
    write_mmio32(base + RTL_REG_RX_DESC_ADDR_LOW, rx as u32);
    mmio_write_barrier();
}

fn rtl_build_tx_test_packet(index: usize, mac: [u8; 6]) -> usize {
    let payload = b"xiaojin-os rtl tx test";
    unsafe {
        let buffer = &mut RTL_TX_BUFFERS.0[index];
        for byte in buffer.iter_mut() {
            *byte = 0;
        }
        for byte in &mut buffer[0..6] {
            *byte = 0xff;
        }
        buffer[6..12].copy_from_slice(&mac);
        buffer[12] = 0x88;
        buffer[13] = 0xb5;
        buffer[14..14 + payload.len()].copy_from_slice(payload);
        buffer[14 + payload.len()] = b' ';
        buffer[15 + payload.len()] = b'0' + index as u8;
    }

    60
}

fn rtl_wait_tx_complete(index: usize, timeout_ms: u64) -> bool {
    let freq = read_cntfrq();
    let Some(timeout_ticks) = ms_to_ticks(timeout_ms, freq) else {
        return false;
    };
    let desc_addr = rtl_tx_desc_addr(index, 0);
    let start = read_cntpct();
    loop {
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let opts1 = unsafe { RTL_TX_RINGS.0[index][0].opts1 };
        if opts1 & RTL_DESC_OWN == 0 {
            return true;
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return false;
        }
        core::hint::spin_loop();
    }
}

fn rtl_rearm_rx_desc(index: usize, entry: usize) {
    unsafe {
        let buffer = &mut RTL_RX_BUFFERS.0[index][entry];
        let desc = &mut RTL_RX_RINGS.0[index][entry];
        let mut opts1 = RTL_DESC_OWN | RTL_RX_BUFFER_SIZE as u32;
        if entry == RTL_RX_DESC_COUNT - 1 {
            opts1 |= RTL_DESC_RING_END;
        }
        desc.opts2 = 0;
        desc.addr = buffer.as_mut_ptr() as u64;
        desc.opts1 = opts1;
        clean_dcache_range(buffer.as_ptr() as u64, RTL_RX_BUFFER_SIZE as u64);
        clean_dcache_range(
            desc as *const RtlDesc as u64,
            core::mem::size_of::<RtlDesc>() as u64,
        );
    }
}

fn rtl_tx_ring_addr(index: usize) -> u64 {
    unsafe { RTL_TX_RINGS.0[index].as_ptr() as u64 }
}

fn rtl_rx_ring_addr(index: usize) -> u64 {
    unsafe { RTL_RX_RINGS.0[index].as_ptr() as u64 }
}

fn rtl_tx_desc_addr(index: usize, entry: usize) -> u64 {
    unsafe { &RTL_TX_RINGS.0[index][entry] as *const RtlDesc as u64 }
}

fn rtl_rx_desc_addr(index: usize, entry: usize) -> u64 {
    unsafe { &RTL_RX_RINGS.0[index][entry] as *const RtlDesc as u64 }
}

fn rtl_tx_buffer_addr(index: usize) -> u64 {
    unsafe { RTL_TX_BUFFERS.0[index].as_ptr() as u64 }
}

fn rtl_rx_buffer_addr(index: usize, entry: usize) -> u64 {
    unsafe { RTL_RX_BUFFERS.0[index][entry].as_ptr() as u64 }
}

fn print_rtl_mac(uart: &mut Uart, label: &str, mac: Result<[u8; 6], ExceptionSnapshot>) {
    match mac {
        Ok(mac) => {
            let valid = is_likely_valid_mac(mac);
            let _ = writeln!(
                uart,
                "{label:<18} {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} valid={}",
                mac[0],
                mac[1],
                mac[2],
                mac[3],
                mac[4],
                mac[5],
                yes_no(valid)
            );
        }
        Err(snapshot) => {
            let _ = writeln!(
                uart,
                "{label:<18} fault ec=0x{:02x} far=0x{:016x}",
                snapshot.esr >> ESR_EC_SHIFT,
                snapshot.far
            );
        }
    }
}

fn is_likely_valid_mac(mac: [u8; 6]) -> bool {
    let all_zero = mac.iter().all(|byte| *byte == 0);
    let all_ff = mac.iter().all(|byte| *byte == 0xff);
    !all_zero && !all_ff && mac[0] & 0x01 == 0
}

fn print_rtl_link_status(uart: &mut Uart, phy_status: u32) {
    let speed = if phy_status & RTL_PHY_1000_FULL != 0 {
        "1000"
    } else if phy_status & RTL_PHY_100M != 0 {
        "100"
    } else if phy_status & RTL_PHY_10M != 0 {
        "10"
    } else {
        "unknown-or-2.5g"
    };
    let _ = writeln!(
        uart,
        "    link          up={} full-duplex={} speed={} raw=0x{:02x}",
        yes_no(phy_status & RTL_PHY_LINK_STATUS != 0),
        yes_no(phy_status & RTL_PHY_FULL_DUPLEX != 0),
        speed,
        phy_status & 0xff
    );
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

fn print_pcie_reg(uart: &mut Uart, label: &str, addr: u64, result: Result<u32, ExceptionSnapshot>) {
    match result {
        Ok(value) => {
            let _ = writeln!(uart, "{label:<18} @ 0x{addr:016x}: 0x{value:08x}");
        }
        Err(snapshot) => {
            let _ = writeln!(
                uart,
                "{label:<18} @ 0x{addr:016x}: fault ec=0x{:02x} far=0x{:016x}",
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

fn enabled_name(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn pcie_ltssm_state_name(state: u32) -> &'static str {
    match state {
        PCIE_CLIENT_LTSSM_L0 => "L0-or-entry",
        0x00 => "detect/disabled",
        0x01..=0x04 => "detect/polling",
        0x05..=0x10 => "configuration",
        _ => "other",
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

fn clean_dcache_range(addr: u64, len: u64) {
    if len == 0 {
        return;
    }

    let ctr = read_ctr_el0();
    let dcache_line = cache_line_size(ctr, 16);
    if dcache_line == 0 {
        return;
    }

    let end = addr.saturating_add(len);
    let mut current = align_down(addr, dcache_line);
    while current < end {
        unsafe {
            asm!("dc cvac, {0}", in(reg) current);
        }
        current = current.saturating_add(dcache_line);
    }
    unsafe {
        asm!("dsb sy");
    }
}

fn invalidate_dcache_range(addr: u64, len: u64) {
    if len == 0 {
        return;
    }

    let ctr = read_ctr_el0();
    let dcache_line = cache_line_size(ctr, 16);
    if dcache_line == 0 {
        return;
    }

    let end = addr.saturating_add(len);
    let mut current = align_down(addr, dcache_line);
    while current < end {
        unsafe {
            asm!("dc ivac, {0}", in(reg) current);
        }
        current = current.saturating_add(dcache_line);
    }
    unsafe {
        asm!("dsb sy");
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
