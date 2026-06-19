#![no_std]
#![no_main]

use core::arch::{asm, global_asm};
use core::fmt::{self, Write};
use core::panic::PanicInfo;
use core::ptr::{addr_of, addr_of_mut, read_volatile, write_volatile};
use core::str;

global_asm!(include_str!("boot.S"));

const UART2_BASE: usize = 0xfe66_0000;
const UART_RBR_THR: usize = 0x00;
const UART_LSR: usize = 0x14;
const UART_LSR_DR: u32 = 1 << 0;
const UART_LSR_THRE: u32 = 1 << 5;
const PIPE_GRF_BASE: u64 = 0xfdc5_0000;
const PIPE_PHY_GRF_BASE: u64 = 0xfdc9_0000;
const PCIE30_PHY_GRF_BASE: u64 = 0xfdcb_8000;
const PCIE2X1_APB_BASE: u64 = 0xfe26_0000;
const PCIE3X1_APB_BASE: u64 = 0xfe27_0000;
const PCIE3X2_APB_BASE: u64 = 0xfe28_0000;
const PCIE2X1_DBI_BASE: u64 = 0x0000_0003_c000_0000;
const PCIE2X1_CFG_BASE: u64 = 0xf400_0000;
const PCIE2X1_IO_BASE: u64 = 0xf410_0000;
const PCIE2X1_MEM32_BASE: u64 = 0xf420_0000;
const PCIE2X1_MEM64_BASE: u64 = 0x0000_0003_0000_0000;
const COMBPHY2_BASE: u64 = 0xfe84_0000;
const PCIE30_PHY_BASE: u64 = 0xfe8c_0000;
const PMUCRU_BASE: u64 = 0xfdd0_0000;
const CRU_BASE: u64 = 0xfdd2_0000;
const RK3568_CLKGATE_BASE: u64 = 0x300;
const RK3568_SOFTRST_BASE: u64 = 0x400;
const RK3568_PMU_CLKSEL_BASE: u64 = 0x100;
const RK3568_PMU_CLKGATE_BASE: u64 = 0x180;
const RK3568_PMU_CLKSEL_CON9: u64 = RK3568_PMU_CLKSEL_BASE + 9 * 4;
const PCIE30_PHY_GRF_CON1: u64 = 0x04;
const PCIE30_PHY_GRF_CON6: u64 = 0x18;
const PCIE30_PHY_GRF_CON9: u64 = 0x24;
const PCIE30_PHY_GRF_STATUS0: u64 = 0x80;
const PCIE30_PHY_SRAM_INIT_DONE: u32 = 1 << 14;
const PCIE30_PHY_DA_OCM: u32 = (1 << 31) | (1 << 15);
const PCIE30_PHY_WR_EN: u32 = 0x000f_0000;
const PCIE30_PHY_BIFURCATION_LANE_0_1: u32 = 1 << 0;
const RK3568_PIPE_PHY_GRF_STATUS: u64 = 0x34;
const RK3568_PIPE_PHY_READY_BIT: u32 = 1 << 6;
const RK3568_PIPE_PHY_READY_VALUE: u32 = 0;
const RK3568_COMBPHY_REG6: u64 = 0x14;
const RK3568_COMBPHY_REG7: u64 = 0x18;
const RK3568_COMBPHY_REG8: u64 = 0x1c;
const RK3568_COMBPHY_REG11: u64 = 0x28;
const RK3568_COMBPHY_REG12: u64 = 0x2c;
const RK3568_COMBPHY_REG13: u64 = 0x30;
const RK3568_COMBPHY_REG14: u64 = 0x34;
const RK3568_COMBPHY_REG15: u64 = 0x38;
const RK3568_COMBPHY_REG16: u64 = 0x3c;
const RK3568_COMBPHY_REG18: u64 = 0x44;
const RK3568_COMBPHY_REG30: u64 = 0x74;
const RK3568_COMBPHY_REG32: u64 = 0x7c;
const RK3568_COMBPHY_REG33: u64 = 0x80;
const PCIE_CLIENT_GENERAL_CON: u64 = 0x000;
const PCIE_CLIENT_INTR_STATUS_MISC: u64 = 0x010;
const PCIE_CLIENT_INTR_MASK_MISC: u64 = 0x024;
const PCIE_CLIENT_POWER_CON: u64 = 0x02c;
const PCIE_CLIENT_CDM_RASDES_TBA_INFO_CMN: u64 = 0x154;
const PCIE_CLIENT_HOT_RESET_CTRL: u64 = 0x180;
const PCIE_CLIENT_LTSSM_STATUS: u64 = 0x300;
const PCIE_CLIENT_DBG_FIFO_MODE_CON: u64 = 0x310;
const PCIE_CLIENT_DBG_FIFO_STATUS: u64 = 0x350;
const PCIE_CLIENT_LTSSM_ENABLE_BIT: u32 = 1 << 2;
const PCIE_CLIENT_RC_MODE_BIT: u32 = 1 << 6;
const PCIE_CLIENT_SET_RC_MODE: u32 = 0x00f0_0040;
const PCIE_CLIENT_ENABLE_LTSSM_VALUE: u32 = 0x0004_0004;
const PCIE_CLIENT_DISABLE_LTSSM_VALUE: u32 = 0x0004_0000;
const PCIE_CLIENT_HOT_RESET_ENHANCE: u32 = 0x0010_0010;
const PCIE_CLIENT_CLKREQ_PULL_DOWN_NOT_READY: u32 = 0x3001_1000;
const PCIE_CLIENT_DBG_EN: u32 = 0xffff_0007;
const PCIE_CLIENT_DBG_DIS: u32 = 0xffff_0000;
const PCIE_CLIENT_LINKUP_MASK: u32 = 0x3 << 16;
const PCIE_CLIENT_LINKUP: u32 = 0x3 << 16;
const PCIE_CLIENT_LTSSM_STATUS_MASK: u32 = 0x3f;
const PCIE_DBG_FIFO_RATE_SHIFT: u32 = 20;
const PCIE_DBG_FIFO_RATE_MASK: u32 = 0x7 << PCIE_DBG_FIFO_RATE_SHIFT;
const PCIE_DBG_FIFO_L1SUB_SHIFT: u32 = 8;
const PCIE_DBG_FIFO_L1SUB_MASK: u32 = 0x7 << PCIE_DBG_FIFO_L1SUB_SHIFT;
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
const RTL_RX_ACCEPT_ALL_PHYS: u32 = 0x01;
const RTL_TX_DMA_BURST: u32 = 7 << 8;
const RTL_TX_INTER_FRAME_GAP: u32 = 3 << 24;
const RTL_RX_BUFFER_SIZE: usize = 2048;
const RTL_TX_BUFFER_SIZE: usize = 1536;
const RTL_TX_DESC_COUNT: usize = 8;
const RTL_RX_DESC_COUNT: usize = 8;
const RTL_DEVICE_COUNT: usize = 2;
const RTL_INIT_TIMEOUT_MS: u64 = 100;
const NET_LOOP_DURATION_MS: u64 = 120_000;
const NET_SNIFF_DURATION_MS: u64 = 10_000;
const NET_SNIFF_MAX_PRINT: u32 = 16;
const NET_RX_WATCHDOG_MS: u64 = 10_000;
const NET_UDP_PORT: u16 = 5510;
const NET_HOST_PORT: u16 = 5511;
const NET_AUTO_DHCP: bool = true;
const NET_AUTO_DHCP_TIMEOUT_MS: u64 = 2_500;
const NET_AUTO_DHCP_RETRY_MS: u64 = 15_000;
const DEFAULT_NET_DEVICE_IPS: [[u8; 4]; RTL_DEVICE_COUNT] =
    [[192, 168, 80, 251], [192, 168, 80, 250]];
const DEFAULT_NET_NETMASKS: [[u8; 4]; RTL_DEVICE_COUNT] = [[255, 255, 255, 0], [255, 255, 255, 0]];
const DEFAULT_NET_GATEWAYS: [[u8; 4]; RTL_DEVICE_COUNT] = [[0, 0, 0, 0], [0, 0, 0, 0]];
const DEFAULT_NET_DNS: [[u8; 4]; RTL_DEVICE_COUNT] = [[0, 0, 0, 0], [0, 0, 0, 0]];
const DEFAULT_NET_BROADCAST_IP: [u8; 4] = [192, 168, 80, 255];
const IPV4_ANY_IP: [u8; 4] = [0, 0, 0, 0];
const IPV4_BROADCAST_IP: [u8; 4] = [255, 255, 255, 255];
const XIAOJIN_IDENTITY_FORMAT: &str = "xiaojin-device-identity-v1";
const XIAOJIN_DEVICE_ID: &str = "r5c-lab";
const XIAOJIN_BOARD_ID: &str = "nanopi-r5c";
const XIAOJIN_SOC_ID: &str = "rk3568";
const XIAOJIN_ARCH_ID: &str = "aarch64";
const COLD_BOOT_AUTO_NET_LISTEN: bool = true;
const COLD_BOOT_SERIAL_ESCAPE_MS: u64 = 2_000;
const NET_BROADCAST_MAC: [u8; 6] = [0xff; 6];
const DHCP_CLIENT_PORT: u16 = 68;
const DHCP_SERVER_PORT: u16 = 67;
const DHCP_PACKET_BYTES: usize = 320;
const DHCP_MIN_PACKET_BYTES: usize = 300;
const DHCP_TIMEOUT_MS: u64 = 5_000;
const DHCP_MAGIC_COOKIE: [u8; 4] = [99, 130, 83, 99];
const DHCP_MSG_DISCOVER: u8 = 1;
const DHCP_MSG_OFFER: u8 = 2;
const DHCP_MSG_REQUEST: u8 = 3;
const DHCP_MSG_ACK: u8 = 5;
const DHCP_OPT_SUBNET_MASK: u8 = 1;
const DHCP_OPT_ROUTER: u8 = 3;
const DHCP_OPT_DNS: u8 = 6;
const DHCP_OPT_MESSAGE_TYPE: u8 = 53;
const DHCP_OPT_SERVER_ID: u8 = 54;
const DHCP_OPT_PARAMETER_REQUEST_LIST: u8 = 55;
const DHCP_OPT_MAX_MESSAGE_SIZE: u8 = 57;
const DHCP_OPT_CLIENT_ID: u8 = 61;
const DHCP_OPT_LEASE_TIME: u8 = 51;
const DHCP_OPT_HOSTNAME: u8 = 12;
const DHCP_OPT_REQUESTED_IP: u8 = 50;
const DHCP_OPT_END: u8 = 255;
const ARP_RESOLVE_TIMEOUT_MS: u64 = 1_000;
const ARP_RESOLVE_ATTEMPTS: u32 = 3;
const DNS_SERVER_PORT: u16 = 53;
const DNS_CLIENT_PORT_BASE: u16 = 53530;
const DNS_PACKET_BYTES: usize = 512;
const DNS_TIMEOUT_MS: u64 = 4_000;
const DNS_MAX_NAME_LEN: usize = 96;
const TCP_CLIENT_PORT_BASE: u16 = 49152;
const TCP_CONNECT_TIMEOUT_MS: u64 = 4_000;
const HTTP_TIMEOUT_MS: u64 = 5_000;
const HTTP_IDLE_TIMEOUT_MS: u64 = 500;
const HTTP_REQUEST_BYTES: usize = 768;
const HTTP_RESPONSE_MAX_BYTES: usize = 8 * 1024;
const HTTP_RESPONSE_SAMPLE_BYTES: usize = 2048;
const TCP_FLAG_PSH: u16 = 0x008;
const TCP_FLAG_SYN: u16 = 0x002;
const TCP_FLAG_RST: u16 = 0x004;
const TCP_FLAG_ACK: u16 = 0x010;
const TCP_FLAG_FIN: u16 = 0x001;
const NETLOAD_BEGIN_MAGIC: &[u8; 4] = b"XJB1";
const NETLOAD_DATA_MAGIC: &[u8; 4] = b"XJD1";
const NETLOAD_COMMIT_MAGIC: &[u8; 4] = b"XJC1";
const NETCONFIG_SET_IP_MAGIC: &[u8; 4] = b"XJI1";
const NETCONTROL_DHCP_MAGIC: &[u8; 4] = b"XJDH";
const NETCONTROL_QUERY_MAGIC: &[u8; 4] = b"XJQ1";
const NETCONTROL_PERSIST_MAGIC: &[u8; 4] = b"XJPS";
const NETCONTROL_BOOT_SLOT_MAGIC: &[u8; 4] = b"XJBT";
const NETCONTROL_REBOOT_MAGIC: &[u8; 4] = b"XJRB";
const NETCONTROL_EVENT_MAGIC: &[u8; 4] = b"XJEV";
const NETCONTROL_DB_BLOB_BEGIN_MAGIC: &[u8; 4] = b"XJBB";
const NETCONTROL_DB_BLOB_DATA_MAGIC: &[u8; 4] = b"XJBD";
const NETCONTROL_DB_BLOB_COMMIT_MAGIC: &[u8; 4] = b"XJBC";
const NETCONTROL_DB_BLOB_READ_MAGIC: &[u8; 4] = b"XJBR";
const NETCONTROL_QUERY_BYTES: usize = 384;
const NETLOAD_FLAG_GO: u32 = 1;
const NETLOAD_BOOT_MAGIC_ADDR: u64 = 0x0587_e800;
const NETLOAD_BOOT_MAGIC: u64 = 0x584a_4e45_5442_4f4f;
const NET_CONFIG_BOOT_MAGIC_ADDR: u64 = 0x0587_e820;
const NET_CONFIG_BOOT_MAGIC: u64 = 0x584a_4950_4346_4732;
const NET_CONFIG_BOOT_IPS_ADDR: u64 = NET_CONFIG_BOOT_MAGIC_ADDR + 8;
const NET_CONFIG_BOOT_NETMASKS_ADDR: u64 = NET_CONFIG_BOOT_IPS_ADDR + RTL_DEVICE_COUNT as u64 * 4;
const NET_CONFIG_BOOT_GATEWAYS_ADDR: u64 =
    NET_CONFIG_BOOT_NETMASKS_ADDR + RTL_DEVICE_COUNT as u64 * 4;
const NET_CONFIG_BOOT_DNS_ADDR: u64 = NET_CONFIG_BOOT_GATEWAYS_ADDR + RTL_DEVICE_COUNT as u64 * 4;
const ETH_TYPE_IPV4: u16 = 0x0800;
const ETH_TYPE_ARP: u16 = 0x0806;
const ARP_HTYPE_ETHERNET: u16 = 1;
const ARP_REQUEST: u16 = 1;
const ARP_REPLY: u16 = 2;
const IPV4_PROTO_TCP: u8 = 6;
const IPV4_PROTO_UDP: u8 = 17;
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
const GPIO2_BASE: u64 = 0xfe75_0000;
const GPIO3_BASE: u64 = 0xfe76_0000;
const SD_DWMMC_BASE: u64 = 0xfe2b_0000;
const SD_DWMMC_WINDOW_SIZE: u64 = 0x1000;
const SD_BLOCK_BYTES: usize = 512;
const DB_FILE_NAME_STR: &str = "xiaojin-db.dat";
const DB_FILE_NAME: &[u8] = b"xiaojin-db.dat";
const DB_MAGIC: &[u8; 8] = b"XJDB0001";
const DB_CATALOG_MAGIC: &[u8; 8] = b"XJCAT001";
const DB_WAL_MAGIC: &[u8; 8] = b"XJWAL001";
const DB_TABLE_MAGIC: &[u8; 8] = b"XJTBL001";
const DB_BTREE_MAGIC: &[u8; 8] = b"XJBT0001";
const DB_BTREE_LEAF_MAGIC: &[u8; 8] = b"XJLF0001";
const DB_EVENT_META_MAGIC: &[u8; 8] = b"XJEVMT01";
const DB_EVENT_PAGE_MAGIC: &[u8; 8] = b"XJEVNT01";
const DB_BLOB_META_MAGIC: &[u8; 8] = b"XJBLMT01";
const DB_FORMAT_VERSION: u32 = 1;
const DB_PAGE_BYTES: u32 = SD_BLOCK_BYTES as u32;
const DB_ROOT_CATALOG_PAGE: u32 = 1;
const DB_WAL_START_PAGE: u32 = 2;
const DB_WAL_PAGE_COUNT: u32 = 64;
const DB_DATA_START_PAGE: u32 = DB_WAL_START_PAGE + DB_WAL_PAGE_COUNT;
const DB_TABLE_ROOT_PAGE: u32 = DB_DATA_START_PAGE;
const DB_BTREE_ROOT_PAGE: u32 = 128;
const DB_BTREE_LEAF_START_PAGE: u32 = DB_BTREE_ROOT_PAGE + 1;
const DB_BTREE_LEAF_COUNT: u32 = 16;
const DB_BTREE_KEYS_PER_LEAF_RANGE: u32 = 1024;
const DB_BTREE_SPLIT_RECORDS: usize = DB_TABLE_RECORDS as usize + 1;
const DB_BTREE_SCAN_MAX_ROWS: usize = 8;
const DB_EVENT_META_PAGE: u32 = 192;
const DB_EVENT_START_PAGE: u32 = DB_EVENT_META_PAGE + 1;
const DB_EVENT_PAGE_COUNT: u32 = 256;
const DB_EVENT_PAYLOAD_OFFSET: usize = 64;
const DB_EVENT_PAYLOAD_BYTES: usize = DB_SUPERBLOCK_CRC_OFFSET - DB_EVENT_PAYLOAD_OFFSET;
const DB_EVENT_LIST_MAX_ROWS: usize = 8;
const DB_BLOB_META_PAGE: u32 = 512;
const DB_BLOB_START_PAGE: u32 = DB_BLOB_META_PAGE + 1;
const DB_BLOB_FLAG_ACTIVE: u32 = 1;
const DB_BLOB_FLAG_COMMITTED: u32 = 2;
const DB_BLOB_PACKET_BYTES: usize = SD_BLOCK_BYTES;
const DB_TABLE_RECORD_BASE: usize = 64;
const DB_TABLE_RECORD_BYTES: usize = 32;
const DB_TABLE_VALUE_BYTES: usize = 24;
const DB_TABLE_RECORDS: u32 = 13;
const DB_TABLE_RECORD_USED: u16 = 1;
const DB_TABLE_KEY_MAINT_HMAC: u32 = 0x584a_4d48; // "XJMH"
const DB_WAL_KIND_TEST: u32 = 1;
const DB_WAL_KIND_PUT: u32 = 2;
const DB_WAL_KIND_BPUT: u32 = 3;
const DB_WAL_KIND_DELETE: u32 = 4;
const DB_EVENT_KIND_MAINT_STATUS: u32 = 1000;
const DB_EVENT_KIND_MAINT_VERIFY: u32 = 1001;
const DB_EVENT_KIND_MAINT_PULL_EXEC: u32 = 1002;
const DB_EVENT_SOURCE_MAINTAINER: u32 = 2;
const DB_SUPERBLOCK_CRC_OFFSET: usize = SD_BLOCK_BYTES - 4;
const SD_CMD_READ_SINGLE_BLOCK: u32 = 17;
const SD_CMD_WRITE_SINGLE_BLOCK: u32 = 24;
const DWMCI_CTRL: u64 = 0x000;
const DWMCI_BLKSIZ: u64 = 0x01c;
const DWMCI_BYTCNT: u64 = 0x020;
const DWMCI_CMDARG: u64 = 0x028;
const DWMCI_CMD: u64 = 0x02c;
const DWMCI_RINTSTS: u64 = 0x044;
const DWMCI_STATUS: u64 = 0x048;
const DWMCI_BMOD: u64 = 0x080;
const DWMCI_DBADDR: u64 = 0x088;
const DWMCI_IDSTS: u64 = 0x08c;
const DWMCI_IDINTEN: u64 = 0x090;
const DWMCI_CTRL_FIFO_RESET: u32 = 1 << 1;
const DWMCI_CTRL_DMA_RESET: u32 = 1 << 2;
const DWMCI_CTRL_DMA_ENABLE: u32 = 1 << 5;
const DWMCI_CTRL_IDMAC_ENABLE: u32 = 1 << 25;
const DWMCI_CMD_RESP_EXPECT: u32 = 1 << 6;
const DWMCI_CMD_CHECK_CRC: u32 = 1 << 8;
const DWMCI_CMD_DATA_EXP: u32 = 1 << 9;
const DWMCI_CMD_WRITE: u32 = 1 << 10;
const DWMCI_CMD_WAIT_PRVDATA: u32 = 1 << 13;
const DWMCI_CMD_USE_HOLD_REG: u32 = 1 << 29;
const DWMCI_CMD_START: u32 = 1 << 31;
const DWMCI_RINTSTS_CDONE: u32 = 1 << 2;
const DWMCI_RINTSTS_DTO: u32 = 1 << 3;
const DWMCI_RINTSTS_RE: u32 = 1 << 1;
const DWMCI_RINTSTS_RCRC: u32 = 1 << 6;
const DWMCI_RINTSTS_DCRC: u32 = 1 << 7;
const DWMCI_RINTSTS_RTO: u32 = 1 << 8;
const DWMCI_RINTSTS_DRTO: u32 = 1 << 9;
const DWMCI_RINTSTS_HTO: u32 = 1 << 10;
const DWMCI_RINTSTS_FRUN: u32 = 1 << 11;
const DWMCI_RINTSTS_HLE: u32 = 1 << 12;
const DWMCI_RINTSTS_SBE: u32 = 1 << 13;
const DWMCI_RINTSTS_EBE: u32 = 1 << 15;
const DWMCI_RINTSTS_ERRORS: u32 = DWMCI_RINTSTS_RE
    | DWMCI_RINTSTS_RCRC
    | DWMCI_RINTSTS_DCRC
    | DWMCI_RINTSTS_RTO
    | DWMCI_RINTSTS_DRTO
    | DWMCI_RINTSTS_HTO
    | DWMCI_RINTSTS_FRUN
    | DWMCI_RINTSTS_HLE
    | DWMCI_RINTSTS_SBE
    | DWMCI_RINTSTS_EBE;
const DWMCI_STATUS_BUSY: u32 = 1 << 9;
const DWMCI_BMOD_SWR: u32 = 1 << 0;
const DWMCI_BMOD_FB: u32 = 1 << 1;
const DWMCI_BMOD_IDMAC_ENABLE: u32 = 1 << 7;
const DWMCI_IDSTS_ERRORS: u32 = 0x1e;
const DWMCI_IDMAC_OWN: u32 = 1 << 31;
const DWMCI_IDMAC_CH: u32 = 1 << 4;
const DWMCI_IDMAC_FS: u32 = 1 << 3;
const DWMCI_IDMAC_LD: u32 = 1 << 2;
const DWMCI_READ_TIMEOUT_MS: u64 = 1_000;
const GPIO_V2_PORT_DR: u64 = 0x00;
const GPIO_V2_PORT_DDR: u64 = 0x08;
const GPIO_V2_EXT_PORT: u64 = 0x70;
const GPIO_V2_VERSION_ID: u64 = 0x78;
const PCIE_POWER_PIN: u8 = 28;
const PCIE20_PINCTRL_PIN: u8 = 26;
const GPIO_TYPE_V2: u32 = 0x0100_0c2b;
const GPIO_TYPE_V2_1: u32 = 0x0101_157c;
const PCIE_ETH_RESET_COUNT: usize = 2;

const LOAD_ADDR: u64 = 0x0208_0000;
const DRAM_BANK0_START: u64 = 0x0020_0000;
const DRAM_BANK0_END: u64 = 0x0840_0000;
const DRAM_BANK1_START: u64 = 0x0940_0000;
const DRAM_BANK1_END: u64 = 0xf000_0000;
const MAX_LINE_LEN: usize = 128;
const MAX_HEXDUMP_LEN: usize = 256;
const MAX_SLEEP_MS: u64 = 10_000;
const DEFAULT_BENCH_ITERS: u64 = 1_000_000;
const MAX_BENCH_ITERS: u64 = 20_000_000;
const XLOAD_SLOT0_ADDR: u64 = 0x0208_0000;
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
static mut RTL_TX_NEXT: [usize; RTL_DEVICE_COUNT] = [0; RTL_DEVICE_COUNT];
static mut NETLOAD_STATE: NetLoadState = NetLoadState::empty();
static mut DB_BLOB_IMPORT_STATE: DbBlobImportState = DbBlobImportState::empty();
static mut LAST_NET_IMAGE: NetImageState = NetImageState::empty();
static mut SD_IDMAC_DESC: SdDmaDescStorage = SdDmaDescStorage(SdIdmacDesc::zero());
static mut SD_BLOCK_BUFFER: SdBlockStorage = SdBlockStorage([0; SD_BLOCK_BYTES]);
static mut SD_WRITE_BUFFER: SdBlockStorage = SdBlockStorage([0; SD_BLOCK_BYTES]);
static mut PROBE_ACTIVE: u8 = 0;
static mut PROBE_FAULTED: u8 = 0;

fn probe_flag_set(flag: *mut u8, value: bool) {
    unsafe {
        write_volatile(flag, u8::from(value));
    }
}

fn probe_flag_get(flag: *const u8) -> bool {
    unsafe { read_volatile(flag) != 0 }
}

fn probe_flag_take(flag: *mut u8) -> bool {
    let value = probe_flag_get(flag.cast_const());
    probe_flag_set(flag, false);
    value
}

struct MmioRegion {
    name: &'static str,
    start: u64,
    end: u64,
    description: &'static str,
}

struct SdReg {
    name: &'static str,
    offset: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct SdIdmacDesc {
    flags: u32,
    cnt: u32,
    addr: u32,
    next: u32,
}

impl SdIdmacDesc {
    const fn zero() -> Self {
        Self {
            flags: 0,
            cnt: 0,
            addr: 0,
            next: 0,
        }
    }
}

#[repr(C, align(64))]
struct SdDmaDescStorage(SdIdmacDesc);

#[repr(C, align(64))]
struct SdBlockStorage([u8; SD_BLOCK_BYTES]);

#[derive(Clone, Copy)]
struct Fat32Facts {
    part_lba: u32,
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    fat_sectors: u32,
    total_sectors: u32,
    root_cluster: u32,
    first_fat_lba: u32,
    data_lba: u32,
    root_lba: u32,
}

#[derive(Clone, Copy)]
struct FatDirTarget {
    name: &'static str,
    found: bool,
    cluster: u32,
    size: u32,
}

#[derive(Clone, Copy)]
struct FatDirEntryInfo {
    found: bool,
    attr: u8,
    cluster: u32,
    size: u32,
}

#[derive(Clone, Copy)]
struct DbFileInfo {
    facts: Fat32Facts,
    entry: FatDirEntryInfo,
}

#[derive(Clone, Copy)]
struct DbSuperblock {
    initialized: bool,
    version: u32,
    page_bytes: u32,
    page_count: u32,
    file_bytes: u32,
    root_page: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbCatalog {
    initialized: bool,
    version: u32,
    page_bytes: u32,
    page_count: u32,
    wal_start_page: u32,
    wal_page_count: u32,
    wal_next_page: u32,
    data_start_page: u32,
    table_root_page: u32,
    sequence: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbPageSummary {
    page: u32,
    crc32: u32,
    first16: [u8; 16],
}

#[derive(Clone, Copy)]
struct DbTablePage {
    initialized: bool,
    version: u32,
    root_page: u32,
    record_count: u32,
    capacity: u32,
    sequence: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbBtreeRoot {
    initialized: bool,
    version: u32,
    root_page: u32,
    leaf_start_page: u32,
    leaf_count: u32,
    leaf_capacity: u32,
    record_count: u32,
    sequence: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbBtreeLeaf {
    initialized: bool,
    version: u32,
    page: u32,
    record_count: u32,
    capacity: u32,
    min_key: u32,
    max_key: u32,
    sequence: u32,
    next_leaf: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbRowValue {
    key: u32,
    value_len: usize,
    value: [u8; DB_TABLE_VALUE_BYTES],
}

#[derive(Clone, Copy)]
struct DbBtreeScan {
    start_key: u32,
    requested_limit: u32,
    row_count: usize,
    visited_leaves: u32,
    rows: [DbRowValue; DB_BTREE_SCAN_MAX_ROWS],
}

#[derive(Clone, Copy)]
struct DbEventMeta {
    initialized: bool,
    version: u32,
    meta_page: u32,
    start_page: u32,
    page_count: u32,
    next_page: u32,
    event_count: u32,
    sequence: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbEventPage {
    initialized: bool,
    version: u32,
    page: u32,
    sequence: u32,
    ticks: u64,
    kind: u32,
    source: u32,
    payload_len: usize,
    payload_crc: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbBlobMeta {
    initialized: bool,
    version: u32,
    meta_page: u32,
    start_page: u32,
    page_count: u32,
    capacity_bytes: u32,
    expected_size: u32,
    received_size: u32,
    expected_crc: u32,
    committed_crc: u32,
    sequence: u32,
    flags: u32,
    crc_expected: u32,
    crc_actual: u32,
}

#[derive(Clone, Copy)]
struct DbEventList {
    start_sequence: u32,
    requested_limit: u32,
    row_count: usize,
    scanned_pages: u32,
    rows: [DbEventPage; DB_EVENT_LIST_MAX_ROWS],
}

#[derive(Clone, Copy)]
struct FatChainSummary {
    ok: bool,
    clusters: u32,
    expected_clusters: u32,
    contiguous: bool,
    first: u32,
    last: u32,
    terminal: u32,
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
struct NetPollStats {
    rx_packets: u32,
    arp_replies: u32,
    udp_replies: u32,
    control_replies: u32,
}

#[derive(Clone, Copy)]
struct WifiPcieInitResult {
    link_up: bool,
    ltssm: u32,
    cfg0: Option<u32>,
}

#[derive(Clone, Copy)]
struct WifiPcieTraceResult {
    init: WifiPcieInitResult,
    rasdes: u32,
    intr_misc: u32,
    intr_mask_misc: u32,
    dbg_mode: u32,
    samples: [u32; 8],
}

impl NetPollStats {
    const fn empty() -> Self {
        Self {
            rx_packets: 0,
            arp_replies: 0,
            udp_replies: 0,
            control_replies: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct NetLoadState {
    active: bool,
    addr: u64,
    size: u32,
    expected_crc: u32,
    received: u32,
    go: bool,
}

impl NetLoadState {
    const fn empty() -> Self {
        Self {
            active: false,
            addr: 0,
            size: 0,
            expected_crc: 0,
            received: 0,
            go: false,
        }
    }
}

#[derive(Clone, Copy)]
struct DbBlobImportState {
    active: bool,
    size: u32,
    expected_crc: u32,
    received: u32,
    rolling_crc: u32,
    fast_lba: bool,
    fast_lba_base: u32,
    page_count: u32,
    seq_lba: bool,
    seq_facts: Fat32Facts,
    seq_sector_index: u32,
    seq_lba_current: u32,
    seq_cluster: u32,
    seq_sector_in_cluster: u32,
}

impl DbBlobImportState {
    const fn empty() -> Self {
        Self {
            active: false,
            size: 0,
            expected_crc: 0,
            received: 0,
            rolling_crc: 0xffff_ffff,
            fast_lba: false,
            fast_lba_base: 0,
            page_count: 0,
            seq_lba: false,
            seq_facts: Fat32Facts {
                part_lba: 0,
                bytes_per_sector: 0,
                sectors_per_cluster: 0,
                reserved_sectors: 0,
                fats: 0,
                fat_sectors: 0,
                total_sectors: 0,
                root_cluster: 0,
                first_fat_lba: 0,
                data_lba: 0,
                root_lba: 0,
            },
            seq_sector_index: 0,
            seq_lba_current: 0,
            seq_cluster: 0,
            seq_sector_in_cluster: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct NetImageState {
    valid: bool,
    addr: u64,
    size: u32,
    crc32: u32,
}

impl NetImageState {
    const fn empty() -> Self {
        Self {
            valid: false,
            addr: 0,
            size: 0,
            crc32: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct DhcpLease {
    message_type: u8,
    ip: [u8; 4],
    netmask: [u8; 4],
    gateway: [u8; 4],
    dns: [u8; 4],
    server: [u8; 4],
    lease_seconds: u32,
}

impl DhcpLease {
    fn empty(index: usize) -> Self {
        Self {
            message_type: 0,
            ip: [0, 0, 0, 0],
            netmask: DEFAULT_NET_NETMASKS[index],
            gateway: [0, 0, 0, 0],
            dns: [0, 0, 0, 0],
            server: [0, 0, 0, 0],
            lease_seconds: 0,
        }
    }
}

#[derive(Clone, Copy)]
struct DhcpExchange {
    offer: DhcpLease,
    ack: DhcpLease,
}

#[derive(Clone, Copy)]
enum DhcpError {
    DiscoverTx,
    OfferTimeout,
    RequestTx,
    AckTimeout,
}

impl DhcpError {
    fn as_str(self) -> &'static str {
        match self {
            Self::DiscoverTx => "discover-tx",
            Self::OfferTimeout => "offer-timeout",
            Self::RequestTx => "request-tx",
            Self::AckTimeout => "ack-timeout",
        }
    }
}

#[derive(Clone, Copy)]
struct DnsResult {
    server: [u8; 4],
    next_hop: [u8; 4],
    next_hop_mac: [u8; 6],
    answer: [u8; 4],
    txid: u16,
}

#[derive(Clone, Copy)]
enum DnsError {
    BadName,
    MissingDns,
    MissingRoute,
    ArpTimeout,
    QueryTx,
    ResponseTimeout,
}

impl DnsError {
    fn as_str(self) -> &'static str {
        match self {
            Self::BadName => "bad-name",
            Self::MissingDns => "missing-dns",
            Self::MissingRoute => "missing-route",
            Self::ArpTimeout => "arp-timeout",
            Self::QueryTx => "query-tx",
            Self::ResponseTimeout => "response-timeout",
        }
    }
}

#[derive(Clone, Copy)]
struct TcpConnectResult {
    dst_ip: [u8; 4],
    next_hop: [u8; 4],
    next_hop_mac: [u8; 6],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    flags: u16,
    dns_txid: u16,
}

#[derive(Clone, Copy)]
struct TcpSession {
    dst_ip: [u8; 4],
    next_hop: [u8; 4],
    next_hop_mac: [u8; 6],
    src_port: u16,
    dst_port: u16,
    local_seq: u32,
    remote_seq_next: u32,
    initial_seq: u32,
    synack_flags: u16,
    dns_txid: u16,
}

#[derive(Clone, Copy)]
enum TcpError {
    BadHost,
    BadPort,
    Dns(DnsError),
    MissingRoute,
    ArpTimeout,
    SynTx,
    ResponseTimeout,
    Reset,
}

impl TcpError {
    fn as_str(self) -> &'static str {
        match self {
            Self::BadHost => "bad-host",
            Self::BadPort => "bad-port",
            Self::Dns(error) => error.as_str(),
            Self::MissingRoute => "missing-route",
            Self::ArpTimeout => "arp-timeout",
            Self::SynTx => "syn-tx",
            Self::ResponseTimeout => "response-timeout",
            Self::Reset => "reset",
        }
    }
}

#[derive(Clone, Copy)]
struct HttpGetResult {
    tcp: TcpSession,
    status_code: u16,
    response_bytes: usize,
    payload_bytes: usize,
    segments: u32,
    fin: bool,
    truncated: bool,
    sample_len: usize,
    sample: [u8; HTTP_RESPONSE_SAMPLE_BYTES],
}

#[derive(Clone, Copy)]
struct HttpPostResult {
    tcp: TcpSession,
    status_code: u16,
    response_bytes: usize,
    payload_bytes: usize,
    segments: u32,
    fin: bool,
    truncated: bool,
    sent_payload_len: u32,
    sample_len: usize,
    sample: [u8; HTTP_RESPONSE_SAMPLE_BYTES],
}

#[derive(Clone, Copy)]
struct HttpTraceResult {
    tcp: TcpSession,
    response_bytes: usize,
    payload_bytes: usize,
    segments: u32,
    fin: bool,
    truncated: bool,
    sample_len: usize,
    sample: [u8; HTTP_RESPONSE_SAMPLE_BYTES],
}

#[derive(Clone, Copy)]
struct HttpDebugResult {
    tcp: TcpSession,
    sent_payload_len: u32,
    segment: TcpDebugSegment,
}

#[derive(Clone, Copy)]
struct TcpDebugSegment {
    seq: u32,
    ack: u32,
    flags: u16,
    segment_len: usize,
    payload_len: usize,
    sample_len: usize,
    sample: [u8; HTTP_RESPONSE_SAMPLE_BYTES],
}

#[derive(Clone, Copy)]
enum HttpError {
    BadPath,
    BadBody,
    RequestTooLong,
    Tcp(TcpError),
    RequestTx,
    ResponseTimeout,
    MalformedResponse,
}

impl HttpError {
    fn as_str(self) -> &'static str {
        match self {
            Self::BadPath => "bad-path",
            Self::BadBody => "bad-body",
            Self::RequestTooLong => "request-too-long",
            Self::Tcp(error) => error.as_str(),
            Self::RequestTx => "request-tx",
            Self::ResponseTimeout => "response-timeout",
            Self::MalformedResponse => "malformed-response",
        }
    }
}

#[derive(Clone, Copy)]
enum SdReadError {
    ProbeFault,
    AddrTooHigh,
    TimeoutReset,
    TimeoutBusy,
    TimeoutCommand,
    TimeoutData,
    StatusError(u32),
    IdmacError(u32),
    DmaOwnTimeout,
    InvalidLayout,
}

impl SdReadError {
    fn as_str(self) -> &'static str {
        match self {
            Self::ProbeFault => "probe-fault",
            Self::AddrTooHigh => "addr-too-high",
            Self::TimeoutReset => "timeout-reset",
            Self::TimeoutBusy => "timeout-busy",
            Self::TimeoutCommand => "timeout-command",
            Self::TimeoutData => "timeout-data",
            Self::StatusError(_) => "status-error",
            Self::IdmacError(_) => "idmac-error",
            Self::DmaOwnTimeout => "dma-own-timeout",
            Self::InvalidLayout => "invalid-layout",
        }
    }
}

#[derive(Clone, Copy)]
enum PersistError {
    Sd(SdReadError),
    ImageTooLarge,
    SlotMissing,
    BadChain,
    CrcMismatch(u32),
}

impl PersistError {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sd(error) => error.as_str(),
            Self::ImageTooLarge => "image-too-large",
            Self::SlotMissing => "slot-missing",
            Self::BadChain => "bad-chain",
            Self::CrcMismatch(_) => "crc-mismatch",
        }
    }
}

impl From<SdReadError> for PersistError {
    fn from(value: SdReadError) -> Self {
        Self::Sd(value)
    }
}

#[derive(Clone, Copy)]
enum BootSlotError {
    Sd(SdReadError),
    ExtlinuxDirMissing,
    ExtlinuxConfMissing,
    ConfigTooLarge,
    BadChain,
    VerifyFailed,
}

impl BootSlotError {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sd(error) => error.as_str(),
            Self::ExtlinuxDirMissing => "extlinux-dir-missing",
            Self::ExtlinuxConfMissing => "extlinux-conf-missing",
            Self::ConfigTooLarge => "config-too-large",
            Self::BadChain => "bad-chain",
            Self::VerifyFailed => "verify-failed",
        }
    }
}

impl From<SdReadError> for BootSlotError {
    fn from(value: SdReadError) -> Self {
        Self::Sd(value)
    }
}

#[derive(Clone, Copy)]
enum DbError {
    Sd(SdReadError),
    MissingFile,
    FileTooSmall,
    BadChain,
    BadSuperblock,
    BadCatalog,
    BadTable,
    BadBtreeRoot,
    BadBtreeLeaf,
    BadEventMeta,
    BadEventPage,
    BadBlobMeta,
    TableMissing,
    TableFull,
    BtreeMissing,
    BtreeFull,
    EventMissing,
    BlobMissing,
    BlobTooLarge,
    BadBlobOffset,
    BadValue,
    KeyMissing,
    PageOutOfRange,
    WriteFailed,
}

impl DbError {
    fn as_str(self) -> &'static str {
        match self {
            Self::Sd(error) => error.as_str(),
            Self::MissingFile => "db-file-missing",
            Self::FileTooSmall => "db-file-too-small",
            Self::BadChain => "db-bad-chain",
            Self::BadSuperblock => "db-bad-superblock",
            Self::BadCatalog => "db-bad-catalog",
            Self::BadTable => "db-bad-table",
            Self::BadBtreeRoot => "db-bad-btree-root",
            Self::BadBtreeLeaf => "db-bad-btree-leaf",
            Self::BadEventMeta => "db-bad-event-meta",
            Self::BadEventPage => "db-bad-event-page",
            Self::BadBlobMeta => "db-bad-blob-meta",
            Self::TableMissing => "db-table-missing",
            Self::TableFull => "db-table-full",
            Self::BtreeMissing => "db-btree-missing",
            Self::BtreeFull => "db-btree-full",
            Self::EventMissing => "db-event-missing",
            Self::BlobMissing => "db-blob-missing",
            Self::BlobTooLarge => "db-blob-too-large",
            Self::BadBlobOffset => "db-bad-blob-offset",
            Self::BadValue => "db-bad-value",
            Self::KeyMissing => "db-key-missing",
            Self::PageOutOfRange => "db-page-out-of-range",
            Self::WriteFailed => "db-write-failed",
        }
    }
}

impl From<SdReadError> for DbError {
    fn from(value: SdReadError) -> Self {
        Self::Sd(value)
    }
}

#[derive(Clone, Copy)]
struct NetRuntimeConfig {
    ips: [[u8; 4]; RTL_DEVICE_COUNT],
    netmasks: [[u8; 4]; RTL_DEVICE_COUNT],
    gateways: [[u8; 4]; RTL_DEVICE_COUNT],
    dns: [[u8; 4]; RTL_DEVICE_COUNT],
}

impl NetRuntimeConfig {
    const fn default() -> Self {
        Self {
            ips: DEFAULT_NET_DEVICE_IPS,
            netmasks: DEFAULT_NET_NETMASKS,
            gateways: DEFAULT_NET_GATEWAYS,
            dns: DEFAULT_NET_DNS,
        }
    }
}

static mut NET_CONFIG: NetRuntimeConfig = NetRuntimeConfig::default();

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
        name: "gpio2",
        start: GPIO2_BASE,
        end: GPIO2_BASE + 0x100,
        description: "RK3568 GPIO bank 2",
    },
    MmioRegion {
        name: "gpio3",
        start: GPIO3_BASE,
        end: GPIO3_BASE + 0x100,
        description: "RK3568 GPIO bank 3",
    },
    MmioRegion {
        name: "sdmmc0",
        start: SD_DWMMC_BASE,
        end: SD_DWMMC_BASE + SD_DWMMC_WINDOW_SIZE,
        description: "RK3568 DW-MMC SD controller, U-Boot mmc1",
    },
];

const SD_DWMMC_REGS: &[SdReg] = &[
    SdReg {
        name: "ctrl",
        offset: 0x000,
    },
    SdReg {
        name: "pwren",
        offset: 0x004,
    },
    SdReg {
        name: "clkdiv",
        offset: 0x008,
    },
    SdReg {
        name: "clkena",
        offset: 0x010,
    },
    SdReg {
        name: "ctype",
        offset: 0x018,
    },
    SdReg {
        name: "blksiz",
        offset: 0x01c,
    },
    SdReg {
        name: "bytcnt",
        offset: 0x020,
    },
    SdReg {
        name: "cmd",
        offset: 0x02c,
    },
    SdReg {
        name: "mintsts",
        offset: 0x040,
    },
    SdReg {
        name: "rintsts",
        offset: 0x044,
    },
    SdReg {
        name: "status",
        offset: 0x048,
    },
    SdReg {
        name: "fifoth",
        offset: 0x04c,
    },
    SdReg {
        name: "cdetect",
        offset: 0x050,
    },
    SdReg {
        name: "wrtprt",
        offset: 0x054,
    },
    SdReg {
        name: "usrid",
        offset: 0x068,
    },
    SdReg {
        name: "verid",
        offset: 0x06c,
    },
    SdReg {
        name: "hcon",
        offset: 0x070,
    },
    SdReg {
        name: "uhsreg",
        offset: 0x074,
    },
    SdReg {
        name: "bmod",
        offset: 0x080,
    },
    SdReg {
        name: "idsts",
        offset: 0x08c,
    },
];

const GPIO0: GpioBank = GpioBank {
    name: "gpio0",
    base: GPIO0_BASE,
};
const GPIO2: GpioBank = GpioBank {
    name: "gpio2",
    base: GPIO2_BASE,
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

const PCIE2_CONTROLLER: PcieController = PcieController {
    name: "pcie2x1",
    apb_base: PCIE2X1_APB_BASE,
    dbi_base: PCIE2X1_DBI_BASE,
    cfg_base: PCIE2X1_CFG_BASE,
    mem32_base: PCIE2X1_MEM32_BASE,
    root_bus: 0x00,
    reset_pin: "gpio3 PC1",
};

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

const PCIE2_CLOCK_GATES: &[CruBit] = &[
    CruBit {
        name: "aclk_pcie20_mst",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 0,
    },
    CruBit {
        name: "aclk_pcie20_slv",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 1,
    },
    CruBit {
        name: "aclk_pcie20_dbi",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 2,
    },
    CruBit {
        name: "pclk_pcie20",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 3,
    },
    CruBit {
        name: "clk_pcie20_aux_ndft",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 4,
    },
    CruBit {
        name: "clk_pcie20_pipe_dft",
        base: CRU_BASE,
        offset: RK3568_CLKGATE_BASE + 12 * 4,
        bit: 5,
    },
];

const PCIE2_RESETS: &[CruBit] = &[
    CruBit {
        name: "srst_p_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 0,
    },
    CruBit {
        name: "srst_pcie20_powerup",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 1,
    },
    CruBit {
        name: "srst_mstr_areset_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 2,
    },
    CruBit {
        name: "srst_slv_areset_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 3,
    },
    CruBit {
        name: "srst_dbi_areset_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 4,
    },
    CruBit {
        name: "srst_breset_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 5,
    },
    CruBit {
        name: "srst_perst_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 6,
    },
    CruBit {
        name: "srst_core_rst_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 7,
    },
    CruBit {
        name: "srst_nsticky_rst_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 8,
    },
    CruBit {
        name: "srst_sticky_rst_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 9,
    },
    CruBit {
        name: "srst_pwr_rst_pcie20",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 10 * 4,
        bit: 10,
    },
];

const COMBPHY2_CLOCK_GATES: &[CruBit] = &[CruBit {
    name: "pclk_pipephy2",
    base: CRU_BASE,
    offset: RK3568_CLKGATE_BASE + 34 * 4,
    bit: 6,
}];

const PCIEPHY2_REF_CLOCK_GATES: &[CruBit] = &[
    CruBit {
        name: "clk_pciephy2_div",
        base: PMUCRU_BASE,
        offset: RK3568_PMU_CLKGATE_BASE + 2 * 4,
        bit: 11,
    },
    CruBit {
        name: "clk_pciephy2_osc0",
        base: PMUCRU_BASE,
        offset: RK3568_PMU_CLKGATE_BASE + 2 * 4,
        bit: 12,
    },
];

const COMBPHY2_RESETS: &[CruBit] = &[
    CruBit {
        name: "srst_p_pipephy2",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 28 * 4,
        bit: 8,
    },
    CruBit {
        name: "srst_pipephy2",
        base: CRU_BASE,
        offset: RK3568_SOFTRST_BASE + 28 * 4,
        bit: 9,
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
    load_net_config_from_boot();
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
    if take_netload_boot_flag() {
        uart.put_str("Netload boot flag detected; entering network listener.\n");
        command_net_polling(&mut uart, "net listen", None);
    } else if COLD_BOOT_AUTO_NET_LISTEN {
        let _ = writeln!(
            uart,
            "Cold boot network listener armed; press any serial key within {COLD_BOOT_SERIAL_ESCAPE_MS}ms for shell."
        );
        if uart.get_byte_timeout(COLD_BOOT_SERIAL_ESCAPE_MS).is_some() {
            uart.put_str("Serial escape detected; staying in shell.\n");
        } else {
            uart.put_str("No serial escape detected; entering network listener.\n");
            command_net_polling(&mut uart, "net listen", None);
        }
    }
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
    } else if command.eq_ignore_ascii_case("identity") {
        print_identity(uart);
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
        command_net(uart, &mut parts);
    } else if command.eq_ignore_ascii_case("maint") {
        command_maint(uart, &mut parts);
    } else if command.eq_ignore_ascii_case("wifi") || command.eq_ignore_ascii_case("wlan") {
        command_wifi(uart, &mut parts);
    } else if command.eq_ignore_ascii_case("sd") {
        command_sd(uart, parts.next(), parts.next());
    } else if command.eq_ignore_ascii_case("db") {
        command_db(uart, &mut parts);
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
    uart.put_str("  identity             show stable device identity and fingerprint\n");
    uart.put_str("  mem                  show current memory map facts\n");
    uart.put_str("  regs                 show selected CPU registers\n");
    uart.put_str("  time                 show generic timer registers and uptime\n");
    uart.put_str("  uptime               show elapsed kernel time\n");
    uart.put_str("  sleep <ms>           busy-wait with generic timer, max 10000\n");
    uart.put_str("  bench [iters]        run timer-backed CPU microbenchmarks\n");
    uart.put_str("  net facts            show wired-network bring-up plan\n");
    uart.put_str("  net ips              show runtime IP config\n");
    uart.put_str("  net ip <if> <ip> confirm\n");
    uart.put_str("                       set runtime IP for pcie3x1/pcie3x2\n");
    uart.put_str("  net dhcp <if> confirm\n");
    uart.put_str("                       request IPv4 config from a DHCP server\n");
    uart.put_str("  net dns <if> <name> confirm\n");
    uart.put_str("                       resolve one DNS A record through runtime DNS\n");
    uart.put_str("  net tcp <if> <host|ip> <port> confirm\n");
    uart.put_str("                       prove outbound TCP SYN/SYN-ACK/ACK path\n");
    uart.put_str("  net http <if> <host|ip> <port> <path> confirm\n");
    uart.put_str("                       perform minimal HTTP/1.0 GET over outbound TCP\n");
    uart.put_str("  net http-post <if> <host|ip> <port> <path> <body> confirm\n");
    uart.put_str("                       perform minimal HTTP/1.0 POST over outbound TCP\n");
    uart.put_str("  maint poll <if> <host|ip> <port> <path> confirm\n");
    uart.put_str("                       pull one cleartext maintainer endpoint summary\n");
    uart.put_str("  maint status <if> <host|ip> <port> <path> confirm\n");
    uart.put_str("                       parse cleartext maintainer status and pending summary\n");
    uart.put_str("  maint audit <if> <host|ip> <port> <path> confirm\n");
    uart.put_str("                       append maintainer status summary into XiaojinDB events\n");
    uart.put_str("  maint verify <if> <host|ip> <port> <path> confirm\n");
    uart.put_str("  maint verify-hmac <if> <host|ip> <port> <path> <key-hex> confirm\n");
    uart.put_str("  maint verify-local <if> <host|ip> <port> <path> confirm\n");
    uart.put_str("                       verify pending command digest and audit the result\n");
    uart.put_str("  maint pull-exec <if> <host|ip> <port> <poll-path> [event-path] confirm\n");
    uart.put_str("                       verify and execute one signed read-only xj_query\n");
    uart.put_str("  maint key-status confirm\n");
    uart.put_str("  maint key-set <key-hex> confirm\n");
    uart.put_str("  maint key-clear confirm\n");
    uart.put_str("  maint ack-intent <if> <host|ip> <port> <poll-path> <event-path> confirm\n");
    uart.put_str(
        "                       verify pending command digest and POST an ACK intent event\n",
    );
    uart.put_str("  maint event <if> <host|ip> <port> <path> <kind> confirm\n");
    uart.put_str(
        "                       POST one public maintainer event to the GCP control plane\n",
    );
    uart.put_str("  net loop confirm     run ARP/IPv4/UDP responder for 120s\n");
    uart.put_str("  net listen confirm   run ARP/IPv4/UDP responder until Ctrl-C\n");
    uart.put_str("  wifi facts           show R5C WiFi hardware and bring-up facts\n");
    uart.put_str("  wifi status          show current WiFi implementation status\n");
    uart.put_str("  wifi pcie            show RTL8822CE candidate PCIe2 x1 address map\n");
    uart.put_str("  wifi cru             show read-only PCIe2 clock/reset status\n");
    uart.put_str("  wifi apb             show read-only PCIe2 APB status\n");
    uart.put_str("  wifi phy             show read-only combphy2 status\n");
    uart.put_str("  wifi power           show PCIe 3.3V regulator GPIO state\n");
    uart.put_str("  wifi link            show PCIe2 link/DBI/config safety status\n");
    uart.put_str("  wifi trace confirm   retrain PCIe2 and show LTSSM debug FIFO evidence\n");
    uart.put_str("  wifi sideband <in|low|high> confirm\n");
    uart.put_str("  wifi pcie-init confirm\n");
    uart.put_str("  wifi pcie-init-high confirm\n");
    uart.put_str("  wifi pcie-init-linux confirm\n");
    uart.put_str("                       initialize PCIe2/combphy2 and probe WiFi config\n");
    uart.put_str("  wifi scan confirm    reserved station-mode scan command\n");
    uart.put_str("  wifi join <ssid> confirm\n");
    uart.put_str("                       reserved station-mode association command\n");
    uart.put_str("  wifi ap <ssid> confirm\n");
    uart.put_str("                       reserved AP-mode bring-up command\n");
    uart.put_str("  sd facts             show SD/MMC OTA storage plan\n");
    uart.put_str("  sd regs confirm      read SD controller probe registers\n");
    uart.put_str("  sd read0 confirm     read SD LBA0 and print MBR/BPB summary\n");
    uart.put_str("  sd bpb confirm       read FAT32 BPB from the first partition\n");
    uart.put_str("  sd root confirm      scan FAT32 root dir for Xiaojin slots\n");
    uart.put_str("  sd slots confirm     verify FAT chains for A/B slot files\n");
    uart.put_str("  db facts             show XiaojinDB page-store plan\n");
    uart.put_str("  db status confirm    read XiaojinDB superblock from SD page file\n");
    uart.put_str("  db init confirm      initialize XiaojinDB superblock in SD page file\n");
    uart.put_str("  db catalog confirm   read XiaojinDB catalog page\n");
    uart.put_str("  db catalog-init confirm\n");
    uart.put_str("                       initialize XiaojinDB catalog page\n");
    uart.put_str("  db page <n> confirm  read one XiaojinDB page summary\n");
    uart.put_str("  db wal-append confirm\n");
    uart.put_str("                       append one test WAL page and advance catalog\n");
    uart.put_str("  db table confirm     read fixed key/value table metadata\n");
    uart.put_str("  db table-init confirm\n");
    uart.put_str("                       initialize fixed key/value root table page\n");
    uart.put_str("  db put <key> <value> confirm\n");
    uart.put_str("                       upsert one u32 key and <=24-byte value\n");
    uart.put_str("  db get <key> confirm read one u32 key from the fixed table\n");
    uart.put_str("  db btree confirm    read experimental B+tree root metadata\n");
    uart.put_str("  db btree-init confirm\n");
    uart.put_str("                       initialize experimental B+tree root and leaf pages\n");
    uart.put_str("  db bput <key> <value> confirm\n");
    uart.put_str("                       WAL-log and upsert one B+tree key/value record\n");
    uart.put_str("  db bget <key> confirm\n");
    uart.put_str("                       read one key from the experimental B+tree\n");
    uart.put_str("  db bscan <start> <limit> confirm\n");
    uart.put_str("                       scan up to 8 B+tree rows in key order\n");
    uart.put_str("  db bleaf <page> confirm\n");
    uart.put_str("                       read one B+tree leaf header for diagnosis\n");
    uart.put_str("  db event-init confirm\n");
    uart.put_str("                       initialize append-only router event log pages\n");
    uart.put_str("  db event-status confirm\n");
    uart.put_str("                       read router event log metadata\n");
    uart.put_str("  db events <start-seq> <limit> confirm\n");
    uart.put_str("                       list up to 8 router event page headers\n");
    uart.put_str("  db event <seq> confirm\n");
    uart.put_str("                       read one router event page header\n");
    uart.put_str("  db blob-status confirm\n");
    uart.put_str("                       read PostgreSQL backup blob metadata\n");
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
    uart.put_str("  net runtime-ip-config yes\n");
    uart.put_str("  net udp-ip-config yes\n");
    uart.put_str("  net dhcp-client yes\n");
    uart.put_str("  net auto-dhcp yes\n");
    uart.put_str("  net default-route yes\n");
    uart.put_str("  net arp-resolve yes\n");
    uart.put_str("  net dns-client yes\n");
    uart.put_str("  net udp-dhcp-control yes\n");
    uart.put_str("  net udp-query-control yes\n");
    uart.put_str("  net routed-udp-loader yes\n");
    uart.put_str("  net http-post yes\n");
    uart.put_str("  maint http-poll yes\n");
    uart.put_str("  maint pending-summary yes\n");
    uart.put_str("  maint command-digest yes\n");
    uart.put_str("  maint command-verify yes\n");
    uart.put_str("  maint command-verify-hmac yes\n");
    uart.put_str("  maint local-hmac-key yes\n");
    uart.put_str("  maint pull-exec yes\n");
    uart.put_str("  maint ack-intent yes\n");
    uart.put_str("  maint remote-event yes\n");
    uart.put_str("  maint local-audit yes\n");
    uart.put_str("  identity facts yes\n");
    uart.put_str("  wifi facts yes\n");
    uart.put_str("  wifi pcie-facts yes\n");
    uart.put_str("  wifi link-readonly yes\n");
    uart.put_str("  wifi station planned\n");
    uart.put_str("  wifi ap planned\n");
    uart.put_str("  sd probe-readonly yes\n");
    uart.put_str("  sd writer yes\n");
    uart.put_str("  db page-store yes\n");
    uart.put_str("  db btree yes\n");
    let _ = writeln!(uart, "  xload link-address 0x{LOAD_ADDR:016x}");
    let _ = writeln!(
        uart,
        "  xload trampoline-address 0x{XLOAD_TRAMPOLINE_ADDR:016x}"
    );
}

fn write_caps_query_summary<W: Write>(out: &mut W) {
    let _ = write!(
        out,
        "XJQ OK caps net-dev=yes dhcp=yes auto-dhcp=yes route=yes arp=yes dns=yes tcp-connect=yes http-get=yes http-post=yes http-trace=yes http-debug=yes maint-poll=yes maint-status=yes maint-pending-summary=yes maint-command-digest=yes maint-command-verify=yes maint-command-verify-hmac=yes maint-local-hmac-key=yes maint-pull-exec=yes maint-ack-intent=yes maint-remote-event=yes maint-local-audit=yes identity=yes query=yes reboot=yes wifi=facts-only wifi-pcie=facts wifi-cru=read-only wifi-apb=read-only wifi-phy=read-only wifi-power=read-only wifi-link=read-only wifi-trace=confirm wifi-pcie-init=confirm wifi-pcie-init-high=confirm wifi-station=planned wifi-ap=planned sd-probe=yes sd-writer=yes ab-persist=yes boot-slot=extlinux db=page-store,catalog,wal,fixed-table,btree,event-log,blob-backup,blob-restore"
    );
}

fn print_about(uart: &mut Uart) {
    uart.put_str("Xiaojin OS 0.1.0\n");
    let _ = writeln!(uart, "device: {XIAOJIN_DEVICE_ID}");
    let _ = writeln!(uart, "board: {XIAOJIN_BOARD_ID}");
    let _ = writeln!(uart, "soc: {XIAOJIN_SOC_ID}");
    uart.put_str("kernel: AArch64 no_std Rust\n");
    uart.put_str("control: UART2 shell\n");
    uart.put_str("network target: PCIe RTL8125 wired loader\n");
    uart.put_str("wireless target: RTL8822CE over R5C M.2 E-Key, driver planned\n");
}

fn print_identity(uart: &mut Uart) {
    write_identity_summary(uart);
    uart.put_str("\n");
}

fn write_identity_summary<W: Write>(out: &mut W) {
    let fingerprint = device_identity_digest();
    let _ = write!(
        out,
        "identity format={XIAOJIN_IDENTITY_FORMAT} device_id={XIAOJIN_DEVICE_ID} board_id={XIAOJIN_BOARD_ID} soc={XIAOJIN_SOC_ID} arch={XIAOJIN_ARCH_ID} fingerprint="
    );
    write_sha256_hex(out, &fingerprint);
}

fn command_wifi<'a>(uart: &mut Uart, parts: &mut core::str::SplitWhitespace<'a>) {
    let action = parts.next();
    match action {
        None | Some("facts") => print_wifi_facts(uart),
        Some("status") => print_wifi_status(uart),
        Some("roadmap") => print_wifi_roadmap(uart),
        Some("pcie") => print_wifi_pcie_facts(uart),
        Some("cru") => print_wifi_cru_status(uart),
        Some("apb") => print_wifi_apb_status(uart),
        Some("phy") => print_wifi_phy_status(uart),
        Some("power") => print_wifi_power_status(uart),
        Some("link") => print_wifi_link_status(uart),
        Some("trace") => command_wifi_trace(uart, parts.next()),
        Some("sideband") => command_wifi_sideband(uart, parts.next(), parts.next()),
        Some("pcie-init") => command_wifi_pcie_init(uart, parts.next()),
        Some("pcie-init-high") => command_wifi_pcie_init_high(uart, parts.next()),
        Some("pcie-init-linux") => command_wifi_pcie_init_linux(uart, parts.next()),
        Some("scan") => command_wifi_scan(uart, parts.next()),
        Some("join") => command_wifi_ssid_not_ready(uart, "join", parts.next(), parts.next()),
        Some("ap") => command_wifi_ssid_not_ready(uart, "ap", parts.next(), parts.next()),
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  wifi facts\n");
            uart.put_str("  wifi status\n");
            uart.put_str("  wifi roadmap\n");
            uart.put_str("  wifi pcie\n");
            uart.put_str("  wifi cru\n");
            uart.put_str("  wifi apb\n");
            uart.put_str("  wifi phy\n");
            uart.put_str("  wifi power\n");
            uart.put_str("  wifi link\n");
            uart.put_str("  wifi trace confirm\n");
            uart.put_str("  wifi sideband <in|low|high> confirm\n");
            uart.put_str("  wifi pcie-init confirm\n");
            uart.put_str("  wifi pcie-init-high confirm\n");
            uart.put_str("  wifi pcie-init-linux confirm\n");
            uart.put_str("  wifi scan confirm\n");
            uart.put_str("  wifi join <ssid> confirm\n");
            uart.put_str("  wifi ap <ssid> confirm\n");
        }
    }
}

fn print_wifi_facts(uart: &mut Uart) {
    uart.put_str("R5C WiFi facts:\n");
    uart.put_str("  module          RTL8822CE family M.2 E-Key WiFi/BT module\n");
    uart.put_str("  wifi path       candidate PCIe2 x1 behind R5C M.2 E-Key slot\n");
    uart.put_str("  bt path         sideband likely USB/UART; not enumerated by Xiaojin OS yet\n");
    uart.put_str("  current stage   facts-only, no WiFi register writes\n");
    uart.put_str("  station mode    planned: enumerate, load firmware, scan, WPA, DHCP, route\n");
    uart.put_str(
        "  AP mode         planned: SoftAP, beacon/probe/auth/assoc, DHCP server, NAT/route\n",
    );
    uart.put_str("  rescue rule     keep wired RTL8125 net-dev/net-ota as priority path\n");
}

fn print_wifi_status(uart: &mut Uart) {
    uart.put_str("wifi status:\n");
    uart.put_str("  state           facts-only\n");
    uart.put_str("  link            down\n");
    uart.put_str("  scan            unavailable\n");
    uart.put_str("  join            unavailable\n");
    uart.put_str("  ap              unavailable\n");
    uart.put_str("  reason          RTL8822CE PCIe/firmware/802.11/WPA driver not implemented\n");
}

fn print_wifi_roadmap(uart: &mut Uart) {
    uart.put_str("wifi roadmap:\n");
    uart.put_str("  1 pcie2x1 facts + safe reset/status probes\n");
    uart.put_str("  2 config-space enumeration for RTL8822CE without touching wired NICs\n");
    uart.put_str("  3 firmware loader and MMIO/interrupt/DMA skeleton\n");
    uart.put_str("  4 station scan/auth/assoc plus WPA2-PSK and DHCP route\n");
    uart.put_str("  5 AP mode: beacon/probe/auth/assoc plus local DHCP service\n");
    uart.put_str("  6 optional bridge/NAT from WiFi AP to wired/cloud maintenance path\n");
}

fn print_wifi_pcie_facts(uart: &mut Uart) {
    uart.put_str("wifi PCIe facts from R5C device tree:\n");
    uart.put_str("  node            /pcie@fe260000\n");
    uart.put_str("  compatible      rockchip,rk3568-pcie\n");
    uart.put_str("  role            candidate RTL8822CE M.2 E-Key WiFi path\n");
    uart.put_str("  bus-range       0x00..0x0f\n");
    uart.put_str("  lanes           1\n");
    uart.put_str("  reset           gpio3 PC1 active-high\n");
    let _ = writeln!(
        uart,
        "  dbi             0x{PCIE2X1_DBI_BASE:016x}..0x{:016x}",
        PCIE2X1_DBI_BASE + 0x0040_0000
    );
    let _ = writeln!(
        uart,
        "  apb             0x{PCIE2X1_APB_BASE:016x}..0x{:016x}",
        PCIE2X1_APB_BASE + 0x0001_0000
    );
    let _ = writeln!(
        uart,
        "  cfg             0x{PCIE2X1_CFG_BASE:016x}..0x{:016x}",
        PCIE2X1_CFG_BASE + 0x0010_0000
    );
    let _ = writeln!(
        uart,
        "  io              0x{PCIE2X1_IO_BASE:016x}..0x{:016x}",
        PCIE2X1_IO_BASE + 0x0010_0000
    );
    let _ = writeln!(
        uart,
        "  mem32           0x{PCIE2X1_MEM32_BASE:016x}..0x{:016x}",
        PCIE2X1_MEM32_BASE + 0x01e0_0000
    );
    let _ = writeln!(
        uart,
        "  mem64           0x{PCIE2X1_MEM64_BASE:016x}..0x{:016x}",
        PCIE2X1_MEM64_BASE + 0x4000_0000
    );
    uart.put_str(
        "  safe rule       do not read DBI/config until pcie2 clock/reset is implemented\n",
    );
}

fn print_wifi_cru_status(uart: &mut Uart) {
    uart.put_str("wifi PCIe2 CRU status:\n");
    uart.put_str("  clocks          gate bit set means disabled\n");
    for gate in PCIE2_CLOCK_GATES {
        print_cru_bit(uart, *gate, false);
    }
    uart.put_str("  resets          reset bit set means asserted\n");
    for reset in PCIE2_RESETS {
        print_cru_bit(uart, *reset, true);
    }
    uart.put_str("  reset-gpio      pcie2x1 M.2 E-Key module reset line\n");
    print_pcie_reset_pin(uart, &PCIE_RESET_PINS[2]);
    uart.put_str("  safe rule       read-only status; no pcie2 clock/reset writes were made\n");
}

fn print_wifi_apb_status(uart: &mut Uart) {
    uart.put_str("wifi PCIe2 APB status:\n");
    uart.put_str("  safe rule       read-only APB status; no DBI/config/window reads\n");
    print_pcie_controller_apb_status(uart, PCIE2_CONTROLLER);
}

fn print_wifi_phy_status(uart: &mut Uart) {
    uart.put_str("wifi combphy2 status:\n");
    uart.put_str("  node            /phy@fe840000 rockchip,rk3568-naneng-combphy\n");
    uart.put_str("  safe rule       read-only combphy2/GRF/CRU status; no writes\n");
    uart.put_str("  clocks          gate bit set means disabled\n");
    for gate in PCIEPHY2_REF_CLOCK_GATES {
        print_cru_bit(uart, *gate, false);
    }
    for gate in COMBPHY2_CLOCK_GATES {
        print_cru_bit(uart, *gate, false);
    }
    print_pcie_reg(
        uart,
        "  pmucru.clksel9",
        PMUCRU_BASE + RK3568_PMU_CLKSEL_CON9,
        probe_read32(PMUCRU_BASE + RK3568_PMU_CLKSEL_CON9),
    );
    uart.put_str("  resets          reset bit set means asserted\n");
    for reset in COMBPHY2_RESETS {
        print_cru_bit(uart, *reset, true);
    }
    print_pcie_reg(
        uart,
        "  pipe-phy-grf.con0",
        PIPE_PHY_GRF_BASE,
        probe_read32(PIPE_PHY_GRF_BASE),
    );
    print_pcie_reg(
        uart,
        "  pipe-phy-grf.con1",
        PIPE_PHY_GRF_BASE + 0x04,
        probe_read32(PIPE_PHY_GRF_BASE + 0x04),
    );
    print_pcie_reg(
        uart,
        "  pipe-phy-grf.con2",
        PIPE_PHY_GRF_BASE + 0x08,
        probe_read32(PIPE_PHY_GRF_BASE + 0x08),
    );
    print_pcie_reg(
        uart,
        "  pipe-phy-grf.con3",
        PIPE_PHY_GRF_BASE + 0x0c,
        probe_read32(PIPE_PHY_GRF_BASE + 0x0c),
    );
    let status = probe_read32(PIPE_PHY_GRF_BASE + RK3568_PIPE_PHY_GRF_STATUS);
    print_pcie_reg(
        uart,
        "  pipe-phy-grf.status",
        PIPE_PHY_GRF_BASE + RK3568_PIPE_PHY_GRF_STATUS,
        status,
    );
    if let Ok(status) = status {
        let _ = writeln!(
            uart,
            "  ready-bit       {}",
            enabled_name((status & RK3568_PIPE_PHY_READY_BIT) == RK3568_PIPE_PHY_READY_VALUE)
        );
    }
    print_pcie_reg(
        uart,
        "  pipe-grf.con0",
        PIPE_GRF_BASE,
        probe_read32(PIPE_GRF_BASE),
    );
    print_combphy_reg(uart, "  phy.reg6", RK3568_COMBPHY_REG6);
    print_combphy_reg(uart, "  phy.reg7", RK3568_COMBPHY_REG7);
    print_combphy_reg(uart, "  phy.reg8", RK3568_COMBPHY_REG8);
    print_combphy_reg(uart, "  phy.reg11", RK3568_COMBPHY_REG11);
    print_combphy_reg(uart, "  phy.reg12", RK3568_COMBPHY_REG12);
    print_combphy_reg(uart, "  phy.reg13", RK3568_COMBPHY_REG13);
    print_combphy_reg(uart, "  phy.reg14", RK3568_COMBPHY_REG14);
    print_combphy_reg(uart, "  phy.reg15", RK3568_COMBPHY_REG15);
    print_combphy_reg(uart, "  phy.reg16", RK3568_COMBPHY_REG16);
    print_combphy_reg(uart, "  phy.reg18", RK3568_COMBPHY_REG18);
    print_combphy_reg(uart, "  phy.reg30", RK3568_COMBPHY_REG30);
    print_combphy_reg(uart, "  phy.reg32", RK3568_COMBPHY_REG32);
    print_combphy_reg(uart, "  phy.reg33", RK3568_COMBPHY_REG33);
}

fn print_wifi_power_status(uart: &mut Uart) {
    uart.put_str("wifi PCIe power status:\n");
    uart.put_str("  regulator       vcc3v3_pcie fixed regulator\n");
    uart.put_str("  gpio            gpio0 PD4 active-high enable\n");
    let direction = read_gpio_v2_direction(GPIO0, PCIE_POWER_PIN);
    let level = read_gpio_v2_level(GPIO0, PCIE_POWER_PIN);
    match (direction, level) {
        (Ok(direction), Ok(level)) => {
            let _ = writeln!(
                uart,
                "  state           dir={} level={} power={}",
                gpio_direction_name(direction),
                bit_name(level),
                if level { "enabled" } else { "disabled" }
            );
        }
        (direction, level) => {
            print_probe_result(uart, "  direction", direction.map(bool_to_u32));
            print_probe_result(uart, "  level", level.map(bool_to_u32));
        }
    }
}

fn print_wifi_link_status(uart: &mut Uart) {
    uart.put_str("wifi PCIe2 link status:\n");
    uart.put_str("  mode            read-only; DBI reads only when clock/reset state is safe\n");
    print_wifi_power_status(uart);
    print_gpio_pin_status(
        uart,
        "  pinctrl.pcie20-reset",
        GPIO2,
        "PD2",
        PCIE20_PINCTRL_PIN,
    );
    print_wifi_cru_status(uart);
    print_wifi_phy_status(uart);
    print_wifi_apb_status(uart);

    if wifi_pcie2_dbi_safe() {
        uart.put_str("  dbi             safe-to-read yes\n");
        print_pcie_reg(
            uart,
            "  dbi.vendor",
            PCIE2X1_DBI_BASE,
            probe_read32(PCIE2X1_DBI_BASE),
        );
        print_pcie_reg(
            uart,
            "  dbi.command",
            PCIE2X1_DBI_BASE + 0x04,
            probe_read32(PCIE2X1_DBI_BASE + 0x04),
        );
        print_pcie_reg(
            uart,
            "  dbi.class",
            PCIE2X1_DBI_BASE + 0x08,
            probe_read32(PCIE2X1_DBI_BASE + 0x08),
        );
        print_pcie_reg(
            uart,
            "  dbi.atu-view",
            PCIE2X1_DBI_BASE + PCIE_ATU_VIEWPORT,
            probe_read32(PCIE2X1_DBI_BASE + PCIE_ATU_VIEWPORT),
        );
    } else {
        uart.put_str("  dbi             safe-to-read no; skipped to avoid bus stall\n");
    }

    let ltssm = probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_LTSSM_STATUS).unwrap_or(0);
    let link_up = ltssm & PCIE_CLIENT_LINKUP_MASK == PCIE_CLIENT_LINKUP;
    if link_up {
        pcie_program_cfg_atu(
            PCIE2_CONTROLLER,
            PCIE2_CONTROLLER.root_bus.wrapping_add(1),
            0,
            0,
            PCIE_ATU_TYPE_CFG0,
        );
        delay_ms(1);
        print_pcie_reg(
            uart,
            "  cfg.vendor",
            PCIE2X1_CFG_BASE,
            probe_read32(PCIE2X1_CFG_BASE),
        );
        print_pcie_reg(
            uart,
            "  cfg.command",
            PCIE2X1_CFG_BASE + 0x04,
            probe_read32(PCIE2X1_CFG_BASE + 0x04),
        );
        print_pcie_reg(
            uart,
            "  cfg.class",
            PCIE2X1_CFG_BASE + 0x08,
            probe_read32(PCIE2X1_CFG_BASE + 0x08),
        );
    } else {
        uart.put_str("  cfg             skipped; PCIe2 link is not up\n");
    }
}

fn command_wifi_pcie_init(uart: &mut Uart, confirm: Option<&str>) {
    if !matches!(confirm, Some(value) if value.eq_ignore_ascii_case("confirm")) {
        uart.put_str("usage: wifi pcie-init confirm\n");
        uart.put_str(
            "scope: writes only PCIe2/combphy2 WiFi path; wired PCIe3 rescue remains untouched\n",
        );
        return;
    }

    uart.put_str("wifi PCIe2 init:\n");
    uart.put_str("  reset-policy    hold-high release-low\n");
    let result = wifi_pcie2_init_sequence();
    print_wifi_pcie_init_result(uart, result);
}

fn command_wifi_pcie_init_high(uart: &mut Uart, confirm: Option<&str>) {
    if !matches!(confirm, Some(value) if value.eq_ignore_ascii_case("confirm")) {
        uart.put_str("usage: wifi pcie-init-high confirm\n");
        uart.put_str(
            "scope: writes only PCIe2/combphy2 WiFi path; reset policy is hold-low release-high\n",
        );
        return;
    }

    uart.put_str("wifi PCIe2 init-high:\n");
    uart.put_str("  reset-policy    hold-low release-high\n");
    let result = wifi_pcie2_init_sequence_release_high();
    print_wifi_pcie_init_result(uart, result);
}

fn command_wifi_pcie_init_linux(uart: &mut Uart, confirm: Option<&str>) {
    if !matches!(confirm, Some(value) if value.eq_ignore_ascii_case("confirm")) {
        uart.put_str("usage: wifi pcie-init-linux confirm\n");
        uart.put_str(
            "scope: writes only PCIe2/combphy2 WiFi path using upstream Linux probe/start-link order\n",
        );
        return;
    }

    uart.put_str("wifi PCIe2 init-linux:\n");
    uart.put_str("  reset-policy    upstream-order pipe-reset then hold-low release-high\n");
    let result = wifi_pcie2_init_sequence_linux_order();
    print_wifi_pcie_init_result(uart, result);
}

fn command_wifi_trace(uart: &mut Uart, confirm: Option<&str>) {
    if !matches!(confirm, Some(value) if value.eq_ignore_ascii_case("confirm")) {
        uart.put_str("usage: wifi trace confirm\n");
        uart.put_str(
            "scope: retrains only PCIe2/combphy2 WiFi path and samples Rockchip LTSSM debug FIFO\n",
        );
        return;
    }

    uart.put_str("wifi PCIe2 trace:\n");
    uart.put_str("  reset-policy    upstream-order with LTSSM debug FIFO enabled\n");
    let result = wifi_pcie2_trace_sequence();
    print_wifi_pcie_init_result(uart, result.init);
    let _ = writeln!(uart, "  rasdes          0x{:08x}", result.rasdes);
    let _ = writeln!(uart, "  intr-misc       0x{:08x}", result.intr_misc);
    let _ = writeln!(uart, "  intr-mask-misc  0x{:08x}", result.intr_mask_misc);
    let _ = writeln!(uart, "  dbg-mode        0x{:08x}", result.dbg_mode);
    for (index, sample) in result.samples.iter().copied().enumerate() {
        let state = sample & PCIE_CLIENT_LTSSM_STATUS_MASK;
        let rate = (sample & PCIE_DBG_FIFO_RATE_MASK) >> PCIE_DBG_FIFO_RATE_SHIFT;
        let l1sub = (sample & PCIE_DBG_FIFO_L1SUB_MASK) >> PCIE_DBG_FIFO_L1SUB_SHIFT;
        let _ = writeln!(
            uart,
            "  sample[{index}]      raw=0x{sample:08x} state=0x{state:02x} state-name={} rate={} l1sub={}",
            pcie_ltssm_state_name(state),
            rate,
            l1sub
        );
    }
}

fn print_wifi_pcie_init_result(uart: &mut Uart, result: WifiPcieInitResult) {
    let _ = writeln!(
        uart,
        "  result          link={} ltssm=0x{:08x} cfg0={}",
        if result.link_up { "up" } else { "down" },
        result.ltssm,
        match result.cfg0 {
            Some(value) => {
                if value == 0x0000_0000 || value == 0xffff_ffff {
                    "empty"
                } else {
                    "device"
                }
            }
            None => "skipped",
        }
    );
    print_wifi_cru_status(uart);
    print_wifi_phy_status(uart);
    print_wifi_apb_status(uart);
    if let Some(cfg0) = result.cfg0 {
        let _ = writeln!(uart, "  cfg0            0x{cfg0:08x}");
        if cfg0 != 0 && cfg0 != 0xffff_ffff {
            let command = probe_read32(PCIE2X1_CFG_BASE + 0x04).unwrap_or(0);
            let class = probe_read32(PCIE2X1_CFG_BASE + 0x08).unwrap_or(0);
            let _ = writeln!(uart, "  cfg-command     0x{command:08x}");
            let _ = writeln!(uart, "  cfg-class       0x{class:08x}");
        }
    }
}

fn command_wifi_sideband(uart: &mut Uart, mode: Option<&str>, confirm: Option<&str>) {
    if !matches!(confirm, Some(value) if value.eq_ignore_ascii_case("confirm")) {
        uart.put_str("usage: wifi sideband <in|low|high> confirm\n");
        uart.put_str("scope: writes only DTB pcie20-reset-pin GPIO2 PD2 sideband\n");
        return;
    }

    match mode {
        Some(value) if value.eq_ignore_ascii_case("in") => {
            gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DDR, PCIE20_PINCTRL_PIN, false);
            uart.put_str("wifi sideband: GPIO2 PD2 set to input\n");
        }
        Some(value) if value.eq_ignore_ascii_case("low") => {
            gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DR, PCIE20_PINCTRL_PIN, false);
            gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DDR, PCIE20_PINCTRL_PIN, true);
            uart.put_str("wifi sideband: GPIO2 PD2 driven low\n");
        }
        Some(value) if value.eq_ignore_ascii_case("high") => {
            gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DR, PCIE20_PINCTRL_PIN, true);
            gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DDR, PCIE20_PINCTRL_PIN, true);
            uart.put_str("wifi sideband: GPIO2 PD2 driven high\n");
        }
        _ => {
            uart.put_str("usage: wifi sideband <in|low|high> confirm\n");
        }
    }
    print_gpio_pin_status(
        uart,
        "  pinctrl.pcie20-reset",
        GPIO2,
        "PD2",
        PCIE20_PINCTRL_PIN,
    );
}

fn command_wifi_scan(uart: &mut Uart, confirm: Option<&str>) {
    if !matches!(confirm, Some(value) if value.eq_ignore_ascii_case("confirm")) {
        uart.put_str("usage: wifi scan confirm\n");
        return;
    }
    uart.put_str("wifi scan: not implemented; run `wifi roadmap` for driver prerequisites\n");
}

fn command_wifi_ssid_not_ready(
    uart: &mut Uart,
    feature: &str,
    ssid: Option<&str>,
    confirm: Option<&str>,
) {
    if ssid.is_none() || !matches!(confirm, Some(value) if value.eq_ignore_ascii_case("confirm")) {
        let _ = writeln!(uart, "usage: wifi {feature} <ssid> confirm");
        return;
    }
    let _ = writeln!(
        uart,
        "wifi {feature}: not implemented; run `wifi roadmap` for driver prerequisites"
    );
}

fn write_wifi_facts_summary<W: Write>(out: &mut W) {
    let _ = write!(
        out,
        "wifi chip=rtl8822ce bus=candidate-pcie2x1 slot=m2-e-key stage=facts-only station=planned ap=planned driver=not-loaded firmware=needed"
    );
}

fn write_wifi_status_summary<W: Write>(out: &mut W) {
    let _ = write!(
        out,
        "wifi status=facts-only link=down scan=no join=no ap=no reason=rtl8822ce-driver-not-implemented"
    );
}

fn write_wifi_pcie_summary<W: Write>(out: &mut W) {
    let _ = write!(
        out,
        "wifi-pcie node=/pcie@fe260000 compatible=rockchip,rk3568-pcie bus=0x00..0x0f lanes=1 reset=gpio3-PC1-active-high dbi=0x{PCIE2X1_DBI_BASE:016x} apb=0x{PCIE2X1_APB_BASE:016x} cfg=0x{PCIE2X1_CFG_BASE:016x} mem32=0x{PCIE2X1_MEM32_BASE:016x} mem64=0x{PCIE2X1_MEM64_BASE:016x} safe=no-dbi-read-before-clock-reset"
    );
}

fn write_wifi_cru_summary<W: Write>(out: &mut W) {
    let _ = write!(out, "wifi-cru clk-gate-bit");
    write_cru_bit_raw_summary(out, "mst", PCIE2_CLOCK_GATES[0]);
    write_cru_bit_raw_summary(out, "slv", PCIE2_CLOCK_GATES[1]);
    write_cru_bit_raw_summary(out, "dbi", PCIE2_CLOCK_GATES[2]);
    write_cru_bit_raw_summary(out, "pclk", PCIE2_CLOCK_GATES[3]);
    write_cru_bit_raw_summary(out, "aux", PCIE2_CLOCK_GATES[4]);
    write_cru_bit_raw_summary(out, "pipe", PCIE2_CLOCK_GATES[5]);
    let _ = write!(out, " rst-bit");
    write_cru_bit_raw_summary(out, "p", PCIE2_RESETS[0]);
    write_cru_bit_raw_summary(out, "pwrup", PCIE2_RESETS[1]);
    write_cru_bit_raw_summary(out, "mstr", PCIE2_RESETS[2]);
    write_cru_bit_raw_summary(out, "slv", PCIE2_RESETS[3]);
    write_cru_bit_raw_summary(out, "dbi", PCIE2_RESETS[4]);
    write_cru_bit_raw_summary(out, "b", PCIE2_RESETS[5]);
    write_cru_bit_raw_summary(out, "perst", PCIE2_RESETS[6]);
    write_cru_bit_raw_summary(out, "core", PCIE2_RESETS[7]);
    write_cru_bit_raw_summary(out, "nsticky", PCIE2_RESETS[8]);
    write_cru_bit_raw_summary(out, "sticky", PCIE2_RESETS[9]);
    write_cru_bit_raw_summary(out, "pwr", PCIE2_RESETS[10]);
    write_pcie_reset_pin_summary(out, &PCIE_RESET_PINS[2]);
    let _ = write!(out, " mode=read-only");
}

fn write_wifi_apb_summary<W: Write>(out: &mut W) {
    let controller = PCIE2_CONTROLLER;
    let _ = write!(
        out,
        "wifi-apb name={} apb=0x{:016x}",
        controller.name, controller.apb_base
    );
    write_probe32_summary(
        out,
        "general",
        controller.apb_base + PCIE_CLIENT_GENERAL_CON,
    );
    write_probe32_summary(
        out,
        "misc",
        controller.apb_base + PCIE_CLIENT_INTR_STATUS_MISC,
    );
    write_probe32_summary(out, "power", controller.apb_base + PCIE_CLIENT_POWER_CON);
    write_probe32_summary(
        out,
        "hot-reset",
        controller.apb_base + PCIE_CLIENT_HOT_RESET_CTRL,
    );
    write_probe32_summary(out, "ltssm", controller.apb_base + PCIE_CLIENT_LTSSM_STATUS);
    let _ = write!(out, " mode=read-only");
}

fn write_wifi_phy_summary<W: Write>(out: &mut W) {
    let _ = write!(out, "wifi-phy base=0x{COMBPHY2_BASE:016x}");
    write_cru_bit_raw_summary(out, "ref-div-gate", PCIEPHY2_REF_CLOCK_GATES[0]);
    write_cru_bit_raw_summary(out, "ref-osc-gate", PCIEPHY2_REF_CLOCK_GATES[1]);
    match probe_read32(PMUCRU_BASE + RK3568_PMU_CLKSEL_CON9) {
        Ok(value) => {
            let ref_div = (value >> 8) & 0x7;
            let ref_mux = (value >> 11) & 0x1;
            let _ = write!(out, " ref-div={ref_div} ref-mux={ref_mux}");
        }
        Err(_) => {
            let _ = write!(out, " ref-div=fault ref-mux=fault");
        }
    }
    write_cru_bit_raw_summary(out, "pclk-gate", COMBPHY2_CLOCK_GATES[0]);
    write_cru_bit_raw_summary(out, "rst-p", COMBPHY2_RESETS[0]);
    write_cru_bit_raw_summary(out, "rst-phy", COMBPHY2_RESETS[1]);
    write_probe32_summary(out, "grf0", PIPE_PHY_GRF_BASE);
    write_probe32_summary(out, "grf1", PIPE_PHY_GRF_BASE + 0x04);
    write_probe32_summary(out, "grf2", PIPE_PHY_GRF_BASE + 0x08);
    write_probe32_summary(out, "grf3", PIPE_PHY_GRF_BASE + 0x0c);
    match probe_read32(PIPE_PHY_GRF_BASE + RK3568_PIPE_PHY_GRF_STATUS) {
        Ok(status) => {
            let ready = if (status & RK3568_PIPE_PHY_READY_BIT) == RK3568_PIPE_PHY_READY_VALUE {
                1
            } else {
                0
            };
            let _ = write!(out, " status=0x{status:08x} ready={ready}");
        }
        Err(_) => {
            let _ = write!(out, " status=fault ready=fault");
        }
    }
    write_probe32_summary(out, "phy6", COMBPHY2_BASE + RK3568_COMBPHY_REG6);
    write_probe32_summary(out, "phy32", COMBPHY2_BASE + RK3568_COMBPHY_REG32);
    write_probe32_summary(out, "phy33", COMBPHY2_BASE + RK3568_COMBPHY_REG33);
    let _ = write!(out, " mode=read-only");
}

fn write_wifi_power_summary<W: Write>(out: &mut W) {
    let _ = write!(
        out,
        "wifi-power regulator=vcc3v3_pcie gpio=gpio0-PD4 active=high"
    );
    match (
        read_gpio_v2_direction(GPIO0, PCIE_POWER_PIN),
        read_gpio_v2_level(GPIO0, PCIE_POWER_PIN),
    ) {
        (Ok(direction), Ok(level)) => {
            let _ = write!(
                out,
                " dir={} level={} power={}",
                gpio_direction_name(direction),
                bit_name(level),
                if level { "enabled" } else { "disabled" }
            );
        }
        _ => {
            let _ = write!(out, " status=fault");
        }
    }
}

fn write_wifi_link_summary<W: Write>(out: &mut W) {
    let ltssm = probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_LTSSM_STATUS).unwrap_or(0);
    let state = ltssm & PCIE_CLIENT_LTSSM_STATUS_MASK;
    let link_up = ltssm & PCIE_CLIENT_LINKUP_MASK == PCIE_CLIENT_LINKUP;
    let dbi_safe = wifi_pcie2_dbi_safe();
    let _ = write!(
        out,
        "wifi-link link={} ltssm=0x{ltssm:08x} state=0x{state:02x} state-name={} dbi-safe={}",
        if link_up { "up" } else { "down" },
        pcie_ltssm_state_name(state),
        if dbi_safe { "yes" } else { "no" }
    );
    write_probe32_summary(out, "general", PCIE2X1_APB_BASE + PCIE_CLIENT_GENERAL_CON);
    write_probe32_summary(out, "misc", PCIE2X1_APB_BASE + PCIE_CLIENT_INTR_STATUS_MISC);
    write_probe32_summary(out, "power", PCIE2X1_APB_BASE + PCIE_CLIENT_POWER_CON);
    write_probe32_summary(
        out,
        "hot-reset",
        PCIE2X1_APB_BASE + PCIE_CLIENT_HOT_RESET_CTRL,
    );
    write_gpio_pin_summary(out, "pinctrl-pd2", GPIO2, PCIE20_PINCTRL_PIN);
    if dbi_safe {
        write_probe32_summary(out, "dbi0", PCIE2X1_DBI_BASE);
        write_probe32_summary(out, "dbi4", PCIE2X1_DBI_BASE + 0x04);
        write_probe32_summary(out, "dbi8", PCIE2X1_DBI_BASE + 0x08);
        write_probe32_summary(out, "atu-view", PCIE2X1_DBI_BASE + PCIE_ATU_VIEWPORT);
    } else {
        let _ = write!(
            out,
            " dbi0=skipped dbi4=skipped dbi8=skipped atu-view=skipped"
        );
    }
    if link_up {
        pcie_program_cfg_atu(
            PCIE2_CONTROLLER,
            PCIE2_CONTROLLER.root_bus.wrapping_add(1),
            0,
            0,
            PCIE_ATU_TYPE_CFG0,
        );
        delay_ms(1);
        write_probe32_summary(out, "cfg0", PCIE2X1_CFG_BASE);
        write_probe32_summary(out, "cfg4", PCIE2X1_CFG_BASE + 0x04);
        write_probe32_summary(out, "cfg8", PCIE2X1_CFG_BASE + 0x08);
    } else {
        let _ = write!(out, " cfg0=skipped cfg4=skipped cfg8=skipped");
    }
}

fn write_wifi_sideband_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _wifi = parts.next();
    let _sideband = parts.next();
    let mode = parts.next().unwrap_or("");
    let confirm = parts.next().unwrap_or("");

    if !confirm.eq_ignore_ascii_case("confirm") {
        let _ = write!(
            out,
            "wifi-sideband error=missing-confirm usage=wifi-sideband-<in|low|high>-confirm"
        );
        return;
    }

    if mode.eq_ignore_ascii_case("in") {
        gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DDR, PCIE20_PINCTRL_PIN, false);
        let _ = write!(out, "wifi-sideband mode=in");
    } else if mode.eq_ignore_ascii_case("low") {
        gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DR, PCIE20_PINCTRL_PIN, false);
        gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DDR, PCIE20_PINCTRL_PIN, true);
        let _ = write!(out, "wifi-sideband mode=low");
    } else if mode.eq_ignore_ascii_case("high") {
        gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DR, PCIE20_PINCTRL_PIN, true);
        gpio_v2_write_bit(GPIO2, GPIO_V2_PORT_DDR, PCIE20_PINCTRL_PIN, true);
        let _ = write!(out, "wifi-sideband mode=high");
    } else {
        let _ = write!(
            out,
            "wifi-sideband error=bad-mode usage=wifi-sideband-<in|low|high>-confirm"
        );
        return;
    }

    write_gpio_pin_summary(out, "pinctrl-pd2", GPIO2, PCIE20_PINCTRL_PIN);
}

fn write_wifi_pcie_init_summary<W: Write>(out: &mut W) {
    let _ = write!(out, "reset-policy=hold-high-release-low ");
    let result = wifi_pcie2_init_sequence();
    write_wifi_pcie_init_result_summary(out, result);
}

fn write_wifi_pcie_init_high_summary<W: Write>(out: &mut W) {
    let _ = write!(out, "reset-policy=hold-low-release-high ");
    let result = wifi_pcie2_init_sequence_release_high();
    write_wifi_pcie_init_result_summary(out, result);
}

fn write_wifi_pcie_init_linux_summary<W: Write>(out: &mut W) {
    let _ = write!(out, "reset-policy=linux-order-hold-low-release-high ");
    let result = wifi_pcie2_init_sequence_linux_order();
    write_wifi_pcie_init_result_summary(out, result);
}

fn write_wifi_trace_summary<W: Write>(out: &mut W) {
    let result = wifi_pcie2_trace_sequence();
    let _ = write!(out, "wifi-trace ");
    write_wifi_pcie_init_result_summary(out, result.init);
    let _ = write!(
        out,
        " rasdes=0x{:08x} intr-misc=0x{:08x} intr-mask-misc=0x{:08x} dbg-mode=0x{:08x}",
        result.rasdes, result.intr_misc, result.intr_mask_misc, result.dbg_mode
    );
    for (index, sample) in result.samples.iter().copied().enumerate() {
        let state = sample & PCIE_CLIENT_LTSSM_STATUS_MASK;
        let rate = (sample & PCIE_DBG_FIFO_RATE_MASK) >> PCIE_DBG_FIFO_RATE_SHIFT;
        let l1sub = (sample & PCIE_DBG_FIFO_L1SUB_MASK) >> PCIE_DBG_FIFO_L1SUB_SHIFT;
        let _ = write!(
            out,
            " s{index}=0x{sample:08x}/0x{state:02x}/{}/{rate}/{l1sub}",
            pcie_ltssm_state_name(state)
        );
    }
}

fn write_wifi_pcie_init_result_summary<W: Write>(out: &mut W, result: WifiPcieInitResult) {
    let _ = write!(
        out,
        "wifi-pcie-init link={} ltssm=0x{:08x}",
        if result.link_up { "up" } else { "down" },
        result.ltssm
    );
    match result.cfg0 {
        Some(cfg0) => {
            let _ = write!(out, " cfg0=0x{cfg0:08x}");
            if cfg0 != 0 && cfg0 != 0xffff_ffff {
                let vendor = cfg0 & 0xffff;
                let device = cfg0 >> 16;
                let _ = write!(out, " vendor=0x{vendor:04x} device=0x{device:04x}");
            }
        }
        None => {
            let _ = write!(out, " cfg0=skipped");
        }
    }
    write_cru_bit_raw_summary(out, "pwrup-rst", PCIE2_RESETS[1]);
    write_cru_bit_raw_summary(out, "phy-rst", COMBPHY2_RESETS[1]);
    write_probe32_summary(
        out,
        "phy-status",
        PIPE_PHY_GRF_BASE + RK3568_PIPE_PHY_GRF_STATUS,
    );
}

fn command_net<'a>(uart: &mut Uart, parts: &mut core::str::SplitWhitespace<'a>) {
    let action = parts.next();
    match action {
        None | Some("facts") => print_net_facts(uart),
        Some("ips") => print_net_ips(uart),
        Some("ip") => command_net_ip(uart, parts.next(), parts.next(), parts.next()),
        Some("dhcp") => command_net_dhcp(uart, parts.next(), parts.next()),
        Some("dns") => command_net_dns(uart, parts.next(), parts.next(), parts.next()),
        Some("tcp") => {
            command_net_tcp(uart, parts.next(), parts.next(), parts.next(), parts.next())
        }
        Some("http") => command_net_http(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("http-post") => command_net_http_post(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("loop") => command_net_loop(uart, parts.next()),
        Some("listen") => command_net_listen(uart, parts.next()),
        Some("sniff") => command_net_sniff(uart, parts.next()),
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  net facts\n");
            uart.put_str("  net ips\n");
            uart.put_str("  net ip <pcie3x1|pcie3x2|0|1> <a.b.c.d> confirm\n");
            uart.put_str("  net dhcp <pcie3x1|pcie3x2|0|1> confirm\n");
            uart.put_str("  net dns <pcie3x1|pcie3x2|0|1> <name> confirm\n");
            uart.put_str("  net tcp <pcie3x1|pcie3x2|0|1> <host|ip> <port> confirm\n");
            uart.put_str("  net http <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
            uart.put_str(
                "  net http-post <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> <body> confirm\n",
            );
            uart.put_str("  net loop confirm\n");
            uart.put_str("  net listen confirm\n");
            uart.put_str("  net sniff confirm\n");
        }
    }
}

fn command_maint<'a>(uart: &mut Uart, parts: &mut core::str::SplitWhitespace<'a>) {
    let action = parts.next();
    match action {
        Some("poll") => command_maint_poll(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("status") => command_maint_status(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("audit") => command_maint_audit(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("verify") => command_maint_verify(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("verify-hmac") => command_maint_verify_hmac(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("verify-local") => command_maint_verify_local(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("pull-exec") => command_maint_pull_exec(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("key-status") => command_maint_key_status(uart, parts.next()),
        Some("key-set") => command_maint_key_set(uart, parts.next(), parts.next()),
        Some("key-clear") => command_maint_key_clear(uart, parts.next()),
        Some("ack-intent") => command_maint_ack_intent(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        Some("event") => command_maint_event(
            uart,
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ),
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  maint poll <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
            uart.put_str("  maint status <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
            uart.put_str("  maint audit <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
            uart.put_str("  maint verify <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
            uart.put_str("  maint verify-hmac <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> <key-hex> confirm\n");
            uart.put_str(
                "  maint verify-local <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n",
            );
            uart.put_str(
                "  maint pull-exec <pcie3x1|pcie3x2|0|1> <host|ip> <port> <poll-path> [event-path] confirm\n",
            );
            uart.put_str("  maint key-status confirm\n");
            uart.put_str("  maint key-set <key-hex> confirm\n");
            uart.put_str("  maint key-clear confirm\n");
            uart.put_str(
                "  maint ack-intent <pcie3x1|pcie3x2|0|1> <host|ip> <port> <poll-path> <event-path> confirm\n",
            );
            uart.put_str(
                "  maint event <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> <kind> confirm\n",
            );
        }
    }
}

fn print_net_ips(uart: &mut Uart) {
    uart.put_str("runtime network config:\n");
    for (index, controller) in PCIE_CONTROLLERS.iter().enumerate() {
        let ip = net_device_ip(index);
        let netmask = net_device_netmask(index);
        let gateway = net_device_gateway(index);
        let dns = net_device_dns(index);
        let _ = writeln!(
            uart,
            "  {} ip={}.{}.{}.{} mask={}.{}.{}.{} gateway={}.{}.{}.{} dns={}.{}.{}.{}",
            controller.name,
            ip[0],
            ip[1],
            ip[2],
            ip[3],
            netmask[0],
            netmask[1],
            netmask[2],
            netmask[3],
            gateway[0],
            gateway[1],
            gateway[2],
            gateway[3],
            dns[0],
            dns[1],
            dns[2],
            dns[3]
        );
    }
}

fn write_net_ips_summary<W: Write>(out: &mut W) {
    let _ = write!(out, "net-ips");
    for (index, controller) in PCIE_CONTROLLERS.iter().enumerate() {
        let ip = net_device_ip(index);
        let netmask = net_device_netmask(index);
        let gateway = net_device_gateway(index);
        let dns = net_device_dns(index);
        let _ = write!(
            out,
            " {}={}.{}.{}.{} mask={}.{}.{}.{} gateway={}.{}.{}.{} dns={}.{}.{}.{}",
            controller.name,
            ip[0],
            ip[1],
            ip[2],
            ip[3],
            netmask[0],
            netmask[1],
            netmask[2],
            netmask[3],
            gateway[0],
            gateway[1],
            gateway[2],
            gateway[3],
            dns[0],
            dns[1],
            dns[2],
            dns[3]
        );
    }
}

fn write_net_dns_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _net = parts.next();
    let _dns = parts.next();
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR dns-usage");
        return;
    };
    let (index, name) = if let Some(index) = parse_net_interface(first) {
        let Some(name) = parts.next() else {
            let _ = write!(out, "XJQ ERR dns-usage");
            return;
        };
        (index, name)
    } else {
        (default_index, first)
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR dns-usage");
        return;
    }

    match net_dns_resolve_a(index, name, DNS_TIMEOUT_MS) {
        Ok(result) => {
            let _ = write!(
                out,
                "XJQ OK dns iface={} name={} ip={}.{}.{}.{} server={}.{}.{}.{} next-hop={}.{}.{}.{} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} txid=0x{:04x}",
                PCIE_CONTROLLERS[index].name,
                name,
                result.answer[0],
                result.answer[1],
                result.answer[2],
                result.answer[3],
                result.server[0],
                result.server[1],
                result.server[2],
                result.server[3],
                result.next_hop[0],
                result.next_hop[1],
                result.next_hop[2],
                result.next_hop[3],
                result.next_hop_mac[0],
                result.next_hop_mac[1],
                result.next_hop_mac[2],
                result.next_hop_mac[3],
                result.next_hop_mac[4],
                result.next_hop_mac[5],
                result.txid
            );
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR dns {}", error.as_str());
        }
    }
}

fn write_net_tcp_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _net = parts.next();
    let _tcp = parts.next();
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR tcp-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR tcp-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port) = parts.next().and_then(parse_u16) else {
        let _ = write!(out, "XJQ ERR tcp-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR tcp-usage");
        return;
    }

    match net_tcp_connect(index, host, port, TCP_CONNECT_TIMEOUT_MS) {
        Ok(result) => {
            let _ = write!(
                out,
                "XJQ OK tcp iface={} host={} ip={}.{}.{}.{} port={} src-port={} next-hop={}.{}.{}.{} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} seq=0x{:08x} ack=0x{:08x} flags=0x{:03x} dns-txid=0x{:04x}",
                PCIE_CONTROLLERS[index].name,
                host,
                result.dst_ip[0],
                result.dst_ip[1],
                result.dst_ip[2],
                result.dst_ip[3],
                result.dst_port,
                result.src_port,
                result.next_hop[0],
                result.next_hop[1],
                result.next_hop[2],
                result.next_hop[3],
                result.next_hop_mac[0],
                result.next_hop_mac[1],
                result.next_hop_mac[2],
                result.next_hop_mac[3],
                result.next_hop_mac[4],
                result.next_hop_mac[5],
                result.seq,
                result.ack,
                result.flags,
                result.dns_txid
            );
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR tcp {}", error.as_str());
        }
    }
}

fn write_net_http_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _net = parts.next();
    let _http = parts.next();
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR http-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port) = parts.next().and_then(parse_u16) else {
        let _ = write!(out, "XJQ ERR http-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR http-usage");
        return;
    }

    match net_http_get(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let _ = write!(
                out,
                "XJQ OK http iface={} host={} ip={}.{}.{}.{} port={} status={} bytes={} payload={} segments={} fin={} truncated={} src-port={} first-line=\"",
                PCIE_CONTROLLERS[index].name,
                host,
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.status_code,
                result.response_bytes,
                result.payload_bytes,
                result.segments,
                yes_no(result.fin),
                yes_no(result.truncated),
                result.tcp.src_port
            );
            write_http_sample_line(out, &result.sample[..result.sample_len]);
            let _ = write!(out, "\"");
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR http {}", error.as_str());
        }
    }
}

fn write_net_http_post_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _net = parts.next();
    let _http_post = parts.next();
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-post-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR http-post-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port) = parts.next().and_then(parse_u16) else {
        let _ = write!(out, "XJQ ERR http-post-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-post-usage");
        return;
    };
    let Some(body) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-post-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR http-post-usage");
        return;
    }

    match net_http_post(index, host, port, path, body.as_bytes(), HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let _ = write!(
                out,
                "XJQ OK http-post iface={} host={} ip={}.{}.{}.{} port={} status={} bytes={} payload={} segments={} fin={} truncated={} sent={} src-port={} first-line=\"",
                PCIE_CONTROLLERS[index].name,
                host,
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.status_code,
                result.response_bytes,
                result.payload_bytes,
                result.segments,
                yes_no(result.fin),
                yes_no(result.truncated),
                result.sent_payload_len,
                result.tcp.src_port
            );
            write_http_sample_line(out, &result.sample[..result.sample_len]);
            let _ = write!(out, "\"");
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR http-post {}", error.as_str());
        }
    }
}

fn write_net_http_trace_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _net = parts.next();
    let _http_trace = parts.next();
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-trace-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR http-trace-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port) = parts.next().and_then(parse_u16) else {
        let _ = write!(out, "XJQ ERR http-trace-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-trace-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR http-trace-usage");
        return;
    }

    match net_http_trace(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let _ = write!(
                out,
                "XJQ OK http-trace iface={} host={} ip={}.{}.{}.{} port={} bytes={} payload={} segments={} fin={} truncated={} src-port={} first-line=\"",
                PCIE_CONTROLLERS[index].name,
                host,
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.response_bytes,
                result.payload_bytes,
                result.segments,
                yes_no(result.fin),
                yes_no(result.truncated),
                result.tcp.src_port
            );
            write_http_sample_line(out, &result.sample[..result.sample_len]);
            let _ = write!(out, "\" raw=\"");
            write_http_raw_sample_line(out, &result.sample[..result.sample_len]);
            let _ = write!(out, "\"");
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR http-trace {}", error.as_str());
        }
    }
}

fn write_net_http_debug_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _net = parts.next();
    let _http_debug = parts.next();
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-debug-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR http-debug-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port) = parts.next().and_then(parse_u16) else {
        let _ = write!(out, "XJQ ERR http-debug-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR http-debug-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR http-debug-usage");
        return;
    }

    match net_http_debug(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let expected_ack = result.tcp.local_seq.wrapping_add(result.sent_payload_len);
            let _ = write!(
                out,
                "XJQ OK http-debug iface={} host={} ip={}.{}.{}.{} port={} src-port={} sent={} expected-ack=0x{expected_ack:08x} seg-seq=0x{:08x} seg-ack=0x{:08x} flags=0x{:03x} segment={} payload={} first-line=\"",
                PCIE_CONTROLLERS[index].name,
                host,
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.tcp.src_port,
                result.sent_payload_len,
                result.segment.seq,
                result.segment.ack,
                result.segment.flags,
                result.segment.segment_len,
                result.segment.payload_len
            );
            write_http_sample_line(out, &result.segment.sample[..result.segment.sample_len]);
            let _ = write!(out, "\" raw=\"");
            write_http_raw_sample_line(out, &result.segment.sample[..result.segment.sample_len]);
            let _ = write!(out, "\"");
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR http-debug {}", error.as_str());
        }
    }
}

fn write_maint_poll_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("poll") {
        let _ = write!(out, "XJQ ERR maint-poll-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-poll-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-poll-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-poll-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-poll-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR maint-poll-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-poll-port");
        return;
    };

    write_maint_poll_summary(out, index, host, port, path);
}

fn write_maint_status_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("status") {
        let _ = write!(out, "XJQ ERR maint-status-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-status-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-status-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-status-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-status-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR maint-status-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-status-port");
        return;
    };

    write_maint_status_summary(out, index, host, port, path);
}

fn write_maint_audit_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("audit") {
        let _ = write!(out, "XJQ ERR maint-audit-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-audit-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-audit-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-audit-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-audit-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR maint-audit-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-audit-port");
        return;
    };

    write_maint_audit_summary(out, index, host, port, path);
}

fn write_maint_verify_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("verify") {
        let _ = write!(out, "XJQ ERR maint-verify-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-verify-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR maint-verify-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-verify-port");
        return;
    };

    write_maint_verify_summary(out, index, host, port, path, MaintHmacKey::None);
}

fn write_maint_verify_hmac_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("verify-hmac") {
        let _ = write!(out, "XJQ ERR maint-verify-hmac-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-hmac-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-verify-hmac-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-hmac-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-hmac-usage");
        return;
    };
    let Some(key_hex) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-hmac-usage");
        return;
    };
    if parts.next() != Some("confirm") || parts.next().is_some() {
        let _ = write!(out, "XJQ ERR maint-verify-hmac-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-verify-hmac-port");
        return;
    };

    write_maint_verify_summary(out, index, host, port, path, MaintHmacKey::Hex(key_hex));
}

fn write_maint_verify_local_query_summary<W: Write>(
    default_index: usize,
    query: &str,
    out: &mut W,
) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("verify-local") {
        let _ = write!(out, "XJQ ERR maint-verify-local-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-local-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-verify-local-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-local-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-verify-local-usage");
        return;
    };
    if parts.next() != Some("confirm") || parts.next().is_some() {
        let _ = write!(out, "XJQ ERR maint-verify-local-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-verify-local-port");
        return;
    };
    let Ok(row) = get_maint_hmac_key_row() else {
        let _ = write!(out, "XJQ ERR maint-verify-local key-missing");
        return;
    };

    write_maint_verify_summary(
        out,
        index,
        host,
        port,
        path,
        MaintHmacKey::Raw(&row.value[..row.value_len]),
    );
}

fn write_maint_pull_exec_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("pull-exec") {
        let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
        return;
    };
    let next = parts.next();
    let event_path = match next {
        Some("confirm") => {
            if parts.next().is_some() {
                let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
                return;
            }
            None
        }
        Some(event_path) => {
            if parts.next() != Some("confirm") || parts.next().is_some() {
                let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
                return;
            }
            Some(event_path)
        }
        None => {
            let _ = write!(out, "XJQ ERR maint-pull-exec-usage");
            return;
        }
    };
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-pull-exec-port");
        return;
    };
    if let Some(event_path) = event_path {
        if !event_path.starts_with('/') {
            let _ = write!(out, "XJQ ERR maint-pull-exec-event-path");
            return;
        }
    }
    let Ok(row) = get_maint_hmac_key_row() else {
        let _ = write!(out, "XJQ ERR maint-pull-exec key-missing");
        return;
    };

    write_maint_pull_exec_summary(
        out,
        index,
        host,
        port,
        path,
        event_path,
        MaintHmacKey::Raw(&row.value[..row.value_len]),
    );
}

fn write_maint_key_status_summary<W: Write>(out: &mut W) {
    match get_maint_hmac_key_row() {
        Ok(row) => {
            let digest = sha256_digest(&row.value[..row.value_len]);
            let _ = write!(
                out,
                "XJQ OK maint-key provisioned=yes key={DB_TABLE_KEY_MAINT_HMAC} key-len={} key-sha256=",
                row.value_len
            );
            write_sha256_hex(out, &digest);
        }
        Err(DbError::KeyMissing) | Err(DbError::TableMissing) => {
            let _ = write!(
                out,
                "XJQ OK maint-key provisioned=no key={DB_TABLE_KEY_MAINT_HMAC} key-len=0 key-sha256=none"
            );
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-key-status {}", error.as_str());
        }
    }
}

fn write_maint_key_set_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("key-set") {
        let _ = write!(out, "XJQ ERR maint-key-set-usage");
        return;
    }
    let Some(key_hex) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-key-set-usage");
        return;
    };
    if parts.next() != Some("confirm") || parts.next().is_some() {
        let _ = write!(out, "XJQ ERR maint-key-set-usage");
        return;
    }
    write_maint_key_set_summary(out, key_hex);
}

fn write_maint_key_set_summary<W: Write>(out: &mut W, key_hex: &str) {
    let mut key = [0u8; DB_TABLE_VALUE_BYTES];
    let Some(key_len) = decode_hex_to_buf(key_hex.as_bytes(), &mut key) else {
        let _ = write!(out, "XJQ ERR maint-key-set bad-key");
        return;
    };
    if key_len == 0 {
        let _ = write!(out, "XJQ ERR maint-key-set bad-key");
        return;
    }
    match put_db_row(DB_TABLE_KEY_MAINT_HMAC, &key[..key_len]) {
        Ok((_info, _catalog, table, row)) => {
            let digest = sha256_digest(&row.value[..row.value_len]);
            let _ = write!(
                out,
                "XJQ OK maint-key-set provisioned=yes key={} key-len={} table-records={} key-sha256=",
                row.key, row.value_len, table.record_count
            );
            write_sha256_hex(out, &digest);
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-key-set {}", error.as_str());
        }
    }
}

fn write_maint_key_clear_summary<W: Write>(out: &mut W) {
    match delete_db_row(DB_TABLE_KEY_MAINT_HMAC) {
        Ok((_info, _catalog, table)) => {
            let _ = write!(
                out,
                "XJQ OK maint-key-clear provisioned=no key={DB_TABLE_KEY_MAINT_HMAC} table-records={} key-sha256=none",
                table.record_count
            );
        }
        Err(DbError::KeyMissing) | Err(DbError::TableMissing) => {
            let _ = write!(
                out,
                "XJQ OK maint-key-clear provisioned=no key={DB_TABLE_KEY_MAINT_HMAC} already-clear=yes key-sha256=none"
            );
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-key-clear {}", error.as_str());
        }
    }
}

fn write_maint_ack_intent_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("ack-intent") {
        let _ = write!(out, "XJQ ERR maint-ack-intent-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-ack-intent-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-ack-intent-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-ack-intent-usage");
        return;
    };
    let Some(poll_path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-ack-intent-usage");
        return;
    };
    let Some(event_path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-ack-intent-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR maint-ack-intent-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-ack-intent-port");
        return;
    };

    write_maint_ack_intent_summary(out, index, host, port, poll_path, event_path);
}

fn write_maint_event_query_summary<W: Write>(default_index: usize, query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _maint = parts.next();
    if parts.next() != Some("event") {
        let _ = write!(out, "XJQ ERR maint-event-usage");
        return;
    }
    let Some(first) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-event-usage");
        return;
    };
    let (index, host) = if let Some(index) = parse_net_interface(first) {
        let Some(host) = parts.next() else {
            let _ = write!(out, "XJQ ERR maint-event-usage");
            return;
        };
        (index, host)
    } else {
        (default_index, first)
    };
    let Some(port_text) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-event-usage");
        return;
    };
    let Some(path) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-event-usage");
        return;
    };
    let Some(kind) = parts.next() else {
        let _ = write!(out, "XJQ ERR maint-event-usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR maint-event-usage");
        return;
    }
    let Some(port) = parse_u16(port_text) else {
        let _ = write!(out, "XJQ ERR maint-event-port");
        return;
    };

    write_maint_event_summary(out, index, host, port, path, kind);
}

fn command_sd(uart: &mut Uart, action: Option<&str>, confirm: Option<&str>) {
    match action {
        None | Some("facts") => print_sd_facts(uart),
        Some("regs") => {
            if confirm != Some("confirm") {
                uart.put_str("usage: sd regs confirm\n");
                uart.put_str("scope: read-only DW-MMC register probe, no block I/O\n");
            } else {
                print_sd_regs(uart);
            }
        }
        Some("read0") => {
            if confirm != Some("confirm") {
                uart.put_str("usage: sd read0 confirm\n");
                uart.put_str("scope: read SD LBA0 into RAM and print a short MBR/BPB summary\n");
            } else {
                print_sd_lba0(uart);
            }
        }
        Some("bpb") => {
            if confirm != Some("confirm") {
                uart.put_str("usage: sd bpb confirm\n");
                uart.put_str("scope: read the first partition boot sector and print FAT32 facts\n");
            } else {
                print_sd_bpb(uart);
            }
        }
        Some("root") => {
            if confirm != Some("confirm") {
                uart.put_str("usage: sd root confirm\n");
                uart.put_str("scope: read one FAT32 root cluster and locate Xiaojin slot files\n");
            } else {
                print_sd_root(uart);
            }
        }
        Some("slots") => {
            if confirm != Some("confirm") {
                uart.put_str("usage: sd slots confirm\n");
                uart.put_str("scope: scan FAT chains for preallocated A/B slot files\n");
            } else {
                print_sd_slots(uart);
            }
        }
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  sd facts\n");
            uart.put_str("  sd regs confirm\n");
            uart.put_str("  sd read0 confirm\n");
            uart.put_str("  sd bpb confirm\n");
            uart.put_str("  sd root confirm\n");
            uart.put_str("  sd slots confirm\n");
        }
    }
}

fn command_db<'a>(uart: &mut Uart, parts: &mut core::str::SplitWhitespace<'a>) {
    let action = parts.next();
    match action {
        None | Some("facts") => print_db_facts(uart),
        Some("status") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db status confirm\n");
                uart.put_str("scope: read XiaojinDB page-file superblock, no writes\n");
            } else {
                print_db_status(uart);
            }
        }
        Some("init") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db init confirm\n");
                uart.put_str("scope: write XiaojinDB superblock into existing page file\n");
            } else {
                print_db_init(uart);
            }
        }
        Some("catalog") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db catalog confirm\n");
                uart.put_str("scope: read XiaojinDB catalog page, no writes\n");
            } else {
                print_db_catalog(uart);
            }
        }
        Some("catalog-init") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db catalog-init confirm\n");
                uart.put_str("scope: write XiaojinDB catalog page into page 1\n");
            } else {
                print_db_catalog_init(uart);
            }
        }
        Some("page") => {
            let page = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db page <n> confirm\n");
                uart.put_str(
                    "scope: read one XiaojinDB page and print a checksum/header summary\n",
                );
            } else {
                print_db_page(uart, page);
            }
        }
        Some("wal-append") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db wal-append confirm\n");
                uart.put_str("scope: write one test WAL page and advance catalog metadata\n");
            } else {
                print_db_wal_append(uart);
            }
        }
        Some("table") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db table confirm\n");
                uart.put_str("scope: read fixed key/value root table metadata\n");
            } else {
                print_db_table(uart);
            }
        }
        Some("table-init") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db table-init confirm\n");
                uart.put_str("scope: initialize fixed key/value root table page\n");
            } else {
                print_db_table_init(uart);
            }
        }
        Some("put") => {
            let key = parts.next();
            let value = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db put <key> <value> confirm\n");
                uart.put_str("scope: WAL-log and upsert one fixed-table record\n");
            } else {
                print_db_put(uart, key, value);
            }
        }
        Some("get") => {
            let key = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db get <key> confirm\n");
                uart.put_str("scope: read one fixed-table record\n");
            } else {
                print_db_get(uart, key);
            }
        }
        Some("btree") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db btree confirm\n");
                uart.put_str("scope: read experimental B+tree root metadata\n");
            } else {
                print_db_btree(uart);
            }
        }
        Some("btree-init") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db btree-init confirm\n");
                uart.put_str("scope: initialize experimental B+tree root and leaf pages\n");
            } else {
                print_db_btree_init(uart);
            }
        }
        Some("bput") => {
            let key = parts.next();
            let value = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db bput <key> <value> confirm\n");
                uart.put_str("scope: WAL-log and upsert one B+tree record\n");
            } else {
                print_db_bput(uart, key, value);
            }
        }
        Some("bget") => {
            let key = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db bget <key> confirm\n");
                uart.put_str("scope: read one B+tree record\n");
            } else {
                print_db_bget(uart, key);
            }
        }
        Some("bscan") => {
            let start = parts.next();
            let limit = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db bscan <start> <limit> confirm\n");
                uart.put_str("scope: scan up to 8 B+tree rows in key order\n");
            } else {
                print_db_bscan(uart, start, limit);
            }
        }
        Some("bleaf") => {
            let page = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db bleaf <page> confirm\n");
                uart.put_str("scope: read one B+tree leaf header for diagnosis\n");
            } else {
                print_db_bleaf(uart, page);
            }
        }
        Some("event-init") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db event-init confirm\n");
                uart.put_str("scope: initialize append-only router event log metadata\n");
            } else {
                print_db_event_init(uart);
            }
        }
        Some("event-status") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db event-status confirm\n");
                uart.put_str("scope: read router event log metadata\n");
            } else {
                print_db_event_status(uart);
            }
        }
        Some("events") => {
            let start = parts.next();
            let limit = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db events <start-seq> <limit> confirm\n");
                uart.put_str("scope: list up to 8 router event page headers\n");
            } else {
                print_db_events(uart, start, limit);
            }
        }
        Some("event") => {
            let sequence = parts.next();
            let confirm = parts.next();
            if confirm != Some("confirm") {
                uart.put_str("usage: db event <seq> confirm\n");
                uart.put_str("scope: read one router event page header\n");
            } else {
                print_db_event(uart, sequence);
            }
        }
        Some("blob-status") => {
            if parts.next() != Some("confirm") {
                uart.put_str("usage: db blob-status confirm\n");
                uart.put_str("scope: read XiaojinDB backup blob metadata\n");
            } else {
                print_db_blob_status(uart);
            }
        }
        _ => {
            uart.put_str("usage:\n");
            uart.put_str("  db facts\n");
            uart.put_str("  db status confirm\n");
            uart.put_str("  db init confirm\n");
            uart.put_str("  db catalog confirm\n");
            uart.put_str("  db catalog-init confirm\n");
            uart.put_str("  db page <n> confirm\n");
            uart.put_str("  db wal-append confirm\n");
            uart.put_str("  db table confirm\n");
            uart.put_str("  db table-init confirm\n");
            uart.put_str("  db put <key> <value> confirm\n");
            uart.put_str("  db get <key> confirm\n");
            uart.put_str("  db btree confirm\n");
            uart.put_str("  db btree-init confirm\n");
            uart.put_str("  db bput <key> <value> confirm\n");
            uart.put_str("  db bget <key> confirm\n");
            uart.put_str("  db bscan <start> <limit> confirm\n");
            uart.put_str("  db bleaf <page> confirm\n");
            uart.put_str("  db event-init confirm\n");
            uart.put_str("  db event-status confirm\n");
            uart.put_str("  db events <start-seq> <limit> confirm\n");
            uart.put_str("  db event <seq> confirm\n");
            uart.put_str("  db blob-status confirm\n");
        }
    }
}

fn print_sd_facts<W: Write>(out: &mut W) {
    let _ = writeln!(out, "SD/MMC OTA storage facts:");
    let _ = writeln!(out, "  board path       NanoPi R5C TF slot");
    let _ = writeln!(out, "  u-boot device    mmc1 / dwmmc@fe2b0000");
    let _ = writeln!(
        out,
        "  controller       0x{SD_DWMMC_BASE:016x}..0x{:016x}",
        SD_DWMMC_BASE + SD_DWMMC_WINDOW_SIZE
    );
    let _ = writeln!(
        out,
        "  stage now        FAT32 read/write of preallocated files"
    );
    let _ = writeln!(
        out,
        "  next             FAT32 root directory + cluster-chain scan"
    );
    let _ = writeln!(
        out,
        "  ota target       overwrite inactive preallocated A/B slot"
    );
    let _ = writeln!(
        out,
        "  writer           enabled for existing preallocated files"
    );
}

fn print_sd_regs<W: Write>(out: &mut W) {
    let _ = writeln!(out, "sdmmc0 DW-MMC registers:");
    for reg in SD_DWMMC_REGS {
        let addr = SD_DWMMC_BASE + reg.offset;
        match probe_read32(addr) {
            Ok(value) => {
                let _ = writeln!(
                    out,
                    "  {:<8} +0x{:03x} @ 0x{addr:016x}: 0x{value:08x}",
                    reg.name, reg.offset
                );
            }
            Err(snapshot) => {
                let _ = writeln!(
                    out,
                    "  {:<8} +0x{:03x} @ 0x{addr:016x}: fault ec=0x{:02x} far=0x{:016x}",
                    reg.name,
                    reg.offset,
                    snapshot.esr >> ESR_EC_SHIFT,
                    snapshot.far
                );
            }
        }
    }
}

fn write_sd_reg_summary<W: Write>(out: &mut W) {
    let _ = write!(out, "sd-regs base=0x{SD_DWMMC_BASE:016x}");
    for reg in SD_DWMMC_REGS {
        let addr = SD_DWMMC_BASE + reg.offset;
        match probe_read32(addr) {
            Ok(value) => {
                let _ = write!(out, " {}=0x{value:08x}", reg.name);
            }
            Err(_) => {
                let _ = write!(out, " {}=fault", reg.name);
            }
        }
    }
}

fn print_sd_lba0<W: Write>(out: &mut W) {
    match sd_read_lba0() {
        Ok(()) => {
            let _ = writeln!(out, "sd read0 ok:");
            write_sd_lba0_details(out, "  ");
        }
        Err(error) => {
            let _ = writeln!(out, "sd read0 error: {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = writeln!(out, "  bits=0x{bits:08x}");
            }
        }
    }
}

fn write_sd_lba0_summary<W: Write>(out: &mut W) {
    match sd_read_lba0() {
        Ok(()) => {
            let _ = write!(out, "OK ");
            write_sd_lba0_details(out, "");
        }
        Err(error) => {
            let _ = write!(out, "ERR sd-read0 {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = write!(out, " bits=0x{bits:08x}");
            }
        }
    }
}

fn write_sd_lba0_details<W: Write>(out: &mut W, prefix: &str) {
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let sig = read_le16(&block, 510).unwrap_or(0);
    let p0 = 446usize;
    let p0_status = block[p0];
    let p0_type = block[p0 + 4];
    let p0_lba = read_le32(&block, p0 + 8).unwrap_or(0);
    let p0_sectors = read_le32(&block, p0 + 12).unwrap_or(0);
    let _ = write!(
        out,
        "{prefix}sig=0x{sig:04x} p0_status=0x{p0_status:02x} p0_type=0x{p0_type:02x} p0_lba={p0_lba} p0_sectors={p0_sectors} first16="
    );
    for byte in &block[..16] {
        let _ = write!(out, "{byte:02x}");
    }
    let _ = writeln!(out);
}

fn print_sd_bpb<W: Write>(out: &mut W) {
    match read_first_partition_bpb() {
        Ok(part_lba) => {
            let _ = writeln!(out, "sd bpb ok:");
            write_sd_bpb_details(out, "  ", part_lba);
        }
        Err(error) => {
            let _ = writeln!(out, "sd bpb error: {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = writeln!(out, "  bits=0x{bits:08x}");
            }
        }
    }
}

fn write_sd_bpb_summary<W: Write>(out: &mut W) {
    match read_first_partition_bpb() {
        Ok(part_lba) => {
            let _ = write!(out, "OK ");
            write_sd_bpb_details(out, "", part_lba);
        }
        Err(error) => {
            let _ = write!(out, "ERR sd-bpb {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = write!(out, " bits=0x{bits:08x}");
            }
        }
    }
}

fn read_first_partition_bpb() -> Result<u32, SdReadError> {
    sd_read_lba0()?;
    let mut mbr = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            mbr.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let part_lba = read_le32(&mbr, 446 + 8).unwrap_or(0);
    if part_lba == 0 {
        return Err(SdReadError::InvalidLayout);
    }
    sd_read_lba(part_lba)?;
    Ok(part_lba)
}

fn write_sd_bpb_details<W: Write>(out: &mut W, prefix: &str, part_lba: u32) {
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let sig = read_le16(&block, 510).unwrap_or(0);
    let bytes_per_sector = read_le16(&block, 11).unwrap_or(0);
    let sectors_per_cluster = block[13];
    let reserved_sectors = read_le16(&block, 14).unwrap_or(0);
    let fats = block[16];
    let total_sectors = read_le32(&block, 32).unwrap_or(0);
    let fat_sectors = read_le32(&block, 36).unwrap_or(0);
    let root_cluster = read_le32(&block, 44).unwrap_or(0);
    let fsinfo_sector = read_le16(&block, 48).unwrap_or(0);
    let backup_boot_sector = read_le16(&block, 50).unwrap_or(0);
    let first_fat_lba = part_lba.saturating_add(reserved_sectors as u32);
    let data_lba = first_fat_lba.saturating_add((fats as u32).saturating_mul(fat_sectors));
    let root_lba = data_lba.saturating_add(
        root_cluster
            .saturating_sub(2)
            .saturating_mul(sectors_per_cluster as u32),
    );
    let _ = writeln!(
        out,
        "{prefix}part_lba={part_lba} sig=0x{sig:04x} bps={bytes_per_sector} spc={sectors_per_cluster} reserved={reserved_sectors} fats={fats} fat_sectors={fat_sectors} total_sectors={total_sectors} root_cluster={root_cluster} fsinfo={fsinfo_sector} backup={backup_boot_sector} first_fat_lba={first_fat_lba} data_lba={data_lba} root_lba={root_lba}"
    );
}

fn print_sd_root<W: Write>(out: &mut W) {
    match scan_sd_root_targets() {
        Ok((facts, targets)) => {
            let _ = writeln!(out, "sd root ok:");
            write_sd_root_details(out, "  ", facts, &targets);
        }
        Err(error) => {
            let _ = writeln!(out, "sd root error: {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = writeln!(out, "  bits=0x{bits:08x}");
            }
        }
    }
}

fn write_sd_root_summary<W: Write>(out: &mut W) {
    match scan_sd_root_targets() {
        Ok((facts, targets)) => {
            let _ = write!(out, "OK ");
            write_sd_root_details(out, "", facts, &targets);
        }
        Err(error) => {
            let _ = write!(out, "ERR sd-root {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = write!(out, " bits=0x{bits:08x}");
            }
        }
    }
}

fn write_sd_root_details<W: Write>(
    out: &mut W,
    prefix: &str,
    facts: Fat32Facts,
    targets: &[FatDirTarget; 4],
) {
    let _ = write!(
        out,
        "{prefix}part_lba={} first_fat_lba={} data_lba={} root_lba={} root_cluster={} bps={} spc={} reserved={} fats={} fat_sectors={} total_sectors={} ",
        facts.part_lba,
        facts.first_fat_lba,
        facts.data_lba,
        facts.root_lba,
        facts.root_cluster,
        facts.bytes_per_sector,
        facts.sectors_per_cluster,
        facts.reserved_sectors,
        facts.fats,
        facts.fat_sectors,
        facts.total_sectors
    );
    for target in targets {
        if target.found {
            let _ = write!(
                out,
                "{} cluster={} size={} ",
                target.name, target.cluster, target.size
            );
        } else {
            let _ = write!(out, "{} missing ", target.name);
        }
    }
    let _ = writeln!(out);
}

fn print_sd_slots<W: Write>(out: &mut W) {
    match scan_sd_slot_chains() {
        Ok((facts, targets, chains)) => {
            let _ = writeln!(out, "sd slots ok:");
            write_sd_slots_details(out, "  ", facts, &targets, &chains);
        }
        Err(error) => {
            let _ = writeln!(out, "sd slots error: {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = writeln!(out, "  bits=0x{bits:08x}");
            }
        }
    }
}

fn write_sd_slots_summary<W: Write>(out: &mut W) {
    match scan_sd_slot_chains() {
        Ok((facts, targets, chains)) => {
            let _ = write!(out, "OK ");
            write_sd_slots_details(out, "", facts, &targets, &chains);
        }
        Err(error) => {
            let _ = write!(out, "ERR sd-slots {}", error.as_str());
            if let SdReadError::StatusError(bits) | SdReadError::IdmacError(bits) = error {
                let _ = write!(out, " bits=0x{bits:08x}");
            }
        }
    }
}

fn write_sd_slots_details<W: Write>(
    out: &mut W,
    prefix: &str,
    facts: Fat32Facts,
    targets: &[FatDirTarget; 4],
    chains: &[FatChainSummary; 2],
) {
    let cluster_bytes = (facts.bytes_per_sector as u32) * (facts.sectors_per_cluster as u32);
    let _ = write!(
        out,
        "{prefix}cluster_bytes={} first_fat_lba={} data_lba={} ",
        cluster_bytes, facts.first_fat_lba, facts.data_lba
    );
    for index in 0..2 {
        let target = targets[index];
        let chain = chains[index];
        if !target.found {
            let _ = write!(out, "{} missing ", target.name);
        } else {
            let _ = write!(
                out,
                "{} ok={} first={} last={} clusters={}/{} contiguous={} terminal=0x{:08x} size={} ",
                target.name,
                yes_no(chain.ok),
                chain.first,
                chain.last,
                chain.clusters,
                chain.expected_clusters,
                yes_no(chain.contiguous),
                chain.terminal,
                target.size
            );
        }
    }
    let _ = writeln!(out);
}

fn scan_sd_slot_chains(
) -> Result<(Fat32Facts, [FatDirTarget; 4], [FatChainSummary; 2]), SdReadError> {
    let (facts, targets) = scan_sd_root_targets()?;
    let chains = [
        scan_fat_chain(facts, targets[0])?,
        scan_fat_chain(facts, targets[1])?,
    ];
    Ok((facts, targets, chains))
}

fn read_extlinux_info() -> Result<(Fat32Facts, FatDirEntryInfo, FatDirEntryInfo, u8), BootSlotError>
{
    let facts = read_fat32_facts()?;
    if facts.bytes_per_sector != SD_BLOCK_BYTES as u16 || facts.sectors_per_cluster == 0 {
        return Err(BootSlotError::Sd(SdReadError::InvalidLayout));
    }

    let extlinux = find_fat_entry_in_dir(facts, facts.root_cluster, b"extlinux")?;
    if !extlinux.found || extlinux.cluster < 2 || extlinux.attr & 0x10 == 0 {
        return Err(BootSlotError::ExtlinuxDirMissing);
    }

    let config = find_fat_entry_in_dir(facts, extlinux.cluster, b"extlinux.conf")?;
    if !config.found || config.cluster < 2 || config.size == 0 {
        return Err(BootSlotError::ExtlinuxConfMissing);
    }

    let default_slot = read_extlinux_default_slot(facts, config)?;
    Ok((facts, extlinux, config, default_slot))
}

fn write_sd_extlinux_summary<W: Write>(out: &mut W) {
    match read_extlinux_info() {
        Ok((_facts, extlinux, config, default_slot)) => {
            let _ = write!(
                out,
                "OK extlinux cluster={} extlinux.conf cluster={} size={} default={}",
                extlinux.cluster,
                config.cluster,
                config.size,
                extlinux_slot_label(default_slot)
            );
        }
        Err(error) => {
            let _ = write!(out, "ERR sd-extlinux {}", error.as_str());
            if let BootSlotError::Sd(SdReadError::StatusError(bits))
            | BootSlotError::Sd(SdReadError::IdmacError(bits)) = error
            {
                let _ = write!(out, " bits=0x{bits:08x}");
            }
        }
    }
}

fn print_db_facts<W: Write>(out: &mut W) {
    let _ = writeln!(out, "XiaojinDB page-store facts:");
    let _ = writeln!(out, "  file             {DB_FILE_NAME_STR}");
    let _ = writeln!(out, "  page bytes       {DB_PAGE_BYTES}");
    let _ = writeln!(out, "  superblock       page 0, little-endian fields");
    let _ = writeln!(
        out,
        "  persistence      existing FAT32 file, no allocation yet"
    );
    let _ = writeln!(
        out,
        "  fixed table      page {DB_TABLE_ROOT_PAGE}, {DB_TABLE_RECORDS} records, {DB_TABLE_VALUE_BYTES}-byte values"
    );
    let _ = writeln!(
        out,
        "  btree            root page {DB_BTREE_ROOT_PAGE}, leaves {}..{}, {DB_TABLE_RECORDS} records/leaf",
        DB_BTREE_LEAF_START_PAGE,
        DB_BTREE_LEAF_START_PAGE + DB_BTREE_LEAF_COUNT - 1
    );
    let _ = writeln!(
        out,
        "  blob backup      meta page {DB_BLOB_META_PAGE}, data starts at page {DB_BLOB_START_PAGE}"
    );
}

fn print_db_status<W: Write>(out: &mut W) {
    match read_db_superblock() {
        Ok((info, superblock)) => {
            let _ = writeln!(out, "db status ok:");
            write_db_superblock_details(out, "  ", info, superblock);
        }
        Err(error) => {
            let _ = writeln!(out, "db status error: {}", error.as_str());
            if let DbError::Sd(SdReadError::StatusError(bits))
            | DbError::Sd(SdReadError::IdmacError(bits)) = error
            {
                let _ = writeln!(out, "  bits=0x{bits:08x}");
            }
        }
    }
}

fn print_db_init<W: Write>(out: &mut W) {
    match init_db_superblock() {
        Ok((info, superblock)) => {
            let _ = writeln!(out, "db init ok:");
            write_db_superblock_details(out, "  ", info, superblock);
        }
        Err(error) => {
            let _ = writeln!(out, "db init error: {}", error.as_str());
            if let DbError::Sd(SdReadError::StatusError(bits))
            | DbError::Sd(SdReadError::IdmacError(bits)) = error
            {
                let _ = writeln!(out, "  bits=0x{bits:08x}");
            }
        }
    }
}

fn print_db_catalog<W: Write>(out: &mut W) {
    match read_db_catalog() {
        Ok((info, catalog)) => {
            let _ = writeln!(out, "db catalog ok:");
            write_db_catalog_details(out, "  ", info, catalog);
        }
        Err(error) => {
            let _ = writeln!(out, "db catalog error: {}", error.as_str());
        }
    }
}

fn print_db_catalog_init<W: Write>(out: &mut W) {
    match init_db_catalog() {
        Ok((info, catalog)) => {
            let _ = writeln!(out, "db catalog-init ok:");
            write_db_catalog_details(out, "  ", info, catalog);
        }
        Err(error) => {
            let _ = writeln!(out, "db catalog-init error: {}", error.as_str());
        }
    }
}

fn print_db_page<W: Write>(out: &mut W, page: Option<&str>) {
    let Some(page) = page.and_then(parse_u64) else {
        uart_like_usage(out, "db page <n> confirm");
        return;
    };
    match read_db_page_summary(page as u32) {
        Ok((_info, summary)) => {
            let _ = writeln!(out, "db page ok:");
            write_db_page_summary_details(out, "  ", summary);
        }
        Err(error) => {
            let _ = writeln!(out, "db page error: {}", error.as_str());
        }
    }
}

fn print_db_wal_append<W: Write>(out: &mut W) {
    match append_db_wal_record() {
        Ok((_info, catalog, summary)) => {
            let _ = writeln!(out, "db wal-append ok:");
            write_db_catalog_details_for_wal(out, "  ", catalog);
            write_db_page_summary_details(out, "  ", summary);
        }
        Err(error) => {
            let _ = writeln!(out, "db wal-append error: {}", error.as_str());
        }
    }
}

fn print_db_table<W: Write>(out: &mut W) {
    match read_db_table() {
        Ok((_info, _catalog, table)) => {
            let _ = writeln!(out, "db table ok:");
            write_db_table_details(out, "  ", table);
        }
        Err(error) => {
            let _ = writeln!(out, "db table error: {}", error.as_str());
        }
    }
}

fn print_db_table_init<W: Write>(out: &mut W) {
    match init_db_table() {
        Ok((_info, catalog, table)) => {
            let _ = writeln!(out, "db table-init ok:");
            write_db_catalog_details_for_wal(out, "  ", catalog);
            write_db_table_details(out, "  ", table);
        }
        Err(error) => {
            let _ = writeln!(out, "db table-init error: {}", error.as_str());
        }
    }
}

fn print_db_put<W: Write>(out: &mut W, key: Option<&str>, value: Option<&str>) {
    let Some(key) = key.and_then(parse_u64) else {
        uart_like_usage(out, "db put <key> <value> confirm");
        return;
    };
    let Some(value) = value else {
        uart_like_usage(out, "db put <key> <value> confirm");
        return;
    };
    match put_db_row(key as u32, value.as_bytes()) {
        Ok((_info, _catalog, table, row)) => {
            let _ = writeln!(out, "db put ok:");
            write_db_table_details(out, "  ", table);
            write_db_row_details(out, "  ", row);
        }
        Err(error) => {
            let _ = writeln!(out, "db put error: {}", error.as_str());
        }
    }
}

fn print_db_get<W: Write>(out: &mut W, key: Option<&str>) {
    let Some(key) = key.and_then(parse_u64) else {
        uart_like_usage(out, "db get <key> confirm");
        return;
    };
    match get_db_row(key as u32) {
        Ok((_info, _catalog, table, row)) => {
            let _ = writeln!(out, "db get ok:");
            write_db_table_details(out, "  ", table);
            write_db_row_details(out, "  ", row);
        }
        Err(error) => {
            let _ = writeln!(out, "db get error: {}", error.as_str());
        }
    }
}

fn print_db_btree<W: Write>(out: &mut W) {
    match read_db_btree_root() {
        Ok((_info, _catalog, root)) => {
            let _ = writeln!(out, "db btree ok:");
            write_db_btree_root_details(out, "  ", root);
        }
        Err(error) => {
            let _ = writeln!(out, "db btree error: {}", error.as_str());
        }
    }
}

fn print_db_btree_init<W: Write>(out: &mut W) {
    match init_db_btree() {
        Ok((_info, catalog, root)) => {
            let _ = writeln!(out, "db btree-init ok:");
            write_db_catalog_details_for_wal(out, "  ", catalog);
            write_db_btree_root_details(out, "  ", root);
        }
        Err(error) => {
            let _ = writeln!(out, "db btree-init error: {}", error.as_str());
        }
    }
}

fn print_db_bput<W: Write>(out: &mut W, key: Option<&str>, value: Option<&str>) {
    let Some(key) = key.and_then(parse_u64) else {
        uart_like_usage(out, "db bput <key> <value> confirm");
        return;
    };
    let Some(value) = value else {
        uart_like_usage(out, "db bput <key> <value> confirm");
        return;
    };
    match put_db_btree_row(key as u32, value.as_bytes()) {
        Ok((_info, _catalog, root, leaf, row)) => {
            let _ = writeln!(out, "db bput ok:");
            write_db_btree_root_details(out, "  ", root);
            write_db_btree_leaf_details(out, "  ", leaf);
            write_db_row_details(out, "  ", row);
        }
        Err(error) => {
            let _ = writeln!(out, "db bput error: {}", error.as_str());
        }
    }
}

fn print_db_bget<W: Write>(out: &mut W, key: Option<&str>) {
    let Some(key) = key.and_then(parse_u64) else {
        uart_like_usage(out, "db bget <key> confirm");
        return;
    };
    match get_db_btree_row(key as u32) {
        Ok((_info, _catalog, root, leaf, row)) => {
            let _ = writeln!(out, "db bget ok:");
            write_db_btree_root_details(out, "  ", root);
            write_db_btree_leaf_details(out, "  ", leaf);
            write_db_row_details(out, "  ", row);
        }
        Err(error) => {
            let _ = writeln!(out, "db bget error: {}", error.as_str());
        }
    }
}

fn print_db_bscan<W: Write>(out: &mut W, start: Option<&str>, limit: Option<&str>) {
    let Some(start) = start.and_then(parse_u64) else {
        uart_like_usage(out, "db bscan <start> <limit> confirm");
        return;
    };
    let Some(limit) = limit.and_then(parse_u64) else {
        uart_like_usage(out, "db bscan <start> <limit> confirm");
        return;
    };
    match scan_db_btree_rows(start as u32, limit as u32) {
        Ok((_info, _catalog, root, scan)) => {
            let _ = writeln!(out, "db bscan ok:");
            write_db_btree_root_details(out, "  ", root);
            write_db_btree_scan_details(out, "  ", scan);
        }
        Err(error) => {
            let _ = writeln!(out, "db bscan error: {}", error.as_str());
        }
    }
}

fn print_db_bleaf<W: Write>(out: &mut W, page: Option<&str>) {
    let Some(page) = page.and_then(parse_u64) else {
        uart_like_usage(out, "db bleaf <page> confirm");
        return;
    };
    match read_db_btree_leaf_raw(page as u32) {
        Ok((_info, _catalog, _root, leaf)) => {
            let _ = writeln!(out, "db bleaf ok:");
            write_db_btree_leaf_details(out, "  ", leaf);
        }
        Err(error) => {
            let _ = writeln!(out, "db bleaf error: {}", error.as_str());
        }
    }
}

fn print_db_event_init<W: Write>(out: &mut W) {
    match init_db_event_log() {
        Ok((_info, _catalog, meta)) => {
            let _ = writeln!(out, "db event-init ok:");
            write_db_event_meta_details(out, "  ", meta);
        }
        Err(error) => {
            let _ = writeln!(out, "db event-init error: {}", error.as_str());
        }
    }
}

fn print_db_event_status<W: Write>(out: &mut W) {
    match read_db_event_meta() {
        Ok((_info, _catalog, meta)) => {
            let _ = writeln!(out, "db event-status ok:");
            write_db_event_meta_details(out, "  ", meta);
        }
        Err(error) => {
            let _ = writeln!(out, "db event-status error: {}", error.as_str());
        }
    }
}

fn print_db_events<W: Write>(out: &mut W, start: Option<&str>, limit: Option<&str>) {
    let Some(start) = start.and_then(parse_u64) else {
        uart_like_usage(out, "db events <start-seq> <limit> confirm");
        return;
    };
    let Some(limit) = limit.and_then(parse_u64) else {
        uart_like_usage(out, "db events <start-seq> <limit> confirm");
        return;
    };
    match list_db_events(start as u32, limit as u32) {
        Ok((_info, _catalog, meta, list)) => {
            let _ = writeln!(out, "db events ok:");
            write_db_event_meta_details(out, "  ", meta);
            write_db_event_list_details(out, "  ", list);
        }
        Err(error) => {
            let _ = writeln!(out, "db events error: {}", error.as_str());
        }
    }
}

fn print_db_event<W: Write>(out: &mut W, sequence: Option<&str>) {
    let Some(sequence) = sequence.and_then(parse_u64) else {
        uart_like_usage(out, "db event <seq> confirm");
        return;
    };
    match read_db_event_by_sequence(sequence as u32) {
        Ok((_info, _catalog, _meta, event)) => {
            let _ = writeln!(out, "db event ok:");
            write_db_event_page_details(out, "  ", event);
        }
        Err(error) => {
            let _ = writeln!(out, "db event error: {}", error.as_str());
        }
    }
}

fn print_db_blob_status<W: Write>(out: &mut W) {
    match read_db_blob_meta() {
        Ok((_info, _catalog, meta)) => {
            let _ = writeln!(out, "db blob-status ok:");
            write_db_blob_meta_details(out, "  ", meta);
        }
        Err(error) => {
            let _ = writeln!(out, "db blob-status error: {}", error.as_str());
        }
    }
}

fn uart_like_usage<W: Write>(out: &mut W, usage: &str) {
    let _ = writeln!(out, "usage: {usage}");
}

fn write_db_status_summary<W: Write>(out: &mut W) {
    match read_db_superblock() {
        Ok((info, superblock)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_superblock_summary(out, info, superblock);
        }
        Err(error) => write_db_error_summary(out, "status", error),
    }
}

fn write_db_init_summary<W: Write>(out: &mut W) {
    match init_db_superblock() {
        Ok((info, superblock)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_superblock_summary(out, info, superblock);
        }
        Err(error) => write_db_error_summary(out, "init", error),
    }
}

fn write_db_catalog_summary<W: Write>(out: &mut W) {
    match read_db_catalog() {
        Ok((info, catalog)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_catalog_summary_inner(out, info, catalog);
        }
        Err(error) => write_db_error_summary(out, "catalog", error),
    }
}

fn write_db_catalog_init_summary<W: Write>(out: &mut W) {
    match init_db_catalog() {
        Ok((info, catalog)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_catalog_summary_inner(out, info, catalog);
        }
        Err(error) => write_db_error_summary(out, "catalog-init", error),
    }
}

fn write_db_page_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _page = parts.next();
    let Some(page) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-page usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR db-page usage");
        return;
    }
    match read_db_page_summary(page as u32) {
        Ok((_info, summary)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_page_summary_inner(out, summary);
        }
        Err(error) => write_db_error_summary(out, "page", error),
    }
}

fn write_db_wal_append_summary<W: Write>(out: &mut W) {
    match append_db_wal_record() {
        Ok((_info, catalog, summary)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_catalog_wal_summary_inner(out, catalog);
            let _ = write!(out, " ");
            write_db_page_summary_inner(out, summary);
        }
        Err(error) => write_db_error_summary(out, "wal-append", error),
    }
}

fn write_db_table_summary<W: Write>(out: &mut W) {
    match read_db_table() {
        Ok((_info, _catalog, table)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_table_summary_inner(out, table);
        }
        Err(error) => write_db_error_summary(out, "table", error),
    }
}

fn write_db_table_init_summary<W: Write>(out: &mut W) {
    match init_db_table() {
        Ok((_info, catalog, table)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_catalog_wal_summary_inner(out, catalog);
            let _ = write!(out, " ");
            write_db_table_summary_inner(out, table);
        }
        Err(error) => write_db_error_summary(out, "table-init", error),
    }
}

fn write_db_put_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _put = parts.next();
    let Some(key) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-put usage");
        return;
    };
    let Some(value) = parts.next() else {
        let _ = write!(out, "XJQ ERR db-put usage");
        return;
    };
    if parts.next() != Some("confirm") {
        let _ = write!(out, "XJQ ERR db-put usage");
        return;
    }
    match put_db_row(key as u32, value.as_bytes()) {
        Ok((_info, _catalog, table, row)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_table_summary_inner(out, table);
            let _ = write!(out, " ");
            write_db_row_summary_inner(out, row);
        }
        Err(error) => write_db_error_summary(out, "put", error),
    }
}

fn write_db_get_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _get = parts.next();
    let Some(key) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-get usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR db-get usage");
        return;
    }
    match get_db_row(key as u32) {
        Ok((_info, _catalog, table, row)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_table_summary_inner(out, table);
            let _ = write!(out, " ");
            write_db_row_summary_inner(out, row);
        }
        Err(error) => write_db_error_summary(out, "get", error),
    }
}

fn write_db_btree_summary<W: Write>(out: &mut W) {
    match read_db_btree_root() {
        Ok((_info, _catalog, root)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_btree_root_summary_inner(out, root);
        }
        Err(error) => write_db_error_summary(out, "btree", error),
    }
}

fn write_db_btree_init_summary<W: Write>(out: &mut W) {
    match init_db_btree() {
        Ok((_info, catalog, root)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_catalog_wal_summary_inner(out, catalog);
            let _ = write!(out, " ");
            write_db_btree_root_summary_inner(out, root);
        }
        Err(error) => write_db_error_summary(out, "btree-init", error),
    }
}

fn write_db_bput_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _bput = parts.next();
    let Some(key) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-bput usage");
        return;
    };
    let Some(value) = parts.next() else {
        let _ = write!(out, "XJQ ERR db-bput usage");
        return;
    };
    if parts.next() != Some("confirm") {
        let _ = write!(out, "XJQ ERR db-bput usage");
        return;
    }
    match put_db_btree_row(key as u32, value.as_bytes()) {
        Ok((_info, _catalog, root, leaf, row)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_btree_root_summary_inner(out, root);
            let _ = write!(out, " ");
            write_db_btree_leaf_summary_inner(out, leaf);
            let _ = write!(out, " ");
            write_db_row_summary_inner(out, row);
        }
        Err(error) => write_db_error_summary(out, "bput", error),
    }
}

fn write_db_bget_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _bget = parts.next();
    let Some(key) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-bget usage");
        return;
    };
    if !matches!(parts.next(), None | Some("confirm")) {
        let _ = write!(out, "XJQ ERR db-bget usage");
        return;
    }
    match get_db_btree_row(key as u32) {
        Ok((_info, _catalog, root, leaf, row)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_btree_root_summary_inner(out, root);
            let _ = write!(out, " ");
            write_db_btree_leaf_summary_inner(out, leaf);
            let _ = write!(out, " ");
            write_db_row_summary_inner(out, row);
        }
        Err(error) => write_db_error_summary(out, "bget", error),
    }
}

fn write_db_bscan_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _bscan = parts.next();
    let Some(start) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-bscan usage");
        return;
    };
    let Some(limit) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-bscan usage");
        return;
    };
    if parts.next() != Some("confirm") {
        let _ = write!(out, "XJQ ERR db-bscan usage");
        return;
    }
    match scan_db_btree_rows(start as u32, limit as u32) {
        Ok((_info, _catalog, root, scan)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_btree_root_summary_inner(out, root);
            let _ = write!(out, " ");
            write_db_btree_scan_summary_inner(out, scan);
        }
        Err(error) => write_db_error_summary(out, "bscan", error),
    }
}

fn write_db_bleaf_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _bleaf = parts.next();
    let Some(page) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-bleaf usage");
        return;
    };
    if parts.next() != Some("confirm") {
        let _ = write!(out, "XJQ ERR db-bleaf usage");
        return;
    }
    match read_db_btree_leaf_raw(page as u32) {
        Ok((_info, _catalog, _root, leaf)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_btree_leaf_summary_inner(out, leaf);
            let _ = write!(
                out,
                " initialized={} version={} flags=0x{:08x} crc-actual=0x{:08x}",
                yes_no(leaf.initialized),
                leaf.version,
                leaf.flags,
                leaf.crc_actual
            );
        }
        Err(error) => write_db_error_summary(out, "bleaf", error),
    }
}

fn write_db_event_init_summary<W: Write>(out: &mut W) {
    match init_db_event_log() {
        Ok((_info, _catalog, meta)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_event_meta_summary_inner(out, meta);
        }
        Err(error) => write_db_error_summary(out, "event-init", error),
    }
}

fn write_db_event_status_summary<W: Write>(out: &mut W) {
    match read_db_event_meta() {
        Ok((_info, _catalog, meta)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_event_meta_summary_inner(out, meta);
        }
        Err(error) => write_db_error_summary(out, "event-status", error),
    }
}

fn write_db_events_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _events = parts.next();
    let Some(start) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-events usage");
        return;
    };
    let Some(limit) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-events usage");
        return;
    };
    if parts.next() != Some("confirm") {
        let _ = write!(out, "XJQ ERR db-events usage");
        return;
    }
    match list_db_events(start as u32, limit as u32) {
        Ok((_info, _catalog, meta, list)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_event_meta_summary_inner(out, meta);
            let _ = write!(out, " ");
            write_db_event_list_summary_inner(out, list);
        }
        Err(error) => write_db_error_summary(out, "events", error),
    }
}

fn write_db_event_query_summary<W: Write>(query: &str, out: &mut W) {
    let mut parts = query.split_whitespace();
    let _db = parts.next();
    let _event = parts.next();
    let Some(sequence) = parts.next().and_then(parse_u64) else {
        let _ = write!(out, "XJQ ERR db-event usage");
        return;
    };
    if parts.next() != Some("confirm") {
        let _ = write!(out, "XJQ ERR db-event usage");
        return;
    }
    let mut payload = [0u8; DB_EVENT_PAYLOAD_BYTES];
    match read_db_event_payload_by_sequence(sequence as u32, &mut payload) {
        Ok((_info, _catalog, _meta, event, payload_len)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_event_page_summary_inner(out, event);
            write_db_event_payload_preview(out, &payload[..payload_len]);
        }
        Err(error) => write_db_error_summary(out, "event", error),
    }
}

fn write_db_blob_status_summary<W: Write>(out: &mut W) {
    match read_db_blob_meta() {
        Ok((_info, _catalog, meta)) => {
            let _ = write!(out, "XJQ OK ");
            write_db_blob_meta_summary_inner(out, meta);
        }
        Err(error) => write_db_error_summary(out, "blob-status", error),
    }
}

fn write_db_error_summary<W: Write>(out: &mut W, action: &str, error: DbError) {
    let _ = write!(out, "XJQ ERR db-{action} {}", error.as_str());
    if let DbError::Sd(SdReadError::StatusError(bits))
    | DbError::Sd(SdReadError::IdmacError(bits)) = error
    {
        let _ = write!(out, " bits=0x{bits:08x}");
    }
}

fn write_db_superblock_details<W: Write>(
    out: &mut W,
    prefix: &str,
    info: DbFileInfo,
    superblock: DbSuperblock,
) {
    let _ = writeln!(
        out,
        "{prefix}file={DB_FILE_NAME_STR} cluster={} size={} pages={} initialized={} version={} page_bytes={} page_count={} file_bytes={} root_page={} flags=0x{:08x} crc_expected=0x{:08x} crc_actual=0x{:08x} crc_ok={}",
        info.entry.cluster,
        info.entry.size,
        info.entry.size / DB_PAGE_BYTES,
        yes_no(superblock.initialized),
        superblock.version,
        superblock.page_bytes,
        superblock.page_count,
        superblock.file_bytes,
        superblock.root_page,
        superblock.flags,
        superblock.crc_expected,
        superblock.crc_actual,
        yes_no(superblock.crc_expected == superblock.crc_actual)
    );
}

fn write_db_superblock_summary<W: Write>(out: &mut W, info: DbFileInfo, superblock: DbSuperblock) {
    let _ = write!(
        out,
        "db file={DB_FILE_NAME_STR} cluster={} size={} pages={} initialized={} version={} page-bytes={} page-count={} file-bytes={} root-page={} flags=0x{:08x} crc=0x{:08x} crc-ok={}",
        info.entry.cluster,
        info.entry.size,
        info.entry.size / DB_PAGE_BYTES,
        yes_no(superblock.initialized),
        superblock.version,
        superblock.page_bytes,
        superblock.page_count,
        superblock.file_bytes,
        superblock.root_page,
        superblock.flags,
        superblock.crc_expected,
        yes_no(superblock.crc_expected == superblock.crc_actual)
    );
}

fn write_db_catalog_details<W: Write>(
    out: &mut W,
    prefix: &str,
    info: DbFileInfo,
    catalog: DbCatalog,
) {
    let _ = writeln!(
        out,
        "{prefix}file={DB_FILE_NAME_STR} cluster={} initialized={} version={} page_bytes={} page_count={} wal_start={} wal_pages={} wal_next={} data_start={} table_root={} sequence={} flags=0x{:08x} crc_expected=0x{:08x} crc_actual=0x{:08x} crc_ok={}",
        info.entry.cluster,
        yes_no(catalog.initialized),
        catalog.version,
        catalog.page_bytes,
        catalog.page_count,
        catalog.wal_start_page,
        catalog.wal_page_count,
        catalog.wal_next_page,
        catalog.data_start_page,
        catalog.table_root_page,
        catalog.sequence,
        catalog.flags,
        catalog.crc_expected,
        catalog.crc_actual,
        yes_no(catalog.crc_expected == catalog.crc_actual)
    );
}

fn write_db_catalog_details_for_wal<W: Write>(out: &mut W, prefix: &str, catalog: DbCatalog) {
    let _ = writeln!(
        out,
        "{prefix}catalog wal_start={} wal_pages={} wal_next={} sequence={} crc_ok={}",
        catalog.wal_start_page,
        catalog.wal_page_count,
        catalog.wal_next_page,
        catalog.sequence,
        yes_no(catalog.crc_expected == catalog.crc_actual)
    );
}

fn write_db_catalog_summary_inner<W: Write>(out: &mut W, info: DbFileInfo, catalog: DbCatalog) {
    let _ = write!(
        out,
        "db-catalog file={DB_FILE_NAME_STR} cluster={} initialized={} version={} page-bytes={} page-count={} wal-start={} wal-pages={} wal-next={} data-start={} table-root={} sequence={} flags=0x{:08x} crc=0x{:08x} crc-ok={}",
        info.entry.cluster,
        yes_no(catalog.initialized),
        catalog.version,
        catalog.page_bytes,
        catalog.page_count,
        catalog.wal_start_page,
        catalog.wal_page_count,
        catalog.wal_next_page,
        catalog.data_start_page,
        catalog.table_root_page,
        catalog.sequence,
        catalog.flags,
        catalog.crc_expected,
        yes_no(catalog.crc_expected == catalog.crc_actual)
    );
}

fn write_db_catalog_wal_summary_inner<W: Write>(out: &mut W, catalog: DbCatalog) {
    let _ = write!(
        out,
        "db-wal wal-start={} wal-pages={} wal-next={} sequence={} catalog-crc-ok={}",
        catalog.wal_start_page,
        catalog.wal_page_count,
        catalog.wal_next_page,
        catalog.sequence,
        yes_no(catalog.crc_expected == catalog.crc_actual)
    );
}

fn write_db_page_summary_details<W: Write>(out: &mut W, prefix: &str, summary: DbPageSummary) {
    let _ = write!(out, "{prefix}");
    write_db_page_summary_inner(out, summary);
    let _ = writeln!(out);
}

fn write_db_page_summary_inner<W: Write>(out: &mut W, summary: DbPageSummary) {
    let _ = write!(
        out,
        "db-page page={} crc32=0x{:08x} first16=",
        summary.page, summary.crc32
    );
    for byte in summary.first16 {
        let _ = write!(out, "{byte:02x}");
    }
}

fn write_db_table_details<W: Write>(out: &mut W, prefix: &str, table: DbTablePage) {
    let _ = writeln!(
        out,
        "{prefix}table initialized={} version={} root_page={} records={} capacity={} sequence={} flags=0x{:08x} crc_expected=0x{:08x} crc_actual=0x{:08x} crc_ok={}",
        yes_no(table.initialized),
        table.version,
        table.root_page,
        table.record_count,
        table.capacity,
        table.sequence,
        table.flags,
        table.crc_expected,
        table.crc_actual,
        yes_no(table.crc_expected == table.crc_actual)
    );
}

fn write_db_table_summary_inner<W: Write>(out: &mut W, table: DbTablePage) {
    let _ = write!(
        out,
        "db-table initialized={} version={} root-page={} records={} capacity={} sequence={} flags=0x{:08x} crc=0x{:08x} crc-ok={}",
        yes_no(table.initialized),
        table.version,
        table.root_page,
        table.record_count,
        table.capacity,
        table.sequence,
        table.flags,
        table.crc_expected,
        yes_no(table.crc_expected == table.crc_actual)
    );
}

fn write_db_btree_root_details<W: Write>(out: &mut W, prefix: &str, root: DbBtreeRoot) {
    let _ = writeln!(
        out,
        "{prefix}btree-root initialized={} version={} root_page={} leaf_start={} leaf_count={} leaf_capacity={} records={} sequence={} flags=0x{:08x} crc_expected=0x{:08x} crc_actual=0x{:08x} crc_ok={}",
        yes_no(root.initialized),
        root.version,
        root.root_page,
        root.leaf_start_page,
        root.leaf_count,
        root.leaf_capacity,
        root.record_count,
        root.sequence,
        root.flags,
        root.crc_expected,
        root.crc_actual,
        yes_no(root.crc_expected == root.crc_actual)
    );
}

fn write_db_btree_root_summary_inner<W: Write>(out: &mut W, root: DbBtreeRoot) {
    let _ = write!(
        out,
        "db-btree initialized={} version={} root-page={} leaf-start={} leaf-count={} leaf-capacity={} records={} sequence={} flags=0x{:08x} crc=0x{:08x} crc-ok={}",
        yes_no(root.initialized),
        root.version,
        root.root_page,
        root.leaf_start_page,
        root.leaf_count,
        root.leaf_capacity,
        root.record_count,
        root.sequence,
        root.flags,
        root.crc_expected,
        yes_no(root.crc_expected == root.crc_actual)
    );
}

fn write_db_btree_leaf_details<W: Write>(out: &mut W, prefix: &str, leaf: DbBtreeLeaf) {
    let _ = writeln!(
        out,
        "{prefix}btree-leaf initialized={} version={} page={} records={} capacity={} min_key={} max_key={} next_leaf={} sequence={} flags=0x{:08x} crc_expected=0x{:08x} crc_actual=0x{:08x} crc_ok={}",
        yes_no(leaf.initialized),
        leaf.version,
        leaf.page,
        leaf.record_count,
        leaf.capacity,
        leaf.min_key,
        leaf.max_key,
        leaf.next_leaf,
        leaf.sequence,
        leaf.flags,
        leaf.crc_expected,
        leaf.crc_actual,
        yes_no(leaf.crc_expected == leaf.crc_actual)
    );
}

fn write_db_btree_leaf_summary_inner<W: Write>(out: &mut W, leaf: DbBtreeLeaf) {
    let _ = write!(
        out,
        "db-btree-leaf page={} records={} capacity={} min-key={} max-key={} next-leaf={} sequence={} crc=0x{:08x} crc-ok={}",
        leaf.page,
        leaf.record_count,
        leaf.capacity,
        leaf.min_key,
        leaf.max_key,
        leaf.next_leaf,
        leaf.sequence,
        leaf.crc_expected,
        yes_no(leaf.crc_expected == leaf.crc_actual)
    );
}

fn write_db_btree_scan_details<W: Write>(out: &mut W, prefix: &str, scan: DbBtreeScan) {
    let _ = writeln!(
        out,
        "{prefix}scan start={} requested_limit={} rows={} visited_leaves={}",
        scan.start_key, scan.requested_limit, scan.row_count, scan.visited_leaves
    );
    for index in 0..scan.row_count {
        let _ = write!(out, "{prefix}row{index} ");
        write_db_row_summary_inner(out, scan.rows[index]);
        let _ = writeln!(out);
    }
}

fn write_db_btree_scan_summary_inner<W: Write>(out: &mut W, scan: DbBtreeScan) {
    let _ = write!(
        out,
        "db-bscan start={} requested-limit={} rows={} visited-leaves={}",
        scan.start_key, scan.requested_limit, scan.row_count, scan.visited_leaves
    );
    for index in 0..scan.row_count {
        let _ = write!(out, " row{index}=");
        write_db_row_pair(out, scan.rows[index]);
    }
}

fn write_db_event_meta_details<W: Write>(out: &mut W, prefix: &str, meta: DbEventMeta) {
    let _ = writeln!(
        out,
        "{prefix}event-meta initialized={} version={} meta_page={} start_page={} pages={} next_page={} events={} sequence={} flags=0x{:08x} crc_expected=0x{:08x} crc_actual=0x{:08x} crc_ok={}",
        yes_no(meta.initialized),
        meta.version,
        meta.meta_page,
        meta.start_page,
        meta.page_count,
        meta.next_page,
        meta.event_count,
        meta.sequence,
        meta.flags,
        meta.crc_expected,
        meta.crc_actual,
        yes_no(meta.crc_expected == meta.crc_actual)
    );
}

fn write_db_event_meta_summary_inner<W: Write>(out: &mut W, meta: DbEventMeta) {
    let _ = write!(
        out,
        "db-event-meta initialized={} version={} meta-page={} start-page={} pages={} next-page={} events={} sequence={} flags=0x{:08x} crc=0x{:08x} crc-ok={}",
        yes_no(meta.initialized),
        meta.version,
        meta.meta_page,
        meta.start_page,
        meta.page_count,
        meta.next_page,
        meta.event_count,
        meta.sequence,
        meta.flags,
        meta.crc_expected,
        yes_no(meta.crc_expected == meta.crc_actual)
    );
}

fn write_db_blob_meta_details<W: Write>(out: &mut W, prefix: &str, meta: DbBlobMeta) {
    let _ = writeln!(
        out,
        "{prefix}blob initialized={} version={} meta_page={} start_page={} pages={} capacity={} expected_size={} received_size={} expected_crc=0x{:08x} committed_crc=0x{:08x} sequence={} flags=0x{:08x} active={} committed={} crc_expected=0x{:08x} crc_actual=0x{:08x} crc_ok={}",
        yes_no(meta.initialized),
        meta.version,
        meta.meta_page,
        meta.start_page,
        meta.page_count,
        meta.capacity_bytes,
        meta.expected_size,
        meta.received_size,
        meta.expected_crc,
        meta.committed_crc,
        meta.sequence,
        meta.flags,
        yes_no(meta.flags & DB_BLOB_FLAG_ACTIVE != 0),
        yes_no(meta.flags & DB_BLOB_FLAG_COMMITTED != 0),
        meta.crc_expected,
        meta.crc_actual,
        yes_no(meta.crc_expected == meta.crc_actual)
    );
}

fn write_db_blob_meta_summary_inner<W: Write>(out: &mut W, meta: DbBlobMeta) {
    let _ = write!(
        out,
        "db-blob initialized={} version={} meta-page={} start-page={} pages={} capacity={} expected-size={} received-size={} expected-crc=0x{:08x} committed-crc=0x{:08x} sequence={} flags=0x{:08x} active={} committed={} crc=0x{:08x} crc-ok={}",
        yes_no(meta.initialized),
        meta.version,
        meta.meta_page,
        meta.start_page,
        meta.page_count,
        meta.capacity_bytes,
        meta.expected_size,
        meta.received_size,
        meta.expected_crc,
        meta.committed_crc,
        meta.sequence,
        meta.flags,
        yes_no(meta.flags & DB_BLOB_FLAG_ACTIVE != 0),
        yes_no(meta.flags & DB_BLOB_FLAG_COMMITTED != 0),
        meta.crc_expected,
        yes_no(meta.crc_expected == meta.crc_actual)
    );
}

fn write_db_event_page_details<W: Write>(out: &mut W, prefix: &str, event: DbEventPage) {
    let _ = write!(out, "{prefix}");
    write_db_event_page_summary_inner(out, event);
    let _ = writeln!(
        out,
        " initialized={} version={} ticks={} flags=0x{:08x} crc_actual=0x{:08x}",
        yes_no(event.initialized),
        event.version,
        event.ticks,
        event.flags,
        event.crc_actual
    );
}

fn write_db_event_page_summary_inner<W: Write>(out: &mut W, event: DbEventPage) {
    let _ = write!(
        out,
        "db-event page={} seq={} kind={} source={} payload-len={} payload-crc=0x{:08x} page-crc=0x{:08x} crc-ok={}",
        event.page,
        event.sequence,
        event.kind,
        event.source,
        event.payload_len,
        event.payload_crc,
        event.crc_expected,
        yes_no(event.crc_expected == event.crc_actual)
    );
}

fn write_db_event_payload_preview<W: Write>(out: &mut W, payload: &[u8]) {
    let _ = write!(out, " payload=\"");
    for byte in payload {
        match *byte {
            b'\\' => {
                let _ = write!(out, "\\\\");
            }
            b'"' => {
                let _ = write!(out, "\\\"");
            }
            b'\n' => {
                let _ = write!(out, "\\n");
            }
            b'\r' => {
                let _ = write!(out, "\\r");
            }
            b'\t' => {
                let _ = write!(out, "\\t");
            }
            0x20..=0x7e => {
                let _ = write!(out, "{}", *byte as char);
            }
            _ => {
                let _ = write!(out, "\\x{:02x}", *byte);
            }
        }
    }
    let _ = write!(out, "\"");
}

fn write_db_event_list_details<W: Write>(out: &mut W, prefix: &str, list: DbEventList) {
    let _ = writeln!(
        out,
        "{prefix}events start_sequence={} requested_limit={} rows={} scanned_pages={}",
        list.start_sequence, list.requested_limit, list.row_count, list.scanned_pages
    );
    for index in 0..list.row_count {
        let _ = write!(out, "{prefix}event{index} ");
        write_db_event_page_summary_inner(out, list.rows[index]);
        let _ = writeln!(out);
    }
}

fn write_db_event_list_summary_inner<W: Write>(out: &mut W, list: DbEventList) {
    let _ = write!(
        out,
        "db-events start-sequence={} requested-limit={} rows={} scanned-pages={}",
        list.start_sequence, list.requested_limit, list.row_count, list.scanned_pages
    );
    for index in 0..list.row_count {
        let event = list.rows[index];
        let _ = write!(
            out,
            " event{index}=seq:{}:kind:{}:source:{}:len:{}:crc:0x{:08x}",
            event.sequence, event.kind, event.source, event.payload_len, event.payload_crc
        );
    }
}

fn write_db_row_details<W: Write>(out: &mut W, prefix: &str, row: DbRowValue) {
    let _ = write!(out, "{prefix}");
    write_db_row_summary_inner(out, row);
    let _ = writeln!(out);
}

fn write_db_row_summary_inner<W: Write>(out: &mut W, row: DbRowValue) {
    let _ = write!(
        out,
        "db-row key={} value-len={} value=\"",
        row.key, row.value_len
    );
    for &byte in &row.value[..row.value_len] {
        let ch = if byte.is_ascii_graphic() || byte == b' ' {
            byte
        } else {
            b'.'
        };
        let _ = write!(out, "{}", ch as char);
    }
    let _ = write!(out, "\"");
}

fn write_db_row_pair<W: Write>(out: &mut W, row: DbRowValue) {
    let _ = write!(out, "{}:\"", row.key);
    for &byte in &row.value[..row.value_len] {
        let ch = if byte.is_ascii_graphic() || byte == b' ' {
            byte
        } else {
            b'.'
        };
        let _ = write!(out, "{}", ch as char);
    }
    let _ = write!(out, "\"");
}

fn locate_db_file() -> Result<DbFileInfo, DbError> {
    let facts = read_fat32_facts()?;
    if facts.bytes_per_sector != SD_BLOCK_BYTES as u16 || facts.sectors_per_cluster == 0 {
        return Err(DbError::Sd(SdReadError::InvalidLayout));
    }
    let entry = find_fat_entry_in_dir(facts, facts.root_cluster, DB_FILE_NAME)?;
    if !entry.found || entry.cluster < 2 {
        return Err(DbError::MissingFile);
    }
    if entry.size < SD_BLOCK_BYTES as u32 {
        return Err(DbError::FileTooSmall);
    }
    Ok(DbFileInfo { facts, entry })
}

fn read_db_superblock() -> Result<(DbFileInfo, DbSuperblock), DbError> {
    let info = locate_db_file()?;
    read_file_sector(info.facts, info.entry, 0)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    Ok((info, parse_db_superblock(&block)))
}

fn init_db_superblock() -> Result<(DbFileInfo, DbSuperblock), DbError> {
    let info = locate_db_file()?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    build_db_superblock(info.entry.size, &mut block);
    write_file_sector(info.facts, info.entry, 0, &block)?;
    read_db_superblock()
}

fn read_db_catalog() -> Result<(DbFileInfo, DbCatalog), DbError> {
    let (info, superblock) = read_db_superblock()?;
    validate_db_superblock(superblock)?;
    read_file_sector(info.facts, info.entry, DB_ROOT_CATALOG_PAGE)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    Ok((info, parse_db_catalog(&block)))
}

fn init_db_catalog() -> Result<(DbFileInfo, DbCatalog), DbError> {
    let (info, superblock) = read_db_superblock()?;
    validate_db_superblock(superblock)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    build_db_catalog(superblock.page_count, DB_WAL_START_PAGE, 0, 0, &mut block);
    write_file_sector(info.facts, info.entry, DB_ROOT_CATALOG_PAGE, &block)?;
    read_db_catalog()
}

fn append_db_wal_record() -> Result<(DbFileInfo, DbCatalog, DbPageSummary), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    let new_catalog = append_db_wal_event(info, catalog, DB_WAL_KIND_TEST, 0, b"test")?;
    let page = previous_wal_page(new_catalog);
    let (_info3, summary) = read_db_page_summary(page)?;
    Ok((info, new_catalog, summary))
}

fn append_db_wal_event(
    info: DbFileInfo,
    catalog: DbCatalog,
    kind: u32,
    key: u32,
    value: &[u8],
) -> Result<DbCatalog, DbError> {
    validate_db_catalog(catalog)?;
    let page = catalog.wal_next_page;
    let sequence = catalog.sequence.wrapping_add(1);
    let mut wal = [0u8; SD_BLOCK_BYTES];
    build_db_wal_page(page, sequence, kind, key, value, &mut wal)?;
    write_file_sector(info.facts, info.entry, page, &wal)?;

    let next_page = if page + 1 >= catalog.wal_start_page + catalog.wal_page_count {
        catalog.wal_start_page
    } else {
        page + 1
    };
    let mut catalog_page = [0u8; SD_BLOCK_BYTES];
    build_db_catalog(
        catalog.page_count,
        next_page,
        sequence,
        catalog.table_root_page,
        &mut catalog_page,
    );
    write_file_sector(info.facts, info.entry, DB_ROOT_CATALOG_PAGE, &catalog_page)?;

    let (_info2, new_catalog) = read_db_catalog()?;
    Ok(new_catalog)
}

fn previous_wal_page(catalog: DbCatalog) -> u32 {
    if catalog.wal_next_page == catalog.wal_start_page {
        catalog
            .wal_start_page
            .saturating_add(catalog.wal_page_count)
            .saturating_sub(1)
    } else {
        catalog.wal_next_page.saturating_sub(1)
    }
}

fn read_db_table() -> Result<(DbFileInfo, DbCatalog, DbTablePage), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    if catalog.table_root_page == 0 {
        return Err(DbError::TableMissing);
    }
    read_file_sector(info.facts, info.entry, catalog.table_root_page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let table = parse_db_table(&block);
    validate_db_table(table)?;
    Ok((info, catalog, table))
}

fn init_db_table() -> Result<(DbFileInfo, DbCatalog, DbTablePage), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    let sequence = catalog.sequence.wrapping_add(1);
    let mut table_page = [0u8; SD_BLOCK_BYTES];
    build_db_table_page(0, sequence, &mut table_page);
    write_file_sector(info.facts, info.entry, DB_TABLE_ROOT_PAGE, &table_page)?;

    let mut catalog_page = [0u8; SD_BLOCK_BYTES];
    build_db_catalog(
        catalog.page_count,
        catalog.wal_next_page,
        sequence,
        DB_TABLE_ROOT_PAGE,
        &mut catalog_page,
    );
    write_file_sector(info.facts, info.entry, DB_ROOT_CATALOG_PAGE, &catalog_page)?;
    read_db_table()
}

fn put_db_row(
    key: u32,
    value: &[u8],
) -> Result<(DbFileInfo, DbCatalog, DbTablePage, DbRowValue), DbError> {
    if value.is_empty() || value.len() > DB_TABLE_VALUE_BYTES {
        return Err(DbError::BadValue);
    }
    let (info, catalog, table) = read_db_table()?;
    validate_db_table(table)?;
    let new_catalog = append_db_wal_event(info, catalog, DB_WAL_KIND_PUT, key, value)?;

    read_file_sector(info.facts, info.entry, DB_TABLE_ROOT_PAGE)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let mut record_count = table.record_count;
    let Some((slot, existed)) = find_table_slot(&block, key) else {
        return Err(DbError::TableFull);
    };
    write_table_record(&mut block, slot, key, value);
    if !existed {
        record_count = record_count.saturating_add(1);
    }
    update_table_header(
        &mut block,
        record_count,
        new_catalog.sequence,
        DB_TABLE_RECORDS,
    );
    write_file_sector(info.facts, info.entry, DB_TABLE_ROOT_PAGE, &block)?;

    let (_info2, _catalog2, new_table) = read_db_table()?;
    let row = read_table_row(&block, slot).ok_or(DbError::KeyMissing)?;
    Ok((info, new_catalog, new_table, row))
}

fn delete_db_row(key: u32) -> Result<(DbFileInfo, DbCatalog, DbTablePage), DbError> {
    let (info, catalog, table) = read_db_table()?;
    validate_db_table(table)?;

    read_file_sector(info.facts, info.entry, DB_TABLE_ROOT_PAGE)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let mut found = None;
    for slot in 0..DB_TABLE_RECORDS as usize {
        if table_record_key(&block, slot) == Some(key) {
            found = Some(slot);
            break;
        }
    }
    let Some(slot) = found else {
        return Err(DbError::KeyMissing);
    };
    let Some(offset) = table_record_offset(slot) else {
        return Err(DbError::BadTable);
    };
    let new_catalog = append_db_wal_event(info, catalog, DB_WAL_KIND_DELETE, key, b"delete")?;
    for byte in &mut block[offset..offset + DB_TABLE_RECORD_BYTES] {
        *byte = 0;
    }
    let record_count = table.record_count.saturating_sub(1);
    update_table_header(
        &mut block,
        record_count,
        new_catalog.sequence,
        DB_TABLE_RECORDS,
    );
    write_file_sector(info.facts, info.entry, DB_TABLE_ROOT_PAGE, &block)?;
    let (_info2, _catalog2, new_table) = read_db_table()?;
    Ok((info, new_catalog, new_table))
}

fn get_db_row(key: u32) -> Result<(DbFileInfo, DbCatalog, DbTablePage, DbRowValue), DbError> {
    let (info, catalog, table) = read_db_table()?;
    read_file_sector(info.facts, info.entry, catalog.table_root_page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    for slot in 0..DB_TABLE_RECORDS as usize {
        if table_record_key(&block, slot) == Some(key) {
            let row = read_table_row(&block, slot).ok_or(DbError::KeyMissing)?;
            return Ok((info, catalog, table, row));
        }
    }
    Err(DbError::KeyMissing)
}

fn get_maint_hmac_key_row() -> Result<DbRowValue, DbError> {
    let (_info, _catalog, _table, row) = get_db_row(DB_TABLE_KEY_MAINT_HMAC)?;
    if row.value_len == 0 || row.value_len > DB_TABLE_VALUE_BYTES {
        return Err(DbError::BadValue);
    }
    Ok(row)
}

fn read_db_btree_root() -> Result<(DbFileInfo, DbCatalog, DbBtreeRoot), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    read_file_sector(info.facts, info.entry, DB_BTREE_ROOT_PAGE)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let root = parse_db_btree_root(&block);
    validate_db_btree_root(root)?;
    Ok((info, catalog, root))
}

fn init_db_btree() -> Result<(DbFileInfo, DbCatalog, DbBtreeRoot), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    if DB_BTREE_LEAF_START_PAGE + DB_BTREE_LEAF_COUNT > catalog.page_count {
        return Err(DbError::PageOutOfRange);
    }
    let sequence = catalog.sequence.wrapping_add(1);

    for leaf_index in 0..DB_BTREE_LEAF_COUNT {
        let page = DB_BTREE_LEAF_START_PAGE + leaf_index;
        let next_leaf = if leaf_index + 1 < DB_BTREE_LEAF_COUNT {
            page + 1
        } else {
            0
        };
        let mut leaf_page = [0u8; SD_BLOCK_BYTES];
        build_db_btree_leaf_page(page, 0, 0, 0, sequence, next_leaf, &mut leaf_page);
        write_file_sector(info.facts, info.entry, page, &leaf_page)?;
    }

    let mut root_page = [0u8; SD_BLOCK_BYTES];
    build_db_btree_root_page(0, sequence, &mut root_page);
    write_file_sector(info.facts, info.entry, DB_BTREE_ROOT_PAGE, &root_page)?;

    let mut catalog_page = [0u8; SD_BLOCK_BYTES];
    build_db_catalog(
        catalog.page_count,
        catalog.wal_next_page,
        sequence,
        catalog.table_root_page,
        &mut catalog_page,
    );
    write_file_sector(info.facts, info.entry, DB_ROOT_CATALOG_PAGE, &catalog_page)?;
    read_db_btree_root()
}

fn put_db_btree_row(
    key: u32,
    value: &[u8],
) -> Result<(DbFileInfo, DbCatalog, DbBtreeRoot, DbBtreeLeaf, DbRowValue), DbError> {
    if value.is_empty() || value.len() > DB_TABLE_VALUE_BYTES {
        return Err(DbError::BadValue);
    }
    let (info, catalog, root) = read_db_btree_root()?;
    let leaf_page = db_btree_leaf_page_for_key(root, key)?;
    read_file_sector(info.facts, info.entry, leaf_page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let leaf = parse_db_btree_leaf(&block);
    validate_db_btree_leaf(root, leaf)?;

    let slot_result = find_btree_leaf_slot(&block, leaf.record_count, key);

    if !matches!(slot_result, Some((_, true))) {
        if let Ok((existing_leaf, existing_block, existing_slot)) =
            find_db_btree_row_block_anywhere(info, root, key)
        {
            if existing_leaf.page != leaf.page {
                return update_existing_db_btree_row(
                    info,
                    catalog,
                    root,
                    existing_leaf,
                    existing_block,
                    existing_slot,
                    key,
                    value,
                );
            }
        }
    }

    let new_catalog = append_db_wal_event(info, catalog, DB_WAL_KIND_BPUT, key, value)?;

    if matches!(slot_result, Some((_, false)) | None) && leaf.record_count >= leaf.capacity {
        let (new_root, new_leaf, row) =
            split_db_btree_leaf_insert(info, new_catalog, root, leaf, &block, key, value)?;
        return Ok((info, new_catalog, new_root, new_leaf, row));
    }

    let Some((slot, existed)) = slot_result else {
        let (new_root, new_leaf, row) =
            split_db_btree_leaf_insert(info, new_catalog, root, leaf, &block, key, value)?;
        return Ok((info, new_catalog, new_root, new_leaf, row));
    };

    let mut record_count = leaf.record_count;
    if !existed {
        shift_btree_leaf_records_right(&mut block, slot, record_count as usize);
        record_count = record_count.saturating_add(1);
    }
    write_table_record(&mut block, slot, key, value);
    let (min_key, max_key) = btree_leaf_key_range(&block, record_count);
    build_db_btree_leaf_header(
        &mut block,
        leaf.page,
        record_count,
        min_key,
        max_key,
        new_catalog.sequence,
        leaf.next_leaf,
    );
    write_file_sector(info.facts, info.entry, leaf.page, &block)?;

    let root_record_count = if existed {
        root.record_count
    } else {
        root.record_count.saturating_add(1)
    };
    let mut root_page = [0u8; SD_BLOCK_BYTES];
    build_db_btree_root_page(root_record_count, new_catalog.sequence, &mut root_page);
    write_file_sector(info.facts, info.entry, DB_BTREE_ROOT_PAGE, &root_page)?;

    let (_info2, _catalog2, new_root) = read_db_btree_root()?;
    let new_leaf = read_db_btree_leaf(info, new_root, leaf.page)?;
    let row = read_table_row(&block, slot).ok_or(DbError::KeyMissing)?;
    Ok((info, new_catalog, new_root, new_leaf, row))
}

fn get_db_btree_row(
    key: u32,
) -> Result<(DbFileInfo, DbCatalog, DbBtreeRoot, DbBtreeLeaf, DbRowValue), DbError> {
    let (info, catalog, root) = read_db_btree_root()?;
    let leaf_page = db_btree_leaf_page_for_key(root, key)?;
    read_file_sector(info.facts, info.entry, leaf_page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let leaf = parse_db_btree_leaf(&block);
    validate_db_btree_leaf(root, leaf)?;
    for slot in 0..leaf.record_count as usize {
        if table_record_key(&block, slot) == Some(key) {
            let row = read_table_row(&block, slot).ok_or(DbError::KeyMissing)?;
            return Ok((info, catalog, root, leaf, row));
        }
    }
    let (found_leaf, row) = find_db_btree_row_anywhere(info, root, key)?;
    Ok((info, catalog, root, found_leaf, row))
}

fn read_db_btree_leaf(
    info: DbFileInfo,
    root: DbBtreeRoot,
    page: u32,
) -> Result<DbBtreeLeaf, DbError> {
    let (leaf, _block) = read_db_btree_leaf_block(info, root, page)?;
    Ok(leaf)
}

fn read_db_btree_leaf_block(
    info: DbFileInfo,
    root: DbBtreeRoot,
    page: u32,
) -> Result<(DbBtreeLeaf, [u8; SD_BLOCK_BYTES]), DbError> {
    read_file_sector(info.facts, info.entry, page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let leaf = parse_db_btree_leaf(&block);
    validate_db_btree_leaf(root, leaf)?;
    Ok((leaf, block))
}

fn read_db_btree_leaf_raw(
    page: u32,
) -> Result<(DbFileInfo, DbCatalog, DbBtreeRoot, DbBtreeLeaf), DbError> {
    let (info, catalog, root) = read_db_btree_root()?;
    if page < root.leaf_start_page || page >= root.leaf_start_page.saturating_add(root.leaf_count) {
        return Err(DbError::PageOutOfRange);
    }
    read_file_sector(info.facts, info.entry, page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    Ok((info, catalog, root, parse_db_btree_leaf(&block)))
}

fn split_db_btree_leaf_insert(
    info: DbFileInfo,
    catalog: DbCatalog,
    root: DbBtreeRoot,
    leaf: DbBtreeLeaf,
    block: &[u8; SD_BLOCK_BYTES],
    key: u32,
    value: &[u8],
) -> Result<(DbBtreeRoot, DbBtreeLeaf, DbRowValue), DbError> {
    let split_page = find_empty_btree_leaf_page(info, root, leaf.page)?;
    let (split_leaf, _split_block) = read_db_btree_leaf_block(info, root, split_page)?;
    if split_leaf.record_count != 0 {
        return Err(DbError::BtreeFull);
    }

    let mut rows = [empty_db_row(); DB_BTREE_SPLIT_RECORDS];
    let mut inserted = false;
    let mut out_index = 0usize;
    for slot in 0..leaf.record_count as usize {
        let row = read_table_row(block, slot).ok_or(DbError::BadBtreeLeaf)?;
        if !inserted && key < row.key {
            rows[out_index] = db_row_from_parts(key, value)?;
            out_index += 1;
            inserted = true;
        }
        rows[out_index] = row;
        out_index += 1;
    }
    if !inserted {
        rows[out_index] = db_row_from_parts(key, value)?;
        out_index += 1;
    }
    if out_index != DB_BTREE_SPLIT_RECORDS {
        return Err(DbError::BadBtreeLeaf);
    }

    let left_count = DB_BTREE_SPLIT_RECORDS / 2;
    let right_count = DB_BTREE_SPLIT_RECORDS - left_count;
    let candidate_next = if split_page == leaf.next_leaf {
        split_leaf.next_leaf
    } else {
        leaf.next_leaf
    };
    let new_next = if candidate_next > split_page {
        candidate_next
    } else {
        0
    };

    let mut left_page = [0u8; SD_BLOCK_BYTES];
    write_btree_leaf_rows(
        &mut left_page,
        leaf.page,
        &rows,
        0,
        left_count,
        catalog.sequence,
        split_page,
    );
    write_file_sector(info.facts, info.entry, leaf.page, &left_page)?;

    let mut right_page = [0u8; SD_BLOCK_BYTES];
    write_btree_leaf_rows(
        &mut right_page,
        split_page,
        &rows,
        left_count,
        right_count,
        catalog.sequence,
        new_next,
    );
    write_file_sector(info.facts, info.entry, split_page, &right_page)?;

    let root_record_count = root.record_count.saturating_add(1);
    let mut root_page = [0u8; SD_BLOCK_BYTES];
    build_db_btree_root_page(root_record_count, catalog.sequence, &mut root_page);
    write_file_sector(info.facts, info.entry, DB_BTREE_ROOT_PAGE, &root_page)?;

    let (_info2, _catalog2, new_root) = read_db_btree_root()?;
    let (left_leaf, left_block) = read_db_btree_leaf_block(info, new_root, leaf.page)?;
    if let Some(row) = find_row_in_btree_leaf_block(&left_block, left_leaf, key) {
        return Ok((new_root, left_leaf, row));
    }
    let (right_leaf, right_block) = read_db_btree_leaf_block(info, new_root, split_page)?;
    let row =
        find_row_in_btree_leaf_block(&right_block, right_leaf, key).ok_or(DbError::KeyMissing)?;
    Ok((new_root, right_leaf, row))
}

fn find_empty_btree_leaf_page(
    info: DbFileInfo,
    root: DbBtreeRoot,
    after_page: u32,
) -> Result<u32, DbError> {
    let start = after_page.saturating_add(1).max(root.leaf_start_page);
    let end = root.leaf_start_page.saturating_add(root.leaf_count);
    let mut page = start;
    while page < end {
        let (leaf, _block) = read_db_btree_leaf_block(info, root, page)?;
        if leaf.record_count == 0 {
            return Ok(page);
        }
        page += 1;
    }
    Err(DbError::BtreeFull)
}

fn find_db_btree_row_anywhere(
    info: DbFileInfo,
    root: DbBtreeRoot,
    key: u32,
) -> Result<(DbBtreeLeaf, DbRowValue), DbError> {
    let end = root.leaf_start_page.saturating_add(root.leaf_count);
    let mut page = root.leaf_start_page;
    while page < end {
        let (leaf, block) = read_db_btree_leaf_block(info, root, page)?;
        if let Some(row) = find_row_in_btree_leaf_block(&block, leaf, key) {
            return Ok((leaf, row));
        }
        page += 1;
    }
    Err(DbError::KeyMissing)
}

fn find_db_btree_row_block_anywhere(
    info: DbFileInfo,
    root: DbBtreeRoot,
    key: u32,
) -> Result<(DbBtreeLeaf, [u8; SD_BLOCK_BYTES], usize), DbError> {
    let end = root.leaf_start_page.saturating_add(root.leaf_count);
    let mut page = root.leaf_start_page;
    while page < end {
        let (leaf, block) = read_db_btree_leaf_block(info, root, page)?;
        for slot in 0..leaf.record_count as usize {
            if table_record_key(&block, slot) == Some(key) {
                return Ok((leaf, block, slot));
            }
        }
        page += 1;
    }
    Err(DbError::KeyMissing)
}

fn update_existing_db_btree_row(
    info: DbFileInfo,
    catalog: DbCatalog,
    root: DbBtreeRoot,
    leaf: DbBtreeLeaf,
    mut block: [u8; SD_BLOCK_BYTES],
    slot: usize,
    key: u32,
    value: &[u8],
) -> Result<(DbFileInfo, DbCatalog, DbBtreeRoot, DbBtreeLeaf, DbRowValue), DbError> {
    let new_catalog = append_db_wal_event(info, catalog, DB_WAL_KIND_BPUT, key, value)?;
    write_table_record(&mut block, slot, key, value);
    let (min_key, max_key) = btree_leaf_key_range(&block, leaf.record_count);
    build_db_btree_leaf_header(
        &mut block,
        leaf.page,
        leaf.record_count,
        min_key,
        max_key,
        new_catalog.sequence,
        leaf.next_leaf,
    );
    write_file_sector(info.facts, info.entry, leaf.page, &block)?;

    let mut root_page = [0u8; SD_BLOCK_BYTES];
    build_db_btree_root_page(root.record_count, new_catalog.sequence, &mut root_page);
    write_file_sector(info.facts, info.entry, DB_BTREE_ROOT_PAGE, &root_page)?;

    let (_info2, _catalog2, new_root) = read_db_btree_root()?;
    let new_leaf = read_db_btree_leaf(info, new_root, leaf.page)?;
    let row = read_table_row(&block, slot).ok_or(DbError::KeyMissing)?;
    Ok((info, new_catalog, new_root, new_leaf, row))
}

fn scan_db_btree_rows(
    start_key: u32,
    limit: u32,
) -> Result<(DbFileInfo, DbCatalog, DbBtreeRoot, DbBtreeScan), DbError> {
    let (info, catalog, root) = read_db_btree_root()?;
    let requested_limit = limit;
    let limit = limit.clamp(1, DB_BTREE_SCAN_MAX_ROWS as u32) as usize;
    let mut scan = DbBtreeScan {
        start_key,
        requested_limit,
        row_count: 0,
        visited_leaves: 0,
        rows: [empty_db_row(); DB_BTREE_SCAN_MAX_ROWS],
    };
    let mut cursor = start_key;

    while scan.row_count < limit {
        let mut best = empty_db_row();
        let mut best_found = false;
        let end = root.leaf_start_page.saturating_add(root.leaf_count);
        let mut page = root.leaf_start_page;
        while page < end {
            let (leaf, block) = read_db_btree_leaf_block(info, root, page)?;
            scan.visited_leaves = scan.visited_leaves.saturating_add(1);
            for slot in 0..leaf.record_count as usize {
                let Some(row) = read_table_row(&block, slot) else {
                    continue;
                };
                if row.key < cursor {
                    continue;
                }
                if !best_found || row.key < best.key {
                    best = row;
                    best_found = true;
                }
            }
            page += 1;
        }
        if !best_found {
            break;
        }
        scan.rows[scan.row_count] = best;
        scan.row_count += 1;
        if best.key == u32::MAX {
            break;
        }
        cursor = best.key + 1;
    }

    Ok((info, catalog, root, scan))
}

fn find_row_in_btree_leaf_block(
    block: &[u8; SD_BLOCK_BYTES],
    leaf: DbBtreeLeaf,
    key: u32,
) -> Option<DbRowValue> {
    for slot in 0..leaf.record_count as usize {
        if table_record_key(block, slot) == Some(key) {
            return read_table_row(block, slot);
        }
    }
    None
}

fn write_btree_leaf_rows(
    out: &mut [u8; SD_BLOCK_BYTES],
    page: u32,
    rows: &[DbRowValue; DB_BTREE_SPLIT_RECORDS],
    start: usize,
    count: usize,
    sequence: u32,
    next_leaf: u32,
) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    for index in 0..count {
        let row = rows[start + index];
        write_table_record(out, index, row.key, &row.value[..row.value_len]);
    }
    let (min_key, max_key) = btree_leaf_key_range(out, count as u32);
    build_db_btree_leaf_header(
        out,
        page,
        count as u32,
        min_key,
        max_key,
        sequence,
        next_leaf,
    );
}

fn empty_db_row() -> DbRowValue {
    DbRowValue {
        key: 0,
        value_len: 0,
        value: [0u8; DB_TABLE_VALUE_BYTES],
    }
}

fn db_row_from_parts(key: u32, value: &[u8]) -> Result<DbRowValue, DbError> {
    if value.is_empty() || value.len() > DB_TABLE_VALUE_BYTES {
        return Err(DbError::BadValue);
    }
    let mut row = empty_db_row();
    row.key = key;
    row.value_len = value.len();
    row.value[..value.len()].copy_from_slice(value);
    Ok(row)
}

fn empty_db_event_page() -> DbEventPage {
    DbEventPage {
        initialized: false,
        version: 0,
        page: 0,
        sequence: 0,
        ticks: 0,
        kind: 0,
        source: 0,
        payload_len: 0,
        payload_crc: 0,
        flags: 0,
        crc_expected: 0,
        crc_actual: 0,
    }
}

fn read_db_event_meta() -> Result<(DbFileInfo, DbCatalog, DbEventMeta), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    read_file_sector(info.facts, info.entry, DB_EVENT_META_PAGE)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let meta = parse_db_event_meta(&block);
    validate_db_event_meta(meta)?;
    Ok((info, catalog, meta))
}

fn init_db_event_log() -> Result<(DbFileInfo, DbCatalog, DbEventMeta), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    if DB_EVENT_START_PAGE + DB_EVENT_PAGE_COUNT > catalog.page_count {
        return Err(DbError::PageOutOfRange);
    }
    let mut block = [0u8; SD_BLOCK_BYTES];
    let empty = [0u8; SD_BLOCK_BYTES];
    let mut page = DB_EVENT_START_PAGE;
    while page < DB_EVENT_START_PAGE + DB_EVENT_PAGE_COUNT {
        write_file_sector(info.facts, info.entry, page, &empty)?;
        page += 1;
    }
    build_db_event_meta_page(DB_EVENT_START_PAGE, 0, 0, &mut block);
    write_file_sector(info.facts, info.entry, DB_EVENT_META_PAGE, &block)?;
    read_db_event_meta()
}

fn append_db_event(
    kind: u32,
    source: u32,
    payload: &[u8],
) -> Result<(DbFileInfo, DbCatalog, DbEventMeta, DbEventPage), DbError> {
    if payload.is_empty() || payload.len() > DB_EVENT_PAYLOAD_BYTES {
        return Err(DbError::BadValue);
    }
    let (info, catalog, meta) = read_db_event_meta()?;
    let page = meta.next_page;
    let sequence = meta.sequence.wrapping_add(1);
    let mut event_page = [0u8; SD_BLOCK_BYTES];
    build_db_event_page(page, sequence, kind, source, payload, &mut event_page)?;
    write_file_sector(info.facts, info.entry, page, &event_page)?;

    let next_page = if page + 1 >= meta.start_page + meta.page_count {
        meta.start_page
    } else {
        page + 1
    };
    let event_count = meta.event_count.saturating_add(1).min(meta.page_count);
    let mut meta_page = [0u8; SD_BLOCK_BYTES];
    build_db_event_meta_page(next_page, event_count, sequence, &mut meta_page);
    write_file_sector(info.facts, info.entry, DB_EVENT_META_PAGE, &meta_page)?;

    let (_info2, _catalog2, new_meta) = read_db_event_meta()?;
    let event = read_db_event_page(info, new_meta, page)?;
    Ok((info, catalog, new_meta, event))
}

fn read_db_event_by_sequence(
    sequence: u32,
) -> Result<(DbFileInfo, DbCatalog, DbEventMeta, DbEventPage), DbError> {
    let (info, catalog, meta) = read_db_event_meta()?;
    let end = meta.start_page.saturating_add(meta.page_count);
    let mut page = meta.start_page;
    while page < end {
        if let Ok(event) = read_db_event_page(info, meta, page) {
            if event.sequence == sequence {
                return Ok((info, catalog, meta, event));
            }
        }
        page += 1;
    }
    Err(DbError::KeyMissing)
}

fn read_db_event_payload_by_sequence(
    sequence: u32,
    payload: &mut [u8; DB_EVENT_PAYLOAD_BYTES],
) -> Result<(DbFileInfo, DbCatalog, DbEventMeta, DbEventPage, usize), DbError> {
    let (info, catalog, meta, event) = read_db_event_by_sequence(sequence)?;
    read_file_sector(info.facts, info.entry, event.page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let parsed = parse_db_event_page(&block);
    validate_db_event_page(meta, parsed, &block)?;
    if parsed.sequence != sequence {
        return Err(DbError::BadEventPage);
    }
    let payload_len = parsed.payload_len;
    payload[..payload_len]
        .copy_from_slice(&block[DB_EVENT_PAYLOAD_OFFSET..DB_EVENT_PAYLOAD_OFFSET + payload_len]);
    Ok((info, catalog, meta, parsed, payload_len))
}

fn list_db_events(
    start_sequence: u32,
    limit: u32,
) -> Result<(DbFileInfo, DbCatalog, DbEventMeta, DbEventList), DbError> {
    let (info, catalog, meta) = read_db_event_meta()?;
    let requested_limit = limit;
    let limit = limit.clamp(1, DB_EVENT_LIST_MAX_ROWS as u32) as usize;
    let mut list = DbEventList {
        start_sequence,
        requested_limit,
        row_count: 0,
        scanned_pages: 0,
        rows: [empty_db_event_page(); DB_EVENT_LIST_MAX_ROWS],
    };
    let mut cursor = start_sequence;
    while list.row_count < limit {
        let mut best = empty_db_event_page();
        let mut best_found = false;
        let end = meta.start_page.saturating_add(meta.page_count);
        let mut page = meta.start_page;
        while page < end {
            list.scanned_pages = list.scanned_pages.saturating_add(1);
            if let Ok(event) = read_db_event_page(info, meta, page) {
                if event.sequence >= cursor && (!best_found || event.sequence < best.sequence) {
                    best = event;
                    best_found = true;
                }
            }
            page += 1;
        }
        if !best_found {
            break;
        }
        list.rows[list.row_count] = best;
        list.row_count += 1;
        if best.sequence == u32::MAX {
            break;
        }
        cursor = best.sequence + 1;
    }
    Ok((info, catalog, meta, list))
}

fn read_db_event_page(
    info: DbFileInfo,
    meta: DbEventMeta,
    page: u32,
) -> Result<DbEventPage, DbError> {
    if page < meta.start_page || page >= meta.start_page.saturating_add(meta.page_count) {
        return Err(DbError::PageOutOfRange);
    }
    read_file_sector(info.facts, info.entry, page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let event = parse_db_event_page(&block);
    validate_db_event_page(meta, event, &block)?;
    Ok(event)
}

fn read_db_blob_meta() -> Result<(DbFileInfo, DbCatalog, DbBlobMeta), DbError> {
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    if DB_BLOB_START_PAGE >= catalog.page_count {
        return Err(DbError::PageOutOfRange);
    }
    read_file_sector(info.facts, info.entry, DB_BLOB_META_PAGE)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let meta = parse_db_blob_meta(&block);
    validate_db_blob_meta(catalog, meta)?;
    Ok((info, catalog, meta))
}

fn init_db_blob_import(
    expected_size: u32,
    expected_crc: u32,
) -> Result<(DbFileInfo, DbCatalog, DbBlobMeta), DbError> {
    if expected_size == 0 {
        return Err(DbError::BadValue);
    }
    let (info, catalog) = read_db_catalog()?;
    validate_db_catalog(catalog)?;
    let page_count = db_blob_page_count(catalog)?;
    let capacity_bytes = page_count.saturating_mul(DB_PAGE_BYTES);
    if expected_size > capacity_bytes {
        return Err(DbError::BlobTooLarge);
    }
    let sequence = read_db_blob_meta()
        .map(|(_info, _catalog, meta)| meta.sequence.wrapping_add(1))
        .unwrap_or(1);
    let mut block = [0u8; SD_BLOCK_BYTES];
    build_db_blob_meta_page(
        page_count,
        capacity_bytes,
        expected_size,
        0,
        expected_crc,
        0,
        sequence,
        DB_BLOB_FLAG_ACTIVE,
        &mut block,
    );
    write_file_sector(info.facts, info.entry, DB_BLOB_META_PAGE, &block)?;
    read_db_blob_meta()
}

fn write_db_blob_chunk(offset: u32, data: &[u8]) -> Result<DbBlobMeta, DbError> {
    if data.is_empty() || data.len() > DB_BLOB_PACKET_BYTES {
        return Err(DbError::BadValue);
    }
    if offset as usize % SD_BLOCK_BYTES != 0 {
        return Err(DbError::BadBlobOffset);
    }
    let (info, catalog, meta) = read_db_blob_meta()?;
    if meta.flags & DB_BLOB_FLAG_ACTIVE == 0 {
        return Err(DbError::BlobMissing);
    }
    if offset != meta.received_size {
        return Err(DbError::BadBlobOffset);
    }
    let new_received = offset
        .checked_add(data.len() as u32)
        .ok_or(DbError::BlobTooLarge)?;
    if new_received > meta.expected_size {
        return Err(DbError::BlobTooLarge);
    }
    let page_index = offset / DB_PAGE_BYTES;
    if page_index >= meta.page_count {
        return Err(DbError::PageOutOfRange);
    }
    let mut block = [0u8; SD_BLOCK_BYTES];
    block[..data.len()].copy_from_slice(data);
    write_file_sector(
        info.facts,
        info.entry,
        meta.start_page.saturating_add(page_index),
        &block,
    )?;
    let mut meta_block = [0u8; SD_BLOCK_BYTES];
    build_db_blob_meta_page(
        meta.page_count,
        meta.capacity_bytes,
        meta.expected_size,
        new_received,
        meta.expected_crc,
        0,
        meta.sequence,
        DB_BLOB_FLAG_ACTIVE,
        &mut meta_block,
    );
    write_file_sector(info.facts, info.entry, DB_BLOB_META_PAGE, &meta_block)?;
    let (_info2, _catalog2, updated) = read_db_blob_meta()?;
    validate_db_blob_meta(catalog, updated)?;
    Ok(updated)
}

fn db_blob_fast_lba_base(info: DbFileInfo, meta: DbBlobMeta) -> Option<u32> {
    let target = FatDirTarget {
        name: DB_FILE_NAME_STR,
        found: true,
        cluster: info.entry.cluster,
        size: info.entry.size,
    };
    let chain = scan_fat_chain(info.facts, target).ok()?;
    if !chain.ok || !chain.contiguous {
        return None;
    }
    Some(cluster_to_lba(info.facts, info.entry.cluster).saturating_add(meta.start_page))
}

fn write_db_blob_chunk_fast(
    offset: u32,
    data: &[u8],
    fast_lba_base: u32,
    page_count: u32,
    expected_size: u32,
) -> Result<u32, DbError> {
    if data.is_empty() || data.len() > DB_BLOB_PACKET_BYTES {
        return Err(DbError::BadValue);
    }
    if offset as usize % SD_BLOCK_BYTES != 0 {
        return Err(DbError::BadBlobOffset);
    }
    let new_received = offset
        .checked_add(data.len() as u32)
        .ok_or(DbError::BlobTooLarge)?;
    if new_received > expected_size {
        return Err(DbError::BlobTooLarge);
    }
    let page_index = offset / DB_PAGE_BYTES;
    if page_index >= page_count {
        return Err(DbError::PageOutOfRange);
    }
    let mut block = [0u8; SD_BLOCK_BYTES];
    block[..data.len()].copy_from_slice(data);
    unsafe {
        core::ptr::copy_nonoverlapping(
            block.as_ptr(),
            addr_of_mut!(SD_WRITE_BUFFER.0).cast::<u8>(),
            SD_BLOCK_BYTES,
        );
    }
    sd_write_lba(fast_lba_base.saturating_add(page_index)).map_err(|error| match error {
        SdReadError::StatusError(_) | SdReadError::IdmacError(_) => DbError::Sd(error),
        _ => DbError::WriteFailed,
    })?;
    Ok(new_received)
}

fn write_db_blob_chunk_sequential(
    offset: u32,
    data: &[u8],
    expected_size: u32,
) -> Result<u32, DbError> {
    if data.is_empty() || data.len() > DB_BLOB_PACKET_BYTES {
        return Err(DbError::BadValue);
    }
    if offset as usize % SD_BLOCK_BYTES != 0 {
        return Err(DbError::BadBlobOffset);
    }
    let new_received = offset
        .checked_add(data.len() as u32)
        .ok_or(DbError::BlobTooLarge)?;
    if new_received > expected_size {
        return Err(DbError::BlobTooLarge);
    }
    let expected_sector = DB_BLOB_START_PAGE.saturating_add(offset / DB_PAGE_BYTES);
    let (facts, sector_index, lba, cluster, sector_in_cluster) = unsafe {
        (
            DB_BLOB_IMPORT_STATE.seq_facts,
            DB_BLOB_IMPORT_STATE.seq_sector_index,
            DB_BLOB_IMPORT_STATE.seq_lba_current,
            DB_BLOB_IMPORT_STATE.seq_cluster,
            DB_BLOB_IMPORT_STATE.seq_sector_in_cluster,
        )
    };
    if sector_index != expected_sector || lba == 0 || cluster < 2 {
        return Err(DbError::BadBlobOffset);
    }

    let mut block = [0u8; SD_BLOCK_BYTES];
    block[..data.len()].copy_from_slice(data);
    unsafe {
        core::ptr::copy_nonoverlapping(
            block.as_ptr(),
            addr_of_mut!(SD_WRITE_BUFFER.0).cast::<u8>(),
            SD_BLOCK_BYTES,
        );
    }
    sd_write_lba(lba).map_err(|error| match error {
        SdReadError::StatusError(_) | SdReadError::IdmacError(_) => DbError::Sd(error),
        _ => DbError::WriteFailed,
    })?;

    let spc = facts.sectors_per_cluster as u32;
    if spc == 0 {
        return Err(DbError::Sd(SdReadError::InvalidLayout));
    }
    let mut next_cluster = cluster;
    let mut next_sector_in_cluster = sector_in_cluster.saturating_add(1);
    let mut next_lba = lba.saturating_add(1);
    if next_sector_in_cluster >= spc {
        let mut cached_fat_lba = u32::MAX;
        let mut cached_fat = [0u8; SD_BLOCK_BYTES];
        next_cluster = read_fat_entry(facts, cluster, &mut cached_fat_lba, &mut cached_fat)
            .map_err(DbError::Sd)?;
        if next_cluster >= 0x0fff_fff8 {
            next_lba = 0;
        } else if next_cluster < 2 || next_cluster >= 0x0fff_fff7 {
            return Err(DbError::BadChain);
        } else {
            next_lba = cluster_to_lba(facts, next_cluster);
        }
        next_sector_in_cluster = 0;
    }

    unsafe {
        DB_BLOB_IMPORT_STATE.seq_sector_index = sector_index.saturating_add(1);
        DB_BLOB_IMPORT_STATE.seq_lba_current = next_lba;
        DB_BLOB_IMPORT_STATE.seq_cluster = next_cluster;
        DB_BLOB_IMPORT_STATE.seq_sector_in_cluster = next_sector_in_cluster;
        DB_BLOB_IMPORT_STATE.received = new_received;
    }
    Ok(new_received)
}

fn mark_db_blob_import_received(
    received_size: u32,
    actual_crc: u32,
) -> Result<DbBlobMeta, DbError> {
    let (info, _catalog, meta) = read_db_blob_meta()?;
    let mut meta_block = [0u8; SD_BLOCK_BYTES];
    build_db_blob_meta_page(
        meta.page_count,
        meta.capacity_bytes,
        meta.expected_size,
        received_size,
        meta.expected_crc,
        actual_crc,
        meta.sequence,
        DB_BLOB_FLAG_ACTIVE,
        &mut meta_block,
    );
    write_file_sector(info.facts, info.entry, DB_BLOB_META_PAGE, &meta_block)?;
    let (_info2, _catalog2, updated) = read_db_blob_meta()?;
    Ok(updated)
}

fn update_db_blob_import_crc(data: &[u8], new_received: u32) -> u32 {
    unsafe {
        let mut crc = DB_BLOB_IMPORT_STATE.rolling_crc;
        for &byte in data {
            crc = crc32_update(crc, byte);
        }
        DB_BLOB_IMPORT_STATE.rolling_crc = crc;
        DB_BLOB_IMPORT_STATE.received = new_received;
        !crc
    }
}

fn commit_db_blob_import(actual_crc_hint: Option<u32>) -> Result<DbBlobMeta, DbError> {
    let (info, _catalog, meta) = read_db_blob_meta()?;
    if meta.flags & DB_BLOB_FLAG_ACTIVE == 0 {
        return Err(DbError::BlobMissing);
    }
    if meta.received_size != meta.expected_size {
        return Err(DbError::BadBlobOffset);
    }
    let actual_crc = match actual_crc_hint {
        Some(crc) => crc,
        None => db_blob_crc(info, meta)?,
    };
    if actual_crc != meta.expected_crc {
        return Err(DbError::BadBlobMeta);
    }
    let mut block = [0u8; SD_BLOCK_BYTES];
    build_db_blob_meta_page(
        meta.page_count,
        meta.capacity_bytes,
        meta.expected_size,
        meta.received_size,
        meta.expected_crc,
        actual_crc,
        meta.sequence,
        DB_BLOB_FLAG_COMMITTED,
        &mut block,
    );
    write_file_sector(info.facts, info.entry, DB_BLOB_META_PAGE, &block)?;
    let (_info2, _catalog2, committed) = read_db_blob_meta()?;
    Ok(committed)
}

fn db_blob_crc(info: DbFileInfo, meta: DbBlobMeta) -> Result<u32, DbError> {
    let mut crc = 0xffff_ffffu32;
    let mut remaining = meta.expected_size as usize;
    let mut page = 0u32;
    let fast_lba_base = db_blob_fast_lba_base(info, meta);
    let mut seq = if fast_lba_base.is_none() {
        file_sector_location(info.facts, info.entry, meta.start_page).ok()
    } else {
        None
    };
    let mut cached_fat_lba = u32::MAX;
    let mut cached_fat = [0u8; SD_BLOCK_BYTES];
    while remaining > 0 {
        if page >= meta.page_count {
            return Err(DbError::PageOutOfRange);
        }
        if let Some(base) = fast_lba_base {
            sd_read_lba(base.saturating_add(page)).map_err(DbError::Sd)?;
        } else if let Some((lba, cluster, sector_in_cluster)) = seq {
            if lba == 0 || cluster < 2 {
                return Err(DbError::BadChain);
            }
            sd_read_lba(lba).map_err(DbError::Sd)?;

            let spc = info.facts.sectors_per_cluster as u32;
            if spc == 0 {
                return Err(DbError::Sd(SdReadError::InvalidLayout));
            }
            let mut next_cluster = cluster;
            let mut next_sector_in_cluster = sector_in_cluster.saturating_add(1);
            let mut next_lba = lba.saturating_add(1);
            if next_sector_in_cluster >= spc {
                next_cluster =
                    read_fat_entry(info.facts, cluster, &mut cached_fat_lba, &mut cached_fat)
                        .map_err(DbError::Sd)?;
                if next_cluster >= 0x0fff_fff8 {
                    next_lba = 0;
                } else if next_cluster < 2 || next_cluster >= 0x0fff_fff7 {
                    return Err(DbError::BadChain);
                } else {
                    next_lba = cluster_to_lba(info.facts, next_cluster);
                }
                next_sector_in_cluster = 0;
            }
            seq = Some((next_lba, next_cluster, next_sector_in_cluster));
        } else {
            read_file_sector(info.facts, info.entry, meta.start_page.saturating_add(page))?;
        }
        let mut block = [0u8; SD_BLOCK_BYTES];
        unsafe {
            core::ptr::copy_nonoverlapping(
                addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
                block.as_mut_ptr(),
                SD_BLOCK_BYTES,
            );
        }
        let take = remaining.min(SD_BLOCK_BYTES);
        for &byte in &block[..take] {
            crc = crc32_update(crc, byte);
        }
        remaining -= take;
        page += 1;
    }
    Ok(!crc)
}

fn read_db_blob_chunk(
    offset: u32,
    len: usize,
) -> Result<(DbBlobMeta, [u8; SD_BLOCK_BYTES], usize), DbError> {
    if len == 0 || len > DB_BLOB_PACKET_BYTES {
        return Err(DbError::BadValue);
    }
    if offset as usize % SD_BLOCK_BYTES != 0 {
        return Err(DbError::BadBlobOffset);
    }
    let (info, _catalog, meta) = read_db_blob_meta()?;
    if meta.flags & DB_BLOB_FLAG_COMMITTED == 0 {
        return Err(DbError::BlobMissing);
    }
    if offset >= meta.expected_size {
        return Err(DbError::BadBlobOffset);
    }
    let remaining = meta.expected_size - offset;
    let take = (remaining as usize).min(len);
    let page_index = offset / DB_PAGE_BYTES;
    if page_index >= meta.page_count {
        return Err(DbError::PageOutOfRange);
    }
    if let Some(base) = db_blob_fast_lba_base(info, meta) {
        sd_read_lba(base.saturating_add(page_index)).map_err(DbError::Sd)?;
    } else {
        read_file_sector(
            info.facts,
            info.entry,
            meta.start_page.saturating_add(page_index),
        )?;
    }
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    Ok((meta, block, take))
}

fn db_blob_page_count(catalog: DbCatalog) -> Result<u32, DbError> {
    if DB_BLOB_START_PAGE >= catalog.page_count {
        return Err(DbError::PageOutOfRange);
    }
    Ok(catalog.page_count - DB_BLOB_START_PAGE)
}

fn build_db_superblock(file_bytes: u32, out: &mut [u8; SD_BLOCK_BYTES]) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    out[..DB_MAGIC.len()].copy_from_slice(DB_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, DB_PAGE_BYTES);
    write_le32(out, 16, file_bytes / DB_PAGE_BYTES);
    write_le32(out, 20, file_bytes);
    write_le32(out, 24, DB_ROOT_CATALOG_PAGE);
    write_le32(out, 28, 0);
    let mut len = 64usize;
    let _ = append_bytes(out, &mut len, b"XiaojinDB page store\n");
    let crc = db_superblock_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
}

fn build_db_catalog(
    page_count: u32,
    wal_next_page: u32,
    sequence: u32,
    table_root_page: u32,
    out: &mut [u8; SD_BLOCK_BYTES],
) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    out[..DB_CATALOG_MAGIC.len()].copy_from_slice(DB_CATALOG_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, DB_PAGE_BYTES);
    write_le32(out, 16, page_count);
    write_le32(out, 20, DB_WAL_START_PAGE);
    write_le32(out, 24, DB_WAL_PAGE_COUNT);
    write_le32(out, 28, wal_next_page);
    write_le32(out, 32, DB_DATA_START_PAGE);
    write_le32(out, 36, table_root_page);
    write_le32(out, 40, sequence);
    write_le32(out, 44, 0);
    let mut len = 64usize;
    let _ = append_bytes(out, &mut len, b"XiaojinDB catalog\n");
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
}

fn build_db_wal_page(
    page: u32,
    sequence: u32,
    kind: u32,
    key: u32,
    value: &[u8],
    out: &mut [u8; SD_BLOCK_BYTES],
) -> Result<(), DbError> {
    if value.len() > DB_TABLE_VALUE_BYTES {
        return Err(DbError::BadValue);
    }
    for byte in out.iter_mut() {
        *byte = 0;
    }
    let ticks = read_cntpct();
    out[..DB_WAL_MAGIC.len()].copy_from_slice(DB_WAL_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, page);
    write_le32(out, 16, sequence);
    write_le32(out, 20, ticks as u32);
    write_le32(out, 24, (ticks >> 32) as u32);
    write_le32(out, 28, kind);
    write_le32(out, 32, key);
    write_le32(out, 36, value.len() as u32);
    out[40..40 + value.len()].copy_from_slice(value);
    let mut len = 64usize;
    let _ = append_bytes(out, &mut len, b"XiaojinDB WAL test record\n");
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
    Ok(())
}

fn build_db_table_page(record_count: u32, sequence: u32, out: &mut [u8; SD_BLOCK_BYTES]) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    update_table_header(out, record_count, sequence, DB_TABLE_RECORDS);
}

fn update_table_header(
    out: &mut [u8; SD_BLOCK_BYTES],
    record_count: u32,
    sequence: u32,
    capacity: u32,
) {
    out[..DB_TABLE_MAGIC.len()].copy_from_slice(DB_TABLE_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, DB_TABLE_ROOT_PAGE);
    write_le32(out, 16, record_count);
    write_le32(out, 20, capacity);
    write_le32(out, 24, sequence);
    write_le32(out, 28, 0);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, 0);
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
}

fn build_db_btree_root_page(record_count: u32, sequence: u32, out: &mut [u8; SD_BLOCK_BYTES]) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    out[..DB_BTREE_MAGIC.len()].copy_from_slice(DB_BTREE_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, DB_BTREE_ROOT_PAGE);
    write_le32(out, 16, DB_BTREE_LEAF_START_PAGE);
    write_le32(out, 20, DB_BTREE_LEAF_COUNT);
    write_le32(out, 24, DB_TABLE_RECORDS);
    write_le32(out, 28, record_count);
    write_le32(out, 32, sequence);
    write_le32(out, 36, 0);
    let mut len = 64usize;
    let _ = append_bytes(out, &mut len, b"XiaojinDB experimental B+tree root\n");
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
}

fn build_db_btree_leaf_page(
    page: u32,
    record_count: u32,
    min_key: u32,
    max_key: u32,
    sequence: u32,
    next_leaf: u32,
    out: &mut [u8; SD_BLOCK_BYTES],
) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    build_db_btree_leaf_header(
        out,
        page,
        record_count,
        min_key,
        max_key,
        sequence,
        next_leaf,
    );
}

fn build_db_btree_leaf_header(
    out: &mut [u8; SD_BLOCK_BYTES],
    page: u32,
    record_count: u32,
    min_key: u32,
    max_key: u32,
    sequence: u32,
    next_leaf: u32,
) {
    out[..DB_BTREE_LEAF_MAGIC.len()].copy_from_slice(DB_BTREE_LEAF_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, page);
    write_le32(out, 16, record_count);
    write_le32(out, 20, DB_TABLE_RECORDS);
    write_le32(out, 24, min_key);
    write_le32(out, 28, max_key);
    write_le32(out, 32, sequence);
    write_le32(out, 36, next_leaf);
    write_le32(out, 40, 0);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, 0);
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
}

fn build_db_event_meta_page(
    next_page: u32,
    event_count: u32,
    sequence: u32,
    out: &mut [u8; SD_BLOCK_BYTES],
) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    out[..DB_EVENT_META_MAGIC.len()].copy_from_slice(DB_EVENT_META_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, DB_EVENT_META_PAGE);
    write_le32(out, 16, DB_EVENT_START_PAGE);
    write_le32(out, 20, DB_EVENT_PAGE_COUNT);
    write_le32(out, 24, next_page);
    write_le32(out, 28, event_count);
    write_le32(out, 32, sequence);
    write_le32(out, 36, 0);
    let mut len = 64usize;
    let _ = append_bytes(out, &mut len, b"XiaojinDB router event log\n");
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
}

fn build_db_event_page(
    page: u32,
    sequence: u32,
    kind: u32,
    source: u32,
    payload: &[u8],
    out: &mut [u8; SD_BLOCK_BYTES],
) -> Result<(), DbError> {
    if payload.is_empty() || payload.len() > DB_EVENT_PAYLOAD_BYTES {
        return Err(DbError::BadValue);
    }
    for byte in out.iter_mut() {
        *byte = 0;
    }
    let ticks = read_cntpct();
    let payload_crc = db_payload_crc(payload);
    out[..DB_EVENT_PAGE_MAGIC.len()].copy_from_slice(DB_EVENT_PAGE_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, page);
    write_le32(out, 16, sequence);
    write_le32(out, 20, ticks as u32);
    write_le32(out, 24, (ticks >> 32) as u32);
    write_le32(out, 28, kind);
    write_le32(out, 32, source);
    write_le32(out, 36, payload.len() as u32);
    write_le32(out, 40, payload_crc);
    write_le32(out, 44, 0);
    out[DB_EVENT_PAYLOAD_OFFSET..DB_EVENT_PAYLOAD_OFFSET + payload.len()].copy_from_slice(payload);
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
    Ok(())
}

fn build_db_blob_meta_page(
    page_count: u32,
    capacity_bytes: u32,
    expected_size: u32,
    received_size: u32,
    expected_crc: u32,
    committed_crc: u32,
    sequence: u32,
    flags: u32,
    out: &mut [u8; SD_BLOCK_BYTES],
) {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    let ticks = read_cntpct();
    out[..DB_BLOB_META_MAGIC.len()].copy_from_slice(DB_BLOB_META_MAGIC);
    write_le32(out, 8, DB_FORMAT_VERSION);
    write_le32(out, 12, DB_BLOB_META_PAGE);
    write_le32(out, 16, DB_BLOB_START_PAGE);
    write_le32(out, 20, page_count);
    write_le32(out, 24, capacity_bytes);
    write_le32(out, 28, expected_size);
    write_le32(out, 32, received_size);
    write_le32(out, 36, expected_crc);
    write_le32(out, 40, committed_crc);
    write_le32(out, 44, sequence);
    write_le32(out, 48, flags);
    write_le32(out, 52, ticks as u32);
    write_le32(out, 56, (ticks >> 32) as u32);
    let mut len = 64usize;
    let _ = append_bytes(out, &mut len, b"XiaojinDB PostgreSQL backup blob\n");
    let crc = db_page_crc(out);
    write_le32(out, DB_SUPERBLOCK_CRC_OFFSET, crc);
}

fn parse_db_superblock(block: &[u8; SD_BLOCK_BYTES]) -> DbSuperblock {
    let initialized = &block[..DB_MAGIC.len()] == DB_MAGIC;
    DbSuperblock {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        page_bytes: read_le32(block, 12).unwrap_or(0),
        page_count: read_le32(block, 16).unwrap_or(0),
        file_bytes: read_le32(block, 20).unwrap_or(0),
        root_page: read_le32(block, 24).unwrap_or(0),
        flags: read_le32(block, 28).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_superblock_crc(block),
    }
}

fn parse_db_catalog(block: &[u8; SD_BLOCK_BYTES]) -> DbCatalog {
    let initialized = &block[..DB_CATALOG_MAGIC.len()] == DB_CATALOG_MAGIC;
    DbCatalog {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        page_bytes: read_le32(block, 12).unwrap_or(0),
        page_count: read_le32(block, 16).unwrap_or(0),
        wal_start_page: read_le32(block, 20).unwrap_or(0),
        wal_page_count: read_le32(block, 24).unwrap_or(0),
        wal_next_page: read_le32(block, 28).unwrap_or(0),
        data_start_page: read_le32(block, 32).unwrap_or(0),
        table_root_page: read_le32(block, 36).unwrap_or(0),
        sequence: read_le32(block, 40).unwrap_or(0),
        flags: read_le32(block, 44).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_page_crc(block),
    }
}

fn parse_db_table(block: &[u8; SD_BLOCK_BYTES]) -> DbTablePage {
    let initialized = &block[..DB_TABLE_MAGIC.len()] == DB_TABLE_MAGIC;
    DbTablePage {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        root_page: read_le32(block, 12).unwrap_or(0),
        record_count: read_le32(block, 16).unwrap_or(0),
        capacity: read_le32(block, 20).unwrap_or(0),
        sequence: read_le32(block, 24).unwrap_or(0),
        flags: read_le32(block, 28).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_page_crc(block),
    }
}

fn parse_db_btree_root(block: &[u8; SD_BLOCK_BYTES]) -> DbBtreeRoot {
    let initialized = &block[..DB_BTREE_MAGIC.len()] == DB_BTREE_MAGIC;
    DbBtreeRoot {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        root_page: read_le32(block, 12).unwrap_or(0),
        leaf_start_page: read_le32(block, 16).unwrap_or(0),
        leaf_count: read_le32(block, 20).unwrap_or(0),
        leaf_capacity: read_le32(block, 24).unwrap_or(0),
        record_count: read_le32(block, 28).unwrap_or(0),
        sequence: read_le32(block, 32).unwrap_or(0),
        flags: read_le32(block, 36).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_page_crc(block),
    }
}

fn parse_db_btree_leaf(block: &[u8; SD_BLOCK_BYTES]) -> DbBtreeLeaf {
    let initialized = &block[..DB_BTREE_LEAF_MAGIC.len()] == DB_BTREE_LEAF_MAGIC;
    DbBtreeLeaf {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        page: read_le32(block, 12).unwrap_or(0),
        record_count: read_le32(block, 16).unwrap_or(0),
        capacity: read_le32(block, 20).unwrap_or(0),
        min_key: read_le32(block, 24).unwrap_or(0),
        max_key: read_le32(block, 28).unwrap_or(0),
        sequence: read_le32(block, 32).unwrap_or(0),
        next_leaf: read_le32(block, 36).unwrap_or(0),
        flags: read_le32(block, 40).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_page_crc(block),
    }
}

fn parse_db_event_meta(block: &[u8; SD_BLOCK_BYTES]) -> DbEventMeta {
    let initialized = &block[..DB_EVENT_META_MAGIC.len()] == DB_EVENT_META_MAGIC;
    DbEventMeta {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        meta_page: read_le32(block, 12).unwrap_or(0),
        start_page: read_le32(block, 16).unwrap_or(0),
        page_count: read_le32(block, 20).unwrap_or(0),
        next_page: read_le32(block, 24).unwrap_or(0),
        event_count: read_le32(block, 28).unwrap_or(0),
        sequence: read_le32(block, 32).unwrap_or(0),
        flags: read_le32(block, 36).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_page_crc(block),
    }
}

fn parse_db_event_page(block: &[u8; SD_BLOCK_BYTES]) -> DbEventPage {
    let initialized = &block[..DB_EVENT_PAGE_MAGIC.len()] == DB_EVENT_PAGE_MAGIC;
    let ticks_low = read_le32(block, 20).unwrap_or(0) as u64;
    let ticks_high = read_le32(block, 24).unwrap_or(0) as u64;
    DbEventPage {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        page: read_le32(block, 12).unwrap_or(0),
        sequence: read_le32(block, 16).unwrap_or(0),
        ticks: ticks_low | (ticks_high << 32),
        kind: read_le32(block, 28).unwrap_or(0),
        source: read_le32(block, 32).unwrap_or(0),
        payload_len: read_le32(block, 36).unwrap_or(0) as usize,
        payload_crc: read_le32(block, 40).unwrap_or(0),
        flags: read_le32(block, 44).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_page_crc(block),
    }
}

fn parse_db_blob_meta(block: &[u8; SD_BLOCK_BYTES]) -> DbBlobMeta {
    let initialized = &block[..DB_BLOB_META_MAGIC.len()] == DB_BLOB_META_MAGIC;
    DbBlobMeta {
        initialized,
        version: read_le32(block, 8).unwrap_or(0),
        meta_page: read_le32(block, 12).unwrap_or(0),
        start_page: read_le32(block, 16).unwrap_or(0),
        page_count: read_le32(block, 20).unwrap_or(0),
        capacity_bytes: read_le32(block, 24).unwrap_or(0),
        expected_size: read_le32(block, 28).unwrap_or(0),
        received_size: read_le32(block, 32).unwrap_or(0),
        expected_crc: read_le32(block, 36).unwrap_or(0),
        committed_crc: read_le32(block, 40).unwrap_or(0),
        sequence: read_le32(block, 44).unwrap_or(0),
        flags: read_le32(block, 48).unwrap_or(0),
        crc_expected: read_le32(block, DB_SUPERBLOCK_CRC_OFFSET).unwrap_or(0),
        crc_actual: db_page_crc(block),
    }
}

fn validate_db_superblock(superblock: DbSuperblock) -> Result<(), DbError> {
    if !superblock.initialized
        || superblock.version != DB_FORMAT_VERSION
        || superblock.page_bytes != DB_PAGE_BYTES
        || superblock.root_page != DB_ROOT_CATALOG_PAGE
        || superblock.page_count <= DB_DATA_START_PAGE
        || superblock.crc_expected != superblock.crc_actual
    {
        return Err(DbError::BadSuperblock);
    }
    Ok(())
}

fn validate_db_catalog(catalog: DbCatalog) -> Result<(), DbError> {
    if !catalog.initialized
        || catalog.version != DB_FORMAT_VERSION
        || catalog.page_bytes != DB_PAGE_BYTES
        || catalog.wal_start_page != DB_WAL_START_PAGE
        || catalog.wal_page_count != DB_WAL_PAGE_COUNT
        || catalog.data_start_page != DB_DATA_START_PAGE
        || catalog.wal_next_page < catalog.wal_start_page
        || catalog.wal_next_page >= catalog.wal_start_page + catalog.wal_page_count
        || catalog.page_count <= catalog.data_start_page
        || catalog.crc_expected != catalog.crc_actual
    {
        return Err(DbError::BadCatalog);
    }
    Ok(())
}

fn validate_db_table(table: DbTablePage) -> Result<(), DbError> {
    if !table.initialized
        || table.version != DB_FORMAT_VERSION
        || table.root_page != DB_TABLE_ROOT_PAGE
        || table.capacity != DB_TABLE_RECORDS
        || table.record_count > table.capacity
        || table.crc_expected != table.crc_actual
    {
        return Err(DbError::BadTable);
    }
    Ok(())
}

fn validate_db_btree_root(root: DbBtreeRoot) -> Result<(), DbError> {
    if !root.initialized {
        return Err(DbError::BtreeMissing);
    }
    if root.version != DB_FORMAT_VERSION
        || root.root_page != DB_BTREE_ROOT_PAGE
        || root.leaf_start_page != DB_BTREE_LEAF_START_PAGE
        || root.leaf_count != DB_BTREE_LEAF_COUNT
        || root.leaf_capacity != DB_TABLE_RECORDS
        || root.record_count > root.leaf_count.saturating_mul(root.leaf_capacity)
        || root.crc_expected != root.crc_actual
    {
        return Err(DbError::BadBtreeRoot);
    }
    Ok(())
}

fn validate_db_btree_leaf(root: DbBtreeRoot, leaf: DbBtreeLeaf) -> Result<(), DbError> {
    if !leaf.initialized {
        return Err(DbError::BtreeMissing);
    }
    let leaf_end = root.leaf_start_page.saturating_add(root.leaf_count);
    if leaf.version != DB_FORMAT_VERSION
        || leaf.page < root.leaf_start_page
        || leaf.page >= leaf_end
        || leaf.capacity != root.leaf_capacity
        || leaf.record_count > leaf.capacity
        || (leaf.next_leaf != 0
            && (leaf.next_leaf <= leaf.page
                || leaf.next_leaf < root.leaf_start_page
                || leaf.next_leaf >= leaf_end))
        || leaf.crc_expected != leaf.crc_actual
    {
        return Err(DbError::BadBtreeLeaf);
    }
    Ok(())
}

fn validate_db_event_meta(meta: DbEventMeta) -> Result<(), DbError> {
    if !meta.initialized {
        return Err(DbError::EventMissing);
    }
    if meta.version != DB_FORMAT_VERSION
        || meta.meta_page != DB_EVENT_META_PAGE
        || meta.start_page != DB_EVENT_START_PAGE
        || meta.page_count != DB_EVENT_PAGE_COUNT
        || meta.next_page < meta.start_page
        || meta.next_page >= meta.start_page + meta.page_count
        || meta.event_count > meta.page_count
        || meta.crc_expected != meta.crc_actual
    {
        return Err(DbError::BadEventMeta);
    }
    Ok(())
}

fn validate_db_event_page(
    meta: DbEventMeta,
    event: DbEventPage,
    block: &[u8; SD_BLOCK_BYTES],
) -> Result<(), DbError> {
    if !event.initialized {
        return Err(DbError::EventMissing);
    }
    if event.version != DB_FORMAT_VERSION
        || event.page < meta.start_page
        || event.page >= meta.start_page + meta.page_count
        || event.payload_len == 0
        || event.payload_len > DB_EVENT_PAYLOAD_BYTES
        || event.crc_expected != event.crc_actual
    {
        return Err(DbError::BadEventPage);
    }
    let payload = &block[DB_EVENT_PAYLOAD_OFFSET..DB_EVENT_PAYLOAD_OFFSET + event.payload_len];
    if db_payload_crc(payload) != event.payload_crc {
        return Err(DbError::BadEventPage);
    }
    Ok(())
}

fn validate_db_blob_meta(catalog: DbCatalog, meta: DbBlobMeta) -> Result<(), DbError> {
    if !meta.initialized {
        return Err(DbError::BlobMissing);
    }
    if meta.version != DB_FORMAT_VERSION
        || meta.meta_page != DB_BLOB_META_PAGE
        || meta.start_page != DB_BLOB_START_PAGE
        || meta.page_count == 0
        || meta.start_page.saturating_add(meta.page_count) > catalog.page_count
        || meta.capacity_bytes != meta.page_count.saturating_mul(DB_PAGE_BYTES)
        || meta.expected_size > meta.capacity_bytes
        || meta.received_size > meta.expected_size
        || meta.crc_expected != meta.crc_actual
    {
        return Err(DbError::BadBlobMeta);
    }
    Ok(())
}

fn db_btree_leaf_page_for_key(root: DbBtreeRoot, key: u32) -> Result<u32, DbError> {
    validate_db_btree_root(root)?;
    let mut leaf_index = key / DB_BTREE_KEYS_PER_LEAF_RANGE;
    if leaf_index >= root.leaf_count {
        leaf_index = root.leaf_count.saturating_sub(1);
    }
    Ok(root.leaf_start_page + leaf_index)
}

fn table_record_offset(slot: usize) -> Option<usize> {
    if slot >= DB_TABLE_RECORDS as usize {
        return None;
    }
    let offset = DB_TABLE_RECORD_BASE + slot * DB_TABLE_RECORD_BYTES;
    if offset + DB_TABLE_RECORD_BYTES > DB_SUPERBLOCK_CRC_OFFSET {
        None
    } else {
        Some(offset)
    }
}

fn table_record_key(block: &[u8; SD_BLOCK_BYTES], slot: usize) -> Option<u32> {
    let offset = table_record_offset(slot)?;
    let flags = read_le16(block, offset + 6)?;
    if flags & DB_TABLE_RECORD_USED == 0 {
        return None;
    }
    read_le32(block, offset)
}

fn find_table_slot(block: &[u8; SD_BLOCK_BYTES], key: u32) -> Option<(usize, bool)> {
    let mut empty = None;
    for slot in 0..DB_TABLE_RECORDS as usize {
        let offset = table_record_offset(slot)?;
        let flags = read_le16(block, offset + 6).unwrap_or(0);
        if flags & DB_TABLE_RECORD_USED == 0 {
            if empty.is_none() {
                empty = Some(slot);
            }
            continue;
        }
        if read_le32(block, offset).unwrap_or(0) == key {
            return Some((slot, true));
        }
    }
    empty.map(|slot| (slot, false))
}

fn find_btree_leaf_slot(
    block: &[u8; SD_BLOCK_BYTES],
    record_count: u32,
    key: u32,
) -> Option<(usize, bool)> {
    let count = record_count as usize;
    if count > DB_TABLE_RECORDS as usize {
        return None;
    }
    for slot in 0..count {
        let current = table_record_key(block, slot)?;
        if current == key {
            return Some((slot, true));
        }
        if key < current {
            return if count < DB_TABLE_RECORDS as usize {
                Some((slot, false))
            } else {
                None
            };
        }
    }
    if count < DB_TABLE_RECORDS as usize {
        Some((count, false))
    } else {
        None
    }
}

fn shift_btree_leaf_records_right(
    block: &mut [u8; SD_BLOCK_BYTES],
    start_slot: usize,
    record_count: usize,
) {
    if record_count >= DB_TABLE_RECORDS as usize {
        return;
    }
    for slot in (start_slot..record_count).rev() {
        let Some(src) = table_record_offset(slot) else {
            return;
        };
        let Some(dst) = table_record_offset(slot + 1) else {
            return;
        };
        for index in 0..DB_TABLE_RECORD_BYTES {
            block[dst + index] = block[src + index];
        }
    }
}

fn btree_leaf_key_range(block: &[u8; SD_BLOCK_BYTES], record_count: u32) -> (u32, u32) {
    if record_count == 0 {
        return (0, 0);
    }
    let first = table_record_key(block, 0).unwrap_or(0);
    let last = table_record_key(block, record_count.saturating_sub(1) as usize).unwrap_or(first);
    (first, last)
}

fn write_table_record(block: &mut [u8; SD_BLOCK_BYTES], slot: usize, key: u32, value: &[u8]) {
    let Some(offset) = table_record_offset(slot) else {
        return;
    };
    for byte in &mut block[offset..offset + DB_TABLE_RECORD_BYTES] {
        *byte = 0;
    }
    write_le32(block, offset, key);
    write_le16(block, offset + 4, value.len() as u16);
    write_le16(block, offset + 6, DB_TABLE_RECORD_USED);
    block[offset + 8..offset + 8 + value.len()].copy_from_slice(value);
}

fn read_table_row(block: &[u8; SD_BLOCK_BYTES], slot: usize) -> Option<DbRowValue> {
    let offset = table_record_offset(slot)?;
    let flags = read_le16(block, offset + 6)?;
    if flags & DB_TABLE_RECORD_USED == 0 {
        return None;
    }
    let key = read_le32(block, offset)?;
    let value_len = read_le16(block, offset + 4)? as usize;
    if value_len > DB_TABLE_VALUE_BYTES {
        return None;
    }
    let mut value = [0u8; DB_TABLE_VALUE_BYTES];
    value[..value_len].copy_from_slice(&block[offset + 8..offset + 8 + value_len]);
    Some(DbRowValue {
        key,
        value_len,
        value,
    })
}

fn db_superblock_crc(block: &[u8; SD_BLOCK_BYTES]) -> u32 {
    db_page_crc(block)
}

fn db_page_crc(block: &[u8; SD_BLOCK_BYTES]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in &block[..DB_SUPERBLOCK_CRC_OFFSET] {
        crc = crc32_update(crc, byte);
    }
    !crc
}

fn db_payload_crc(payload: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in payload {
        crc = crc32_update(crc, byte);
    }
    !crc
}

fn read_db_page_summary(page: u32) -> Result<(DbFileInfo, DbPageSummary), DbError> {
    let info = locate_db_file()?;
    let page_count = info.entry.size / DB_PAGE_BYTES;
    if page >= page_count {
        return Err(DbError::PageOutOfRange);
    }
    read_file_sector(info.facts, info.entry, page)?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let mut first16 = [0u8; 16];
    first16.copy_from_slice(&block[..16]);
    Ok((
        info,
        DbPageSummary {
            page,
            crc32: db_page_crc(&block),
            first16,
        },
    ))
}

fn read_file_sector(
    facts: Fat32Facts,
    target: FatDirEntryInfo,
    sector_index: u32,
) -> Result<(), DbError> {
    let lba = file_sector_lba(facts, target, sector_index)?;
    sd_read_lba(lba)?;
    Ok(())
}

fn write_file_sector(
    facts: Fat32Facts,
    target: FatDirEntryInfo,
    sector_index: u32,
    bytes: &[u8; SD_BLOCK_BYTES],
) -> Result<(), DbError> {
    let lba = file_sector_lba(facts, target, sector_index)?;
    unsafe {
        core::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            addr_of_mut!(SD_WRITE_BUFFER.0).cast::<u8>(),
            SD_BLOCK_BYTES,
        );
    }
    sd_write_lba(lba).map_err(|error| match error {
        SdReadError::StatusError(_) | SdReadError::IdmacError(_) => DbError::Sd(error),
        _ => DbError::WriteFailed,
    })?;
    Ok(())
}

fn file_sector_lba(
    facts: Fat32Facts,
    target: FatDirEntryInfo,
    sector_index: u32,
) -> Result<u32, DbError> {
    file_sector_location(facts, target, sector_index).map(|location| location.0)
}

fn file_sector_location(
    facts: Fat32Facts,
    target: FatDirEntryInfo,
    sector_index: u32,
) -> Result<(u32, u32, u32), DbError> {
    if target.cluster < 2 || target.size == 0 {
        return Err(DbError::MissingFile);
    }
    let sectors_needed =
        target.size.saturating_add(SD_BLOCK_BYTES as u32 - 1) / SD_BLOCK_BYTES as u32;
    if sector_index >= sectors_needed {
        return Err(DbError::FileTooSmall);
    }
    let spc = facts.sectors_per_cluster as u32;
    if spc == 0 {
        return Err(DbError::Sd(SdReadError::InvalidLayout));
    }

    let mut current = target.cluster;
    let cluster_offset = sector_index / spc;
    let sector_in_cluster = sector_index % spc;
    let mut cached_fat_lba = u32::MAX;
    let mut cached_fat = [0u8; SD_BLOCK_BYTES];

    for _ in 0..cluster_offset {
        if current < 2 || current >= 0x0fff_fff7 {
            return Err(DbError::BadChain);
        }
        current = read_fat_entry(facts, current, &mut cached_fat_lba, &mut cached_fat)?;
    }
    if current < 2 || current >= 0x0fff_fff7 {
        return Err(DbError::BadChain);
    }
    Ok((
        cluster_to_lba(facts, current).saturating_add(sector_in_cluster),
        current,
        sector_in_cluster,
    ))
}

fn find_fat_entry_in_dir(
    facts: Fat32Facts,
    dir_cluster: u32,
    target_name: &[u8],
) -> Result<FatDirEntryInfo, SdReadError> {
    if dir_cluster < 2 {
        return Err(SdReadError::InvalidLayout);
    }

    let mut lfn = [0u8; 260];
    let mut lfn_len = 0usize;
    let dir_lba = cluster_to_lba(facts, dir_cluster);

    for sector in 0..facts.sectors_per_cluster as u32 {
        sd_read_lba(dir_lba.saturating_add(sector))?;
        let mut block = [0u8; SD_BLOCK_BYTES];
        unsafe {
            core::ptr::copy_nonoverlapping(
                addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
                block.as_mut_ptr(),
                SD_BLOCK_BYTES,
            );
        }

        for entry in block.chunks_exact(32) {
            let first = entry[0];
            if first == 0x00 {
                return Ok(FatDirEntryInfo {
                    found: false,
                    attr: 0,
                    cluster: 0,
                    size: 0,
                });
            }
            if first == 0xe5 {
                lfn_len = 0;
                continue;
            }
            let attr = entry[11];
            if attr == 0x0f {
                collect_lfn_entry(entry, &mut lfn, &mut lfn_len);
                continue;
            }

            let mut name = [0u8; 260];
            let name_len = if lfn_len > 0 {
                name[..lfn_len].copy_from_slice(&lfn[..lfn_len]);
                lfn_len
            } else {
                short_dir_name(entry, &mut name)
            };
            lfn_len = 0;

            if !ascii_eq_ignore_case(&name[..name_len], target_name) {
                continue;
            }

            let cluster = ((read_le16(entry, 20).unwrap_or(0) as u32) << 16)
                | read_le16(entry, 26).unwrap_or(0) as u32;
            let size = read_le32(entry, 28).unwrap_or(0);
            return Ok(FatDirEntryInfo {
                found: true,
                attr,
                cluster,
                size,
            });
        }
    }

    Ok(FatDirEntryInfo {
        found: false,
        attr: 0,
        cluster: 0,
        size: 0,
    })
}

fn scan_fat_chain(facts: Fat32Facts, target: FatDirTarget) -> Result<FatChainSummary, SdReadError> {
    if !target.found || target.cluster < 2 || target.size == 0 {
        return Ok(FatChainSummary {
            ok: false,
            clusters: 0,
            expected_clusters: 0,
            contiguous: false,
            first: target.cluster,
            last: target.cluster,
            terminal: 0,
        });
    }

    let cluster_bytes = (facts.bytes_per_sector as u32) * (facts.sectors_per_cluster as u32);
    if cluster_bytes == 0 {
        return Err(SdReadError::InvalidLayout);
    }
    let expected_clusters = target.size.saturating_add(cluster_bytes - 1) / cluster_bytes;
    let mut current = target.cluster;
    let mut clusters = 0u32;
    let mut last = current;
    let mut terminal = 0u32;
    let mut contiguous = true;
    let mut cached_fat_lba = u32::MAX;
    let mut cached_fat = [0u8; SD_BLOCK_BYTES];

    while clusters <= expected_clusters.saturating_add(4) {
        if current < 2 || current >= 0x0fff_fff7 {
            terminal = current;
            break;
        }
        let next = read_fat_entry(facts, current, &mut cached_fat_lba, &mut cached_fat)?;
        clusters = clusters.saturating_add(1);
        last = current;
        if next >= 0x0fff_fff8 {
            terminal = next;
            break;
        }
        if next != current.saturating_add(1) {
            contiguous = false;
        }
        current = next;
    }

    Ok(FatChainSummary {
        ok: clusters == expected_clusters && terminal >= 0x0fff_fff8,
        clusters,
        expected_clusters,
        contiguous,
        first: target.cluster,
        last,
        terminal,
    })
}

fn read_fat_entry(
    facts: Fat32Facts,
    cluster: u32,
    cached_fat_lba: &mut u32,
    cached_fat: &mut [u8; SD_BLOCK_BYTES],
) -> Result<u32, SdReadError> {
    let fat_offset = cluster.saturating_mul(4);
    let fat_lba = facts
        .first_fat_lba
        .saturating_add(fat_offset / SD_BLOCK_BYTES as u32);
    let entry_offset = (fat_offset % SD_BLOCK_BYTES as u32) as usize;
    if fat_lba != *cached_fat_lba {
        sd_read_lba(fat_lba)?;
        unsafe {
            core::ptr::copy_nonoverlapping(
                addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
                cached_fat.as_mut_ptr(),
                SD_BLOCK_BYTES,
            );
        }
        *cached_fat_lba = fat_lba;
    }
    Ok(read_le32(cached_fat, entry_offset).unwrap_or(0) & 0x0fff_ffff)
}

fn persist_last_image_to_slot(slot: u8, image: NetImageState) -> Result<u32, PersistError> {
    let (facts, targets) = scan_sd_root_targets()?;
    let target = match slot {
        b'a' => targets[0],
        b'b' => targets[1],
        _ => return Err(PersistError::SlotMissing),
    };
    if !target.found || target.cluster < 2 {
        return Err(PersistError::SlotMissing);
    }
    if image.size == 0 || image.size > target.size {
        return Err(PersistError::ImageTooLarge);
    }

    write_image_prefix_to_slot(facts, target, image)?;
    let actual_crc = crc32_slot_prefix(facts, target.cluster, image.size)?;
    if actual_crc != image.crc32 {
        return Err(PersistError::CrcMismatch(actual_crc));
    }
    Ok(actual_crc)
}

fn write_image_prefix_to_slot(
    facts: Fat32Facts,
    target: FatDirTarget,
    image: NetImageState,
) -> Result<(), PersistError> {
    let cluster_bytes = (facts.bytes_per_sector as u32) * (facts.sectors_per_cluster as u32);
    if cluster_bytes == 0 || facts.bytes_per_sector != SD_BLOCK_BYTES as u16 {
        return Err(PersistError::Sd(SdReadError::InvalidLayout));
    }

    let sectors_needed =
        image.size.saturating_add(SD_BLOCK_BYTES as u32 - 1) / SD_BLOCK_BYTES as u32;
    let mut sectors_written = 0u32;
    let mut image_offset = 0u32;
    let mut current = target.cluster;
    let mut cached_fat_lba = u32::MAX;
    let mut cached_fat = [0u8; SD_BLOCK_BYTES];

    while sectors_written < sectors_needed {
        if current < 2 || current >= 0x0fff_fff7 {
            return Err(PersistError::BadChain);
        }
        let cluster_lba = cluster_to_lba(facts, current);
        for sector in 0..facts.sectors_per_cluster as u32 {
            if sectors_written >= sectors_needed {
                break;
            }
            fill_sd_write_buffer_from_image(image.addr, image.size, image_offset);
            sd_write_lba(cluster_lba.saturating_add(sector))?;
            image_offset = image_offset.saturating_add(SD_BLOCK_BYTES as u32);
            sectors_written = sectors_written.saturating_add(1);
        }
        if sectors_written < sectors_needed {
            current = read_fat_entry(facts, current, &mut cached_fat_lba, &mut cached_fat)?;
        }
    }

    Ok(())
}

fn crc32_slot_prefix(
    facts: Fat32Facts,
    first_cluster: u32,
    image_size: u32,
) -> Result<u32, PersistError> {
    let mut remaining = image_size;
    let mut current = first_cluster;
    let mut crc = 0xffff_ffffu32;
    let mut cached_fat_lba = u32::MAX;
    let mut cached_fat = [0u8; SD_BLOCK_BYTES];

    while remaining > 0 {
        if current < 2 || current >= 0x0fff_fff7 {
            return Err(PersistError::BadChain);
        }
        let cluster_lba = cluster_to_lba(facts, current);
        for sector in 0..facts.sectors_per_cluster as u32 {
            if remaining == 0 {
                break;
            }
            sd_read_lba(cluster_lba.saturating_add(sector))?;
            let take = remaining.min(SD_BLOCK_BYTES as u32) as usize;
            let mut block = [0u8; SD_BLOCK_BYTES];
            unsafe {
                core::ptr::copy_nonoverlapping(
                    addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
                    block.as_mut_ptr(),
                    SD_BLOCK_BYTES,
                );
            }
            for byte in &block[..take] {
                crc = crc32_update(crc, *byte);
            }
            remaining = remaining.saturating_sub(take as u32);
        }
        if remaining > 0 {
            current = read_fat_entry(facts, current, &mut cached_fat_lba, &mut cached_fat)?;
        }
    }

    Ok(!crc)
}

fn fill_sd_write_buffer_from_image(image_addr: u64, image_size: u32, image_offset: u32) {
    unsafe {
        core::ptr::write_bytes(
            addr_of_mut!(SD_WRITE_BUFFER.0).cast::<u8>(),
            0,
            SD_BLOCK_BYTES,
        );
        if image_offset < image_size {
            let remaining = image_size - image_offset;
            let take = remaining.min(SD_BLOCK_BYTES as u32) as usize;
            core::ptr::copy_nonoverlapping(
                (image_addr + image_offset as u64) as usize as *const u8,
                addr_of_mut!(SD_WRITE_BUFFER.0).cast::<u8>(),
                take,
            );
        }
    }
}

fn switch_extlinux_default_slot(slot: u8) -> Result<(), BootSlotError> {
    if slot != b'a' && slot != b'b' {
        return Err(BootSlotError::VerifyFailed);
    }
    let (facts, _extlinux, config, _default_slot) = read_extlinux_info()?;
    let mut bytes = [0u8; SD_BLOCK_BYTES];
    let len = build_extlinux_config(slot, &mut bytes);
    if len == 0 || len as u32 > config.size {
        return Err(BootSlotError::ConfigTooLarge);
    }
    write_bytes_to_existing_file(facts, config, &bytes[..len])?;
    let verified = read_extlinux_default_slot(facts, config)?;
    if verified != slot {
        return Err(BootSlotError::VerifyFailed);
    }
    Ok(())
}

fn write_bytes_to_existing_file(
    facts: Fat32Facts,
    target: FatDirEntryInfo,
    bytes: &[u8],
) -> Result<(), BootSlotError> {
    let cluster_bytes = (facts.bytes_per_sector as u32) * (facts.sectors_per_cluster as u32);
    if cluster_bytes == 0 || facts.bytes_per_sector != SD_BLOCK_BYTES as u16 {
        return Err(BootSlotError::Sd(SdReadError::InvalidLayout));
    }
    if target.cluster < 2 || target.size == 0 {
        return Err(BootSlotError::ExtlinuxConfMissing);
    }
    if bytes.len() as u32 > target.size {
        return Err(BootSlotError::ConfigTooLarge);
    }

    let sectors_needed =
        target.size.saturating_add(SD_BLOCK_BYTES as u32 - 1) / SD_BLOCK_BYTES as u32;
    let mut sectors_written = 0u32;
    let mut file_offset = 0u32;
    let mut current = target.cluster;
    let mut cached_fat_lba = u32::MAX;
    let mut cached_fat = [0u8; SD_BLOCK_BYTES];

    while sectors_written < sectors_needed {
        if current < 2 || current >= 0x0fff_fff7 {
            return Err(BootSlotError::BadChain);
        }
        let cluster_lba = cluster_to_lba(facts, current);
        for sector in 0..facts.sectors_per_cluster as u32 {
            if sectors_written >= sectors_needed {
                break;
            }
            fill_sd_write_buffer_from_slice(bytes, file_offset);
            sd_write_lba(cluster_lba.saturating_add(sector))?;
            file_offset = file_offset.saturating_add(SD_BLOCK_BYTES as u32);
            sectors_written = sectors_written.saturating_add(1);
        }
        if sectors_written < sectors_needed {
            current = read_fat_entry(facts, current, &mut cached_fat_lba, &mut cached_fat)?;
        }
    }

    Ok(())
}

fn fill_sd_write_buffer_from_slice(bytes: &[u8], offset: u32) {
    unsafe {
        core::ptr::write_bytes(
            addr_of_mut!(SD_WRITE_BUFFER.0).cast::<u8>(),
            0,
            SD_BLOCK_BYTES,
        );
        let offset = offset as usize;
        if offset < bytes.len() {
            let take = (bytes.len() - offset).min(SD_BLOCK_BYTES);
            core::ptr::copy_nonoverlapping(
                bytes.as_ptr().add(offset),
                addr_of_mut!(SD_WRITE_BUFFER.0).cast::<u8>(),
                take,
            );
        }
    }
}

fn build_extlinux_config(slot: u8, out: &mut [u8; SD_BLOCK_BYTES]) -> usize {
    let mut len = 0usize;
    let label = match slot {
        b'a' => b"xiaojin-a".as_slice(),
        b'b' => b"xiaojin-b".as_slice(),
        _ => return 0,
    };
    if !append_bytes(out, &mut len, b"DEFAULT ") {
        return 0;
    }
    if !append_bytes(out, &mut len, label) {
        return 0;
    }
    if !append_bytes(
        out,
        &mut len,
        b"\nTIMEOUT 1\nMENU TITLE Xiaojin OS A/B\n\nLABEL xiaojin-a\n  LINUX /xiaojin-os-a.bin\n\nLABEL xiaojin-b\n  LINUX /xiaojin-os-b.bin\n",
    ) {
        return 0;
    }
    len
}

fn append_bytes(out: &mut [u8], len: &mut usize, bytes: &[u8]) -> bool {
    if len.saturating_add(bytes.len()) > out.len() {
        return false;
    }
    out[*len..*len + bytes.len()].copy_from_slice(bytes);
    *len += bytes.len();
    true
}

fn read_extlinux_default_slot(
    facts: Fat32Facts,
    config: FatDirEntryInfo,
) -> Result<u8, BootSlotError> {
    if config.cluster < 2 {
        return Err(BootSlotError::ExtlinuxConfMissing);
    }
    sd_read_lba(cluster_to_lba(facts, config.cluster))?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    if ascii_contains(&block, b"DEFAULT xiaojin-a") {
        Ok(b'a')
    } else if ascii_contains(&block, b"DEFAULT xiaojin-b") {
        Ok(b'b')
    } else {
        Ok(b'?')
    }
}

fn ascii_contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|window| {
        window
            .iter()
            .zip(needle.iter())
            .all(|(&a, &b)| ascii_lower(a) == ascii_lower(b))
    })
}

fn extlinux_slot_label(slot: u8) -> &'static str {
    match slot {
        b'a' => "xiaojin-a",
        b'b' => "xiaojin-b",
        _ => "unknown",
    }
}

fn cluster_to_lba(facts: Fat32Facts, cluster: u32) -> u32 {
    facts.data_lba.saturating_add(
        cluster
            .saturating_sub(2)
            .saturating_mul(facts.sectors_per_cluster as u32),
    )
}

fn scan_sd_root_targets() -> Result<(Fat32Facts, [FatDirTarget; 4]), SdReadError> {
    let facts = read_fat32_facts()?;
    if facts.bytes_per_sector != SD_BLOCK_BYTES as u16 || facts.sectors_per_cluster == 0 {
        return Err(SdReadError::InvalidLayout);
    }

    let mut targets = [
        FatDirTarget {
            name: "xiaojin-os-a.bin",
            found: false,
            cluster: 0,
            size: 0,
        },
        FatDirTarget {
            name: "xiaojin-os-b.bin",
            found: false,
            cluster: 0,
            size: 0,
        },
        FatDirTarget {
            name: "xiaojin-os.bin",
            found: false,
            cluster: 0,
            size: 0,
        },
        FatDirTarget {
            name: "bootstate.txt",
            found: false,
            cluster: 0,
            size: 0,
        },
    ];
    let mut lfn = [0u8; 260];
    let mut lfn_len = 0usize;

    for sector in 0..facts.sectors_per_cluster as u32 {
        sd_read_lba(facts.root_lba.saturating_add(sector))?;
        let mut block = [0u8; SD_BLOCK_BYTES];
        unsafe {
            core::ptr::copy_nonoverlapping(
                addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
                block.as_mut_ptr(),
                SD_BLOCK_BYTES,
            );
        }

        for entry in block.chunks_exact(32) {
            let first = entry[0];
            if first == 0x00 {
                return Ok((facts, targets));
            }
            if first == 0xe5 {
                lfn_len = 0;
                continue;
            }
            let attr = entry[11];
            if attr == 0x0f {
                collect_lfn_entry(entry, &mut lfn, &mut lfn_len);
                continue;
            }

            let mut name = [0u8; 260];
            let name_len = if lfn_len > 0 {
                name[..lfn_len].copy_from_slice(&lfn[..lfn_len]);
                lfn_len
            } else {
                short_dir_name(entry, &mut name)
            };

            let cluster = ((read_le16(entry, 20).unwrap_or(0) as u32) << 16)
                | read_le16(entry, 26).unwrap_or(0) as u32;
            let size = read_le32(entry, 28).unwrap_or(0);
            update_dir_targets(&name[..name_len], cluster, size, &mut targets);
            lfn_len = 0;
        }
    }

    Ok((facts, targets))
}

fn collect_lfn_entry(entry: &[u8], lfn: &mut [u8; 260], lfn_len: &mut usize) {
    let order = (entry[0] & 0x1f) as usize;
    if order == 0 {
        return;
    }
    if entry[0] & 0x40 != 0 {
        *lfn_len = 0;
        for byte in lfn.iter_mut() {
            *byte = 0;
        }
    }

    let mut pos = (order - 1) * 13;
    for offset in [1usize, 3, 5, 7, 9, 14, 16, 18, 20, 22, 24, 28, 30] {
        if pos >= lfn.len() || offset + 1 >= entry.len() {
            break;
        }
        let value = (entry[offset] as u16) | ((entry[offset + 1] as u16) << 8);
        if value == 0x0000 {
            break;
        }
        if value != 0xffff {
            let byte = if value <= 0x7f {
                ascii_lower(value as u8)
            } else {
                b'?'
            };
            lfn[pos] = byte;
            *lfn_len = (*lfn_len).max(pos + 1);
        }
        pos += 1;
    }
}

fn short_dir_name(entry: &[u8], out: &mut [u8; 260]) -> usize {
    let mut len = 0usize;
    for &byte in &entry[0..8] {
        if byte == b' ' {
            break;
        }
        out[len] = ascii_lower(byte);
        len += 1;
    }
    let mut ext_len = 0usize;
    for &byte in &entry[8..11] {
        if byte != b' ' {
            ext_len += 1;
        }
    }
    if ext_len > 0 {
        out[len] = b'.';
        len += 1;
        for &byte in &entry[8..11] {
            if byte == b' ' {
                break;
            }
            out[len] = ascii_lower(byte);
            len += 1;
        }
    }
    len
}

fn update_dir_targets(name: &[u8], cluster: u32, size: u32, targets: &mut [FatDirTarget; 4]) {
    for target in targets.iter_mut() {
        if ascii_eq_ignore_case(name, target.name.as_bytes()) {
            target.found = true;
            target.cluster = cluster;
            target.size = size;
        }
    }
}

fn ascii_eq_ignore_case(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right.iter())
        .all(|(&a, &b)| ascii_lower(a) == ascii_lower(b))
}

fn ascii_lower(byte: u8) -> u8 {
    if byte.is_ascii_uppercase() {
        byte + 32
    } else {
        byte
    }
}

fn read_fat32_facts() -> Result<Fat32Facts, SdReadError> {
    let part_lba = read_first_partition_bpb()?;
    let mut block = [0u8; SD_BLOCK_BYTES];
    unsafe {
        core::ptr::copy_nonoverlapping(
            addr_of!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            block.as_mut_ptr(),
            SD_BLOCK_BYTES,
        );
    }
    let bytes_per_sector = read_le16(&block, 11).unwrap_or(0);
    let sectors_per_cluster = block[13];
    let reserved_sectors = read_le16(&block, 14).unwrap_or(0);
    let fats = block[16];
    let total_sectors = read_le32(&block, 32).unwrap_or(0);
    let fat_sectors = read_le32(&block, 36).unwrap_or(0);
    let root_cluster = read_le32(&block, 44).unwrap_or(0);
    if bytes_per_sector == 0
        || sectors_per_cluster == 0
        || reserved_sectors == 0
        || fats == 0
        || fat_sectors == 0
        || root_cluster < 2
    {
        return Err(SdReadError::InvalidLayout);
    }
    let first_fat_lba = part_lba.saturating_add(reserved_sectors as u32);
    let data_lba = first_fat_lba.saturating_add((fats as u32).saturating_mul(fat_sectors));
    let root_lba = data_lba.saturating_add(
        root_cluster
            .saturating_sub(2)
            .saturating_mul(sectors_per_cluster as u32),
    );
    Ok(Fat32Facts {
        part_lba,
        bytes_per_sector,
        sectors_per_cluster,
        reserved_sectors,
        fats,
        fat_sectors,
        total_sectors,
        root_cluster,
        first_fat_lba,
        data_lba,
        root_lba,
    })
}

fn sd_read_lba0() -> Result<(), SdReadError> {
    sd_read_lba(0)
}

fn sd_read_lba(lba: u32) -> Result<(), SdReadError> {
    let desc_addr = unsafe { addr_of!(SD_IDMAC_DESC.0) as u64 };
    let buffer_addr = unsafe { addr_of!(SD_BLOCK_BUFFER.0) as u64 };
    if desc_addr > u32::MAX as u64 || buffer_addr > u32::MAX as u64 {
        return Err(SdReadError::AddrTooHigh);
    }

    unsafe {
        core::ptr::write_bytes(
            addr_of_mut!(SD_BLOCK_BUFFER.0).cast::<u8>(),
            0,
            SD_BLOCK_BYTES,
        );
        core::ptr::write(
            addr_of_mut!(SD_IDMAC_DESC.0),
            SdIdmacDesc {
                flags: DWMCI_IDMAC_OWN | DWMCI_IDMAC_CH | DWMCI_IDMAC_FS | DWMCI_IDMAC_LD,
                cnt: SD_BLOCK_BYTES as u32,
                addr: buffer_addr as u32,
                next: 0,
            },
        );
    }
    clean_dcache_range(desc_addr, core::mem::size_of::<SdIdmacDesc>() as u64);
    clean_dcache_range(buffer_addr, SD_BLOCK_BYTES as u64);
    invalidate_dcache_range(buffer_addr, SD_BLOCK_BYTES as u64);

    wait_sd_not_busy(DWMCI_READ_TIMEOUT_MS)?;

    let ctrl = sd_read_reg(DWMCI_CTRL)?;
    sd_write_reg(DWMCI_RINTSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDINTEN, 0);

    sd_write_reg(
        DWMCI_CTRL,
        ctrl | DWMCI_CTRL_FIFO_RESET | DWMCI_CTRL_DMA_RESET,
    );
    wait_sd_ctrl_clear(
        DWMCI_CTRL_FIFO_RESET | DWMCI_CTRL_DMA_RESET,
        DWMCI_READ_TIMEOUT_MS,
    )?;

    let bmod = sd_read_reg(DWMCI_BMOD)?;
    sd_write_reg(DWMCI_BMOD, bmod | DWMCI_BMOD_SWR);
    wait_sd_bmod_clear(DWMCI_BMOD_SWR, DWMCI_READ_TIMEOUT_MS)?;

    sd_write_reg(DWMCI_DBADDR, desc_addr as u32);
    sd_write_reg(DWMCI_BMOD, DWMCI_BMOD_FB | DWMCI_BMOD_IDMAC_ENABLE);
    sd_write_reg(
        DWMCI_CTRL,
        (ctrl | DWMCI_CTRL_DMA_ENABLE | DWMCI_CTRL_IDMAC_ENABLE)
            & !(DWMCI_CTRL_FIFO_RESET | DWMCI_CTRL_DMA_RESET),
    );
    sd_write_reg(DWMCI_BLKSIZ, SD_BLOCK_BYTES as u32);
    sd_write_reg(DWMCI_BYTCNT, SD_BLOCK_BYTES as u32);
    sd_write_reg(DWMCI_CMDARG, lba);
    sd_write_reg(DWMCI_RINTSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDSTS, 0xffff_ffff);
    mmio_write_barrier();

    sd_write_reg(
        DWMCI_CMD,
        DWMCI_CMD_START
            | DWMCI_CMD_USE_HOLD_REG
            | DWMCI_CMD_WAIT_PRVDATA
            | DWMCI_CMD_DATA_EXP
            | DWMCI_CMD_CHECK_CRC
            | DWMCI_CMD_RESP_EXPECT
            | SD_CMD_READ_SINGLE_BLOCK,
    );
    wait_sd_rint_bit(
        DWMCI_RINTSTS_CDONE,
        DWMCI_READ_TIMEOUT_MS,
        SdReadError::TimeoutCommand,
    )?;
    wait_sd_rint_bit(
        DWMCI_RINTSTS_DTO,
        DWMCI_READ_TIMEOUT_MS,
        SdReadError::TimeoutData,
    )?;

    let idsts = sd_read_reg(DWMCI_IDSTS)?;
    if idsts & DWMCI_IDSTS_ERRORS != 0 {
        return Err(SdReadError::IdmacError(idsts));
    }

    invalidate_dcache_range(desc_addr, core::mem::size_of::<SdIdmacDesc>() as u64);
    let flags = unsafe { read_volatile(addr_of!(SD_IDMAC_DESC.0.flags)) };
    if flags & DWMCI_IDMAC_OWN != 0 {
        return Err(SdReadError::DmaOwnTimeout);
    }

    invalidate_dcache_range(buffer_addr, SD_BLOCK_BYTES as u64);
    let final_ctrl = sd_read_reg(DWMCI_CTRL).unwrap_or(ctrl);
    sd_write_reg(DWMCI_CTRL, final_ctrl & !DWMCI_CTRL_DMA_ENABLE);
    sd_write_reg(DWMCI_RINTSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDSTS, 0xffff_ffff);
    Ok(())
}

fn sd_write_lba(lba: u32) -> Result<(), SdReadError> {
    let desc_addr = unsafe { addr_of!(SD_IDMAC_DESC.0) as u64 };
    let buffer_addr = unsafe { addr_of!(SD_WRITE_BUFFER.0) as u64 };
    if desc_addr > u32::MAX as u64 || buffer_addr > u32::MAX as u64 {
        return Err(SdReadError::AddrTooHigh);
    }

    unsafe {
        core::ptr::write(
            addr_of_mut!(SD_IDMAC_DESC.0),
            SdIdmacDesc {
                flags: DWMCI_IDMAC_OWN | DWMCI_IDMAC_CH | DWMCI_IDMAC_FS | DWMCI_IDMAC_LD,
                cnt: SD_BLOCK_BYTES as u32,
                addr: buffer_addr as u32,
                next: 0,
            },
        );
    }
    clean_dcache_range(buffer_addr, SD_BLOCK_BYTES as u64);
    clean_dcache_range(desc_addr, core::mem::size_of::<SdIdmacDesc>() as u64);

    wait_sd_not_busy(DWMCI_READ_TIMEOUT_MS)?;

    let ctrl = sd_read_reg(DWMCI_CTRL)?;
    sd_write_reg(DWMCI_RINTSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDINTEN, 0);
    sd_write_reg(
        DWMCI_CTRL,
        ctrl | DWMCI_CTRL_FIFO_RESET | DWMCI_CTRL_DMA_RESET,
    );
    wait_sd_ctrl_clear(
        DWMCI_CTRL_FIFO_RESET | DWMCI_CTRL_DMA_RESET,
        DWMCI_READ_TIMEOUT_MS,
    )?;

    let bmod = sd_read_reg(DWMCI_BMOD)?;
    sd_write_reg(DWMCI_BMOD, bmod | DWMCI_BMOD_SWR);
    wait_sd_bmod_clear(DWMCI_BMOD_SWR, DWMCI_READ_TIMEOUT_MS)?;

    sd_write_reg(DWMCI_DBADDR, desc_addr as u32);
    sd_write_reg(DWMCI_BMOD, DWMCI_BMOD_FB | DWMCI_BMOD_IDMAC_ENABLE);
    sd_write_reg(
        DWMCI_CTRL,
        (ctrl | DWMCI_CTRL_DMA_ENABLE | DWMCI_CTRL_IDMAC_ENABLE)
            & !(DWMCI_CTRL_FIFO_RESET | DWMCI_CTRL_DMA_RESET),
    );
    sd_write_reg(DWMCI_BLKSIZ, SD_BLOCK_BYTES as u32);
    sd_write_reg(DWMCI_BYTCNT, SD_BLOCK_BYTES as u32);
    sd_write_reg(DWMCI_CMDARG, lba);
    sd_write_reg(DWMCI_RINTSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDSTS, 0xffff_ffff);
    mmio_write_barrier();

    sd_write_reg(
        DWMCI_CMD,
        DWMCI_CMD_START
            | DWMCI_CMD_USE_HOLD_REG
            | DWMCI_CMD_WAIT_PRVDATA
            | DWMCI_CMD_DATA_EXP
            | DWMCI_CMD_WRITE
            | DWMCI_CMD_CHECK_CRC
            | DWMCI_CMD_RESP_EXPECT
            | SD_CMD_WRITE_SINGLE_BLOCK,
    );
    wait_sd_rint_bit(
        DWMCI_RINTSTS_CDONE,
        DWMCI_READ_TIMEOUT_MS,
        SdReadError::TimeoutCommand,
    )?;
    wait_sd_rint_bit(
        DWMCI_RINTSTS_DTO,
        DWMCI_READ_TIMEOUT_MS,
        SdReadError::TimeoutData,
    )?;

    let idsts = sd_read_reg(DWMCI_IDSTS)?;
    if idsts & DWMCI_IDSTS_ERRORS != 0 {
        return Err(SdReadError::IdmacError(idsts));
    }
    invalidate_dcache_range(desc_addr, core::mem::size_of::<SdIdmacDesc>() as u64);
    let flags = unsafe { read_volatile(addr_of!(SD_IDMAC_DESC.0.flags)) };
    if flags & DWMCI_IDMAC_OWN != 0 {
        return Err(SdReadError::DmaOwnTimeout);
    }

    let final_ctrl = sd_read_reg(DWMCI_CTRL).unwrap_or(ctrl);
    sd_write_reg(DWMCI_CTRL, final_ctrl & !DWMCI_CTRL_DMA_ENABLE);
    sd_write_reg(DWMCI_RINTSTS, 0xffff_ffff);
    sd_write_reg(DWMCI_IDSTS, 0xffff_ffff);
    Ok(())
}

fn sd_read_reg(offset: u64) -> Result<u32, SdReadError> {
    probe_read32(SD_DWMMC_BASE + offset).map_err(|_| SdReadError::ProbeFault)
}

fn sd_write_reg(offset: u64, value: u32) {
    write_mmio32(SD_DWMMC_BASE + offset, value);
}

fn wait_sd_ctrl_clear(mask: u32, timeout_ms: u64) -> Result<u32, SdReadError> {
    wait_sd_reg_clear(DWMCI_CTRL, mask, timeout_ms, SdReadError::TimeoutReset)
}

fn wait_sd_bmod_clear(mask: u32, timeout_ms: u64) -> Result<u32, SdReadError> {
    wait_sd_reg_clear(DWMCI_BMOD, mask, timeout_ms, SdReadError::TimeoutReset)
}

fn wait_sd_reg_clear(
    offset: u64,
    mask: u32,
    timeout_ms: u64,
    timeout_error: SdReadError,
) -> Result<u32, SdReadError> {
    let freq = read_cntfrq();
    let Some(timeout_ticks) = ms_to_ticks(timeout_ms, freq) else {
        return Err(timeout_error);
    };
    let start = read_cntpct();
    loop {
        let value = sd_read_reg(offset)?;
        if value & mask == 0 {
            return Ok(value);
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return Err(timeout_error);
        }
        core::hint::spin_loop();
    }
}

fn wait_sd_not_busy(timeout_ms: u64) -> Result<u32, SdReadError> {
    wait_sd_reg_clear(
        DWMCI_STATUS,
        DWMCI_STATUS_BUSY,
        timeout_ms,
        SdReadError::TimeoutBusy,
    )
}

fn wait_sd_rint_bit(
    bit: u32,
    timeout_ms: u64,
    timeout_error: SdReadError,
) -> Result<u32, SdReadError> {
    let freq = read_cntfrq();
    let Some(timeout_ticks) = ms_to_ticks(timeout_ms, freq) else {
        return Err(timeout_error);
    };
    let start = read_cntpct();
    loop {
        let status = sd_read_reg(DWMCI_RINTSTS)?;
        if status & DWMCI_RINTSTS_ERRORS != 0 {
            return Err(SdReadError::StatusError(status));
        }
        if status & bit != 0 {
            return Ok(status);
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return Err(timeout_error);
        }
        core::hint::spin_loop();
    }
}

fn command_net_ip(uart: &mut Uart, iface: Option<&str>, ip: Option<&str>, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net ip <pcie3x1|pcie3x2|0|1> <a.b.c.d> confirm\n");
        uart.put_str("scope: update runtime IP config and preserve it across net-dev reloads\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(ip_text) = ip else {
        uart.put_str("missing IPv4 address\n");
        return;
    };
    let Some(new_ip) = parse_ipv4(ip_text) else {
        uart.put_str("invalid IPv4 address\n");
        return;
    };
    if !valid_unicast_ipv4(new_ip) {
        uart.put_str("refusing non-unicast IPv4 address\n");
        return;
    }
    set_net_device_ip(index, new_ip);
    let _ = writeln!(
        uart,
        "{} ip set to {}.{}.{}.{}",
        PCIE_CONTROLLERS[index].name, new_ip[0], new_ip[1], new_ip[2], new_ip[3]
    );
}

fn command_net_dhcp(uart: &mut Uart, iface: Option<&str>, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net dhcp <pcie3x1|pcie3x2|0|1> confirm\n");
        uart.put_str("scope: request runtime IPv4/mask/gateway/dns from a DHCP server\n");
        uart.put_str("note: this is manual for now; cold boot keeps the static fallback IPs\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };

    let controller = PCIE_CONTROLLERS[index];
    let mac = rtl_local_mac(index);
    let xid = 0x584a_0000u32 ^ (read_cntpct() as u32).rotate_left(7) ^ ((index as u32) << 8);
    let _ = writeln!(
        uart,
        "net dhcp {} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} xid=0x{xid:08x}:",
        controller.name, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );

    command_pcie_link_start(uart, Some("confirm"));
    command_pcie_rtl_init(uart, Some("confirm"));

    uart.put_str("  discover        broadcast DHCPDISCOVER\n");
    match net_dhcp_exchange(index, xid) {
        Ok(exchange) => {
            print_dhcp_lease(uart, "  offer", exchange.offer);
            uart.put_str("  request         broadcast DHCPREQUEST\n");
            print_dhcp_lease(uart, "  ack", exchange.ack);
            set_net_device_dhcp_lease(index, exchange.ack);
            let _ = writeln!(
                uart,
                "  bound           {} ip={}.{}.{}.{}",
                controller.name,
                exchange.ack.ip[0],
                exchange.ack.ip[1],
                exchange.ack.ip[2],
                exchange.ack.ip[3]
            );
            let _ = net_send_gratuitous_arp(index, ARP_REQUEST);
            let _ = net_send_gratuitous_arp(index, ARP_REPLY);
        }
        Err(error) => {
            let _ = writeln!(uart, "  error           {}", error.as_str());
        }
    }
}

fn command_net_dns(
    uart: &mut Uart,
    iface: Option<&str>,
    name: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net dns <pcie3x1|pcie3x2|0|1> <name> confirm\n");
        uart.put_str("scope: resolve one A record using runtime DNS/gateway state\n");
        uart.put_str("note: run `net dhcp <if> confirm` first when dns/gateway are 0.0.0.0\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(name) = name else {
        uart.put_str("missing DNS name\n");
        return;
    };

    let _ = writeln!(uart, "net dns {} {name}:", PCIE_CONTROLLERS[index].name);
    match net_dns_resolve_a(index, name, DNS_TIMEOUT_MS) {
        Ok(result) => {
            let _ = writeln!(
                uart,
                "  answer          {}.{}.{}.{}",
                result.answer[0], result.answer[1], result.answer[2], result.answer[3]
            );
            let _ = writeln!(
                uart,
                "  server          {}.{}.{}.{}",
                result.server[0], result.server[1], result.server[2], result.server[3]
            );
            let _ = writeln!(
                uart,
                "  next-hop        {}.{}.{}.{} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} txid=0x{:04x}",
                result.next_hop[0],
                result.next_hop[1],
                result.next_hop[2],
                result.next_hop[3],
                result.next_hop_mac[0],
                result.next_hop_mac[1],
                result.next_hop_mac[2],
                result.next_hop_mac[3],
                result.next_hop_mac[4],
                result.next_hop_mac[5],
                result.txid
            );
        }
        Err(error) => {
            let _ = writeln!(uart, "  error           {}", error.as_str());
        }
    }
}

fn command_net_tcp(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net tcp <pcie3x1|pcie3x2|0|1> <host|ip> <port> confirm\n");
        uart.put_str("scope: prove outbound TCP connect using SYN/SYN-ACK/ACK\n");
        uart.put_str("note: run `net dhcp <if> confirm` first for routed internet targets\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };

    let _ = writeln!(
        uart,
        "net tcp {} {host}:{port}:",
        PCIE_CONTROLLERS[index].name
    );
    match net_tcp_connect(index, host, port, TCP_CONNECT_TIMEOUT_MS) {
        Ok(result) => print_tcp_connect_result(uart, result),
        Err(error) => {
            let _ = writeln!(uart, "  error           {}", error.as_str());
        }
    }
}

fn print_tcp_connect_result(uart: &mut Uart, result: TcpConnectResult) {
    let _ = writeln!(
        uart,
        "  connected       {}.{}.{}.{}:{} src-port={} flags=0x{:03x}",
        result.dst_ip[0],
        result.dst_ip[1],
        result.dst_ip[2],
        result.dst_ip[3],
        result.dst_port,
        result.src_port,
        result.flags
    );
    let _ = writeln!(
        uart,
        "  next-hop        {}.{}.{}.{} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        result.next_hop[0],
        result.next_hop[1],
        result.next_hop[2],
        result.next_hop[3],
        result.next_hop_mac[0],
        result.next_hop_mac[1],
        result.next_hop_mac[2],
        result.next_hop_mac[3],
        result.next_hop_mac[4],
        result.next_hop_mac[5]
    );
    let _ = writeln!(
        uart,
        "  seq             0x{:08x} ack=0x{:08x} dns-txid=0x{:04x}",
        result.seq, result.ack, result.dns_txid
    );
}

fn command_net_http(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net http <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
        uart.put_str("scope: perform one HTTP/1.0 GET over the outbound TCP probe path\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };

    let _ = writeln!(
        uart,
        "net http {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    match net_http_get(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => print_http_get_result(uart, result),
        Err(error) => {
            let _ = writeln!(uart, "  error           {}", error.as_str());
        }
    }
}

fn command_net_http_post(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    body: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str(
            "usage: net http-post <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> <body> confirm\n",
        );
        uart.put_str("scope: perform one small HTTP/1.0 POST over the outbound TCP probe path\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };
    let Some(body) = body else {
        uart.put_str("missing body\n");
        return;
    };

    let _ = writeln!(
        uart,
        "net http-post {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    match net_http_post(index, host, port, path, body.as_bytes(), HTTP_TIMEOUT_MS) {
        Ok(result) => print_http_post_result(uart, result),
        Err(error) => {
            let _ = writeln!(uart, "  error           {}", error.as_str());
        }
    }
}

fn command_maint_event(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    kind: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str(
            "usage: maint event <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> <kind> confirm\n",
        );
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };
    let Some(kind) = kind else {
        uart.put_str("missing event kind\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint event {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_event_summary(uart, index, host, port, path, kind);
    uart.put_str("\n");
}

fn command_maint_ack_intent(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    poll_path: Option<&str>,
    event_path: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str(
            "usage: maint ack-intent <pcie3x1|pcie3x2|0|1> <host|ip> <port> <poll-path> <event-path> confirm\n",
        );
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(poll_path) = poll_path else {
        uart.put_str("missing poll path\n");
        return;
    };
    let Some(event_path) = event_path else {
        uart.put_str("missing event path\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint ack-intent {} http://{host}:{port}{poll_path} -> {event_path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_ack_intent_summary(uart, index, host, port, poll_path, event_path);
    uart.put_str("\n");
}

fn command_maint_verify(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str("usage: maint verify <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint verify {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_verify_summary(uart, index, host, port, path, MaintHmacKey::None);
    uart.put_str("\n");
}

fn command_maint_verify_hmac(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    key_hex: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str(
            "usage: maint verify-hmac <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> <key-hex> confirm\n",
        );
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };
    let Some(key_hex) = key_hex else {
        uart.put_str("missing HMAC key hex\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint verify-hmac {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_verify_summary(uart, index, host, port, path, MaintHmacKey::Hex(key_hex));
    uart.put_str("\n");
}

fn command_maint_verify_local(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str(
            "usage: maint verify-local <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n",
        );
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };
    let Ok(row) = get_maint_hmac_key_row() else {
        uart.put_str("maint verify-local error: key-missing\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint verify-local {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_verify_summary(
        uart,
        index,
        host,
        port,
        path,
        MaintHmacKey::Raw(&row.value[..row.value_len]),
    );
    uart.put_str("\n");
}

fn command_maint_pull_exec(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    event_path_or_confirm: Option<&str>,
    confirm: Option<&str>,
) {
    let event_path = match (event_path_or_confirm, confirm) {
        (Some("confirm"), None) => None,
        (Some(event_path), Some("confirm")) => Some(event_path),
        _ => {
            uart.put_str(
                "usage: maint pull-exec <pcie3x1|pcie3x2|0|1> <host|ip> <port> <poll-path> [event-path] confirm\n",
            );
            return;
        }
    };
    if let Some(event_path) = event_path {
        if !event_path.starts_with('/') {
            uart.put_str("event path must start with '/'\n");
            return;
        }
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };
    let Ok(row) = get_maint_hmac_key_row() else {
        uart.put_str("maint pull-exec error: key-missing\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint pull-exec {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_pull_exec_summary(
        uart,
        index,
        host,
        port,
        path,
        event_path,
        MaintHmacKey::Raw(&row.value[..row.value_len]),
    );
    uart.put_str("\n");
}

fn command_maint_key_status(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: maint key-status confirm\n");
        return;
    }
    write_maint_key_status_summary(uart);
    uart.put_str("\n");
}

fn command_maint_key_set(uart: &mut Uart, key_hex: Option<&str>, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: maint key-set <key-hex> confirm\n");
        return;
    }
    let Some(key_hex) = key_hex else {
        uart.put_str("missing HMAC key hex\n");
        return;
    };
    write_maint_key_set_summary(uart, key_hex);
    uart.put_str("\n");
}

fn command_maint_key_clear(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: maint key-clear confirm\n");
        return;
    }
    write_maint_key_clear_summary(uart);
    uart.put_str("\n");
}

fn command_maint_poll(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str("usage: maint poll <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint poll {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_poll_summary(uart, index, host, port, path);
    uart.put_str("\n");
}

fn command_maint_status(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str("usage: maint status <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint status {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_status_summary(uart, index, host, port, path);
    uart.put_str("\n");
}

fn command_maint_audit(
    uart: &mut Uart,
    iface: Option<&str>,
    host: Option<&str>,
    port: Option<&str>,
    path: Option<&str>,
    confirm: Option<&str>,
) {
    if confirm != Some("confirm") {
        uart.put_str("usage: maint audit <pcie3x1|pcie3x2|0|1> <host|ip> <port> <path> confirm\n");
        return;
    }
    let Some(iface) = iface else {
        uart.put_str("missing interface\n");
        return;
    };
    let Some(index) = parse_net_interface(iface) else {
        uart.put_str("unknown interface; use pcie3x1, pcie3x2, 0, or 1\n");
        return;
    };
    let Some(host) = host else {
        uart.put_str("missing host\n");
        return;
    };
    let Some(port) = port.and_then(parse_u16) else {
        uart.put_str("missing or invalid TCP port\n");
        return;
    };
    let Some(path) = path else {
        uart.put_str("missing path\n");
        return;
    };

    let _ = writeln!(
        uart,
        "maint audit {} http://{host}:{port}{path}:",
        PCIE_CONTROLLERS[index].name
    );
    write_maint_audit_summary(uart, index, host, port, path);
    uart.put_str("\n");
}

fn print_http_get_result(uart: &mut Uart, result: HttpGetResult) {
    let _ = writeln!(
        uart,
        "  status          {} bytes={} payload={} segments={} fin={} truncated={}",
        result.status_code,
        result.response_bytes,
        result.payload_bytes,
        result.segments,
        yes_no(result.fin),
        yes_no(result.truncated)
    );
    let _ = writeln!(
        uart,
        "  peer            {}.{}.{}.{}:{} src-port={}",
        result.tcp.dst_ip[0],
        result.tcp.dst_ip[1],
        result.tcp.dst_ip[2],
        result.tcp.dst_ip[3],
        result.tcp.dst_port,
        result.tcp.src_port
    );
    uart.put_str("  first-line      ");
    write_http_sample_line(uart, &result.sample[..result.sample_len]);
    uart.put_str("\n");
}

fn print_http_post_result(uart: &mut Uart, result: HttpPostResult) {
    let _ = writeln!(
        uart,
        "  status          {} bytes={} payload={} segments={} fin={} truncated={} sent={}",
        result.status_code,
        result.response_bytes,
        result.payload_bytes,
        result.segments,
        yes_no(result.fin),
        yes_no(result.truncated),
        result.sent_payload_len
    );
    let _ = writeln!(
        uart,
        "  peer            {}.{}.{}.{}:{} src-port={}",
        result.tcp.dst_ip[0],
        result.tcp.dst_ip[1],
        result.tcp.dst_ip[2],
        result.tcp.dst_ip[3],
        result.tcp.dst_port,
        result.tcp.src_port
    );
    uart.put_str("  first-line      ");
    write_http_sample_line(uart, &result.sample[..result.sample_len]);
    uart.put_str("\n");
}

fn write_maint_poll_summary<W: Write>(
    out: &mut W,
    index: usize,
    host: &str,
    port: u16,
    path: &str,
) {
    match net_http_get(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let _ = write!(
                out,
                "XJQ OK maint-poll iface={} host={} ip={}.{}.{}.{} port={} status={} bytes={} payload={} segments={} fin={} truncated={} src-port={} first-line=\"",
                PCIE_CONTROLLERS[index].name,
                host,
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.status_code,
                result.response_bytes,
                result.payload_bytes,
                result.segments,
                yes_no(result.fin),
                yes_no(result.truncated),
                result.tcp.src_port
            );
            write_http_sample_line(out, &result.sample[..result.sample_len]);
            let _ = write!(out, "\" body=\"");
            write_http_body_sample_line(out, &result.sample[..result.sample_len]);
            let _ = write!(out, "\"");
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-poll {}", error.as_str());
        }
    }
}

fn write_maint_status_summary<W: Write>(
    out: &mut W,
    index: usize,
    host: &str,
    port: u16,
    path: &str,
) {
    match net_http_get(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let Some(body) = find_http_body_start(&result.sample[..result.sample_len])
                .map(|start| &result.sample[start..result.sample_len])
            else {
                let _ = write!(out, "XJQ ERR maint-status no-body");
                return;
            };
            let _ = write!(
                out,
                "XJQ OK maint-status iface={} host={} ip={}.{}.{}.{} port={} status={} bytes={} payload={} ",
                PCIE_CONTROLLERS[index].name,
                host,
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.status_code,
                result.response_bytes,
                result.payload_bytes
            );
            write_maint_field(out, body, b"device_id");
            write_maint_field(out, body, b"registered");
            write_maint_field(out, body, b"board_id");
            write_maint_field(out, body, b"arch");
            write_maint_field(out, body, b"db_event_sequence");
            write_maint_field(out, body, b"artifacts");
            write_maint_field(out, body, b"commands_pending");
            write_maint_field(out, body, b"commands_running");
            write_maint_field(out, body, b"commands_succeeded");
            write_maint_field(out, body, b"commands_failed");
            write_maint_field(out, body, b"commands_acknowledged");
            write_maint_field(out, body, b"next_pending_command_id");
            write_maint_field(out, body, b"next_pending_command_kind");
            write_maint_field(out, body, b"next_pending_command_updated_unix");
            write_maint_field(out, body, b"next_pending_payload_sha256");
            write_maint_field(out, body, b"next_pending_command_sha256");
            write_maint_field(out, body, b"next_pending_signer_id");
            write_maint_field(out, body, b"next_pending_signature_algorithm");
            write_maint_field(out, body, b"next_pending_signature_sha256");
            write_maint_field(out, body, b"next_pending_signature_present");
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-status {}", error.as_str());
        }
    }
}

fn write_maint_audit_summary<W: Write>(
    out: &mut W,
    index: usize,
    host: &str,
    port: u16,
    path: &str,
) {
    match net_http_get(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let Some(body) = find_http_body_start(&result.sample[..result.sample_len])
                .map(|start| &result.sample[start..result.sample_len])
            else {
                let _ = write!(out, "XJQ ERR maint-audit no-body");
                return;
            };

            let mut event = NetReplyBuf::new();
            let _ = write!(
                event,
                "maint-status-v1 iface={} host=",
                PCIE_CONTROLLERS[index].name
            );
            write_ascii_token(&mut event, host.as_bytes());
            let _ = write!(
                event,
                " ip={}.{}.{}.{} port={} http_status={} response_bytes={} payload_bytes={} ",
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.status_code,
                result.response_bytes,
                result.payload_bytes
            );
            write_maint_event_fields(&mut event, body);

            if event.as_slice().len() > DB_EVENT_PAYLOAD_BYTES {
                let _ = write!(
                    out,
                    "XJQ ERR maint-audit payload-too-long len={} max={DB_EVENT_PAYLOAD_BYTES}",
                    event.as_slice().len()
                );
                return;
            }

            match append_db_event(
                DB_EVENT_KIND_MAINT_STATUS,
                DB_EVENT_SOURCE_MAINTAINER,
                event.as_slice(),
            ) {
                Ok((_info, _catalog, meta, db_event)) => {
                    let _ = write!(
                        out,
                        "XJQ OK maint-audit iface={} host={} ip={}.{}.{}.{} port={} status={} seq={} page={} kind={} source={} payload-len={} payload-crc=0x{:08x} events={} next-page={} ",
                        PCIE_CONTROLLERS[index].name,
                        host,
                        result.tcp.dst_ip[0],
                        result.tcp.dst_ip[1],
                        result.tcp.dst_ip[2],
                        result.tcp.dst_ip[3],
                        result.tcp.dst_port,
                        result.status_code,
                        db_event.sequence,
                        db_event.page,
                        db_event.kind,
                        db_event.source,
                        db_event.payload_len,
                        db_event.payload_crc,
                        meta.event_count,
                        meta.next_page
                    );
                    write_maint_field(out, body, b"registered");
                    write_maint_field(out, body, b"commands_pending");
                    write_maint_field(out, body, b"next_pending_command_id");
                    write_maint_field(out, body, b"next_pending_command_kind");
                    write_maint_field(out, body, b"next_pending_command_sha256");
                }
                Err(error) => {
                    let _ = write!(out, "XJQ ERR maint-audit {}", error.as_str());
                    if let DbError::Sd(SdReadError::StatusError(bits))
                    | DbError::Sd(SdReadError::IdmacError(bits)) = error
                    {
                        let _ = write!(out, " bits=0x{bits:08x}");
                    }
                }
            }
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-audit {}", error.as_str());
        }
    }
}

fn write_maint_event_summary<W: Write>(
    out: &mut W,
    index: usize,
    host: &str,
    port: u16,
    path: &str,
    kind: &str,
) {
    if !is_public_maint_event_kind(kind) {
        let _ = write!(out, "XJQ ERR maint-event bad-kind");
        return;
    }
    let mut body = NetReplyBuf::new();
    if write!(
        body,
        "{{\"events\":[{{\"kind\":\"{}\",\"payload\":{{\"source\":\"xiaojin-os\",\"iface\":\"{}\"}}}}]}}",
        kind, PCIE_CONTROLLERS[index].name
    )
    .is_err()
    {
        let _ = write!(out, "XJQ ERR maint-event body-too-long");
        return;
    }

    match net_http_post(index, host, port, path, body.as_slice(), HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let _ = write!(
                out,
                "XJQ OK maint-event iface={} host={} ip={}.{}.{}.{} port={} status={} bytes={} payload={} segments={} fin={} truncated={} sent={} src-port={} kind={} first-line=\"",
                PCIE_CONTROLLERS[index].name,
                host,
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.status_code,
                result.response_bytes,
                result.payload_bytes,
                result.segments,
                yes_no(result.fin),
                yes_no(result.truncated),
                result.sent_payload_len,
                result.tcp.src_port,
                kind
            );
            write_http_sample_line(out, &result.sample[..result.sample_len]);
            let _ = write!(out, "\"");
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-event {}", error.as_str());
        }
    }
}

fn write_maint_ack_intent_summary<W: Write>(
    out: &mut W,
    index: usize,
    host: &str,
    port: u16,
    poll_path: &str,
    event_path: &str,
) {
    match net_http_get(index, host, port, poll_path, HTTP_TIMEOUT_MS) {
        Ok(poll) => {
            let Some(body) = find_http_body_start(&poll.sample[..poll.sample_len])
                .map(|start| &poll.sample[start..poll.sample_len])
            else {
                let _ = write!(out, "XJQ ERR maint-ack-intent no-body");
                return;
            };

            let command_id = find_maint_text_field(body, b"next_pending_command_id");
            let kind = find_maint_text_field(body, b"next_pending_command_kind");
            let payload_sha = find_maint_text_field(body, b"next_pending_payload_sha256");
            let command_sha = find_maint_text_field(body, b"next_pending_command_sha256");
            let signer_id = find_maint_text_field(body, b"next_pending_signer_id");
            let signature_sha = find_maint_text_field(body, b"next_pending_signature_sha256");
            let signature_present = find_maint_text_field(body, b"next_pending_signature_present");

            let mut calculated = [0u8; 32];
            let (pending, digest_ok, digest_calculated, reason) =
                match (command_id, kind, payload_sha, command_sha) {
                    (Some(id), _, _, _) if ascii_eq(id, b"none") => {
                        (false, false, false, "no-pending")
                    }
                    (Some(id), Some(kind), Some(payload_sha), Some(command_sha)) => {
                        if !is_sha256_hex(payload_sha) {
                            (true, false, false, "bad-payload-sha")
                        } else if !is_sha256_hex(command_sha) {
                            (true, false, false, "bad-command-sha")
                        } else if let Some(digest) = maint_command_digest(id, kind, payload_sha) {
                            calculated = digest;
                            if sha256_hex_matches(&calculated, command_sha) {
                                (true, true, true, "ok")
                            } else {
                                (true, false, true, "mismatch")
                            }
                        } else {
                            (true, false, false, "canonical-too-long")
                        }
                    }
                    _ => (true, false, false, "missing-field"),
                };

            let identity_fingerprint = device_identity_digest();
            let mut post_body = NetReplyBuf::new();
            let _ = write!(
                post_body,
                "{{\"events\":[{{\"kind\":\"maint_ack_intent\",\"payload\":{{\"s\":\"xj\",\"di\":\"{XIAOJIN_DEVICE_ID}\",\"if\":\"{}\",\"fp\":\"",
                PCIE_CONTROLLERS[index].name,
            );
            write_sha256_hex(&mut post_body, &identity_fingerprint);
            let _ = write!(
                post_body,
                "\",\"p\":{},\"d\":{},\"r\":\"{}\",\"h\":\"",
                if pending { "true" } else { "false" },
                if digest_ok { "true" } else { "false" },
                reason
            );
            if let Some(command_sha) = command_sha {
                write_ascii_token(&mut post_body, command_sha);
            } else {
                let _ = write!(post_body, "none");
            }
            let _ = write!(post_body, "\",\"x\":\"");
            if digest_calculated {
                write_sha256_hex(&mut post_body, &calculated);
            } else {
                let _ = write!(post_body, "none");
            }
            let _ = write!(post_body, "\",\"si\":\"");
            if let Some(signer_id) = signer_id {
                write_ascii_token(&mut post_body, signer_id);
            } else {
                let _ = write!(post_body, "none");
            }
            let _ = write!(post_body, "\",\"sg\":\"");
            if let Some(signature_sha) = signature_sha {
                write_ascii_token(&mut post_body, signature_sha);
            } else {
                let _ = write!(post_body, "none");
            }
            let _ = write!(post_body, "\",\"sp\":\"");
            if let Some(signature_present) = signature_present {
                write_ascii_token(&mut post_body, signature_present);
            } else {
                let _ = write!(post_body, "no");
            }
            let _ = write!(post_body, "\"}}}}]}}");

            if post_body.as_slice().len() > 1024 {
                let _ = write!(
                    out,
                    "XJQ ERR maint-ack-intent body-too-long len={}",
                    post_body.as_slice().len()
                );
                return;
            }

            match net_http_post(
                index,
                host,
                port,
                event_path,
                post_body.as_slice(),
                HTTP_TIMEOUT_MS,
            ) {
                Ok(event) => {
                    let _ = write!(
                        out,
                        "XJQ OK maint-ack-intent iface={} host={} ip={}.{}.{}.{} port={} poll-status={} event-status={} pending={} digest-ok={} reason={} sent={} src-port={} first-line=\"",
                        PCIE_CONTROLLERS[index].name,
                        host,
                        event.tcp.dst_ip[0],
                        event.tcp.dst_ip[1],
                        event.tcp.dst_ip[2],
                        event.tcp.dst_ip[3],
                        event.tcp.dst_port,
                        poll.status_code,
                        event.status_code,
                        yes_no(pending),
                        yes_no(digest_ok),
                        reason,
                        event.sent_payload_len,
                        event.tcp.src_port
                    );
                    write_http_sample_line(out, &event.sample[..event.sample_len]);
                    let _ = write!(out, "\" ");
                    write_maint_field(out, body, b"device_id");
                    write_maint_field(out, body, b"next_pending_command_id");
                    write_maint_field(out, body, b"next_pending_command_kind");
                    write_maint_field(out, body, b"next_pending_command_sha256");
                    write_maint_field(out, body, b"next_pending_signer_id");
                    write_maint_field(out, body, b"next_pending_signature_sha256");
                    write_maint_field(out, body, b"next_pending_signature_present");
                    let _ = write!(out, "calculated=");
                    if digest_calculated {
                        write_sha256_hex(out, &calculated);
                    } else {
                        let _ = write!(out, "none");
                    }
                }
                Err(error) => {
                    let _ = write!(out, "XJQ ERR maint-ack-intent event {}", error.as_str());
                }
            }
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-ack-intent poll {}", error.as_str());
        }
    }
}

fn write_maint_verify_summary<W: Write>(
    out: &mut W,
    index: usize,
    host: &str,
    port: u16,
    path: &str,
    hmac_key: MaintHmacKey<'_>,
) {
    match net_http_get(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let Some(body) = find_http_body_start(&result.sample[..result.sample_len])
                .map(|start| &result.sample[start..result.sample_len])
            else {
                let _ = write!(out, "XJQ ERR maint-verify no-body");
                return;
            };

            let command_id = find_maint_text_field(body, b"next_pending_command_id");
            let kind = find_maint_text_field(body, b"next_pending_command_kind");
            let payload_sha = find_maint_text_field(body, b"next_pending_payload_sha256");
            let command_sha = find_maint_text_field(body, b"next_pending_command_sha256");
            let signature_algorithm =
                find_maint_text_field(body, b"next_pending_signature_algorithm");
            let signature_sha = find_maint_text_field(body, b"next_pending_signature_sha256");
            let signature_present = find_maint_text_field(body, b"next_pending_signature_present");

            let mut calculated = [0u8; 32];
            let (pending, digest_ok, digest_calculated, reason) =
                match (command_id, kind, payload_sha, command_sha) {
                    (Some(id), _, _, _) if ascii_eq(id, b"none") => {
                        (false, false, false, "no-pending")
                    }
                    (Some(id), Some(kind), Some(payload_sha), Some(command_sha)) => {
                        if !is_sha256_hex(payload_sha) {
                            (true, false, false, "bad-payload-sha")
                        } else if !is_sha256_hex(command_sha) {
                            (true, false, false, "bad-command-sha")
                        } else if let Some(digest) = maint_command_digest(id, kind, payload_sha) {
                            calculated = digest;
                            if sha256_hex_matches(&calculated, command_sha) {
                                (true, true, true, "ok")
                            } else {
                                (true, false, true, "mismatch")
                            }
                        } else {
                            (true, false, false, "canonical-too-long")
                        }
                    }
                    _ => (true, false, false, "missing-field"),
                };
            let mut signature_calculated = [0u8; 32];
            let (signature_checked, signature_ok, signature_calculated_valid, signature_reason) =
                if hmac_key.is_some() {
                    match maint_hmac_signature_digest(command_sha, signature_algorithm, hmac_key) {
                        MaintHmacCheckInput::NoPending => (true, false, false, "no-pending"),
                        MaintHmacCheckInput::DigestNotOk => (true, false, false, "digest-not-ok"),
                        MaintHmacCheckInput::BadKey => (true, false, false, "bad-key"),
                        MaintHmacCheckInput::BadAlgorithm => (true, false, false, "bad-algorithm"),
                        MaintHmacCheckInput::Calculated(digest) => {
                            signature_calculated = digest;
                            match (signature_present, signature_sha) {
                                (Some(present), _) if !ascii_eq(present, b"yes") => {
                                    (true, false, true, "signature-missing")
                                }
                                (_, Some(signature_sha))
                                    if sha256_hex_matches(&signature_calculated, signature_sha) =>
                                {
                                    (true, true, true, "ok")
                                }
                                (_, Some(signature_sha)) if !is_sha256_hex(signature_sha) => {
                                    (true, false, true, "bad-signature-sha")
                                }
                                (_, Some(_)) => (true, false, true, "signature-mismatch"),
                                _ => (true, false, true, "missing-signature-sha"),
                            }
                        }
                    }
                } else {
                    (false, false, false, "not-requested")
                };

            let mut event = NetReplyBuf::new();
            let _ = write!(
                event,
                "maint-command-verify-v1 iface={} host=",
                PCIE_CONTROLLERS[index].name
            );
            write_ascii_token(&mut event, host.as_bytes());
            let _ = write!(
                event,
                " ip={}.{}.{}.{} port={} http_status={} pending={} digest-ok={} reason={} ",
                result.tcp.dst_ip[0],
                result.tcp.dst_ip[1],
                result.tcp.dst_ip[2],
                result.tcp.dst_ip[3],
                result.tcp.dst_port,
                result.status_code,
                yes_no(pending),
                yes_no(digest_ok),
                reason
            );
            if signature_checked {
                let _ = write!(
                    event,
                    "sig-check=yes sig-ok={} sig-reason={} ",
                    yes_no(signature_ok),
                    signature_reason
                );
            } else {
                let _ = write!(event, "sig-check=no sig-ok=no sig-reason=not-requested ");
            }
            write_maint_compact_field(&mut event, "device_id", body, b"device_id");
            write_maint_compact_field(&mut event, "command_id", body, b"next_pending_command_id");
            write_maint_compact_field(&mut event, "kind", body, b"next_pending_command_kind");
            write_maint_compact_field(
                &mut event,
                "command_sha",
                body,
                b"next_pending_command_sha256",
            );
            write_maint_compact_field(&mut event, "signer", body, b"next_pending_signer_id");
            write_maint_compact_field(&mut event, "sigsha", body, b"next_pending_signature_sha256");
            write_maint_compact_field(&mut event, "sigp", body, b"next_pending_signature_present");

            if event.as_slice().len() > DB_EVENT_PAYLOAD_BYTES {
                let _ = write!(
                    out,
                    "XJQ ERR maint-verify payload-too-long len={} max={DB_EVENT_PAYLOAD_BYTES}",
                    event.as_slice().len()
                );
                return;
            }

            match append_db_event(
                DB_EVENT_KIND_MAINT_VERIFY,
                DB_EVENT_SOURCE_MAINTAINER,
                event.as_slice(),
            ) {
                Ok((_info, _catalog, meta, db_event)) => {
                    let _ = write!(
                        out,
                        "XJQ OK maint-verify iface={} host={} ip={}.{}.{}.{} port={} status={} pending={} digest-ok={} reason={} seq={} page={} kind={} source={} payload-len={} payload-crc=0x{:08x} events={} next-page={} ",
                        PCIE_CONTROLLERS[index].name,
                        host,
                        result.tcp.dst_ip[0],
                        result.tcp.dst_ip[1],
                        result.tcp.dst_ip[2],
                        result.tcp.dst_ip[3],
                        result.tcp.dst_port,
                        result.status_code,
                        yes_no(pending),
                        yes_no(digest_ok),
                        reason,
                        db_event.sequence,
                        db_event.page,
                        db_event.kind,
                        db_event.source,
                        db_event.payload_len,
                        db_event.payload_crc,
                        meta.event_count,
                        meta.next_page
                    );
                    write_maint_field(out, body, b"device_id");
                    write_maint_field(out, body, b"next_pending_command_id");
                    write_maint_field(out, body, b"next_pending_command_kind");
                    write_maint_field(out, body, b"next_pending_payload_sha256");
                    write_maint_field(out, body, b"next_pending_command_sha256");
                    write_maint_field(out, body, b"next_pending_signer_id");
                    write_maint_field(out, body, b"next_pending_signature_algorithm");
                    write_maint_field(out, body, b"next_pending_signature_sha256");
                    write_maint_field(out, body, b"next_pending_signature_present");
                    let _ = write!(
                        out,
                        "signature-checked={} signature-ok={} signature-reason={} signature-calculated=",
                        yes_no(signature_checked),
                        yes_no(signature_ok),
                        signature_reason
                    );
                    if signature_calculated_valid {
                        write_sha256_hex(out, &signature_calculated);
                    } else {
                        let _ = write!(out, "none");
                    }
                    let _ = write!(out, " ");
                    let _ = write!(out, "calculated=");
                    if digest_calculated {
                        write_sha256_hex(out, &calculated);
                    } else {
                        let _ = write!(out, "none");
                    }
                }
                Err(error) => {
                    let _ = write!(out, "XJQ ERR maint-verify {}", error.as_str());
                    if let DbError::Sd(SdReadError::StatusError(bits))
                    | DbError::Sd(SdReadError::IdmacError(bits)) = error
                    {
                        let _ = write!(out, " bits=0x{bits:08x}");
                    }
                }
            }
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-verify {}", error.as_str());
        }
    }
}

fn write_maint_pull_exec_summary<W: Write>(
    out: &mut W,
    index: usize,
    host: &str,
    port: u16,
    path: &str,
    event_path: Option<&str>,
    hmac_key: MaintHmacKey<'_>,
) {
    match net_http_get(index, host, port, path, HTTP_TIMEOUT_MS) {
        Ok(result) => {
            let Some(body) = find_http_body_start(&result.sample[..result.sample_len])
                .map(|start| &result.sample[start..result.sample_len])
            else {
                let _ = write!(out, "XJQ ERR maint-pull-exec no-body");
                return;
            };

            let command_id = find_maint_text_field(body, b"next_pending_command_id");
            let kind = find_maint_text_field(body, b"next_pending_command_kind");
            let payload_sha = find_maint_text_field(body, b"next_pending_payload_sha256");
            let command_sha = find_maint_text_field(body, b"next_pending_command_sha256");
            let signature_algorithm =
                find_maint_text_field(body, b"next_pending_signature_algorithm");
            let signature_sha = find_maint_text_field(body, b"next_pending_signature_sha256");
            let signature_present = find_maint_text_field(body, b"next_pending_signature_present");
            let public_query_present =
                find_maint_text_field(body, b"next_pending_public_query_present");
            let public_query_hex = find_maint_text_field(body, b"next_pending_public_query_hex");
            let public_query_sha = find_maint_text_field(body, b"next_pending_public_query_sha256");

            let Some(command_id) = command_id else {
                let _ = write!(out, "XJQ ERR maint-pull-exec missing-command-id");
                return;
            };
            if ascii_eq(command_id, b"none") {
                let _ = write!(out, "XJQ ERR maint-pull-exec no-pending");
                return;
            }
            let Some(kind) = kind else {
                let _ = write!(out, "XJQ ERR maint-pull-exec missing-kind");
                return;
            };
            if !ascii_eq(kind, b"xj_query") {
                let _ = write!(out, "XJQ ERR maint-pull-exec unsupported-kind kind=");
                write_ascii_token(out, kind);
                return;
            }
            let (Some(payload_sha), Some(command_sha)) = (payload_sha, command_sha) else {
                let _ = write!(out, "XJQ ERR maint-pull-exec missing-digest");
                return;
            };
            if !is_sha256_hex(payload_sha) || !is_sha256_hex(command_sha) {
                let _ = write!(out, "XJQ ERR maint-pull-exec bad-digest");
                return;
            }
            if !matches!(public_query_present, Some(value) if ascii_eq(value, b"yes")) {
                let _ = write!(out, "XJQ ERR maint-pull-exec public-query-missing");
                return;
            }
            let Some(public_query_hex) = public_query_hex else {
                let _ = write!(out, "XJQ ERR maint-pull-exec missing-query-hex");
                return;
            };
            let mut query_buf = [0u8; NETCONTROL_QUERY_BYTES];
            let Some(query_len) = decode_hex_to_buf(public_query_hex, &mut query_buf) else {
                let _ = write!(out, "XJQ ERR maint-pull-exec bad-query-hex");
                return;
            };
            let query_bytes = &query_buf[..query_len];
            if !is_public_xj_query(query_bytes) {
                let _ = write!(out, "XJQ ERR maint-pull-exec query-denied query=");
                write_ascii_token(out, query_bytes);
                return;
            }
            if let Some(public_query_sha) = public_query_sha {
                let query_digest = sha256_digest(query_bytes);
                if !sha256_hex_matches(&query_digest, public_query_sha) {
                    let _ = write!(out, "XJQ ERR maint-pull-exec query-sha-mismatch");
                    return;
                }
            }
            let Some(payload_digest) = maint_public_query_payload_digest(query_bytes) else {
                let _ = write!(out, "XJQ ERR maint-pull-exec payload-canonical-too-long");
                return;
            };
            if !sha256_hex_matches(&payload_digest, payload_sha) {
                let _ = write!(out, "XJQ ERR maint-pull-exec payload-mismatch calculated=");
                write_sha256_hex(out, &payload_digest);
                return;
            }
            let Some(command_digest) = maint_command_digest(command_id, kind, payload_sha) else {
                let _ = write!(out, "XJQ ERR maint-pull-exec command-canonical-too-long");
                return;
            };
            if !sha256_hex_matches(&command_digest, command_sha) {
                let _ = write!(out, "XJQ ERR maint-pull-exec command-mismatch calculated=");
                write_sha256_hex(out, &command_digest);
                return;
            }

            let signature_calculated =
                match maint_hmac_signature_digest(Some(command_sha), signature_algorithm, hmac_key)
                {
                    MaintHmacCheckInput::NoPending => {
                        let _ = write!(out, "XJQ ERR maint-pull-exec signature-no-pending");
                        return;
                    }
                    MaintHmacCheckInput::DigestNotOk => {
                        let _ = write!(out, "XJQ ERR maint-pull-exec signature-bad-digest");
                        return;
                    }
                    MaintHmacCheckInput::BadKey => {
                        let _ = write!(out, "XJQ ERR maint-pull-exec signature-bad-key");
                        return;
                    }
                    MaintHmacCheckInput::BadAlgorithm => {
                        let _ = write!(out, "XJQ ERR maint-pull-exec signature-bad-algorithm");
                        return;
                    }
                    MaintHmacCheckInput::Calculated(digest) => digest,
                };
            if !matches!(signature_present, Some(value) if ascii_eq(value, b"yes")) {
                let _ = write!(out, "XJQ ERR maint-pull-exec signature-missing");
                return;
            }
            let Some(signature_sha) = signature_sha else {
                let _ = write!(out, "XJQ ERR maint-pull-exec signature-sha-missing");
                return;
            };
            if !sha256_hex_matches(&signature_calculated, signature_sha) {
                let _ = write!(
                    out,
                    "XJQ ERR maint-pull-exec signature-mismatch calculated="
                );
                write_sha256_hex(out, &signature_calculated);
                return;
            }

            let Ok(query) = str::from_utf8(query_bytes) else {
                let _ = write!(out, "XJQ ERR maint-pull-exec query-utf8");
                return;
            };
            let mut query_result = NetReplyBuf::new();
            if !write_public_xj_query_result(query, &mut query_result) {
                let _ = write!(out, "XJQ ERR maint-pull-exec query-denied");
                return;
            }

            let mut event = NetReplyBuf::new();
            let result_digest = sha256_digest(query_result.as_slice());
            let _ = write!(
                event,
                "maint-pull-exec-v1 iface={} host=",
                PCIE_CONTROLLERS[index].name
            );
            write_ascii_token(&mut event, host.as_bytes());
            let _ = write!(
                event,
                " port={} http={} query=",
                result.tcp.dst_port, result.status_code
            );
            write_ascii_token(&mut event, query_bytes);
            let _ = write!(event, " command_id=");
            write_ascii_token(&mut event, command_id);
            let _ = write!(event, " command_sha=");
            write_ascii_token(&mut event, command_sha);
            let _ = write!(event, " sig-ok=yes result-sha=");
            write_sha256_hex(&mut event, &result_digest);
            if event.as_slice().len() > DB_EVENT_PAYLOAD_BYTES {
                let _ = write!(
                    out,
                    "XJQ ERR maint-pull-exec payload-too-long len={} max={DB_EVENT_PAYLOAD_BYTES}",
                    event.as_slice().len()
                );
                return;
            }

            match append_db_event(
                DB_EVENT_KIND_MAINT_PULL_EXEC,
                DB_EVENT_SOURCE_MAINTAINER,
                event.as_slice(),
            ) {
                Ok((_info, _catalog, meta, db_event)) => {
                    let report = event_path.map(|event_path| {
                        post_maint_pull_exec_event(
                            index,
                            host,
                            port,
                            event_path,
                            command_id,
                            command_sha,
                            signature_sha,
                            query_bytes,
                            &result_digest,
                            db_event,
                        )
                    });
                    let _ = write!(
                        out,
                        "XJQ OK maint-pull-exec iface={} host={} ip={}.{}.{}.{} port={} status={} signature-ok=yes seq={} page={} events={} next-page={} query=\"",
                        PCIE_CONTROLLERS[index].name,
                        host,
                        result.tcp.dst_ip[0],
                        result.tcp.dst_ip[1],
                        result.tcp.dst_ip[2],
                        result.tcp.dst_ip[3],
                        result.tcp.dst_port,
                        result.status_code,
                        db_event.sequence,
                        db_event.page,
                        meta.event_count,
                        meta.next_page
                    );
                    write_http_sample_line(out, query_bytes);
                    let _ = write!(out, "\" command_id=");
                    write_ascii_token(out, command_id);
                    match report {
                        Some(Ok(report)) => {
                            let _ = write!(
                                out,
                                " report=ok report-status={} report-bytes={} report-sent={} report-src-port={}",
                                report.status_code,
                                report.response_bytes,
                                report.sent_payload_len,
                                report.tcp.src_port
                            );
                        }
                        Some(Err(error)) => {
                            let _ = write!(out, " report=err report-error={}", error.as_str());
                        }
                        None => {
                            let _ = write!(out, " report=skipped");
                        }
                    }
                    let _ = write!(out, " result=\"");
                    write_http_sample_line(out, query_result.as_slice());
                    let _ = write!(out, "\"");
                }
                Err(error) => {
                    let _ = write!(out, "XJQ ERR maint-pull-exec {}", error.as_str());
                    if let DbError::Sd(SdReadError::StatusError(bits))
                    | DbError::Sd(SdReadError::IdmacError(bits)) = error
                    {
                        let _ = write!(out, " bits=0x{bits:08x}");
                    }
                }
            }
        }
        Err(error) => {
            let _ = write!(out, "XJQ ERR maint-pull-exec {}", error.as_str());
        }
    }
}

fn post_maint_pull_exec_event(
    index: usize,
    host: &str,
    port: u16,
    event_path: &str,
    command_id: &[u8],
    command_sha: &[u8],
    signature_sha: &[u8],
    query: &[u8],
    result_digest: &[u8; 32],
    db_event: DbEventPage,
) -> Result<HttpPostResult, HttpError> {
    let mut post_body = NetReplyBuf::new();
    write!(
        post_body,
        "{{\"events\":[{{\"kind\":\"maint_pull_exec\",\"payload\":{{\"source\":\"xiaojin-os\",\"device_id\":\"{XIAOJIN_DEVICE_ID}\",\"iface\":\"{}\",\"command_id\":\"",
        PCIE_CONTROLLERS[index].name,
    )
    .map_err(|_| HttpError::BadBody)?;
    if !write_json_ascii_value(&mut post_body, command_id, 96) {
        return Err(HttpError::BadBody);
    }
    write!(post_body, "\",\"command_sha256\":\"").map_err(|_| HttpError::BadBody)?;
    if !write_json_ascii_value(&mut post_body, command_sha, 64) {
        return Err(HttpError::BadBody);
    }
    write!(post_body, "\",\"signature_sha256\":\"").map_err(|_| HttpError::BadBody)?;
    if !write_json_ascii_value(&mut post_body, signature_sha, 64) {
        return Err(HttpError::BadBody);
    }
    write!(post_body, "\",\"query\":\"").map_err(|_| HttpError::BadBody)?;
    if !write_json_ascii_value(&mut post_body, query, 96) {
        return Err(HttpError::BadBody);
    }
    write!(post_body, "\",\"result_sha256\":\"").map_err(|_| HttpError::BadBody)?;
    write_sha256_hex(&mut post_body, result_digest);
    write!(
        post_body,
        "\",\"db_seq\":{},\"db_page\":{},\"ok\":true}}}}]}}",
        db_event.sequence, db_event.page
    )
    .map_err(|_| HttpError::BadBody)?;

    if post_body.as_slice().len() > 1024 {
        return Err(HttpError::BadBody);
    }

    net_http_post(
        index,
        host,
        port,
        event_path,
        post_body.as_slice(),
        HTTP_TIMEOUT_MS,
    )
}

fn write_json_ascii_value(buffer: &mut NetReplyBuf, value: &[u8], max: usize) -> bool {
    for byte in value.iter().copied().take(max) {
        let out = match byte {
            b'"' | b'\\' => '_',
            0x20..=0x7e => byte as char,
            _ => '_',
        };
        if write!(buffer, "{out}").is_err() {
            return false;
        }
    }
    true
}

fn maint_public_query_payload_digest(query: &[u8]) -> Option<[u8; 32]> {
    if !is_public_xj_query(query) {
        return None;
    }
    let mut canonical = NetReplyBuf::new();
    for part in [b"{\"query\":\"".as_slice(), query, b"\"}".as_slice()] {
        if !net_reply_push_checked(&mut canonical, part) {
            return None;
        }
    }
    Some(sha256_digest(canonical.as_slice()))
}

fn is_public_xj_query(query: &[u8]) -> bool {
    if query.is_empty()
        || query.len() > 96
        || query.first().is_some_and(|byte| byte.is_ascii_whitespace())
        || query.last().is_some_and(|byte| byte.is_ascii_whitespace())
        || query.iter().any(|byte| {
            !(byte.is_ascii_graphic() || *byte == b' ') || matches!(*byte, b'"' | b'\\')
        })
    {
        return false;
    }
    let Ok(query) = str::from_utf8(query) else {
        return false;
    };

    [
        "caps",
        "identity",
        "identity facts",
        "net ips",
        "db facts",
        "db status",
        "db status confirm",
        "db catalog",
        "db catalog confirm",
        "db event-status",
        "db event-status confirm",
        "maint key-status",
        "maint key-status confirm",
        "sd extlinux",
        "sd extlinux confirm",
    ]
    .iter()
    .any(|allowed| query.eq_ignore_ascii_case(allowed))
}

fn write_public_xj_query_result<W: Write>(query: &str, out: &mut W) -> bool {
    if query.eq_ignore_ascii_case("caps") {
        write_caps_query_summary(out);
    } else if query.eq_ignore_ascii_case("identity") || query.eq_ignore_ascii_case("identity facts")
    {
        let _ = write!(out, "XJQ OK ");
        write_identity_summary(out);
    } else if query.eq_ignore_ascii_case("net ips") {
        let _ = write!(out, "XJQ OK ");
        write_net_ips_summary(out);
    } else if query.eq_ignore_ascii_case("db facts") {
        let _ = write!(
            out,
            "XJQ OK db-facts file={DB_FILE_NAME_STR} page-bytes={DB_PAGE_BYTES} superblock=0 catalog={DB_ROOT_CATALOG_PAGE} wal-start={DB_WAL_START_PAGE} wal-pages={DB_WAL_PAGE_COUNT} table-root={DB_TABLE_ROOT_PAGE} table-records={DB_TABLE_RECORDS} btree-root={DB_BTREE_ROOT_PAGE} btree-leaf-start={DB_BTREE_LEAF_START_PAGE} btree-leaves={DB_BTREE_LEAF_COUNT} event-meta={DB_EVENT_META_PAGE} event-start={DB_EVENT_START_PAGE} event-pages={DB_EVENT_PAGE_COUNT} blob-meta={DB_BLOB_META_PAGE} blob-start={DB_BLOB_START_PAGE} value-bytes={DB_TABLE_VALUE_BYTES} persistence=preallocated-fat32"
        );
    } else if query.eq_ignore_ascii_case("db status")
        || query.eq_ignore_ascii_case("db status confirm")
    {
        write_db_status_summary(out);
    } else if query.eq_ignore_ascii_case("db catalog")
        || query.eq_ignore_ascii_case("db catalog confirm")
    {
        write_db_catalog_summary(out);
    } else if query.eq_ignore_ascii_case("db event-status")
        || query.eq_ignore_ascii_case("db event-status confirm")
    {
        write_db_event_status_summary(out);
    } else if query.eq_ignore_ascii_case("maint key-status")
        || query.eq_ignore_ascii_case("maint key-status confirm")
    {
        write_maint_key_status_summary(out);
    } else if query.eq_ignore_ascii_case("sd extlinux")
        || query.eq_ignore_ascii_case("sd extlinux confirm")
    {
        let _ = write!(out, "XJQ ");
        write_sd_extlinux_summary(out);
    } else {
        return false;
    }
    true
}

fn write_maint_event_fields<W: Write>(out: &mut W, body: &[u8]) {
    write_maint_field(out, body, b"device_id");
    write_maint_field(out, body, b"registered");
    write_maint_field(out, body, b"board_id");
    write_maint_field(out, body, b"arch");
    write_maint_field(out, body, b"db_event_sequence");
    write_maint_field(out, body, b"commands_pending");
    write_maint_field(out, body, b"next_pending_command_id");
    write_maint_field(out, body, b"next_pending_command_kind");
    write_maint_field(out, body, b"next_pending_command_sha256");
    write_maint_compact_field(out, "signer", body, b"next_pending_signer_id");
    write_maint_compact_field(out, "sigsha", body, b"next_pending_signature_sha256");
    write_maint_compact_field(out, "sigp", body, b"next_pending_signature_present");
}

fn write_maint_field<W: Write>(out: &mut W, body: &[u8], key: &[u8]) {
    let _ = write!(out, "{}=", AsciiBytes(key));
    if let Some(value) = find_maint_text_field(body, key) {
        write_ascii_token(out, value);
    } else {
        let _ = write!(out, "?");
    }
    let _ = write!(out, " ");
}

fn write_maint_compact_field<W: Write>(out: &mut W, name: &str, body: &[u8], key: &[u8]) {
    let _ = write!(out, "{name}=");
    if let Some(value) = find_maint_text_field(body, key) {
        write_ascii_token(out, value);
    } else {
        let _ = write!(out, "?");
    }
    let _ = write!(out, " ");
}

fn is_public_maint_event_kind(kind: &str) -> bool {
    matches!(
        kind,
        "maint_poll" | "maint_verify" | "maint_ack_intent" | "maint_pull_exec" | "heartbeat"
    )
}

fn find_maint_text_field<'a>(body: &'a [u8], key: &[u8]) -> Option<&'a [u8]> {
    let mut start = 0usize;
    while start < body.len() {
        let mut end = start;
        while end < body.len() && body[end] != b'\n' && body[end] != b'\r' {
            end += 1;
        }
        let line = &body[start..end];
        if line.len() > key.len() && &line[..key.len()] == key && line[key.len()] == b'=' {
            return Some(&line[key.len() + 1..]);
        }
        start = end.saturating_add(1);
    }
    None
}

struct AsciiBytes<'a>(&'a [u8]);

impl<'a> fmt::Display for AsciiBytes<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            if (0x20..=0x7e).contains(byte) {
                write!(f, "{}", *byte as char)?;
            }
        }
        Ok(())
    }
}

fn write_ascii_token<W: Write>(out: &mut W, value: &[u8]) {
    let mut count = 0usize;
    for byte in value {
        let ch = if (0x21..=0x7e).contains(byte) && *byte != b'"' && *byte != b'\\' {
            *byte as char
        } else {
            '_'
        };
        let _ = write!(out, "{ch}");
        count += 1;
        if count >= 64 {
            break;
        }
    }
}

fn write_http_sample_line<W: Write>(out: &mut W, sample: &[u8]) {
    let sample = if let Some(start) = find_http_status_line(sample) {
        &sample[start..]
    } else {
        sample
    };
    let mut count = 0usize;
    for byte in sample {
        if *byte == b'\r' || *byte == b'\n' {
            break;
        }
        let ch = if (0x20..=0x7e).contains(byte) {
            *byte as char
        } else {
            '.'
        };
        let _ = write!(out, "{ch}");
        count += 1;
        if count >= HTTP_RESPONSE_SAMPLE_BYTES {
            break;
        }
    }
}

fn write_http_body_sample_line<W: Write>(out: &mut W, sample: &[u8]) {
    let Some(start) = find_http_body_start(sample) else {
        return;
    };
    let mut count = 0usize;
    for byte in &sample[start..] {
        let ch = if *byte == b'\r' || *byte == b'\n' {
            ' '
        } else if (0x20..=0x7e).contains(byte) {
            *byte as char
        } else {
            '.'
        };
        let _ = write!(out, "{ch}");
        count += 1;
        if count >= 96 {
            break;
        }
    }
}

fn find_http_body_start(sample: &[u8]) -> Option<usize> {
    let mut index = 0usize;
    while index + 3 < sample.len() {
        if sample[index] == b'\r'
            && sample[index + 1] == b'\n'
            && sample[index + 2] == b'\r'
            && sample[index + 3] == b'\n'
        {
            return Some(index + 4);
        }
        index += 1;
    }

    index = 0;
    while index + 1 < sample.len() {
        if sample[index] == b'\n' && sample[index + 1] == b'\n' {
            return Some(index + 2);
        }
        index += 1;
    }
    None
}

fn write_http_raw_sample_line<W: Write>(out: &mut W, sample: &[u8]) {
    let mut count = 0usize;
    for byte in sample {
        match *byte {
            b'\r' => {
                let _ = write!(out, "\\r");
            }
            b'\n' => {
                let _ = write!(out, "\\n");
            }
            b'"' => {
                let _ = write!(out, "\\\"");
            }
            b'\\' => {
                let _ = write!(out, "\\\\");
            }
            0x20..=0x7e => {
                let _ = write!(out, "{}", *byte as char);
            }
            _ => {
                let _ = write!(out, ".");
            }
        }
        count += 1;
        if count >= 192 {
            break;
        }
    }
}

fn print_dhcp_lease(uart: &mut Uart, label: &str, lease: DhcpLease) {
    let _ = writeln!(
        uart,
        "{label:<16} type={} ip={}.{}.{}.{} mask={}.{}.{}.{} gateway={}.{}.{}.{} dns={}.{}.{}.{} server={}.{}.{}.{} lease={}s",
        lease.message_type,
        lease.ip[0],
        lease.ip[1],
        lease.ip[2],
        lease.ip[3],
        lease.netmask[0],
        lease.netmask[1],
        lease.netmask[2],
        lease.netmask[3],
        lease.gateway[0],
        lease.gateway[1],
        lease.gateway[2],
        lease.gateway[3],
        lease.dns[0],
        lease.dns[1],
        lease.dns[2],
        lease.dns[3],
        lease.server[0],
        lease.server[1],
        lease.server[2],
        lease.server[3],
        lease.lease_seconds
    );
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
    uart.put_str("  stage 5          runtime static IP config for routed LAN bring-up\n");
    uart.put_str("  stage 6          manual DHCP client for runtime IP/gateway/DNS facts\n");
    uart.put_str("  stage 7          UDP XJDH control packet can trigger DHCP without TTL\n");
    uart.put_str("  rescue path      UART xload remains the fallback until UDP loader is stable\n");
}

fn command_net_loop(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net loop confirm\n");
        uart.put_str("scope: initialize RTL8125 and answer ARP/UDP xjping for 120 seconds\n");
        return;
    }

    command_net_polling(uart, "net loop", Some(NET_LOOP_DURATION_MS));
}

fn command_net_listen(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net listen confirm\n");
        uart.put_str("scope: initialize RTL8125 and answer ARP/UDP xjping until Ctrl-C\n");
        return;
    }

    command_net_polling(uart, "net listen", None);
}

fn command_net_polling(uart: &mut Uart, label: &str, duration_ms: Option<u64>) {
    let _ = writeln!(uart, "{label}:");
    for (index, controller) in PCIE_CONTROLLERS.iter().enumerate() {
        let ip = net_device_ip(index);
        let mac = rtl_local_mac(index);
        let _ = writeln!(
            uart,
            "  {} ip={}.{}.{}.{} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} udp-port={}",
            controller.name,
            ip[0],
            ip[1],
            ip[2],
            ip[3],
            mac[0],
            mac[1],
            mac[2],
            mac[3],
            mac[4],
            mac[5],
            NET_UDP_PORT
        );
    }

    command_pcie_link_start(uart, Some("confirm"));
    command_pcie_rtl_init(uart, Some("confirm"));
    net_auto_dhcp_if_needed(uart);

    let freq = read_cntfrq();
    let limit_ticks = match duration_ms {
        Some(duration_ms) => {
            let Some(ticks) = ms_to_ticks(duration_ms, freq) else {
                uart.put_str("  timer           invalid CNTFRQ\n");
                return;
            };
            Some(ticks)
        }
        None => None,
    };
    let start = read_cntpct();
    let mut arp_replies = 0u32;
    let mut udp_replies = 0u32;
    let mut control_replies = 0u32;
    let mut rx_packets = 0u32;
    let mut arp_announcements = 0u32;
    let mut udp_hellos = 0u32;
    let announce_ticks = ms_to_ticks(2_000, freq).unwrap_or(1);
    let watchdog_ticks = ms_to_ticks(NET_RX_WATCHDOG_MS, freq).unwrap_or(announce_ticks);
    let auto_dhcp_retry_ticks = ms_to_ticks(NET_AUTO_DHCP_RETRY_MS, freq).unwrap_or(watchdog_ticks);
    let mut next_announce_elapsed = 0u64;
    let mut next_watchdog_elapsed = watchdog_ticks;
    let mut next_auto_dhcp_elapsed = auto_dhcp_retry_ticks;
    let mut last_watchdog_udp_replies = 0u32;
    let mut last_watchdog_control_replies = 0u32;
    let mut last_watchdog_rx_packets = 0u32;
    let mut rx_watchdog_refreshes = 0u32;
    let mut auto_dhcp_retries = 0u32;

    uart.put_str("  state           polling; press Ctrl-C on serial to stop early\n");
    loop {
        let elapsed = read_cntpct().wrapping_sub(start);
        if limit_ticks.is_some_and(|ticks| elapsed >= ticks) {
            break;
        }
        if matches!(uart.get_byte(), Some(0x03)) {
            uart.put_str("  stop            serial Ctrl-C\n");
            break;
        }
        if elapsed >= next_announce_elapsed {
            for index in 0..RTL_DEVICE_COUNT {
                if net_send_gratuitous_arp(index, ARP_REQUEST) {
                    arp_announcements = arp_announcements.wrapping_add(1);
                }
                if net_send_gratuitous_arp(index, ARP_REPLY) {
                    arp_announcements = arp_announcements.wrapping_add(1);
                }
                if net_send_host_hello(index) {
                    udp_hellos = udp_hellos.wrapping_add(1);
                }
            }
            next_announce_elapsed = next_announce_elapsed.wrapping_add(announce_ticks);
        }
        for index in 0..RTL_DEVICE_COUNT {
            let stats = net_poll_device(index);
            arp_replies = arp_replies.wrapping_add(stats.arp_replies);
            udp_replies = udp_replies.wrapping_add(stats.udp_replies);
            control_replies = control_replies.wrapping_add(stats.control_replies);
            rx_packets = rx_packets.wrapping_add(stats.rx_packets);
        }
        if elapsed >= next_watchdog_elapsed {
            let no_udp_progress = udp_replies == last_watchdog_udp_replies;
            let no_control_progress = control_replies == last_watchdog_control_replies
                && rx_packets != last_watchdog_rx_packets;
            if no_udp_progress || no_control_progress {
                for index in 0..RTL_DEVICE_COUNT {
                    rtl_refresh_dma(index);
                    let _ = net_send_host_hello(index);
                }
                rx_watchdog_refreshes = rx_watchdog_refreshes.wrapping_add(1);
            }
            last_watchdog_udp_replies = udp_replies;
            last_watchdog_control_replies = control_replies;
            last_watchdog_rx_packets = rx_packets;
            next_watchdog_elapsed = next_watchdog_elapsed.wrapping_add(watchdog_ticks);
        }
        if NET_AUTO_DHCP && elapsed >= next_auto_dhcp_elapsed {
            if !net_any_device_has_routed_dns() {
                auto_dhcp_retries = auto_dhcp_retries.wrapping_add(1);
                net_auto_dhcp_if_needed(uart);
            }
            next_auto_dhcp_elapsed = read_cntpct()
                .wrapping_sub(start)
                .wrapping_add(auto_dhcp_retry_ticks);
        }
        core::hint::spin_loop();
    }

    let _ = writeln!(
        uart,
        "  done            rx={} arp-announcements={} udp-hellos={} arp-replies={} udp-replies={} control-replies={} rx-watchdog-refreshes={} auto-dhcp-retries={}",
        rx_packets, arp_announcements, udp_hellos, arp_replies, udp_replies, control_replies, rx_watchdog_refreshes, auto_dhcp_retries
    );
}

fn net_auto_dhcp_if_needed(uart: &mut Uart) {
    if !NET_AUTO_DHCP {
        return;
    }
    if net_any_device_has_routed_dns() {
        return;
    }

    let order = [1usize, 0usize];
    for index in order {
        if net_device_has_routed_dns(index) {
            continue;
        }
        let mac = rtl_local_mac(index);
        let xid = 0x584a_0000u32 ^ (read_cntpct() as u32).rotate_left(7) ^ ((index as u32) << 8);
        let _ = writeln!(
            uart,
            "  dhcp-auto      {} mac={:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x} xid=0x{xid:08x}",
            PCIE_CONTROLLERS[index].name, mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
        );
        match net_dhcp_exchange_with_timeout(index, xid, NET_AUTO_DHCP_TIMEOUT_MS) {
            Ok(exchange) => {
                let ack = exchange.ack;
                set_net_device_dhcp_lease(index, ack);
                let _ = net_send_gratuitous_arp(index, ARP_REQUEST);
                let _ = net_send_gratuitous_arp(index, ARP_REPLY);
                let _ = writeln!(
                    uart,
                    "    bound        ip={}.{}.{}.{} mask={}.{}.{}.{} gateway={}.{}.{}.{} dns={}.{}.{}.{} lease={}s",
                    ack.ip[0],
                    ack.ip[1],
                    ack.ip[2],
                    ack.ip[3],
                    ack.netmask[0],
                    ack.netmask[1],
                    ack.netmask[2],
                    ack.netmask[3],
                    ack.gateway[0],
                    ack.gateway[1],
                    ack.gateway[2],
                    ack.gateway[3],
                    ack.dns[0],
                    ack.dns[1],
                    ack.dns[2],
                    ack.dns[3],
                    ack.lease_seconds
                );
                return;
            }
            Err(error) => {
                let _ = writeln!(uart, "    skipped      {}", error.as_str());
            }
        }
    }
}

fn command_net_sniff(uart: &mut Uart, confirm: Option<&str>) {
    if confirm != Some("confirm") {
        uart.put_str("usage: net sniff confirm\n");
        uart.put_str("scope: initialize RTL8125 and print a bounded RX frame sample\n");
        return;
    }

    uart.put_str("net sniff:\n");
    command_pcie_link_start(uart, Some("confirm"));
    command_pcie_rtl_init(uart, Some("confirm"));

    let freq = read_cntfrq();
    let Some(limit_ticks) = ms_to_ticks(NET_SNIFF_DURATION_MS, freq) else {
        uart.put_str("  timer           invalid CNTFRQ\n");
        return;
    };
    let start = read_cntpct();
    let mut rx_packets = 0u32;
    let mut printed = 0u32;

    uart.put_str("  state           sniffing for 10s; press Ctrl-C on serial to stop early\n");
    while read_cntpct().wrapping_sub(start) < limit_ticks && printed < NET_SNIFF_MAX_PRINT {
        if matches!(uart.get_byte(), Some(0x03)) {
            uart.put_str("  stop            serial Ctrl-C\n");
            break;
        }
        for index in 0..RTL_DEVICE_COUNT {
            net_sniff_device(uart, index, &mut rx_packets, &mut printed);
            if printed >= NET_SNIFF_MAX_PRINT {
                break;
            }
        }
        core::hint::spin_loop();
    }

    let _ = writeln!(
        uart,
        "  done            rx={} printed={}",
        rx_packets, printed
    );
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
    uart.put_str("  pcie2x1          enabled on R5C, reset gpio3 C1, candidate M.2 WiFi path\n");
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
                | RTL_RX_ACCEPT_MY_PHYS
                | RTL_RX_ACCEPT_ALL_PHYS,
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
        let (entry, completed) = rtl_send_prepared_frame_with_entry(index, len, 250);
        let desc_addr = rtl_tx_desc_addr(index, entry);
        let intr = probe_read32(base + RTL_REG_INTR_STATUS_8125).unwrap_or(0);
        let status = unsafe { RTL_TX_RINGS.0[index][entry].opts1 };
        let _ = writeln!(
            uart,
            "  {} len={} buffer=0x{buffer_addr:016x} desc={}@0x{desc_addr:016x} complete={} opts1=0x{status:08x} intr=0x{intr:08x}",
            controller.name,
            len,
            entry,
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

fn net_device_ip(index: usize) -> [u8; 4] {
    unsafe { NET_CONFIG.ips[index] }
}

fn net_device_netmask(index: usize) -> [u8; 4] {
    unsafe { NET_CONFIG.netmasks[index] }
}

fn net_device_gateway(index: usize) -> [u8; 4] {
    unsafe { NET_CONFIG.gateways[index] }
}

fn net_device_dns(index: usize) -> [u8; 4] {
    unsafe { NET_CONFIG.dns[index] }
}

fn net_device_has_routed_dns(index: usize) -> bool {
    net_device_gateway(index) != IPV4_ANY_IP && net_device_dns(index) != IPV4_ANY_IP
}

fn net_any_device_has_routed_dns() -> bool {
    for index in 0..RTL_DEVICE_COUNT {
        if net_device_has_routed_dns(index) {
            return true;
        }
    }
    false
}

fn set_net_device_ip(index: usize, ip: [u8; 4]) {
    unsafe {
        NET_CONFIG.ips[index] = ip;
    }
    persist_net_config_to_boot();
}

fn set_net_device_dhcp_lease(index: usize, lease: DhcpLease) {
    unsafe {
        NET_CONFIG.ips[index] = lease.ip;
        NET_CONFIG.netmasks[index] = lease.netmask;
        NET_CONFIG.gateways[index] = lease.gateway;
        NET_CONFIG.dns[index] = lease.dns;
    }
    persist_net_config_to_boot();
}

fn net_broadcast_ip(index: usize) -> [u8; 4] {
    let ip = net_device_ip(index);
    let netmask = net_device_netmask(index);
    if netmask == IPV4_ANY_IP {
        return DEFAULT_NET_BROADCAST_IP;
    }
    [
        ip[0] | !netmask[0],
        ip[1] | !netmask[1],
        ip[2] | !netmask[2],
        ip[3] | !netmask[3],
    ]
}

fn net_accepts_ipv4_dst(index: usize, dst_ip: [u8; 4]) -> bool {
    dst_ip == net_device_ip(index)
        || dst_ip == net_broadcast_ip(index)
        || dst_ip == IPV4_BROADCAST_IP
}

fn persist_net_config_to_boot() {
    unsafe {
        write_volatile(
            NET_CONFIG_BOOT_MAGIC_ADDR as usize as *mut u64,
            NET_CONFIG_BOOT_MAGIC,
        );
        for index in 0..RTL_DEVICE_COUNT {
            write_volatile(
                (NET_CONFIG_BOOT_IPS_ADDR as usize + index * 4) as *mut u32,
                ipv4_to_u32(NET_CONFIG.ips[index]),
            );
            write_volatile(
                (NET_CONFIG_BOOT_NETMASKS_ADDR as usize + index * 4) as *mut u32,
                ipv4_to_u32(NET_CONFIG.netmasks[index]),
            );
            write_volatile(
                (NET_CONFIG_BOOT_GATEWAYS_ADDR as usize + index * 4) as *mut u32,
                ipv4_to_u32(NET_CONFIG.gateways[index]),
            );
            write_volatile(
                (NET_CONFIG_BOOT_DNS_ADDR as usize + index * 4) as *mut u32,
                ipv4_to_u32(NET_CONFIG.dns[index]),
            );
        }
    }
}

fn load_net_config_from_boot() {
    let magic = unsafe { read_volatile(NET_CONFIG_BOOT_MAGIC_ADDR as usize as *const u64) };
    if magic != NET_CONFIG_BOOT_MAGIC {
        return;
    }

    let mut config = NetRuntimeConfig::default();
    for index in 0..RTL_DEVICE_COUNT {
        let raw =
            unsafe { read_volatile((NET_CONFIG_BOOT_IPS_ADDR as usize + index * 4) as *const u32) };
        let candidate = u32_to_ipv4(raw);
        if valid_unicast_ipv4(candidate) {
            config.ips[index] = candidate;
        }

        let raw = unsafe {
            read_volatile((NET_CONFIG_BOOT_NETMASKS_ADDR as usize + index * 4) as *const u32)
        };
        let candidate = u32_to_ipv4(raw);
        if valid_ipv4_netmask(candidate) {
            config.netmasks[index] = candidate;
        }

        let raw = unsafe {
            read_volatile((NET_CONFIG_BOOT_GATEWAYS_ADDR as usize + index * 4) as *const u32)
        };
        let candidate = u32_to_ipv4(raw);
        if valid_optional_unicast_ipv4(candidate) {
            config.gateways[index] = candidate;
        }

        let raw =
            unsafe { read_volatile((NET_CONFIG_BOOT_DNS_ADDR as usize + index * 4) as *const u32) };
        let candidate = u32_to_ipv4(raw);
        if valid_optional_unicast_ipv4(candidate) {
            config.dns[index] = candidate;
        }
    }
    unsafe {
        NET_CONFIG = config;
    }
}

fn parse_net_interface(value: &str) -> Option<usize> {
    if value.eq_ignore_ascii_case("pcie3x1") || value == "0" {
        Some(0)
    } else if value.eq_ignore_ascii_case("pcie3x2") || value == "1" {
        Some(1)
    } else {
        None
    }
}

fn parse_ipv4(value: &str) -> Option<[u8; 4]> {
    let mut out = [0u8; 4];
    let mut count = 0usize;
    for part in value.split('.') {
        if count >= 4 || part.is_empty() {
            return None;
        }
        let Ok(byte) = part.parse::<u8>() else {
            return None;
        };
        out[count] = byte;
        count += 1;
    }
    if count == 4 {
        Some(out)
    } else {
        None
    }
}

fn valid_unicast_ipv4(ip: [u8; 4]) -> bool {
    ip[0] != 0 && ip[0] < 224 && ip != [255, 255, 255, 255] && ip[3] != 0 && ip[3] != 255
}

fn valid_optional_unicast_ipv4(ip: [u8; 4]) -> bool {
    ip == IPV4_ANY_IP || valid_unicast_ipv4(ip)
}

fn valid_ipv4_netmask(mask: [u8; 4]) -> bool {
    let raw = ipv4_to_u32(mask);
    if raw == 0 || raw == u32::MAX {
        return false;
    }
    let inverse = !raw;
    inverse & inverse.wrapping_add(1) == 0
}

fn ipv4_to_u32(ip: [u8; 4]) -> u32 {
    u32::from_be_bytes(ip)
}

fn u32_to_ipv4(value: u32) -> [u8; 4] {
    value.to_be_bytes()
}

fn net_poll_device(index: usize) -> NetPollStats {
    let mut stats = NetPollStats::empty();

    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        stats.rx_packets = stats.rx_packets.wrapping_add(1);
        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);

        let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
        match net_handle_frame(index, frame) {
            Some(NetFrameReply::Arp) => stats.arp_replies = stats.arp_replies.wrapping_add(1),
            Some(NetFrameReply::UdpPing) => stats.udp_replies = stats.udp_replies.wrapping_add(1),
            Some(NetFrameReply::UdpControl) => {
                stats.udp_replies = stats.udp_replies.wrapping_add(1);
                stats.control_replies = stats.control_replies.wrapping_add(1);
            }
            None => {}
        }
        rtl_rearm_rx_desc(index, entry);
    }

    stats
}

fn net_dhcp_exchange(index: usize, xid: u32) -> Result<DhcpExchange, DhcpError> {
    net_dhcp_exchange_with_timeout(index, xid, DHCP_TIMEOUT_MS)
}

fn net_dhcp_exchange_with_timeout(
    index: usize,
    xid: u32,
    timeout_ms: u64,
) -> Result<DhcpExchange, DhcpError> {
    if !net_send_dhcp_discover(index, xid) {
        return Err(DhcpError::DiscoverTx);
    }

    let Some(offer) = net_wait_dhcp_message(index, xid, DHCP_MSG_OFFER, timeout_ms) else {
        return Err(DhcpError::OfferTimeout);
    };

    if !net_send_dhcp_request(index, xid, offer.ip, offer.server) {
        return Err(DhcpError::RequestTx);
    }

    let Some(mut ack) = net_wait_dhcp_message(index, xid, DHCP_MSG_ACK, timeout_ms) else {
        return Err(DhcpError::AckTimeout);
    };
    if ack.server == IPV4_ANY_IP {
        ack.server = offer.server;
    }
    if ack.gateway == IPV4_ANY_IP {
        ack.gateway = offer.gateway;
    }
    if ack.dns == IPV4_ANY_IP {
        ack.dns = offer.dns;
    }
    if ack.netmask == IPV4_ANY_IP {
        ack.netmask = offer.netmask;
    }

    Ok(DhcpExchange { offer, ack })
}

fn net_wait_dhcp_message(
    index: usize,
    xid: u32,
    expected_message_type: u8,
    timeout_ms: u64,
) -> Option<DhcpLease> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Some(lease) = net_poll_dhcp_message(index, xid, expected_message_type) {
            return Some(lease);
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn net_poll_dhcp_message(index: usize, xid: u32, expected_message_type: u8) -> Option<DhcpLease> {
    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);
        let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
        let lease = net_parse_dhcp_frame(index, frame, xid)
            .filter(|lease| lease.message_type == expected_message_type);
        rtl_rearm_rx_desc(index, entry);
        if lease.is_some() {
            return lease;
        }
    }
    None
}

fn net_parse_dhcp_frame(index: usize, frame: &[u8], xid: u32) -> Option<DhcpLease> {
    if frame.len() < 14 {
        return None;
    }

    let mut eth_type = read_be16(frame, 12)?;
    let mut ip_offset = 14usize;
    if eth_type == 0x8100 || eth_type == 0x88a8 {
        if frame.len() < 18 {
            return None;
        }
        eth_type = read_be16(frame, 16)?;
        ip_offset = 18;
    }
    if eth_type != ETH_TYPE_IPV4 || frame.len() < ip_offset + 20 {
        return None;
    }

    let ip = &frame[ip_offset..];
    let version = ip[0] >> 4;
    let ihl = ((ip[0] & 0x0f) as usize) * 4;
    if version != 4 || ihl < 20 || ip.len() < ihl + 8 || ip[9] != IPV4_PROTO_UDP {
        return None;
    }
    let total_len = read_be16(ip, 2)? as usize;
    if total_len < ihl + 8 || frame.len() < ip_offset + total_len {
        return None;
    }

    let udp = &ip[ihl..total_len];
    let src_port = read_be16(udp, 0)?;
    let dst_port = read_be16(udp, 2)?;
    let udp_len = read_be16(udp, 4)? as usize;
    if src_port != DHCP_SERVER_PORT
        || dst_port != DHCP_CLIENT_PORT
        || udp_len < 8
        || udp_len > udp.len()
    {
        return None;
    }

    net_parse_dhcp_payload(index, &udp[8..udp_len], xid)
}

fn net_parse_dhcp_payload(index: usize, payload: &[u8], xid: u32) -> Option<DhcpLease> {
    if payload.len() < 240 {
        return None;
    }
    if payload[0] != 2 || payload[1] != 1 || payload[2] != 6 {
        return None;
    }
    if read_be32(payload, 4)? != xid {
        return None;
    }
    let mac = rtl_local_mac(index);
    if &payload[28..34] != mac.as_slice() {
        return None;
    }
    if &payload[236..240] != DHCP_MAGIC_COOKIE.as_slice() {
        return None;
    }

    let mut lease = DhcpLease::empty(index);
    lease.ip = [payload[16], payload[17], payload[18], payload[19]];

    let mut offset = 240usize;
    while offset < payload.len() {
        let code = payload[offset];
        offset += 1;
        match code {
            0 => continue,
            DHCP_OPT_END => break,
            _ => {}
        }
        if offset >= payload.len() {
            break;
        }
        let len = payload[offset] as usize;
        offset += 1;
        if offset + len > payload.len() {
            break;
        }
        let data = &payload[offset..offset + len];
        match code {
            DHCP_OPT_MESSAGE_TYPE if len >= 1 => lease.message_type = data[0],
            DHCP_OPT_SUBNET_MASK if len >= 4 => {
                let candidate = [data[0], data[1], data[2], data[3]];
                if valid_ipv4_netmask(candidate) {
                    lease.netmask = candidate;
                }
            }
            DHCP_OPT_ROUTER if len >= 4 => {
                let candidate = [data[0], data[1], data[2], data[3]];
                if valid_optional_unicast_ipv4(candidate) {
                    lease.gateway = candidate;
                }
            }
            DHCP_OPT_DNS if len >= 4 => {
                let candidate = [data[0], data[1], data[2], data[3]];
                if valid_optional_unicast_ipv4(candidate) {
                    lease.dns = candidate;
                }
            }
            DHCP_OPT_SERVER_ID if len >= 4 => {
                lease.server = [data[0], data[1], data[2], data[3]];
            }
            DHCP_OPT_LEASE_TIME if len >= 4 => {
                lease.lease_seconds = read_be32(data, 0).unwrap_or(0);
            }
            _ => {}
        }
        offset += len;
    }

    if lease.message_type == 0 || !valid_unicast_ipv4(lease.ip) {
        return None;
    }
    Some(lease)
}

fn net_dns_resolve_a(index: usize, name: &str, timeout_ms: u64) -> Result<DnsResult, DnsError> {
    let server = net_device_dns(index);
    if !valid_unicast_ipv4(server) {
        return Err(DnsError::MissingDns);
    }
    let Some(next_hop) = net_next_hop_ip(index, server) else {
        return Err(DnsError::MissingRoute);
    };
    let Some(next_hop_mac) = net_resolve_mac(index, next_hop) else {
        return Err(DnsError::ArpTimeout);
    };

    let txid = (read_cntpct() as u16) ^ 0x584a ^ ((index as u16) << 8);
    let mut payload = [0u8; DNS_PACKET_BYTES];
    let Some(len) = build_dns_a_query(name, txid, &mut payload) else {
        return Err(DnsError::BadName);
    };
    let client_port = DNS_CLIENT_PORT_BASE.wrapping_add(index as u16);
    if !net_send_udp_packet(
        index,
        next_hop_mac,
        server,
        DNS_SERVER_PORT,
        client_port,
        &payload[..len],
    ) {
        return Err(DnsError::QueryTx);
    }
    let Some(answer) = net_wait_dns_a_response(index, txid, client_port, timeout_ms) else {
        return Err(DnsError::ResponseTimeout);
    };
    Ok(DnsResult {
        server,
        next_hop,
        next_hop_mac,
        answer,
        txid,
    })
}

fn net_next_hop_ip(index: usize, dst_ip: [u8; 4]) -> Option<[u8; 4]> {
    let our_ip = net_device_ip(index);
    let netmask = net_device_netmask(index);
    if ipv4_same_subnet(our_ip, dst_ip, netmask) {
        Some(dst_ip)
    } else {
        let gateway = net_device_gateway(index);
        if valid_unicast_ipv4(gateway) {
            Some(gateway)
        } else {
            None
        }
    }
}

fn ipv4_same_subnet(a: [u8; 4], b: [u8; 4], mask: [u8; 4]) -> bool {
    let mask = ipv4_to_u32(mask);
    (ipv4_to_u32(a) & mask) == (ipv4_to_u32(b) & mask)
}

fn net_resolve_mac(index: usize, target_ip: [u8; 4]) -> Option<[u8; 6]> {
    for _ in 0..ARP_RESOLVE_ATTEMPTS {
        if !net_send_arp_request(index, target_ip) {
            continue;
        }
        if let Some(mac) = net_wait_arp_reply(index, target_ip, ARP_RESOLVE_TIMEOUT_MS) {
            return Some(mac);
        }
    }
    None
}

fn net_send_arp_request(index: usize, target_ip: [u8; 4]) -> bool {
    let our_mac = rtl_local_mac(index);
    let our_ip = net_device_ip(index);
    clear_tx_buffer(index);
    unsafe {
        let tx = &mut RTL_TX_BUFFERS.0[index];
        tx[0..6].copy_from_slice(&NET_BROADCAST_MAC);
        tx[6..12].copy_from_slice(&our_mac);
        write_be16(tx, 12, ETH_TYPE_ARP);
        write_be16(tx, 14, ARP_HTYPE_ETHERNET);
        write_be16(tx, 16, ETH_TYPE_IPV4);
        tx[18] = 6;
        tx[19] = 4;
        write_be16(tx, 20, ARP_REQUEST);
        tx[22..28].copy_from_slice(&our_mac);
        tx[28..32].copy_from_slice(&our_ip);
        for byte in &mut tx[32..38] {
            *byte = 0;
        }
        tx[38..42].copy_from_slice(&target_ip);
    }
    rtl_send_prepared_frame(index, 60, 100)
}

fn net_wait_arp_reply(index: usize, target_ip: [u8; 4], timeout_ms: u64) -> Option<[u8; 6]> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Some(mac) = net_poll_arp_reply(index, target_ip) {
            return Some(mac);
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn net_poll_arp_reply(index: usize, target_ip: [u8; 4]) -> Option<[u8; 6]> {
    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);
        let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
        let reply = net_parse_arp_reply(index, frame, target_ip);
        rtl_rearm_rx_desc(index, entry);
        if reply.is_some() {
            return reply;
        }
    }
    None
}

fn net_parse_arp_reply(index: usize, frame: &[u8], expected_ip: [u8; 4]) -> Option<[u8; 6]> {
    if frame.len() < 42 || read_be16(frame, 12)? != ETH_TYPE_ARP {
        return None;
    }
    let arp_offset = 14usize;
    let htype = read_be16(frame, arp_offset)?;
    let ptype = read_be16(frame, arp_offset + 2)?;
    let hlen = frame[arp_offset + 4];
    let plen = frame[arp_offset + 5];
    let opcode = read_be16(frame, arp_offset + 6)?;
    if htype != ARP_HTYPE_ETHERNET
        || ptype != ETH_TYPE_IPV4
        || hlen != 6
        || plen != 4
        || opcode != ARP_REPLY
    {
        return None;
    }
    let sender_ip = [
        frame[arp_offset + 14],
        frame[arp_offset + 15],
        frame[arp_offset + 16],
        frame[arp_offset + 17],
    ];
    let target_ip = [
        frame[arp_offset + 24],
        frame[arp_offset + 25],
        frame[arp_offset + 26],
        frame[arp_offset + 27],
    ];
    if sender_ip != expected_ip || target_ip != net_device_ip(index) {
        return None;
    }
    let mac = [
        frame[arp_offset + 8],
        frame[arp_offset + 9],
        frame[arp_offset + 10],
        frame[arp_offset + 11],
        frame[arp_offset + 12],
        frame[arp_offset + 13],
    ];
    if is_likely_valid_mac(mac) {
        Some(mac)
    } else {
        None
    }
}

fn net_tcp_connect(
    index: usize,
    host: &str,
    dst_port: u16,
    timeout_ms: u64,
) -> Result<TcpConnectResult, TcpError> {
    let session = net_tcp_open(index, host, dst_port, timeout_ms)?;
    let _ = net_send_tcp_packet(
        index,
        session.next_hop_mac,
        net_device_ip(index),
        session.dst_ip,
        session.src_port,
        session.dst_port,
        session.local_seq,
        session.remote_seq_next,
        TCP_FLAG_ACK | TCP_FLAG_RST,
        &[],
    );
    Ok(TcpConnectResult {
        dst_ip: session.dst_ip,
        next_hop: session.next_hop,
        next_hop_mac: session.next_hop_mac,
        src_port: session.src_port,
        dst_port: session.dst_port,
        seq: session.initial_seq,
        ack: session.remote_seq_next,
        flags: session.synack_flags,
        dns_txid: session.dns_txid,
    })
}

fn net_tcp_open(
    index: usize,
    host: &str,
    dst_port: u16,
    timeout_ms: u64,
) -> Result<TcpSession, TcpError> {
    let (dst_ip, dns_txid) = if let Some(ip) = parse_ipv4(host) {
        if !valid_unicast_ipv4(ip) {
            return Err(TcpError::BadHost);
        }
        (ip, 0)
    } else {
        let result = net_dns_resolve_a(index, host, DNS_TIMEOUT_MS).map_err(TcpError::Dns)?;
        (result.answer, result.txid)
    };
    if dst_port == 0 {
        return Err(TcpError::BadPort);
    }

    let Some(next_hop) = net_next_hop_ip(index, dst_ip) else {
        return Err(TcpError::MissingRoute);
    };
    let Some(next_hop_mac) = net_resolve_mac(index, next_hop) else {
        return Err(TcpError::ArpTimeout);
    };

    let ticks = read_cntpct();
    let src_port = TCP_CLIENT_PORT_BASE.wrapping_add((ticks as u16) & 0x3fff);
    let seq = (ticks as u32).rotate_left(13) ^ 0x584a_5443 ^ ((index as u32) << 24);
    let src_ip = net_device_ip(index);

    if !net_send_tcp_packet(
        index,
        next_hop_mac,
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        seq,
        0,
        TCP_FLAG_SYN,
        &[],
    ) {
        return Err(TcpError::SynTx);
    }

    let Some(response) =
        net_wait_tcp_connect_response(index, dst_ip, src_port, dst_port, seq, timeout_ms)
    else {
        return Err(TcpError::ResponseTimeout);
    };
    let (remote_seq, flags) = response?;
    let ack = remote_seq.wrapping_add(1);
    let local_seq = seq.wrapping_add(1);
    let _ = net_send_tcp_packet(
        index,
        next_hop_mac,
        src_ip,
        dst_ip,
        src_port,
        dst_port,
        local_seq,
        ack,
        TCP_FLAG_ACK,
        &[],
    );

    Ok(TcpSession {
        dst_ip,
        next_hop,
        next_hop_mac,
        src_port,
        dst_port,
        local_seq,
        remote_seq_next: ack,
        initial_seq: seq,
        synack_flags: flags,
        dns_txid,
    })
}

fn net_http_get(
    index: usize,
    host: &str,
    dst_port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<HttpGetResult, HttpError> {
    let (session, response, _sent_payload_len) =
        net_http_exchange(index, host, dst_port, path, timeout_ms)?;
    let status_code = parse_http_status_code(&response.sample[..response.sample_len])
        .ok_or(HttpError::MalformedResponse)?;

    Ok(HttpGetResult {
        tcp: session,
        status_code,
        response_bytes: response.response_bytes,
        payload_bytes: response.payload_bytes,
        segments: response.segments,
        fin: response.fin,
        truncated: response.truncated,
        sample_len: response.sample_len,
        sample: response.sample,
    })
}

fn net_http_post(
    index: usize,
    host: &str,
    dst_port: u16,
    path: &str,
    body: &[u8],
    timeout_ms: u64,
) -> Result<HttpPostResult, HttpError> {
    let (session, response, sent_payload_len) =
        net_http_exchange_with_body(index, host, dst_port, path, b"POST", body, timeout_ms)?;
    let status_code = parse_http_status_code(&response.sample[..response.sample_len])
        .ok_or(HttpError::MalformedResponse)?;

    Ok(HttpPostResult {
        tcp: session,
        status_code,
        response_bytes: response.response_bytes,
        payload_bytes: response.payload_bytes,
        segments: response.segments,
        fin: response.fin,
        truncated: response.truncated,
        sent_payload_len,
        sample_len: response.sample_len,
        sample: response.sample,
    })
}

fn net_http_trace(
    index: usize,
    host: &str,
    dst_port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<HttpTraceResult, HttpError> {
    let (session, response, _sent_payload_len) =
        net_http_exchange(index, host, dst_port, path, timeout_ms)?;
    Ok(HttpTraceResult {
        tcp: session,
        response_bytes: response.response_bytes,
        payload_bytes: response.payload_bytes,
        segments: response.segments,
        fin: response.fin,
        truncated: response.truncated,
        sample_len: response.sample_len,
        sample: response.sample,
    })
}

fn net_http_debug(
    index: usize,
    host: &str,
    dst_port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<HttpDebugResult, HttpError> {
    if !path.starts_with('/')
        || path
            .as_bytes()
            .iter()
            .any(|b| *b == b' ' || *b == b'\r' || *b == b'\n')
    {
        return Err(HttpError::BadPath);
    }
    let session = net_tcp_open(index, host, dst_port, timeout_ms).map_err(HttpError::Tcp)?;
    let mut request = HttpRequestBuf::new();
    if write!(
        request,
        "GET {path} HTTP/1.0\r\nHost: {host}\r\nUser-Agent: xiaojin-os/0.1\r\nConnection: close\r\n\r\n"
    )
    .is_err()
    {
        let _ = net_tcp_reset(index, session);
        return Err(HttpError::RequestTooLong);
    }

    if !net_send_tcp_packet(
        index,
        session.next_hop_mac,
        net_device_ip(index),
        session.dst_ip,
        session.src_port,
        session.dst_port,
        session.local_seq,
        session.remote_seq_next,
        TCP_FLAG_ACK | TCP_FLAG_PSH,
        request.as_slice(),
    ) {
        let _ = net_tcp_reset(index, session);
        return Err(HttpError::RequestTx);
    }

    let sent_payload_len = request.as_slice().len() as u32;
    let Some(segment) = net_wait_tcp_debug_segment(index, session, timeout_ms) else {
        let _ = net_tcp_reset(index, session);
        return Err(HttpError::ResponseTimeout);
    };
    let _ = net_tcp_reset(index, session);

    Ok(HttpDebugResult {
        tcp: session,
        sent_payload_len,
        segment,
    })
}

fn net_http_exchange(
    index: usize,
    host: &str,
    dst_port: u16,
    path: &str,
    timeout_ms: u64,
) -> Result<(TcpSession, HttpResponseAccum, u32), HttpError> {
    net_http_exchange_with_body(index, host, dst_port, path, b"GET", &[], timeout_ms)
}

fn net_http_exchange_with_body(
    index: usize,
    host: &str,
    dst_port: u16,
    path: &str,
    method: &[u8],
    body: &[u8],
    timeout_ms: u64,
) -> Result<(TcpSession, HttpResponseAccum, u32), HttpError> {
    if !path.starts_with('/')
        || path
            .as_bytes()
            .iter()
            .any(|b| *b == b' ' || *b == b'\r' || *b == b'\n')
    {
        return Err(HttpError::BadPath);
    }
    if body
        .iter()
        .any(|byte| *byte == b'\r' || *byte == b'\n' || *byte == 0)
    {
        return Err(HttpError::BadBody);
    }
    let session = net_tcp_open(index, host, dst_port, timeout_ms).map_err(HttpError::Tcp)?;
    let mut request = HttpRequestBuf::new();
    let request_ok = if method == b"POST" {
        write!(
            request,
            "POST {path} HTTP/1.0\r\nHost: {host}\r\nUser-Agent: xiaojin-os/0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .and_then(|_| request.write_str(core::str::from_utf8(body).unwrap_or("")))
        .is_ok()
    } else {
        write!(
            request,
            "GET {path} HTTP/1.0\r\nHost: {host}\r\nUser-Agent: xiaojin-os/0.1\r\nConnection: close\r\n\r\n"
        )
        .is_ok()
    };
    if !request_ok {
        let _ = net_tcp_reset(index, session);
        return Err(HttpError::RequestTooLong);
    }

    if !net_send_tcp_packet(
        index,
        session.next_hop_mac,
        net_device_ip(index),
        session.dst_ip,
        session.src_port,
        session.dst_port,
        session.local_seq,
        session.remote_seq_next,
        TCP_FLAG_ACK | TCP_FLAG_PSH,
        request.as_slice(),
    ) {
        let _ = net_tcp_reset(index, session);
        return Err(HttpError::RequestTx);
    }

    let sent_payload_len = request.as_slice().len() as u32;
    let Some(response) = net_wait_http_response(index, session, sent_payload_len, timeout_ms)
    else {
        let _ = net_tcp_reset(index, session);
        return Err(HttpError::ResponseTimeout);
    };
    let response = match response {
        Ok(response) => response,
        Err(error) => {
            let _ = net_tcp_reset(index, session);
            return Err(error);
        }
    };

    let local_seq = session.local_seq.wrapping_add(sent_payload_len);
    let _ = net_send_tcp_packet(
        index,
        session.next_hop_mac,
        net_device_ip(index),
        session.dst_ip,
        session.src_port,
        session.dst_port,
        local_seq,
        response.next_remote_seq,
        TCP_FLAG_ACK | TCP_FLAG_RST,
        &[],
    );

    Ok((session, response, sent_payload_len))
}

#[derive(Clone, Copy)]
struct TcpPayloadResult {
    remote_seq: u32,
    flags: u16,
    response_bytes: usize,
    payload_bytes: usize,
    sample_len: usize,
    sample: [u8; HTTP_RESPONSE_SAMPLE_BYTES],
}

#[derive(Clone, Copy)]
struct HttpResponseAccum {
    next_remote_seq: u32,
    response_bytes: usize,
    payload_bytes: usize,
    segments: u32,
    sample_len: usize,
    sample: [u8; HTTP_RESPONSE_SAMPLE_BYTES],
    fin: bool,
    truncated: bool,
}

fn net_wait_http_response(
    index: usize,
    session: TcpSession,
    sent_payload_len: u32,
    timeout_ms: u64,
) -> Option<Result<HttpResponseAccum, HttpError>> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let idle_ticks = ms_to_ticks(HTTP_IDLE_TIMEOUT_MS, freq)?;
    let start = read_cntpct();
    let mut last_progress = start;
    let local_seq = session.local_seq.wrapping_add(sent_payload_len);
    let mut response = HttpResponseAccum {
        next_remote_seq: session.remote_seq_next,
        response_bytes: 0,
        payload_bytes: 0,
        segments: 0,
        sample_len: 0,
        sample: [0; HTTP_RESPONSE_SAMPLE_BYTES],
        fin: false,
        truncated: false,
    };

    loop {
        if let Some(segment) = net_poll_tcp_payload(index, session, sent_payload_len) {
            let segment = match segment {
                Ok(segment) => segment,
                Err(error) => return Some(Err(error)),
            };
            if segment.payload_bytes > 0 {
                if segment.remote_seq != response.next_remote_seq {
                    continue;
                }
                let remaining = HTTP_RESPONSE_MAX_BYTES.saturating_sub(response.payload_bytes);
                let accepted = if segment.payload_bytes > remaining {
                    remaining
                } else {
                    segment.payload_bytes
                };
                response.response_bytes = response
                    .response_bytes
                    .saturating_add(segment.response_bytes);
                response.payload_bytes = response.payload_bytes.saturating_add(accepted);
                response.segments = response.segments.saturating_add(1);

                let sample_remaining = response.sample.len().saturating_sub(response.sample_len);
                if sample_remaining > 0 && accepted > 0 {
                    let copy_len = core::cmp::min(
                        core::cmp::min(segment.sample_len, sample_remaining),
                        accepted,
                    );
                    let start = response.sample_len;
                    response.sample[start..start + copy_len]
                        .copy_from_slice(&segment.sample[..copy_len]);
                    response.sample_len += copy_len;
                }

                response.next_remote_seq = response
                    .next_remote_seq
                    .wrapping_add(segment.payload_bytes as u32);
                last_progress = read_cntpct();
                let _ = net_send_tcp_packet(
                    index,
                    session.next_hop_mac,
                    net_device_ip(index),
                    session.dst_ip,
                    session.src_port,
                    session.dst_port,
                    local_seq,
                    response.next_remote_seq,
                    TCP_FLAG_ACK,
                    &[],
                );
                if response.payload_bytes >= HTTP_RESPONSE_MAX_BYTES {
                    response.truncated = true;
                    return Some(Ok(response));
                }
            }

            if segment.payload_bytes == 0
                && segment.flags & TCP_FLAG_FIN != 0
                && segment.remote_seq != response.next_remote_seq
            {
                continue;
            }

            if segment.flags & TCP_FLAG_FIN != 0 {
                response.next_remote_seq = response.next_remote_seq.wrapping_add(1);
                response.fin = true;
                let _ = net_send_tcp_packet(
                    index,
                    session.next_hop_mac,
                    net_device_ip(index),
                    session.dst_ip,
                    session.src_port,
                    session.dst_port,
                    local_seq,
                    response.next_remote_seq,
                    TCP_FLAG_ACK,
                    &[],
                );
                return Some(Ok(response));
            }
        }
        let now = read_cntpct();
        if now.wrapping_sub(start) >= timeout_ticks {
            if response.payload_bytes > 0 {
                return Some(Ok(response));
            }
            return None;
        }
        if response.payload_bytes > 0 && now.wrapping_sub(last_progress) >= idle_ticks {
            return Some(Ok(response));
        }
        core::hint::spin_loop();
    }
}

fn net_poll_tcp_payload(
    index: usize,
    session: TcpSession,
    sent_payload_len: u32,
) -> Option<Result<TcpPayloadResult, HttpError>> {
    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);
        let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
        let response = net_parse_tcp_payload(index, frame, session, sent_payload_len);
        if response.is_none() {
            let _ = net_handle_frame(index, frame);
        }
        rtl_rearm_rx_desc(index, entry);
        if response.is_some() {
            return response;
        }
    }
    None
}

fn net_wait_tcp_debug_segment(
    index: usize,
    session: TcpSession,
    timeout_ms: u64,
) -> Option<TcpDebugSegment> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Some(segment) = net_poll_tcp_debug_segment(index, session) {
            return Some(segment);
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn net_poll_tcp_debug_segment(index: usize, session: TcpSession) -> Option<TcpDebugSegment> {
    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);
        let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
        let segment = net_parse_tcp_debug_segment(index, frame, session);
        if segment.is_none() {
            let _ = net_handle_frame(index, frame);
        }
        rtl_rearm_rx_desc(index, entry);
        if segment.is_some() {
            return segment;
        }
    }
    None
}

fn net_parse_tcp_debug_segment(
    index: usize,
    frame: &[u8],
    session: TcpSession,
) -> Option<TcpDebugSegment> {
    let parsed = parse_tcp_ipv4_segment(index, frame)?;
    if parsed.src_ip != session.dst_ip
        || parsed.dst_ip != net_device_ip(index)
        || parsed.src_port != session.dst_port
        || parsed.dst_port != session.src_port
    {
        return None;
    }

    let mut sample = [0u8; HTTP_RESPONSE_SAMPLE_BYTES];
    let sample_len = if parsed.payload.len() > sample.len() {
        sample.len()
    } else {
        parsed.payload.len()
    };
    sample[..sample_len].copy_from_slice(&parsed.payload[..sample_len]);

    Some(TcpDebugSegment {
        seq: parsed.seq,
        ack: parsed.ack,
        flags: parsed.flags,
        segment_len: parsed.segment_len,
        payload_len: parsed.payload.len(),
        sample_len,
        sample,
    })
}

fn net_parse_tcp_payload(
    index: usize,
    frame: &[u8],
    session: TcpSession,
    sent_payload_len: u32,
) -> Option<Result<TcpPayloadResult, HttpError>> {
    let parsed = parse_tcp_ipv4_segment(index, frame)?;
    if parsed.src_ip != session.dst_ip
        || parsed.dst_ip != net_device_ip(index)
        || parsed.src_port != session.dst_port
        || parsed.dst_port != session.src_port
    {
        return None;
    }
    if parsed.flags & TCP_FLAG_RST != 0 {
        return Some(Err(HttpError::Tcp(TcpError::Reset)));
    }
    let expected_ack = session.local_seq.wrapping_add(sent_payload_len);
    if parsed.ack != expected_ack || (parsed.payload.is_empty() && parsed.flags & TCP_FLAG_FIN == 0)
    {
        return None;
    }

    let mut sample = [0u8; HTTP_RESPONSE_SAMPLE_BYTES];
    let sample_len = if parsed.payload.len() > sample.len() {
        sample.len()
    } else {
        parsed.payload.len()
    };
    sample[..sample_len].copy_from_slice(&parsed.payload[..sample_len]);
    Some(Ok(TcpPayloadResult {
        remote_seq: parsed.seq,
        flags: parsed.flags,
        response_bytes: parsed.segment_len,
        payload_bytes: parsed.payload.len(),
        sample_len,
        sample,
    }))
}

#[derive(Clone, Copy)]
struct TcpParsedSegment<'a> {
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    flags: u16,
    segment_len: usize,
    payload: &'a [u8],
}

fn parse_tcp_ipv4_segment<'a>(index: usize, frame: &'a [u8]) -> Option<TcpParsedSegment<'a>> {
    if frame.len() < 14 {
        return None;
    }
    let mut eth_type = read_be16(frame, 12)?;
    let mut ip_offset = 14usize;
    if eth_type == 0x8100 || eth_type == 0x88a8 {
        if frame.len() < 18 {
            return None;
        }
        eth_type = read_be16(frame, 16)?;
        ip_offset = 18;
    }
    if eth_type != ETH_TYPE_IPV4 || frame.len() < ip_offset + 20 {
        return None;
    }

    let ip = &frame[ip_offset..];
    let version = ip[0] >> 4;
    let ihl = ((ip[0] & 0x0f) as usize) * 4;
    if version != 4 || ihl < 20 || ip.len() < ihl + 20 || ip[9] != IPV4_PROTO_TCP {
        return None;
    }
    let total_len = read_be16(ip, 2)? as usize;
    if total_len < ihl + 20 || frame.len() < ip_offset + total_len {
        return None;
    }
    let src_ip = [ip[12], ip[13], ip[14], ip[15]];
    let dst_ip = [ip[16], ip[17], ip[18], ip[19]];
    if dst_ip != net_device_ip(index) {
        return None;
    }

    let tcp = &ip[ihl..total_len];
    let data_offset = ((tcp[12] >> 4) as usize) * 4;
    if data_offset < 20 || tcp.len() < data_offset {
        return None;
    }
    Some(TcpParsedSegment {
        src_ip,
        dst_ip,
        src_port: read_be16(tcp, 0)?,
        dst_port: read_be16(tcp, 2)?,
        seq: read_be32(tcp, 4)?,
        ack: read_be32(tcp, 8)?,
        flags: read_be16(tcp, 12)? & 0x01ff,
        segment_len: tcp.len(),
        payload: &tcp[data_offset..],
    })
}

fn parse_http_status_code(payload: &[u8]) -> Option<u16> {
    let line_start = find_http_status_line(payload)?;
    let payload = &payload[line_start..];
    if payload.len() < 12 {
        return None;
    }
    let mut offset = 5usize;
    while offset < payload.len() && payload[offset] != b' ' {
        offset += 1;
    }
    if offset + 4 > payload.len() || payload[offset] != b' ' {
        return None;
    }
    let code = &payload[offset + 1..offset + 4];
    if !code.iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some(((code[0] - b'0') as u16) * 100 + ((code[1] - b'0') as u16) * 10 + (code[2] - b'0') as u16)
}

fn find_http_status_line(payload: &[u8]) -> Option<usize> {
    let mut offset = 0usize;
    while offset + 5 <= payload.len() {
        if &payload[offset..offset + 5] == b"HTTP/" {
            return Some(offset);
        }
        offset += 1;
    }
    None
}

fn net_tcp_reset(index: usize, session: TcpSession) -> bool {
    net_send_tcp_packet(
        index,
        session.next_hop_mac,
        net_device_ip(index),
        session.dst_ip,
        session.src_port,
        session.dst_port,
        session.local_seq,
        session.remote_seq_next,
        TCP_FLAG_ACK | TCP_FLAG_RST,
        &[],
    )
}

fn net_wait_tcp_connect_response(
    index: usize,
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    timeout_ms: u64,
) -> Option<Result<(u32, u16), TcpError>> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Some(response) =
            net_poll_tcp_connect_response(index, dst_ip, src_port, dst_port, seq)
        {
            return Some(response);
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn net_poll_tcp_connect_response(
    index: usize,
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
) -> Option<Result<(u32, u16), TcpError>> {
    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);
        let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
        let response =
            net_parse_tcp_connect_response(index, frame, dst_ip, src_port, dst_port, seq);
        if response.is_none() {
            let _ = net_handle_frame(index, frame);
        }
        rtl_rearm_rx_desc(index, entry);
        if response.is_some() {
            return response;
        }
    }
    None
}

fn net_parse_tcp_connect_response(
    index: usize,
    frame: &[u8],
    expected_src_ip: [u8; 4],
    expected_dst_port: u16,
    expected_src_port: u16,
    expected_seq: u32,
) -> Option<Result<(u32, u16), TcpError>> {
    if frame.len() < 14 {
        return None;
    }
    let mut eth_type = read_be16(frame, 12)?;
    let mut ip_offset = 14usize;
    if eth_type == 0x8100 || eth_type == 0x88a8 {
        if frame.len() < 18 {
            return None;
        }
        eth_type = read_be16(frame, 16)?;
        ip_offset = 18;
    }
    if eth_type != ETH_TYPE_IPV4 || frame.len() < ip_offset + 20 {
        return None;
    }

    let ip = &frame[ip_offset..];
    let version = ip[0] >> 4;
    let ihl = ((ip[0] & 0x0f) as usize) * 4;
    if version != 4 || ihl < 20 || ip.len() < ihl + 20 || ip[9] != IPV4_PROTO_TCP {
        return None;
    }
    let total_len = read_be16(ip, 2)? as usize;
    if total_len < ihl + 20 || frame.len() < ip_offset + total_len {
        return None;
    }
    let src_ip = [ip[12], ip[13], ip[14], ip[15]];
    let dst_ip = [ip[16], ip[17], ip[18], ip[19]];
    if src_ip != expected_src_ip || dst_ip != net_device_ip(index) {
        return None;
    }

    let tcp = &ip[ihl..total_len];
    let src_port = read_be16(tcp, 0)?;
    let dst_port = read_be16(tcp, 2)?;
    if src_port != expected_src_port || dst_port != expected_dst_port {
        return None;
    }
    let remote_seq = read_be32(tcp, 4)?;
    let ack = read_be32(tcp, 8)?;
    let flags = read_be16(tcp, 12)? & 0x01ff;
    if flags & TCP_FLAG_RST != 0 {
        return Some(Err(TcpError::Reset));
    }
    if flags & (TCP_FLAG_SYN | TCP_FLAG_ACK) == (TCP_FLAG_SYN | TCP_FLAG_ACK)
        && ack == expected_seq.wrapping_add(1)
    {
        return Some(Ok((remote_seq, flags)));
    }
    None
}

fn net_send_tcp_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack: u32,
    flags: u16,
    payload: &[u8],
) -> bool {
    let our_mac = rtl_local_mac(index);
    let tcp_len = 20 + payload.len();
    let ip_len = 20 + tcp_len;
    let frame_len = 14 + ip_len;
    let padded_len = if frame_len < 60 { 60 } else { frame_len };
    if padded_len > RTL_TX_BUFFER_SIZE {
        return false;
    }

    clear_tx_buffer(index);
    unsafe {
        let tx = &mut RTL_TX_BUFFERS.0[index];
        tx[0..6].copy_from_slice(&dst_mac);
        tx[6..12].copy_from_slice(&our_mac);
        write_be16(tx, 12, ETH_TYPE_IPV4);

        let ip = 14;
        tx[ip] = 0x45;
        tx[ip + 1] = 0;
        write_be16(tx, ip + 2, ip_len as u16);
        write_be16(tx, ip + 4, 0x5400 | index as u16);
        write_be16(tx, ip + 6, 0);
        tx[ip + 8] = 64;
        tx[ip + 9] = IPV4_PROTO_TCP;
        write_be16(tx, ip + 10, 0);
        tx[ip + 12..ip + 16].copy_from_slice(&src_ip);
        tx[ip + 16..ip + 20].copy_from_slice(&dst_ip);
        let ip_csum = ipv4_header_checksum(&tx[ip..ip + 20]);
        write_be16(tx, ip + 10, ip_csum);

        let tcp = ip + 20;
        write_be16(tx, tcp, src_port);
        write_be16(tx, tcp + 2, dst_port);
        write_be32(tx, tcp + 4, seq);
        write_be32(tx, tcp + 8, ack);
        write_be16(tx, tcp + 12, (5u16 << 12) | (flags & 0x01ff));
        write_be16(tx, tcp + 14, 64240);
        write_be16(tx, tcp + 16, 0);
        write_be16(tx, tcp + 18, 0);
        tx[tcp + 20..tcp + 20 + payload.len()].copy_from_slice(payload);
        let csum = tcp_checksum(src_ip, dst_ip, &tx[tcp..tcp + tcp_len]);
        write_be16(tx, tcp + 16, csum);
    }

    rtl_send_prepared_frame(index, padded_len, 100)
}

fn build_dns_a_query(name: &str, txid: u16, out: &mut [u8; DNS_PACKET_BYTES]) -> Option<usize> {
    for byte in out.iter_mut() {
        *byte = 0;
    }
    let name = name.trim().trim_end_matches('.');
    if name.is_empty() || name.len() > DNS_MAX_NAME_LEN {
        return None;
    }
    write_be16(out, 0, txid);
    write_be16(out, 2, 0x0100);
    write_be16(out, 4, 1);
    let mut offset = 12usize;
    for label in name.split('.') {
        if label.is_empty() || label.len() > 63 {
            return None;
        }
        if offset + 1 + label.len() + 5 > out.len() {
            return None;
        }
        for byte in label.bytes() {
            let ok = byte.is_ascii_alphanumeric() || byte == b'-';
            if !ok {
                return None;
            }
        }
        out[offset] = label.len() as u8;
        offset += 1;
        out[offset..offset + label.len()].copy_from_slice(label.as_bytes());
        offset += label.len();
    }
    out[offset] = 0;
    offset += 1;
    write_be16(out, offset, 1);
    offset += 2;
    write_be16(out, offset, 1);
    offset += 2;
    Some(offset)
}

fn net_wait_dns_a_response(
    index: usize,
    txid: u16,
    client_port: u16,
    timeout_ms: u64,
) -> Option<[u8; 4]> {
    let freq = read_cntfrq();
    let timeout_ticks = ms_to_ticks(timeout_ms, freq)?;
    let start = read_cntpct();
    loop {
        if let Some(answer) = net_poll_dns_a_response(index, txid, client_port) {
            return Some(answer);
        }
        if read_cntpct().wrapping_sub(start) >= timeout_ticks {
            return None;
        }
        core::hint::spin_loop();
    }
}

fn net_poll_dns_a_response(index: usize, txid: u16, client_port: u16) -> Option<[u8; 4]> {
    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);
        let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
        let answer = net_parse_dns_frame(frame, txid, client_port);
        rtl_rearm_rx_desc(index, entry);
        if answer.is_some() {
            return answer;
        }
    }
    None
}

fn net_parse_dns_frame(frame: &[u8], txid: u16, client_port: u16) -> Option<[u8; 4]> {
    if frame.len() < 14 {
        return None;
    }
    let mut eth_type = read_be16(frame, 12)?;
    let mut ip_offset = 14usize;
    if eth_type == 0x8100 || eth_type == 0x88a8 {
        if frame.len() < 18 {
            return None;
        }
        eth_type = read_be16(frame, 16)?;
        ip_offset = 18;
    }
    if eth_type != ETH_TYPE_IPV4 || frame.len() < ip_offset + 20 {
        return None;
    }
    let ip = &frame[ip_offset..];
    let version = ip[0] >> 4;
    let ihl = ((ip[0] & 0x0f) as usize) * 4;
    if version != 4 || ihl < 20 || ip.len() < ihl + 8 || ip[9] != IPV4_PROTO_UDP {
        return None;
    }
    let total_len = read_be16(ip, 2)? as usize;
    if total_len < ihl + 8 || frame.len() < ip_offset + total_len {
        return None;
    }
    let dst_ip = [ip[16], ip[17], ip[18], ip[19]];
    if dst_ip != net_device_ip(0) && dst_ip != net_device_ip(1) {
        return None;
    }
    let udp = &ip[ihl..total_len];
    let src_port = read_be16(udp, 0)?;
    let dst_port = read_be16(udp, 2)?;
    let udp_len = read_be16(udp, 4)? as usize;
    if src_port != DNS_SERVER_PORT || dst_port != client_port || udp_len < 8 || udp_len > udp.len()
    {
        return None;
    }
    dns_find_a_answer(&udp[8..udp_len], txid)
}

fn dns_find_a_answer(payload: &[u8], txid: u16) -> Option<[u8; 4]> {
    if payload.len() < 12 || read_be16(payload, 0)? != txid {
        return None;
    }
    let flags = read_be16(payload, 2)?;
    if flags & 0x8000 == 0 {
        return None;
    }
    let qd_count = read_be16(payload, 4)? as usize;
    let an_count = read_be16(payload, 6)? as usize;
    let mut offset = 12usize;
    for _ in 0..qd_count {
        offset = dns_skip_name(payload, offset)?;
        if offset + 4 > payload.len() {
            return None;
        }
        offset += 4;
    }
    for _ in 0..an_count {
        offset = dns_skip_name(payload, offset)?;
        if offset + 10 > payload.len() {
            return None;
        }
        let rr_type = read_be16(payload, offset)?;
        let rr_class = read_be16(payload, offset + 2)?;
        let rd_len = read_be16(payload, offset + 8)? as usize;
        offset += 10;
        if offset + rd_len > payload.len() {
            return None;
        }
        if rr_type == 1 && rr_class == 1 && rd_len == 4 {
            let ip = [
                payload[offset],
                payload[offset + 1],
                payload[offset + 2],
                payload[offset + 3],
            ];
            if valid_unicast_ipv4(ip) {
                return Some(ip);
            }
        }
        offset += rd_len;
    }
    None
}

fn dns_skip_name(payload: &[u8], mut offset: usize) -> Option<usize> {
    let mut labels = 0usize;
    loop {
        if offset >= payload.len() || labels > 32 {
            return None;
        }
        let len = payload[offset];
        if len == 0 {
            return Some(offset + 1);
        }
        if len & 0xc0 == 0xc0 {
            if offset + 1 >= payload.len() {
                return None;
            }
            return Some(offset + 2);
        }
        if len & 0xc0 != 0 {
            return None;
        }
        offset = offset.checked_add(1 + len as usize)?;
        labels += 1;
    }
}

fn net_sniff_device(uart: &mut Uart, index: usize, rx_packets: &mut u32, printed: &mut u32) {
    for entry in 0..RTL_RX_DESC_COUNT {
        let desc_addr = rtl_rx_desc_addr(index, entry);
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let status = unsafe { RTL_RX_RINGS.0[index][entry].opts1 };
        if status & RTL_DESC_OWN != 0 {
            continue;
        }

        *rx_packets = rx_packets.wrapping_add(1);
        let len = (status & 0x3fff) as usize;
        let len = if len > RTL_RX_BUFFER_SIZE {
            RTL_RX_BUFFER_SIZE
        } else {
            len
        };
        let buffer_addr = rtl_rx_buffer_addr(index, entry);
        invalidate_dcache_range(buffer_addr, len as u64);

        if *printed < NET_SNIFF_MAX_PRINT {
            let frame = unsafe { core::slice::from_raw_parts(buffer_addr as *const u8, len) };
            print_net_frame_sample(uart, index, entry, status, frame);
            *printed = printed.wrapping_add(1);
        }
        rtl_rearm_rx_desc(index, entry);
    }
}

fn print_net_frame_sample(uart: &mut Uart, index: usize, entry: usize, status: u32, frame: &[u8]) {
    let eth_type = read_be16(frame, 12).unwrap_or(0);
    let inner_eth_type = if eth_type == 0x8100 || eth_type == 0x88a8 {
        read_be16(frame, 16)
    } else {
        None
    };
    let _ = write!(
        uart,
        "  {} rx desc={} len={} opts1=0x{status:08x} eth=0x{eth_type:04x}",
        PCIE_CONTROLLERS[index].name,
        entry,
        frame.len()
    );
    if let Some(inner) = inner_eth_type {
        let _ = write!(uart, " inner=0x{inner:04x}");
    }
    uart.put_str(" head=");
    let sample_len = if frame.len() > 64 { 64 } else { frame.len() };
    for byte in &frame[..sample_len] {
        let _ = write!(uart, "{byte:02x}");
    }
    uart.put_str("\n");
}

enum NetFrameReply {
    Arp,
    UdpPing,
    UdpControl,
}

fn net_handle_frame(index: usize, frame: &[u8]) -> Option<NetFrameReply> {
    if frame.len() < 14 {
        return None;
    }

    let mut eth_type = read_be16(frame, 12)?;
    let mut payload_offset = 14usize;
    if eth_type == 0x8100 || eth_type == 0x88a8 {
        if frame.len() < 18 {
            return None;
        }
        eth_type = read_be16(frame, 16)?;
        payload_offset = 18;
    }

    match eth_type {
        ETH_TYPE_ARP => {
            if net_handle_arp(index, frame, payload_offset) {
                Some(NetFrameReply::Arp)
            } else {
                None
            }
        }
        ETH_TYPE_IPV4 => net_handle_ipv4_udp(index, frame, payload_offset),
        _ => None,
    }
}

fn net_handle_arp(index: usize, frame: &[u8], arp_offset: usize) -> bool {
    if frame.len() < arp_offset + 28 {
        return false;
    }
    let Some(htype) = read_be16(frame, arp_offset) else {
        return false;
    };
    let Some(ptype) = read_be16(frame, arp_offset + 2) else {
        return false;
    };
    let hlen = frame[arp_offset + 4];
    let plen = frame[arp_offset + 5];
    let Some(opcode) = read_be16(frame, arp_offset + 6) else {
        return false;
    };
    if htype != ARP_HTYPE_ETHERNET
        || ptype != ETH_TYPE_IPV4
        || hlen != 6
        || plen != 4
        || opcode != ARP_REQUEST
    {
        return false;
    }

    let target_ip = [
        frame[arp_offset + 24],
        frame[arp_offset + 25],
        frame[arp_offset + 26],
        frame[arp_offset + 27],
    ];
    let our_ip = net_device_ip(index);
    if target_ip != our_ip {
        return false;
    }

    let sender_mac = [
        frame[arp_offset + 8],
        frame[arp_offset + 9],
        frame[arp_offset + 10],
        frame[arp_offset + 11],
        frame[arp_offset + 12],
        frame[arp_offset + 13],
    ];
    let sender_ip = [
        frame[arp_offset + 14],
        frame[arp_offset + 15],
        frame[arp_offset + 16],
        frame[arp_offset + 17],
    ];
    let our_mac = rtl_local_mac(index);

    clear_tx_buffer(index);
    unsafe {
        let tx = &mut RTL_TX_BUFFERS.0[index];
        tx[0..6].copy_from_slice(&sender_mac);
        tx[6..12].copy_from_slice(&our_mac);
        write_be16(tx, 12, ETH_TYPE_ARP);
        write_be16(tx, 14, ARP_HTYPE_ETHERNET);
        write_be16(tx, 16, ETH_TYPE_IPV4);
        tx[18] = 6;
        tx[19] = 4;
        write_be16(tx, 20, ARP_REPLY);
        tx[22..28].copy_from_slice(&our_mac);
        tx[28..32].copy_from_slice(&our_ip);
        tx[32..38].copy_from_slice(&sender_mac);
        tx[38..42].copy_from_slice(&sender_ip);
    }

    rtl_send_prepared_frame(index, 60, 100)
}

fn net_send_gratuitous_arp(index: usize, opcode: u16) -> bool {
    if opcode != ARP_REQUEST && opcode != ARP_REPLY {
        return false;
    }

    let our_mac = rtl_local_mac(index);
    let our_ip = net_device_ip(index);
    clear_tx_buffer(index);
    unsafe {
        let tx = &mut RTL_TX_BUFFERS.0[index];
        for byte in &mut tx[0..6] {
            *byte = 0xff;
        }
        tx[6..12].copy_from_slice(&our_mac);
        write_be16(tx, 12, ETH_TYPE_ARP);
        write_be16(tx, 14, ARP_HTYPE_ETHERNET);
        write_be16(tx, 16, ETH_TYPE_IPV4);
        tx[18] = 6;
        tx[19] = 4;
        write_be16(tx, 20, opcode);
        tx[22..28].copy_from_slice(&our_mac);
        tx[28..32].copy_from_slice(&our_ip);
        if opcode == ARP_REPLY {
            for byte in &mut tx[32..38] {
                *byte = 0xff;
            }
        }
        tx[38..42].copy_from_slice(&our_ip);
    }

    rtl_send_prepared_frame(index, 60, 100)
}

fn net_handle_ipv4_udp(index: usize, frame: &[u8], ip_offset: usize) -> Option<NetFrameReply> {
    if frame.len() < ip_offset + 20 {
        return None;
    }
    let ip = &frame[ip_offset..];
    let version = ip[0] >> 4;
    let ihl = ((ip[0] & 0x0f) as usize) * 4;
    if version != 4 || ihl < 20 || ip.len() < ihl + 8 || ip[9] != IPV4_PROTO_UDP {
        return None;
    }
    let Some(total_len) = read_be16(ip, 2).map(|v| v as usize) else {
        return None;
    };
    if total_len < ihl + 8 || frame.len() < ip_offset + total_len {
        return None;
    }
    let dst_ip = [ip[16], ip[17], ip[18], ip[19]];
    if !net_accepts_ipv4_dst(index, dst_ip) {
        return None;
    }

    let udp = &ip[ihl..total_len];
    let Some(src_port) = read_be16(udp, 0) else {
        return None;
    };
    let Some(dst_port) = read_be16(udp, 2) else {
        return None;
    };
    let Some(udp_len) = read_be16(udp, 4).map(|v| v as usize) else {
        return None;
    };
    if dst_port != NET_UDP_PORT || udp_len < 8 || udp_len > udp.len() {
        return None;
    }
    let payload = &udp[8..udp_len];
    let dst_mac = [frame[6], frame[7], frame[8], frame[9], frame[10], frame[11]];
    let src_ip = [ip[12], ip[13], ip[14], ip[15]];

    if payload.starts_with(NETLOAD_BEGIN_MAGIC)
        || payload.starts_with(NETLOAD_DATA_MAGIC)
        || payload.starts_with(NETLOAD_COMMIT_MAGIC)
    {
        return net_handle_loader_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONFIG_SET_IP_MAGIC) {
        return net_handle_config_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONTROL_DHCP_MAGIC) {
        return net_handle_dhcp_control_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONTROL_QUERY_MAGIC) {
        return net_handle_query_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONTROL_PERSIST_MAGIC) {
        return net_handle_persist_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONTROL_BOOT_SLOT_MAGIC) {
        return net_handle_boot_slot_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONTROL_REBOOT_MAGIC) {
        return net_handle_reboot_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONTROL_EVENT_MAGIC) {
        return net_handle_event_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload.starts_with(NETCONTROL_DB_BLOB_BEGIN_MAGIC)
        || payload.starts_with(NETCONTROL_DB_BLOB_DATA_MAGIC)
        || payload.starts_with(NETCONTROL_DB_BLOB_COMMIT_MAGIC)
        || payload.starts_with(NETCONTROL_DB_BLOB_READ_MAGIC)
    {
        return net_handle_db_blob_packet(index, dst_mac, src_ip, src_port, payload)
            .then_some(NetFrameReply::UdpControl);
    }

    if payload != b"xjping" {
        return None;
    }

    let reply = if index == 0 {
        b"xjpong pcie3x1"
    } else {
        b"xjpong pcie3x2"
    };
    net_send_udp_reply(index, dst_mac, src_ip, src_port, reply).then_some(NetFrameReply::UdpPing)
}

fn net_handle_config_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();
    let mut apply_ip: Option<[u8; 4]> = None;

    if payload.len() < 8 {
        let _ = write!(reply, "XJIP ERR bad-len");
    } else {
        let new_ip = [payload[4], payload[5], payload[6], payload[7]];
        if !valid_unicast_ipv4(new_ip) {
            let _ = write!(reply, "XJIP ERR bad-ip");
        } else {
            let old_ip = net_device_ip(index);
            let _ = write!(
                reply,
                "XJIP OK iface={} old={}.{}.{}.{} new={}.{}.{}.{} apply=after-reply",
                PCIE_CONTROLLERS[index].name,
                old_ip[0],
                old_ip[1],
                old_ip[2],
                old_ip[3],
                new_ip[0],
                new_ip[1],
                new_ip[2],
                new_ip[3]
            );
            apply_ip = Some(new_ip);
        }
    }

    let sent = net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice());
    if let Some(new_ip) = apply_ip {
        busy_wait_ms(50);
        set_net_device_ip(index, new_ip);
    }
    sent
}

fn net_handle_dhcp_control_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();
    let mut apply_lease: Option<DhcpLease> = None;

    if payload.len() != 4 {
        let _ = write!(reply, "XJDH ERR bad-len");
    } else {
        let xid = 0x584a_0000u32 ^ (read_cntpct() as u32).rotate_left(7) ^ ((index as u32) << 8);
        match net_dhcp_exchange(index, xid) {
            Ok(exchange) => {
                let ack = exchange.ack;
                let _ = write!(
                    reply,
                    "XJDH OK iface={} ip={}.{}.{}.{} mask={}.{}.{}.{} gateway={}.{}.{}.{} dns={}.{}.{}.{} server={}.{}.{}.{} lease={} apply=after-reply",
                    PCIE_CONTROLLERS[index].name,
                    ack.ip[0],
                    ack.ip[1],
                    ack.ip[2],
                    ack.ip[3],
                    ack.netmask[0],
                    ack.netmask[1],
                    ack.netmask[2],
                    ack.netmask[3],
                    ack.gateway[0],
                    ack.gateway[1],
                    ack.gateway[2],
                    ack.gateway[3],
                    ack.dns[0],
                    ack.dns[1],
                    ack.dns[2],
                    ack.dns[3],
                    ack.server[0],
                    ack.server[1],
                    ack.server[2],
                    ack.server[3],
                    ack.lease_seconds
                );
                apply_lease = Some(ack);
            }
            Err(error) => {
                let _ = write!(reply, "XJDH ERR {}", error.as_str());
            }
        }
    }

    let sent = net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice());
    if let Some(lease) = apply_lease {
        busy_wait_ms(50);
        set_net_device_dhcp_lease(index, lease);
        let _ = net_send_gratuitous_arp(index, ARP_REQUEST);
        let _ = net_send_gratuitous_arp(index, ARP_REPLY);
    }
    sent
}

fn net_handle_query_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();
    let mut reboot_after_reply = false;

    if payload.len() < NETCONTROL_QUERY_MAGIC.len() {
        let _ = write!(reply, "XJQ ERR bad-len");
    } else {
        let query_payload = &payload[NETCONTROL_QUERY_MAGIC.len()..];
        let mut query_buf = [0u8; NETCONTROL_QUERY_BYTES];
        if query_payload.len() > query_buf.len() {
            let _ = write!(reply, "XJQ ERR query-too-long");
        } else {
            query_buf[..query_payload.len()].copy_from_slice(query_payload);
            let query = match str::from_utf8(&query_buf[..query_payload.len()]) {
                Ok(value) => value.trim(),
                Err(_) => "",
            };
            if query.is_empty() {
                let _ = write!(reply, "XJQ ERR empty-query");
            } else if query.eq_ignore_ascii_case("caps") {
                write_caps_query_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("identity")
                || query.eq_ignore_ascii_case("identity facts")
            {
                let _ = write!(reply, "XJQ OK ");
                write_identity_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("reboot confirm") {
                let _ = write!(reply, "XJQ OK reboot=after-reply");
                reboot_after_reply = true;
            } else if query.eq_ignore_ascii_case("net ips") {
                let _ = write!(reply, "XJQ OK ");
                write_net_ips_summary(&mut reply);
            } else if query.starts_with("net dns ") {
                write_net_dns_query_summary(index, query, &mut reply);
            } else if query.starts_with("net tcp ") {
                write_net_tcp_query_summary(index, query, &mut reply);
            } else if query.starts_with("net http-debug ") {
                write_net_http_debug_query_summary(index, query, &mut reply);
            } else if query.starts_with("net http-trace ") {
                write_net_http_trace_query_summary(index, query, &mut reply);
            } else if query.starts_with("net http-post ") {
                write_net_http_post_query_summary(index, query, &mut reply);
            } else if query.starts_with("net http ") {
                write_net_http_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint poll ") {
                write_maint_poll_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint status ") {
                write_maint_status_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint audit ") {
                write_maint_audit_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint verify-hmac ") {
                write_maint_verify_hmac_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint verify-local ") {
                write_maint_verify_local_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint pull-exec ") {
                write_maint_pull_exec_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint verify ") {
                write_maint_verify_query_summary(index, query, &mut reply);
            } else if query.eq_ignore_ascii_case("maint key-status")
                || query.eq_ignore_ascii_case("maint key-status confirm")
            {
                write_maint_key_status_summary(&mut reply);
            } else if query.starts_with("maint key-set ") {
                write_maint_key_set_query_summary(query, &mut reply);
            } else if query.eq_ignore_ascii_case("maint key-clear")
                || query.eq_ignore_ascii_case("maint key-clear confirm")
            {
                write_maint_key_clear_summary(&mut reply);
            } else if query.starts_with("maint ack-intent ") {
                write_maint_ack_intent_query_summary(index, query, &mut reply);
            } else if query.starts_with("maint event ") {
                write_maint_event_query_summary(index, query, &mut reply);
            } else if query.eq_ignore_ascii_case("wifi facts") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_facts_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi status") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_status_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi pcie") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_pcie_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi cru") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_cru_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi apb") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_apb_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi phy") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_phy_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi power") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_power_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi link") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_link_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi trace confirm") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_trace_summary(&mut reply);
            } else if query.starts_with("wifi sideband ") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_sideband_summary(query, &mut reply);
            } else if query.eq_ignore_ascii_case("wifi pcie-init confirm") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_pcie_init_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi pcie-init-high confirm") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_pcie_init_high_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("wifi pcie-init-linux confirm") {
                let _ = write!(reply, "XJQ OK ");
                write_wifi_pcie_init_linux_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("sd facts") {
                let _ = write!(
                reply,
                "XJQ OK sd-facts controller=dwmmc@fe2b0000 uboot-mmc=1 stage=fat32-preallocated-files ota-target=inactive-ab-slot writer=yes"
            );
            } else if query.eq_ignore_ascii_case("sd regs")
                || query.eq_ignore_ascii_case("sd regs confirm")
            {
                let _ = write!(reply, "XJQ OK ");
                write_sd_reg_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("sd read0")
                || query.eq_ignore_ascii_case("sd read0 confirm")
            {
                let _ = write!(reply, "XJQ ");
                write_sd_lba0_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("sd bpb")
                || query.eq_ignore_ascii_case("sd bpb confirm")
            {
                let _ = write!(reply, "XJQ ");
                write_sd_bpb_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("sd root")
                || query.eq_ignore_ascii_case("sd root confirm")
            {
                let _ = write!(reply, "XJQ ");
                write_sd_root_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("sd slots")
                || query.eq_ignore_ascii_case("sd slots confirm")
            {
                let _ = write!(reply, "XJQ ");
                write_sd_slots_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("sd extlinux")
                || query.eq_ignore_ascii_case("sd extlinux confirm")
            {
                let _ = write!(reply, "XJQ ");
                write_sd_extlinux_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db facts") {
                let _ = write!(
                reply,
                "XJQ OK db-facts file={DB_FILE_NAME_STR} page-bytes={DB_PAGE_BYTES} superblock=0 catalog={DB_ROOT_CATALOG_PAGE} wal-start={DB_WAL_START_PAGE} wal-pages={DB_WAL_PAGE_COUNT} table-root={DB_TABLE_ROOT_PAGE} table-records={DB_TABLE_RECORDS} btree-root={DB_BTREE_ROOT_PAGE} btree-leaf-start={DB_BTREE_LEAF_START_PAGE} btree-leaves={DB_BTREE_LEAF_COUNT} event-meta={DB_EVENT_META_PAGE} event-start={DB_EVENT_START_PAGE} event-pages={DB_EVENT_PAGE_COUNT} blob-meta={DB_BLOB_META_PAGE} blob-start={DB_BLOB_START_PAGE} value-bytes={DB_TABLE_VALUE_BYTES} persistence=preallocated-fat32"
            );
            } else if query.eq_ignore_ascii_case("db status")
                || query.eq_ignore_ascii_case("db status confirm")
            {
                write_db_status_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db init")
                || query.eq_ignore_ascii_case("db init confirm")
            {
                write_db_init_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db catalog")
                || query.eq_ignore_ascii_case("db catalog confirm")
            {
                write_db_catalog_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db catalog-init")
                || query.eq_ignore_ascii_case("db catalog-init confirm")
            {
                write_db_catalog_init_summary(&mut reply);
            } else if query.starts_with("db page ") {
                write_db_page_query_summary(query, &mut reply);
            } else if query.eq_ignore_ascii_case("db wal-append")
                || query.eq_ignore_ascii_case("db wal-append confirm")
            {
                write_db_wal_append_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db table")
                || query.eq_ignore_ascii_case("db table confirm")
            {
                write_db_table_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db table-init")
                || query.eq_ignore_ascii_case("db table-init confirm")
            {
                write_db_table_init_summary(&mut reply);
            } else if query.starts_with("db put ") {
                write_db_put_query_summary(query, &mut reply);
            } else if query.starts_with("db get ") {
                write_db_get_query_summary(query, &mut reply);
            } else if query.eq_ignore_ascii_case("db btree")
                || query.eq_ignore_ascii_case("db btree confirm")
            {
                write_db_btree_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db btree-init")
                || query.eq_ignore_ascii_case("db btree-init confirm")
            {
                write_db_btree_init_summary(&mut reply);
            } else if query.starts_with("db bput ") {
                write_db_bput_query_summary(query, &mut reply);
            } else if query.starts_with("db bget ") {
                write_db_bget_query_summary(query, &mut reply);
            } else if query.starts_with("db bscan ") {
                write_db_bscan_query_summary(query, &mut reply);
            } else if query.starts_with("db bleaf ") {
                write_db_bleaf_query_summary(query, &mut reply);
            } else if query.eq_ignore_ascii_case("db event-init")
                || query.eq_ignore_ascii_case("db event-init confirm")
            {
                write_db_event_init_summary(&mut reply);
            } else if query.eq_ignore_ascii_case("db event-status")
                || query.eq_ignore_ascii_case("db event-status confirm")
            {
                write_db_event_status_summary(&mut reply);
            } else if query.starts_with("db events ") {
                write_db_events_query_summary(query, &mut reply);
            } else if query.starts_with("db event ") {
                write_db_event_query_summary(query, &mut reply);
            } else if query.eq_ignore_ascii_case("db blob-status")
                || query.eq_ignore_ascii_case("db blob-status confirm")
            {
                write_db_blob_status_summary(&mut reply);
            } else {
                let _ = write!(reply, "XJQ ERR unknown-query");
            }
        }
    }

    let sent = net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice());
    if reboot_after_reply {
        busy_wait_ms(50);
        psci_system_reset()
    }
    sent
}

fn net_handle_boot_slot_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();

    if payload.len() < 5 {
        let _ = write!(reply, "XJBT ERR bad-len");
    } else {
        let slot = payload[4];
        if slot != b'a' && slot != b'b' {
            let _ = write!(reply, "XJBT ERR bad-slot");
        } else {
            match switch_extlinux_default_slot(slot) {
                Ok(()) => {
                    let _ = write!(
                        reply,
                        "XJBT OK slot={} default={}",
                        slot as char,
                        extlinux_slot_label(slot)
                    );
                }
                Err(error) => {
                    let _ = write!(reply, "XJBT ERR {}", error.as_str());
                    if let BootSlotError::Sd(SdReadError::StatusError(bits))
                    | BootSlotError::Sd(SdReadError::IdmacError(bits)) = error
                    {
                        let _ = write!(reply, " bits=0x{bits:08x}");
                    }
                }
            }
        }
    }

    net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice())
}

fn net_handle_reboot_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();

    if payload.len() != NETCONTROL_REBOOT_MAGIC.len() {
        let _ = write!(reply, "XJRB ERR bad-len");
        return net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice());
    }

    let _ = write!(reply, "XJRB OK reboot=after-reply");
    let _sent = net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice());
    busy_wait_ms(50);
    psci_system_reset()
}

fn net_handle_event_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();

    if payload.len() < 20 {
        let _ = write!(reply, "XJEV ERR bad-len");
    } else {
        let kind = read_be32(payload, 4).unwrap_or(0);
        let source = read_be32(payload, 8).unwrap_or(0);
        let payload_len = read_be32(payload, 12).unwrap_or(0) as usize;
        let expected_crc = read_be32(payload, 16).unwrap_or(0);
        if payload_len == 0 || payload_len > DB_EVENT_PAYLOAD_BYTES {
            let _ = write!(
                reply,
                "XJEV ERR bad-payload-len max={DB_EVENT_PAYLOAD_BYTES}"
            );
        } else if payload.len() < 20 + payload_len {
            let _ = write!(reply, "XJEV ERR truncated");
        } else {
            let event_payload = &payload[20..20 + payload_len];
            let actual_crc = db_payload_crc(event_payload);
            if actual_crc != expected_crc {
                let _ = write!(
                    reply,
                    "XJEV ERR crc-mismatch expected=0x{expected_crc:08x} actual=0x{actual_crc:08x}"
                );
            } else {
                match append_db_event(kind, source, event_payload) {
                    Ok((_info, _catalog, meta, event)) => {
                        let _ = write!(
                            reply,
                            "XJEV OK seq={} page={} kind={} source={} payload-len={} payload-crc=0x{:08x} events={} next-page={}",
                            event.sequence,
                            event.page,
                            event.kind,
                            event.source,
                            event.payload_len,
                            event.payload_crc,
                            meta.event_count,
                            meta.next_page
                        );
                    }
                    Err(error) => {
                        let _ = write!(reply, "XJEV ERR {}", error.as_str());
                        if let DbError::Sd(SdReadError::StatusError(bits))
                        | DbError::Sd(SdReadError::IdmacError(bits)) = error
                        {
                            let _ = write!(reply, " bits=0x{bits:08x}");
                        }
                    }
                }
            }
        }
    }

    net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice())
}

fn net_handle_db_blob_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();

    if payload.starts_with(NETCONTROL_DB_BLOB_BEGIN_MAGIC) {
        if payload.len() < 16 {
            let _ = write!(reply, "XJBB ERR bad-begin");
        } else {
            let size = read_be32(payload, 4).unwrap_or(0);
            let expected_crc = read_be32(payload, 8).unwrap_or(0);
            let _flags = read_be32(payload, 12).unwrap_or(0);
            match init_db_blob_import(size, expected_crc) {
                Ok((info, _catalog, meta)) => {
                    let fast_lba_base = db_blob_fast_lba_base(info, meta).unwrap_or(0);
                    let fast_lba = fast_lba_base != 0;
                    let seq = file_sector_location(info.facts, info.entry, meta.start_page).ok();
                    let (seq_lba, seq_lba_current, seq_cluster, seq_sector_in_cluster) =
                        if let Some((lba, cluster, sector_in_cluster)) = seq {
                            (true, lba, cluster, sector_in_cluster)
                        } else {
                            (false, 0, 0, 0)
                        };
                    unsafe {
                        DB_BLOB_IMPORT_STATE = DbBlobImportState {
                            active: true,
                            size,
                            expected_crc,
                            received: 0,
                            rolling_crc: 0xffff_ffff,
                            fast_lba,
                            fast_lba_base,
                            page_count: meta.page_count,
                            seq_lba,
                            seq_facts: info.facts,
                            seq_sector_index: meta.start_page,
                            seq_lba_current,
                            seq_cluster,
                            seq_sector_in_cluster,
                        };
                    }
                    let _ = write!(
                        reply,
                        "XJBB READY size={} crc32=0x{:08x} capacity={} page-size={} fast-lba={} seq-lba={}",
                        size, expected_crc, meta.capacity_bytes, DB_PAGE_BYTES, fast_lba, seq_lba
                    );
                }
                Err(error) => write_db_blob_packet_error(&mut reply, "begin", error),
            }
        }
    } else if payload.starts_with(NETCONTROL_DB_BLOB_DATA_MAGIC) {
        if payload.len() < 10 {
            let _ = write!(reply, "XJBB ERR bad-data");
        } else {
            let offset = read_be32(payload, 4).unwrap_or(0);
            let len = read_be16(payload, 8).unwrap_or(0) as usize;
            let data = if payload.len() >= 10 + len {
                &payload[10..10 + len]
            } else {
                &[]
            };
            let (active, received, size, seq_lba, fast_lba, fast_lba_base, page_count) = unsafe {
                (
                    DB_BLOB_IMPORT_STATE.active,
                    DB_BLOB_IMPORT_STATE.received,
                    DB_BLOB_IMPORT_STATE.size,
                    DB_BLOB_IMPORT_STATE.seq_lba,
                    DB_BLOB_IMPORT_STATE.fast_lba,
                    DB_BLOB_IMPORT_STATE.fast_lba_base,
                    DB_BLOB_IMPORT_STATE.page_count,
                )
            };
            if !active {
                let _ = write!(reply, "XJBB ERR not-active");
            } else if len == 0 || len > DB_BLOB_PACKET_BYTES || data.len() != len {
                let _ = write!(
                    reply,
                    "XJBB ERR bad-data-len expected={received} max={DB_BLOB_PACKET_BYTES}"
                );
            } else if offset < received && offset.saturating_add(len as u32) <= received {
                let _ = write!(reply, "XJBB ACK offset={received}");
            } else if offset != received {
                let _ = write!(reply, "XJBB ERR expected={received}");
            } else if offset.saturating_add(len as u32) > size {
                let _ = write!(reply, "XJBB ERR overflow size={size}");
            } else if seq_lba {
                match write_db_blob_chunk_sequential(offset, data, size) {
                    Ok(new_received) => {
                        let actual_crc = update_db_blob_import_crc(data, new_received);
                        if new_received == size {
                            match mark_db_blob_import_received(new_received, actual_crc) {
                                Ok(_) => {
                                    let _ = write!(reply, "XJBB ACK offset={new_received}");
                                }
                                Err(error) => {
                                    write_db_blob_packet_error(&mut reply, "data-mark", error)
                                }
                            }
                        } else {
                            let _ = write!(reply, "XJBB ACK offset={new_received}");
                        }
                    }
                    Err(error) => write_db_blob_packet_error(&mut reply, "data", error),
                }
            } else if fast_lba {
                match write_db_blob_chunk_fast(offset, data, fast_lba_base, page_count, size) {
                    Ok(new_received) => {
                        let actual_crc = update_db_blob_import_crc(data, new_received);
                        if new_received == size {
                            match mark_db_blob_import_received(new_received, actual_crc) {
                                Ok(_) => {
                                    let _ = write!(reply, "XJBB ACK offset={new_received}");
                                }
                                Err(error) => {
                                    write_db_blob_packet_error(&mut reply, "data-mark", error)
                                }
                            }
                        } else {
                            let _ = write!(reply, "XJBB ACK offset={new_received}");
                        }
                    }
                    Err(error) => write_db_blob_packet_error(&mut reply, "data", error),
                }
            } else {
                match write_db_blob_chunk(offset, data) {
                    Ok(meta) => {
                        let actual_crc = update_db_blob_import_crc(data, meta.received_size);
                        if meta.received_size == size {
                            match mark_db_blob_import_received(meta.received_size, actual_crc) {
                                Ok(_) => {
                                    let _ = write!(reply, "XJBB ACK offset={}", meta.received_size);
                                }
                                Err(error) => {
                                    write_db_blob_packet_error(&mut reply, "data-mark", error)
                                }
                            }
                        } else {
                            let _ = write!(reply, "XJBB ACK offset={}", meta.received_size);
                        }
                    }
                    Err(error) => write_db_blob_packet_error(&mut reply, "data", error),
                }
            }
        }
    } else if payload.starts_with(NETCONTROL_DB_BLOB_COMMIT_MAGIC) {
        let active = unsafe { DB_BLOB_IMPORT_STATE.active };
        if !active {
            match read_db_blob_meta() {
                Ok((_info, _catalog, meta)) if meta.flags & DB_BLOB_FLAG_COMMITTED != 0 => {
                    let _ = write!(
                        reply,
                        "XJBB OK size={} crc32=0x{:08x} pages={} committed=yes",
                        meta.expected_size, meta.committed_crc, meta.page_count
                    );
                }
                Ok((_info, _catalog, meta))
                    if meta.flags & DB_BLOB_FLAG_ACTIVE != 0
                        && meta.received_size == meta.expected_size =>
                {
                    if meta.committed_crc != meta.expected_crc {
                        let _ = write!(
                            reply,
                            "XJBB ERR recover-crc-missing received={} size={} expected-crc=0x{:08x} saved-crc=0x{:08x}",
                            meta.received_size, meta.expected_size, meta.expected_crc, meta.committed_crc
                        );
                    } else {
                        match commit_db_blob_import(Some(meta.committed_crc)) {
                            Ok(meta) => {
                                let _ = write!(
                                reply,
                                "XJBB OK size={} crc32=0x{:08x} pages={} committed=yes recovered=yes",
                                meta.expected_size, meta.committed_crc, meta.page_count
                            );
                            }
                            Err(error) => {
                                write_db_blob_packet_error(&mut reply, "commit-recover", error)
                            }
                        }
                    }
                }
                Ok((_info, _catalog, meta)) if meta.flags & DB_BLOB_FLAG_ACTIVE != 0 => {
                    let _ = write!(
                        reply,
                        "XJBB ERR incomplete received={} size={}",
                        meta.received_size, meta.expected_size
                    );
                }
                _ => {
                    let _ = write!(reply, "XJBB ERR not-active");
                }
            }
        } else {
            let (received, size, expected_crc, rolling_crc) = unsafe {
                (
                    DB_BLOB_IMPORT_STATE.received,
                    DB_BLOB_IMPORT_STATE.size,
                    DB_BLOB_IMPORT_STATE.expected_crc,
                    DB_BLOB_IMPORT_STATE.rolling_crc,
                )
            };
            if received != size {
                let _ = write!(reply, "XJBB ERR incomplete received={received} size={size}");
            } else {
                let actual_crc = !rolling_crc;
                if let Err(error) = mark_db_blob_import_received(received, actual_crc) {
                    write_db_blob_packet_error(&mut reply, "commit-mark", error);
                    net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice());
                    return true;
                }
                match commit_db_blob_import(Some(actual_crc)) {
                    Ok(meta) => {
                        if meta.expected_crc != expected_crc {
                            let _ = write!(reply, "XJBB ERR commit stale-state");
                        } else {
                            unsafe {
                                DB_BLOB_IMPORT_STATE = DbBlobImportState::empty();
                            }
                            let _ = write!(
                                reply,
                                "XJBB OK size={} crc32=0x{:08x} pages={} committed=yes",
                                meta.expected_size, meta.committed_crc, meta.page_count
                            );
                        }
                    }
                    Err(error) => write_db_blob_packet_error(&mut reply, "commit", error),
                }
            }
        }
    } else if payload.starts_with(NETCONTROL_DB_BLOB_READ_MAGIC) {
        if payload.len() < 10 {
            let _ = write!(reply, "XJBR ERR bad-read");
        } else {
            let offset = read_be32(payload, 4).unwrap_or(0);
            let len = read_be16(payload, 8).unwrap_or(0) as usize;
            match read_db_blob_chunk(offset, len) {
                Ok((meta, block, take)) => {
                    reply.push_slice(NETCONTROL_DB_BLOB_READ_MAGIC);
                    reply.push_be32(offset);
                    reply.push_be16(take as u16);
                    reply.push_be32(meta.expected_size);
                    reply.push_be32(meta.committed_crc);
                    reply.push_slice(&block[..take]);
                }
                Err(error) => {
                    write_db_blob_packet_error_with_prefix(&mut reply, "XJBR", "read", error)
                }
            }
        }
    } else {
        return false;
    }

    net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice())
}

fn write_db_blob_packet_error<W: Write>(out: &mut W, action: &str, error: DbError) {
    write_db_blob_packet_error_with_prefix(out, "XJBB", action, error);
}

fn write_db_blob_packet_error_with_prefix<W: Write>(
    out: &mut W,
    prefix: &str,
    action: &str,
    error: DbError,
) {
    let _ = write!(out, "{prefix} ERR {action} {}", error.as_str());
    if let DbError::Sd(SdReadError::StatusError(bits))
    | DbError::Sd(SdReadError::IdmacError(bits)) = error
    {
        let _ = write!(out, " bits=0x{bits:08x}");
    }
}

fn net_handle_persist_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();

    if payload.len() < 5 {
        let _ = write!(reply, "XJPS ERR bad-len");
    } else {
        let slot = payload[4];
        let image = unsafe { LAST_NET_IMAGE };
        if !image.valid {
            let _ = write!(reply, "XJPS ERR no-image");
        } else if slot != b'a' && slot != b'b' {
            let _ = write!(reply, "XJPS ERR bad-slot");
        } else {
            match persist_last_image_to_slot(slot, image) {
                Ok(actual_crc) => {
                    let _ = write!(
                        reply,
                        "XJPS OK slot={} size={} crc32=0x{actual_crc:08x}",
                        slot as char, image.size
                    );
                }
                Err(error) => {
                    let _ = write!(reply, "XJPS ERR {}", error.as_str());
                    if let PersistError::Sd(SdReadError::StatusError(bits))
                    | PersistError::Sd(SdReadError::IdmacError(bits)) = error
                    {
                        let _ = write!(reply, " bits=0x{bits:08x}");
                    } else if let PersistError::CrcMismatch(actual) = error {
                        let _ = write!(reply, " actual=0x{actual:08x}");
                    }
                }
            }
        }
    }

    net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice())
}

struct NetReplyBuf {
    bytes: [u8; 1200],
    len: usize,
}

impl NetReplyBuf {
    fn new() -> Self {
        Self {
            bytes: [0; 1200],
            len: 0,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    fn push_slice(&mut self, bytes: &[u8]) {
        let take = bytes.len().min(self.bytes.len().saturating_sub(self.len));
        self.bytes[self.len..self.len + take].copy_from_slice(&bytes[..take]);
        self.len += take;
    }

    fn push_be16(&mut self, value: u16) {
        self.push_slice(&[(value >> 8) as u8, value as u8]);
    }

    fn push_be32(&mut self, value: u32) {
        self.push_slice(&[
            (value >> 24) as u8,
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ]);
    }
}

impl Write for NetReplyBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = self.bytes.len().saturating_sub(self.len);
        if s.len() > remaining {
            return Err(fmt::Error);
        }
        self.bytes[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

struct HttpRequestBuf {
    bytes: [u8; HTTP_REQUEST_BYTES],
    len: usize,
}

impl HttpRequestBuf {
    fn new() -> Self {
        Self {
            bytes: [0; HTTP_REQUEST_BYTES],
            len: 0,
        }
    }

    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl Write for HttpRequestBuf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = self.bytes.len().saturating_sub(self.len);
        if s.len() > remaining {
            return Err(fmt::Error);
        }
        self.bytes[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

fn net_handle_loader_packet(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    src_port: u16,
    payload: &[u8],
) -> bool {
    let mut reply = NetReplyBuf::new();
    let mut reload: Option<(u64, u64)> = None;

    if payload.starts_with(NETLOAD_BEGIN_MAGIC) {
        if payload.len() < 16 {
            let _ = write!(reply, "XJNL ERR bad-begin");
        } else {
            let size = read_be32(payload, 4).unwrap_or(0);
            let expected_crc = read_be32(payload, 8).unwrap_or(0);
            let flags = read_be32(payload, 12).unwrap_or(0);
            let go = flags & NETLOAD_FLAG_GO != 0;
            if size == 0 || size as u64 > XLOAD_MAX_IMAGE_SIZE {
                let _ = write!(reply, "XJNL ERR bad-size max={XLOAD_MAX_IMAGE_SIZE}");
            } else if let Some(addr) = choose_xload_auto_addr(size as u64) {
                unsafe {
                    NETLOAD_STATE = NetLoadState {
                        active: true,
                        addr,
                        size,
                        expected_crc,
                        received: 0,
                        go,
                    };
                }
                let _ = write!(
                    reply,
                    "XJNL READY addr=0x{addr:016x} size={size} crc32=0x{expected_crc:08x} go={}",
                    yes_no(go)
                );
            } else {
                let _ = write!(reply, "XJNL ERR no-safe-slot");
            }
        }
    } else if payload.starts_with(NETLOAD_DATA_MAGIC) {
        if payload.len() < 10 {
            let _ = write!(reply, "XJNL ERR bad-data");
        } else {
            let offset = read_be32(payload, 4).unwrap_or(0);
            let len = read_be16(payload, 8).unwrap_or(0) as usize;
            let data = if payload.len() >= 10 + len {
                &payload[10..10 + len]
            } else {
                &[]
            };
            let handled = unsafe {
                let received = NETLOAD_STATE.received;
                let size = NETLOAD_STATE.size;
                if !NETLOAD_STATE.active {
                    let _ = write!(reply, "XJNL ERR not-active");
                    true
                } else if len == 0 || data.len() != len {
                    let _ = write!(reply, "XJNL ERR bad-data-len");
                    true
                } else if offset < received && offset.saturating_add(len as u32) <= received {
                    let _ = write!(reply, "XJNL ACK offset={received}");
                    true
                } else if offset != received {
                    let _ = write!(reply, "XJNL ERR expected={received}");
                    true
                } else if offset.saturating_add(len as u32) > size {
                    let _ = write!(reply, "XJNL ERR overflow size={size}");
                    true
                } else {
                    let dst = NETLOAD_STATE.addr + offset as u64;
                    for (i, byte) in data.iter().copied().enumerate() {
                        write_volatile((dst as usize + i) as *mut u8, byte);
                    }
                    NETLOAD_STATE.received = NETLOAD_STATE.received.wrapping_add(len as u32);
                    let next = NETLOAD_STATE.received;
                    let _ = write!(reply, "XJNL ACK offset={next}");
                    true
                }
            };
            if !handled {
                return false;
            }
        }
    } else if payload.starts_with(NETLOAD_COMMIT_MAGIC) {
        unsafe {
            if !NETLOAD_STATE.active {
                let _ = write!(reply, "XJNL ERR not-active");
            } else {
                let state = NETLOAD_STATE;
                if state.received != state.size {
                    let _ = write!(
                        reply,
                        "XJNL ERR incomplete received={} size={}",
                        state.received, state.size
                    );
                } else {
                    let actual_crc = crc32_memory(state.addr, state.size as u64);
                    if actual_crc != state.expected_crc {
                        let _ = write!(
                            reply,
                            "XJNL ERR crc32 expected=0x{:08x} actual=0x{actual_crc:08x}",
                            state.expected_crc
                        );
                    } else {
                        flush_loaded_image(state.addr, state.size as u64);
                        let _ = write!(
                            reply,
                            "XJNL OK addr=0x{:016x} size={} crc32=0x{actual_crc:08x} go={}",
                            state.addr,
                            state.size,
                            yes_no(state.go)
                        );
                        if state.go {
                            set_netload_boot_flag();
                            reload = Some((state.addr, state.size as u64));
                        } else {
                            LAST_NET_IMAGE = NetImageState {
                                valid: true,
                                addr: state.addr,
                                size: state.size,
                                crc32: actual_crc,
                            };
                            NETLOAD_STATE = NetLoadState::empty();
                        }
                    }
                }
            }
        }
    } else {
        return false;
    }

    let sent = net_send_udp_reply(index, dst_mac, src_ip, src_port, reply.as_slice());
    if let Some((addr, size)) = reload {
        busy_wait_ms(200);
        reload_to_link_address(addr, size);
    }
    sent
}

fn net_send_host_hello(index: usize) -> bool {
    let payload = if index == 0 {
        b"xjhello pcie3x1"
    } else {
        b"xjhello pcie3x2"
    };
    net_send_udp_packet(
        index,
        NET_BROADCAST_MAC,
        net_broadcast_ip(index),
        NET_HOST_PORT,
        NET_UDP_PORT,
        payload,
    )
}

fn net_send_udp_reply(
    index: usize,
    dst_mac: [u8; 6],
    dst_ip: [u8; 4],
    dst_port: u16,
    payload: &[u8],
) -> bool {
    net_send_udp_packet(index, dst_mac, dst_ip, dst_port, NET_UDP_PORT, payload)
}

fn net_send_udp_packet(
    index: usize,
    dst_mac: [u8; 6],
    dst_ip: [u8; 4],
    dst_port: u16,
    src_port: u16,
    payload: &[u8],
) -> bool {
    net_send_udp_packet_with_src(
        index,
        dst_mac,
        net_device_ip(index),
        dst_ip,
        dst_port,
        src_port,
        payload,
    )
}

fn net_send_udp_packet_with_src(
    index: usize,
    dst_mac: [u8; 6],
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    dst_port: u16,
    src_port: u16,
    payload: &[u8],
) -> bool {
    let our_mac = rtl_local_mac(index);
    let udp_len = 8 + payload.len();
    let ip_len = 20 + udp_len;
    let frame_len = 14 + ip_len;
    let padded_len = if frame_len < 60 { 60 } else { frame_len };

    if padded_len > RTL_TX_BUFFER_SIZE {
        return false;
    }

    clear_tx_buffer(index);
    unsafe {
        let tx = &mut RTL_TX_BUFFERS.0[index];
        tx[0..6].copy_from_slice(&dst_mac);
        tx[6..12].copy_from_slice(&our_mac);
        write_be16(tx, 12, ETH_TYPE_IPV4);

        let ip = 14;
        tx[ip] = 0x45;
        tx[ip + 1] = 0;
        write_be16(tx, ip + 2, ip_len as u16);
        write_be16(tx, ip + 4, 0x3500 | index as u16);
        write_be16(tx, ip + 6, 0);
        tx[ip + 8] = 64;
        tx[ip + 9] = IPV4_PROTO_UDP;
        write_be16(tx, ip + 10, 0);
        tx[ip + 12..ip + 16].copy_from_slice(&src_ip);
        tx[ip + 16..ip + 20].copy_from_slice(&dst_ip);
        let csum = ipv4_header_checksum(&tx[ip..ip + 20]);
        write_be16(tx, ip + 10, csum);

        let udp = ip + 20;
        write_be16(tx, udp, src_port);
        write_be16(tx, udp + 2, dst_port);
        write_be16(tx, udp + 4, udp_len as u16);
        write_be16(tx, udp + 6, 0);
        tx[udp + 8..udp + 8 + payload.len()].copy_from_slice(payload);
    }

    rtl_send_prepared_frame(index, padded_len, 100)
}

fn net_send_dhcp_discover(index: usize, xid: u32) -> bool {
    net_send_dhcp_packet(index, xid, DHCP_MSG_DISCOVER, None, None)
}

fn net_send_dhcp_request(index: usize, xid: u32, requested_ip: [u8; 4], server: [u8; 4]) -> bool {
    let server = if server == IPV4_ANY_IP {
        None
    } else {
        Some(server)
    };
    net_send_dhcp_packet(index, xid, DHCP_MSG_REQUEST, Some(requested_ip), server)
}

fn net_send_dhcp_packet(
    index: usize,
    xid: u32,
    message_type: u8,
    requested_ip: Option<[u8; 4]>,
    server: Option<[u8; 4]>,
) -> bool {
    let mut payload = [0u8; DHCP_PACKET_BYTES];
    let Some(len) = build_dhcp_packet(index, xid, message_type, requested_ip, server, &mut payload)
    else {
        return false;
    };
    net_send_udp_packet_with_src(
        index,
        NET_BROADCAST_MAC,
        IPV4_ANY_IP,
        IPV4_BROADCAST_IP,
        DHCP_SERVER_PORT,
        DHCP_CLIENT_PORT,
        &payload[..len],
    )
}

fn build_dhcp_packet(
    index: usize,
    xid: u32,
    message_type: u8,
    requested_ip: Option<[u8; 4]>,
    server: Option<[u8; 4]>,
    payload: &mut [u8],
) -> Option<usize> {
    if payload.len() < DHCP_MIN_PACKET_BYTES {
        return None;
    }
    for byte in payload.iter_mut() {
        *byte = 0;
    }

    let mac = rtl_local_mac(index);
    payload[0] = 1;
    payload[1] = 1;
    payload[2] = 6;
    payload[3] = 0;
    write_be32(payload, 4, xid);
    write_be16(payload, 10, 0x8000);
    payload[28..34].copy_from_slice(&mac);
    payload[236..240].copy_from_slice(&DHCP_MAGIC_COOKIE);

    let mut offset = 240usize;
    dhcp_put_option(payload, &mut offset, DHCP_OPT_MESSAGE_TYPE, &[message_type])?;

    let mut client_id = [0u8; 7];
    client_id[0] = 1;
    client_id[1..7].copy_from_slice(&mac);
    dhcp_put_option(payload, &mut offset, DHCP_OPT_CLIENT_ID, &client_id)?;

    let max_size = [
        ((DHCP_PACKET_BYTES >> 8) & 0xff) as u8,
        DHCP_PACKET_BYTES as u8,
    ];
    dhcp_put_option(payload, &mut offset, DHCP_OPT_MAX_MESSAGE_SIZE, &max_size)?;

    if let Some(ip) = requested_ip {
        dhcp_put_option(payload, &mut offset, DHCP_OPT_REQUESTED_IP, &ip)?;
    }
    if let Some(server) = server {
        dhcp_put_option(payload, &mut offset, DHCP_OPT_SERVER_ID, &server)?;
    }

    let params = [
        DHCP_OPT_SUBNET_MASK,
        DHCP_OPT_ROUTER,
        DHCP_OPT_DNS,
        DHCP_OPT_LEASE_TIME,
        DHCP_OPT_SERVER_ID,
    ];
    dhcp_put_option(
        payload,
        &mut offset,
        DHCP_OPT_PARAMETER_REQUEST_LIST,
        &params,
    )?;

    let hostname = if index == 0 {
        b"xiaojin-r5c-0".as_slice()
    } else {
        b"xiaojin-r5c-1".as_slice()
    };
    dhcp_put_option(payload, &mut offset, DHCP_OPT_HOSTNAME, hostname)?;

    if offset >= payload.len() {
        return None;
    }
    payload[offset] = DHCP_OPT_END;
    offset += 1;
    if offset < DHCP_MIN_PACKET_BYTES {
        offset = DHCP_MIN_PACKET_BYTES;
    }
    Some(offset)
}

fn dhcp_put_option(payload: &mut [u8], offset: &mut usize, code: u8, data: &[u8]) -> Option<()> {
    if data.len() > 255 || *offset + 2 + data.len() > payload.len() {
        return None;
    }
    payload[*offset] = code;
    payload[*offset + 1] = data.len() as u8;
    payload[*offset + 2..*offset + 2 + data.len()].copy_from_slice(data);
    *offset += 2 + data.len();
    Some(())
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

fn print_gpio_pin_status(uart: &mut Uart, label: &str, bank: GpioBank, pin_name: &str, pin: u8) {
    let direction = read_gpio_v2_direction(bank, pin);
    let level = read_gpio_v2_level(bank, pin);
    match (direction, level) {
        (Ok(direction), Ok(level)) => {
            let _ = writeln!(
                uart,
                "{label:<24} {} {} bit={:<2} dir={:<3} level={}",
                bank.name,
                pin_name,
                pin,
                gpio_direction_name(direction),
                bit_name(level)
            );
        }
        (direction, level) => {
            let _ = writeln!(
                uart,
                "{label:<24} {} {} bit={:<2}",
                bank.name, pin_name, pin
            );
            print_probe_result(uart, "    direction", direction.map(bool_to_u32));
            print_probe_result(uart, "    level", level.map(bool_to_u32));
        }
    }
}

fn print_pcie_reset_pins(uart: &mut Uart) {
    uart.put_str("pcie reset pins:\n");
    for pin in PCIE_RESET_PINS {
        print_pcie_reset_pin(uart, pin);
    }
    uart.put_str("  note            RJ45 release-high behavior was validated by PCIe link-up\n");
}

fn print_pcie_reset_pin(uart: &mut Uart, pin: &PcieResetPin) {
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

fn write_gpio_pin_summary<W: Write>(out: &mut W, label: &str, bank: GpioBank, pin: u8) {
    match (
        read_gpio_v2_direction(bank, pin),
        read_gpio_v2_level(bank, pin),
    ) {
        (Ok(direction), Ok(level)) => {
            let _ = write!(
                out,
                " {label}={}{} dir={} level={}",
                bank.name,
                pin,
                gpio_direction_name(direction),
                bit_name(level)
            );
        }
        _ => {
            let _ = write!(out, " {label}=fault");
        }
    }
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

fn gpio_v2_set_output(bank: GpioBank, pin: u8, level: bool) {
    gpio_v2_write_bit(bank, GPIO_V2_PORT_DR, pin, level);
    gpio_v2_write_bit(bank, GPIO_V2_PORT_DDR, pin, true);
}

fn set_pcie_reset_asserted(asserted: bool) {
    for pin in &PCIE_RESET_PINS[..PCIE_ETH_RESET_COUNT] {
        set_pcie_reset_pin_asserted(pin, asserted);
    }
}

fn set_pcie_reset_pin_asserted(pin: &PcieResetPin, asserted: bool) {
    let level = if pin.active_high { asserted } else { !asserted };
    gpio_v2_set_output(pin.bank, pin.pin, level);
}

fn set_pcie_reset_pin_level(pin: &PcieResetPin, level: bool) {
    gpio_v2_set_output(pin.bank, pin.pin, level);
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

fn cru_bit_set(bit: CruBit) -> Option<bool> {
    probe_read32(bit.base + bit.offset)
        .ok()
        .map(|value| value & (1u32 << bit.bit) != 0)
}

fn cru_gate_is_enabled(bit: CruBit) -> bool {
    matches!(cru_bit_set(bit), Some(false))
}

fn cru_reset_is_deasserted(bit: CruBit) -> bool {
    matches!(cru_bit_set(bit), Some(false))
}

fn cru_gate_enable(bit: CruBit) {
    write_mmio32(bit.base + bit.offset, hiword_update(bit.bit, false));
}

fn cru_reset_assert(bit: CruBit) {
    write_mmio32(bit.base + bit.offset, hiword_update(bit.bit, true));
}

fn cru_reset_deassert(bit: CruBit) {
    write_mmio32(bit.base + bit.offset, hiword_update(bit.bit, false));
}

fn hiword_update(bit: u8, value: bool) -> u32 {
    let mask = 1u32 << (bit + 16);
    let data = if value { 1u32 << bit } else { 0 };
    mask | data
}

fn hiword_update_range(bitstart: u8, bitend: u8, value: u32) -> u32 {
    let width = bitend - bitstart + 1;
    let mask = if width >= 32 {
        u32::MAX
    } else {
        ((1u32 << width) - 1) << bitstart
    };
    (mask << 16) | ((value << bitstart) & mask)
}

fn update_mmio32(addr: u64, mask: u32, value: u32) {
    let current = probe_read32(addr).unwrap_or(0);
    write_mmio32(addr, (current & !mask) | (value & mask));
}

fn combphy_param_write(base: u64, offset: u64, bitstart: u8, bitend: u8, value: u32) {
    write_mmio32(base + offset, hiword_update_range(bitstart, bitend, value));
}

fn wifi_pcie2_dbi_safe() -> bool {
    PCIE2_CLOCK_GATES.iter().copied().all(cru_gate_is_enabled)
        && PCIEPHY2_REF_CLOCK_GATES
            .iter()
            .copied()
            .all(cru_gate_is_enabled)
        && COMBPHY2_CLOCK_GATES
            .iter()
            .copied()
            .all(cru_gate_is_enabled)
        && PCIE2_RESETS.iter().copied().all(cru_reset_is_deasserted)
        && COMBPHY2_RESETS.iter().copied().all(cru_reset_is_deasserted)
}

fn wifi_pcie2_init_sequence() -> WifiPcieInitResult {
    wifi_pcie2_init_sequence_with_reset_release(false)
}

fn wifi_pcie2_init_sequence_release_high() -> WifiPcieInitResult {
    wifi_pcie2_init_sequence_with_reset_release(true)
}

fn wifi_pcie2_enable_clocks() {
    for gate in PCIEPHY2_REF_CLOCK_GATES {
        cru_gate_enable(*gate);
    }
    cru_gate_enable(PCIE_CLOCK_GATES[0]);
    cru_gate_enable(PCIE_CLOCK_GATES[1]);
    for gate in PCIE2_CLOCK_GATES {
        cru_gate_enable(*gate);
    }
    for gate in COMBPHY2_CLOCK_GATES {
        cru_gate_enable(*gate);
    }
}

fn wifi_pcie2_configure_combphy() {
    // RK3568 naneng combphy PCIe mode, mirrored from the upstream Linux driver.
    update_mmio32(
        COMBPHY2_BASE + RK3568_COMBPHY_REG32,
        0x0000_00f0,
        0x0000_0010,
    );
    combphy_param_write(PIPE_PHY_GRF_BASE, 0x00, 0, 15, 0x1000);
    combphy_param_write(PIPE_PHY_GRF_BASE, 0x04, 0, 15, 0x0000);
    combphy_param_write(PIPE_PHY_GRF_BASE, 0x08, 0, 15, 0x0101);
    combphy_param_write(PIPE_PHY_GRF_BASE, 0x0c, 0, 15, 0x0200);
    combphy_param_write(PIPE_PHY_GRF_BASE, 0x04, 13, 14, 0x02);
    update_mmio32(
        COMBPHY2_BASE + RK3568_COMBPHY_REG33,
        0x0000_001c,
        0x0000_0008,
    );
    write_mmio32(COMBPHY2_BASE + RK3568_COMBPHY_REG12, 0x04);
    update_mmio32(
        COMBPHY2_BASE + RK3568_COMBPHY_REG6,
        0x0000_00c0,
        0x0000_0040,
    );
    write_mmio32(COMBPHY2_BASE + RK3568_COMBPHY_REG18, 0x32);
    write_mmio32(COMBPHY2_BASE + RK3568_COMBPHY_REG11, 0xf0);
}

fn wifi_pcie2_configure_rc() {
    write_mmio32(
        PCIE2X1_APB_BASE + PCIE_CLIENT_HOT_RESET_CTRL,
        PCIE_CLIENT_HOT_RESET_ENHANCE,
    );
    write_mmio32(
        PCIE2X1_APB_BASE + PCIE_CLIENT_POWER_CON,
        PCIE_CLIENT_CLKREQ_PULL_DOWN_NOT_READY,
    );
    write_mmio32(
        PCIE2X1_APB_BASE + PCIE_CLIENT_GENERAL_CON,
        PCIE_CLIENT_SET_RC_MODE,
    );
}

fn wifi_pcie2_wait_link_and_cfg() -> WifiPcieInitResult {
    let ltssm = wait_pcie_link(PCIE2_CONTROLLER, PCIE_LINK_TRAIN_TIMEOUT_MS)
        .unwrap_or_else(|| probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_LTSSM_STATUS).unwrap_or(0));
    let link_up = ltssm & PCIE_CLIENT_LINKUP_MASK == PCIE_CLIENT_LINKUP;
    let cfg0 = if link_up {
        pcie_program_cfg_atu(
            PCIE2_CONTROLLER,
            PCIE2_CONTROLLER.root_bus.wrapping_add(1),
            0,
            0,
            PCIE_ATU_TYPE_CFG0,
        );
        delay_ms(1);
        probe_read32(PCIE2X1_CFG_BASE).ok()
    } else {
        None
    };

    WifiPcieInitResult {
        link_up,
        ltssm,
        cfg0,
    }
}

fn wifi_pcie2_init_sequence_with_reset_release(release_high: bool) -> WifiPcieInitResult {
    gpio_v2_write_bit(GPIO0, GPIO_V2_PORT_DR, PCIE_POWER_PIN, true);
    gpio_v2_write_bit(GPIO0, GPIO_V2_PORT_DDR, PCIE_POWER_PIN, true);

    wifi_pcie2_enable_clocks();

    for reset in COMBPHY2_RESETS {
        cru_reset_deassert(*reset);
    }
    for reset in PCIE2_RESETS {
        cru_reset_deassert(*reset);
    }

    wifi_pcie2_configure_combphy();

    set_pcie_reset_pin_level(&PCIE_RESET_PINS[2], !release_high);
    delay_ms(20);

    wifi_pcie2_configure_rc();
    set_pcie_ltssm(PCIE2_CONTROLLER, true);
    delay_ms(100);
    set_pcie_reset_pin_level(&PCIE_RESET_PINS[2], release_high);

    wifi_pcie2_wait_link_and_cfg()
}

fn wifi_pcie2_init_sequence_linux_order() -> WifiPcieInitResult {
    gpio_v2_write_bit(GPIO0, GPIO_V2_PORT_DR, PCIE_POWER_PIN, true);
    gpio_v2_write_bit(GPIO0, GPIO_V2_PORT_DDR, PCIE_POWER_PIN, true);

    set_pcie_reset_pin_level(&PCIE_RESET_PINS[2], false);
    cru_reset_assert(PCIE2_RESETS[1]);
    delay_ms(20);

    wifi_pcie2_enable_clocks();
    for reset in COMBPHY2_RESETS {
        cru_reset_deassert(*reset);
    }
    wifi_pcie2_configure_combphy();

    cru_reset_deassert(PCIE2_RESETS[1]);
    for reset in PCIE2_RESETS {
        cru_reset_deassert(*reset);
    }
    delay_ms(1);

    wifi_pcie2_configure_rc();
    set_pcie_ltssm(PCIE2_CONTROLLER, true);
    delay_ms(100);
    set_pcie_reset_pin_level(&PCIE_RESET_PINS[2], true);

    wifi_pcie2_wait_link_and_cfg()
}

fn wifi_pcie2_trace_sequence() -> WifiPcieTraceResult {
    gpio_v2_write_bit(GPIO0, GPIO_V2_PORT_DR, PCIE_POWER_PIN, true);
    gpio_v2_write_bit(GPIO0, GPIO_V2_PORT_DDR, PCIE_POWER_PIN, true);

    set_pcie_reset_pin_level(&PCIE_RESET_PINS[2], false);
    cru_reset_assert(PCIE2_RESETS[1]);
    delay_ms(20);

    wifi_pcie2_enable_clocks();
    for reset in COMBPHY2_RESETS {
        cru_reset_deassert(*reset);
    }
    wifi_pcie2_configure_combphy();

    cru_reset_deassert(PCIE2_RESETS[1]);
    for reset in PCIE2_RESETS {
        cru_reset_deassert(*reset);
    }
    delay_ms(1);

    wifi_pcie2_configure_rc();
    write_mmio32(
        PCIE2X1_APB_BASE + PCIE_CLIENT_DBG_FIFO_MODE_CON,
        PCIE_CLIENT_DBG_EN,
    );
    set_pcie_ltssm(PCIE2_CONTROLLER, true);
    delay_ms(10);
    set_pcie_reset_pin_level(&PCIE_RESET_PINS[2], true);

    let mut samples = [0u32; 8];
    for sample in samples.iter_mut() {
        delay_ms(10);
        *sample =
            probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_DBG_FIFO_STATUS).unwrap_or(0xffff_ffff);
    }

    let init = wifi_pcie2_wait_link_and_cfg();
    let rasdes = probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_CDM_RASDES_TBA_INFO_CMN).unwrap_or(0);
    let intr_misc = probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_INTR_STATUS_MISC).unwrap_or(0);
    let intr_mask_misc = probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_INTR_MASK_MISC).unwrap_or(0);
    let dbg_mode = probe_read32(PCIE2X1_APB_BASE + PCIE_CLIENT_DBG_FIFO_MODE_CON).unwrap_or(0);
    write_mmio32(
        PCIE2X1_APB_BASE + PCIE_CLIENT_DBG_FIFO_MODE_CON,
        PCIE_CLIENT_DBG_DIS,
    );

    WifiPcieTraceResult {
        init,
        rasdes,
        intr_misc,
        intr_mask_misc,
        dbg_mode,
        samples,
    }
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

fn print_combphy_reg(uart: &mut Uart, label: &str, offset: u64) {
    print_pcie_reg(
        uart,
        label,
        COMBPHY2_BASE + offset,
        probe_read32(COMBPHY2_BASE + offset),
    );
}

fn write_cru_bit_raw_summary<W: Write>(out: &mut W, label: &str, bit: CruBit) {
    match probe_read32(bit.base + bit.offset) {
        Ok(value) => {
            let set = value & (1u32 << bit.bit) != 0;
            let _ = write!(out, " {label}={}", if set { 1 } else { 0 });
        }
        Err(_) => {
            let _ = write!(out, " {label}=fault");
        }
    }
}

fn write_pcie_reset_pin_summary<W: Write>(out: &mut W, pin: &PcieResetPin) {
    match (
        read_gpio_v2_direction(pin.bank, pin.pin),
        read_gpio_v2_level(pin.bank, pin.pin),
    ) {
        (Ok(direction), Ok(level)) => {
            let asserted = if pin.active_high { level } else { !level };
            let _ = write!(
                out,
                " reset-gpio={} dir={} level={} reset={}",
                pin.label,
                gpio_direction_name(direction),
                bit_name(level),
                if asserted { "asserted" } else { "deasserted" }
            );
        }
        _ => {
            let _ = write!(out, " reset-gpio={} status=fault", pin.label);
        }
    }
}

fn write_probe32_summary<W: Write>(out: &mut W, name: &str, addr: u64) {
    match probe_read32(addr) {
        Ok(value) => {
            let _ = write!(out, " {name}=0x{value:08x}");
        }
        Err(_) => {
            let _ = write!(out, " {name}=fault");
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
        RTL_TX_NEXT[index] = 0;
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

fn rtl_refresh_dma(index: usize) {
    let Some(controller) = PCIE_CONTROLLERS.get(index) else {
        return;
    };
    let base = controller.mem32_base;
    pcie_select_rtl_endpoint(*controller);
    pcie_map_rtl_bars(*controller);
    rtl_irq_mask_and_ack(base);
    let _ = rtl_mac_reset(base, RTL_INIT_TIMEOUT_MS);
    rtl_irq_mask_and_ack(base);
    rtl_write_mac(base, rtl_local_mac(index));
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
            | RTL_RX_ACCEPT_MY_PHYS
            | RTL_RX_ACCEPT_ALL_PHYS,
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
}

fn rtl_build_tx_test_packet(index: usize, mac: [u8; 6]) -> usize {
    let payload = b"xiaojin-os rtl tx test";
    clear_tx_buffer(index);
    unsafe {
        let buffer = &mut RTL_TX_BUFFERS.0[index];
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

fn clear_tx_buffer(index: usize) {
    unsafe {
        for byte in RTL_TX_BUFFERS.0[index].iter_mut() {
            *byte = 0;
        }
    }
}

fn rtl_send_prepared_frame(index: usize, len: usize, timeout_ms: u64) -> bool {
    let (_, completed) = rtl_send_prepared_frame_with_entry(index, len, timeout_ms);
    completed
}

fn rtl_send_prepared_frame_with_entry(index: usize, len: usize, timeout_ms: u64) -> (usize, bool) {
    if index >= RTL_DEVICE_COUNT || len == 0 || len > RTL_TX_BUFFER_SIZE {
        return (0, false);
    }

    let base = PCIE_CONTROLLERS[index].mem32_base;
    let buffer_addr = rtl_tx_buffer_addr(index);
    let entry = unsafe {
        let entry = RTL_TX_NEXT[index];
        RTL_TX_NEXT[index] = (entry + 1) % RTL_TX_DESC_COUNT;
        entry
    };
    let desc_addr = rtl_tx_desc_addr(index, entry);
    let ring_end = if entry == RTL_TX_DESC_COUNT - 1 {
        RTL_DESC_RING_END
    } else {
        0
    };
    unsafe {
        let desc = &mut RTL_TX_RINGS.0[index][entry];
        desc.addr = buffer_addr;
        desc.opts2 = 0;
        desc.opts1 = len as u32 | RTL_DESC_FIRST_FRAG | RTL_DESC_LAST_FRAG | ring_end;
        clean_dcache_range(buffer_addr, len as u64);
        clean_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        desc.opts1 |= RTL_DESC_OWN;
        clean_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
    }

    write_mmio16(base + RTL_REG_TX_POLL_8125, 1);
    mmio_write_barrier();
    let completed = rtl_wait_tx_complete(index, entry, timeout_ms);
    (entry, completed)
}

fn rtl_wait_tx_complete(index: usize, entry: usize, timeout_ms: u64) -> bool {
    let freq = read_cntfrq();
    let Some(timeout_ticks) = ms_to_ticks(timeout_ms, freq) else {
        return false;
    };
    let desc_addr = rtl_tx_desc_addr(index, entry);
    let start = read_cntpct();
    loop {
        invalidate_dcache_range(desc_addr, core::mem::size_of::<RtlDesc>() as u64);
        let opts1 = unsafe { RTL_TX_RINGS.0[index][entry].opts1 };
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

fn read_be16(bytes: &[u8], offset: usize) -> Option<u16> {
    if offset + 1 >= bytes.len() {
        return None;
    }
    Some(((bytes[offset] as u16) << 8) | bytes[offset + 1] as u16)
}

fn read_be32(bytes: &[u8], offset: usize) -> Option<u32> {
    if offset + 3 >= bytes.len() {
        return None;
    }
    Some(
        ((bytes[offset] as u32) << 24)
            | ((bytes[offset + 1] as u32) << 16)
            | ((bytes[offset + 2] as u32) << 8)
            | bytes[offset + 3] as u32,
    )
}

fn read_le16(bytes: &[u8], offset: usize) -> Option<u16> {
    if offset + 1 >= bytes.len() {
        return None;
    }
    Some((bytes[offset] as u16) | ((bytes[offset + 1] as u16) << 8))
}

fn read_le32(bytes: &[u8], offset: usize) -> Option<u32> {
    if offset + 3 >= bytes.len() {
        return None;
    }
    Some(
        (bytes[offset] as u32)
            | ((bytes[offset + 1] as u32) << 8)
            | ((bytes[offset + 2] as u32) << 16)
            | ((bytes[offset + 3] as u32) << 24),
    )
}

fn write_le16(bytes: &mut [u8], offset: usize, value: u16) {
    if offset + 1 >= bytes.len() {
        return;
    }
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
}

fn write_le32(bytes: &mut [u8], offset: usize, value: u32) {
    if offset + 3 >= bytes.len() {
        return;
    }
    bytes[offset] = value as u8;
    bytes[offset + 1] = (value >> 8) as u8;
    bytes[offset + 2] = (value >> 16) as u8;
    bytes[offset + 3] = (value >> 24) as u8;
}

fn write_be16(bytes: &mut [u8], offset: usize, value: u16) {
    if offset + 1 >= bytes.len() {
        return;
    }
    bytes[offset] = (value >> 8) as u8;
    bytes[offset + 1] = value as u8;
}

fn write_be32(bytes: &mut [u8], offset: usize, value: u32) {
    if offset + 3 >= bytes.len() {
        return;
    }
    bytes[offset] = (value >> 24) as u8;
    bytes[offset + 1] = (value >> 16) as u8;
    bytes[offset + 2] = (value >> 8) as u8;
    bytes[offset + 3] = value as u8;
}

fn ipv4_header_checksum(header: &[u8]) -> u16 {
    let mut sum = 0u32;
    let mut offset = 0usize;
    while offset + 1 < header.len() {
        sum = sum.wrapping_add(read_be16(header, offset).unwrap_or(0) as u32);
        offset += 2;
    }
    if offset < header.len() {
        sum = sum.wrapping_add((header[offset] as u32) << 8);
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

fn ones_complement_sum_bytes(mut sum: u32, bytes: &[u8]) -> u32 {
    let mut offset = 0usize;
    while offset + 1 < bytes.len() {
        sum = sum.wrapping_add(read_be16(bytes, offset).unwrap_or(0) as u32);
        offset += 2;
    }
    if offset < bytes.len() {
        sum = sum.wrapping_add((bytes[offset] as u32) << 8);
    }
    sum
}

fn ones_complement_finish(mut sum: u32) -> u16 {
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

fn tcp_checksum(src_ip: [u8; 4], dst_ip: [u8; 4], tcp: &[u8]) -> u16 {
    let mut sum = 0u32;
    sum = ones_complement_sum_bytes(sum, &src_ip);
    sum = ones_complement_sum_bytes(sum, &dst_ip);
    sum = sum.wrapping_add(IPV4_PROTO_TCP as u32);
    sum = sum.wrapping_add(tcp.len() as u32);
    sum = ones_complement_sum_bytes(sum, tcp);
    ones_complement_finish(sum)
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
    probe_flag_set(addr_of_mut!(PROBE_FAULTED), false);
    probe_flag_set(addr_of_mut!(PROBE_ACTIVE), true);
    unsafe {
        asm!(
            "ldr {value:w}, [{addr}]",
            addr = in(reg) addr as usize,
            value = inout(reg) value,
            options(nostack, preserves_flags)
        );
    }
    probe_flag_set(addr_of_mut!(PROBE_ACTIVE), false);

    if probe_flag_take(addr_of_mut!(PROBE_FAULTED)) {
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
    psci_system_reset()
}

fn psci_system_reset() -> ! {
    unsafe {
        asm!(
            "smc #0",
            in("x0") PSCI_SYSTEM_RESET,
            out("x1") _,
            out("x2") _,
            out("x3") _,
            out("x4") _,
            out("x5") _,
            out("x6") _,
            out("x7") _,
        );
    }
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

fn parse_u16(token: &str) -> Option<u16> {
    let value = parse_u64(token)?;
    if value == 0 || value > u16::MAX as u64 {
        None
    } else {
        Some(value as u16)
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

fn ascii_eq(left: &[u8], right: &[u8]) -> bool {
    left == right
}

fn trim_ascii_whitespace(value: &[u8]) -> &[u8] {
    let mut start = 0usize;
    let mut end = value.len();
    while start < end && value[start].is_ascii_whitespace() {
        start += 1;
    }
    while end > start && value[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    &value[start..end]
}

fn is_sha256_hex(value: &[u8]) -> bool {
    value.len() == 64 && value.iter().all(|byte| byte.is_ascii_hexdigit())
}

fn hex_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn sha256_hex_matches(digest: &[u8; 32], expected: &[u8]) -> bool {
    if expected.len() != 64 {
        return false;
    }
    for (index, byte) in digest.iter().enumerate() {
        let Some(high) = hex_nibble(expected[index * 2]) else {
            return false;
        };
        let Some(low) = hex_nibble(expected[index * 2 + 1]) else {
            return false;
        };
        if *byte != (high << 4) | low {
            return false;
        }
    }
    true
}

fn write_sha256_hex<W: Write>(out: &mut W, digest: &[u8; 32]) {
    for byte in digest {
        let _ = write!(out, "{byte:02x}");
    }
}

fn device_identity_digest() -> [u8; 32] {
    let mut canonical = NetReplyBuf::new();
    let _ = write!(
        canonical,
        "{XIAOJIN_IDENTITY_FORMAT}\ndevice_id={XIAOJIN_DEVICE_ID}\nboard_id={XIAOJIN_BOARD_ID}\nsoc={XIAOJIN_SOC_ID}\narch={XIAOJIN_ARCH_ID}\n"
    );
    sha256_digest(canonical.as_slice())
}

fn net_reply_push_checked(buffer: &mut NetReplyBuf, bytes: &[u8]) -> bool {
    let before = buffer.as_slice().len();
    buffer.push_slice(bytes);
    buffer.as_slice().len() == before + bytes.len()
}

fn maint_command_digest(command_id: &[u8], kind: &[u8], payload_sha256: &[u8]) -> Option<[u8; 32]> {
    let command_id = trim_ascii_whitespace(command_id);
    let kind = trim_ascii_whitespace(kind);
    let payload_sha256 = trim_ascii_whitespace(payload_sha256);
    let mut canonical = NetReplyBuf::new();
    for part in [
        b"xiaojin-maint-command-v1\ncommand_id=".as_slice(),
        command_id,
        b"\nkind=".as_slice(),
        kind,
        b"\npayload_sha256=".as_slice(),
        payload_sha256,
        b"\n".as_slice(),
    ] {
        if !net_reply_push_checked(&mut canonical, part) {
            return None;
        }
    }
    Some(sha256_digest(canonical.as_slice()))
}

enum MaintHmacCheckInput {
    NoPending,
    DigestNotOk,
    BadKey,
    BadAlgorithm,
    Calculated([u8; 32]),
}

#[derive(Clone, Copy)]
enum MaintHmacKey<'a> {
    None,
    Hex(&'a str),
    Raw(&'a [u8]),
}

impl<'a> MaintHmacKey<'a> {
    fn is_some(self) -> bool {
        !matches!(self, Self::None)
    }
}

fn maint_hmac_signature_digest(
    command_sha: Option<&[u8]>,
    signature_algorithm: Option<&[u8]>,
    key: MaintHmacKey<'_>,
) -> MaintHmacCheckInput {
    let Some(command_sha) = command_sha.map(trim_ascii_whitespace) else {
        return MaintHmacCheckInput::DigestNotOk;
    };
    if ascii_eq(command_sha, b"none") {
        return MaintHmacCheckInput::NoPending;
    }
    if !is_sha256_hex(command_sha) {
        return MaintHmacCheckInput::DigestNotOk;
    }
    let Some(signature_algorithm) = signature_algorithm.map(trim_ascii_whitespace) else {
        return MaintHmacCheckInput::BadAlgorithm;
    };
    if !ascii_eq(signature_algorithm, b"xiaojin-hmac-sha256-v1") {
        return MaintHmacCheckInput::BadAlgorithm;
    }

    let mut decoded_key = [0u8; 128];
    let key_bytes = match key {
        MaintHmacKey::None => return MaintHmacCheckInput::BadKey,
        MaintHmacKey::Hex(key_hex) => {
            let Some(key_len) = decode_hex_to_buf(key_hex.as_bytes(), &mut decoded_key) else {
                return MaintHmacCheckInput::BadKey;
            };
            if key_len == 0 {
                return MaintHmacCheckInput::BadKey;
            }
            &decoded_key[..key_len]
        }
        MaintHmacKey::Raw(key) => {
            if key.is_empty() {
                return MaintHmacCheckInput::BadKey;
            }
            key
        }
    };

    let signature_digest = hmac_sha256_digest(key_bytes, command_sha);
    let mut signature_hex = [0u8; 64];
    write_digest_hex_bytes(&signature_digest, &mut signature_hex);
    MaintHmacCheckInput::Calculated(sha256_digest(&signature_hex))
}

fn decode_hex_to_buf(input: &[u8], out: &mut [u8]) -> Option<usize> {
    let input = trim_ascii_whitespace(input);
    if input.len() % 2 != 0 || input.len() / 2 > out.len() {
        return None;
    }
    let mut index = 0usize;
    while index < input.len() {
        let high = hex_nibble(input[index])?;
        let low = hex_nibble(input[index + 1])?;
        out[index / 2] = (high << 4) | low;
        index += 2;
    }
    Some(input.len() / 2)
}

fn write_digest_hex_bytes(digest: &[u8; 32], out: &mut [u8; 64]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (index, byte) in digest.iter().enumerate() {
        out[index * 2] = HEX[(byte >> 4) as usize];
        out[index * 2 + 1] = HEX[(byte & 0x0f) as usize];
    }
}

fn hmac_sha256_digest(key: &[u8], message: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    if key.len() > key_block.len() {
        let digest = sha256_digest(key);
        key_block[..digest.len()].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }

    let mut inner_pad = [0x36u8; 64];
    let mut outer_pad = [0x5cu8; 64];
    let mut index = 0usize;
    while index < 64 {
        inner_pad[index] ^= key_block[index];
        outer_pad[index] ^= key_block[index];
        index += 1;
    }

    let mut inner_message = [0u8; 128];
    inner_message[..64].copy_from_slice(&inner_pad);
    let inner_len = 64 + message.len().min(64);
    if message.len() <= 64 {
        inner_message[64..inner_len].copy_from_slice(message);
        let inner_digest = sha256_digest(&inner_message[..inner_len]);
        let mut outer_message = [0u8; 96];
        outer_message[..64].copy_from_slice(&outer_pad);
        outer_message[64..].copy_from_slice(&inner_digest);
        return sha256_digest(&outer_message);
    }

    hmac_sha256_digest_slow(&key_block, &inner_pad, &outer_pad, message)
}

fn hmac_sha256_digest_slow(
    _key_block: &[u8; 64],
    inner_pad: &[u8; 64],
    outer_pad: &[u8; 64],
    message: &[u8],
) -> [u8; 32] {
    let mut scratch = [0u8; 256];
    if message.len() + 64 <= scratch.len() {
        scratch[..64].copy_from_slice(inner_pad);
        scratch[64..64 + message.len()].copy_from_slice(message);
        let inner_digest = sha256_digest(&scratch[..64 + message.len()]);
        let mut outer_message = [0u8; 96];
        outer_message[..64].copy_from_slice(outer_pad);
        outer_message[64..].copy_from_slice(&inner_digest);
        return sha256_digest(&outer_message);
    }

    let mut fallback = NetReplyBuf::new();
    fallback.push_slice(inner_pad);
    fallback.push_slice(message);
    let inner_digest = sha256_digest(fallback.as_slice());
    let mut outer_message = [0u8; 96];
    outer_message[..64].copy_from_slice(outer_pad);
    outer_message[64..].copy_from_slice(&inner_digest);
    sha256_digest(&outer_message)
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

fn crc32_memory(addr: u64, len: u64) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for offset in 0..len {
        let byte = unsafe { read_volatile((addr + offset) as usize as *const u8) };
        crc = crc32_update(crc, byte);
    }
    !crc
}

const SHA256_K: [u32; 64] = [
    0x428a_2f98,
    0x7137_4491,
    0xb5c0_fbcf,
    0xe9b5_dba5,
    0x3956_c25b,
    0x59f1_11f1,
    0x923f_82a4,
    0xab1c_5ed5,
    0xd807_aa98,
    0x1283_5b01,
    0x2431_85be,
    0x550c_7dc3,
    0x72be_5d74,
    0x80de_b1fe,
    0x9bdc_06a7,
    0xc19b_f174,
    0xe49b_69c1,
    0xefbe_4786,
    0x0fc1_9dc6,
    0x240c_a1cc,
    0x2de9_2c6f,
    0x4a74_84aa,
    0x5cb0_a9dc,
    0x76f9_88da,
    0x983e_5152,
    0xa831_c66d,
    0xb003_27c8,
    0xbf59_7fc7,
    0xc6e0_0bf3,
    0xd5a7_9147,
    0x06ca_6351,
    0x1429_2967,
    0x27b7_0a85,
    0x2e1b_2138,
    0x4d2c_6dfc,
    0x5338_0d13,
    0x650a_7354,
    0x766a_0abb,
    0x81c2_c92e,
    0x9272_2c85,
    0xa2bf_e8a1,
    0xa81a_664b,
    0xc24b_8b70,
    0xc76c_51a3,
    0xd192_e819,
    0xd699_0624,
    0xf40e_3585,
    0x106a_a070,
    0x19a4_c116,
    0x1e37_6c08,
    0x2748_774c,
    0x34b0_bcb5,
    0x391c_0cb3,
    0x4ed8_aa4a,
    0x5b9c_ca4f,
    0x682e_6ff3,
    0x748f_82ee,
    0x78a5_636f,
    0x84c8_7814,
    0x8cc7_0208,
    0x90be_fffa,
    0xa450_6ceb,
    0xbef9_a3f7,
    0xc671_78f2,
];

fn sha256_digest(input: &[u8]) -> [u8; 32] {
    let mut state = [
        0x6a09_e667u32,
        0xbb67_ae85,
        0x3c6e_f372,
        0xa54f_f53a,
        0x510e_527f,
        0x9b05_688c,
        0x1f83_d9ab,
        0x5be0_cd19,
    ];

    let mut offset = 0usize;
    while input.len().saturating_sub(offset) >= 64 {
        sha256_compress(&mut state, &input[offset..offset + 64]);
        offset += 64;
    }

    let rem = &input[offset..];
    let mut block = [0u8; 64];
    block[..rem.len()].copy_from_slice(rem);
    block[rem.len()] = 0x80;
    if rem.len() >= 56 {
        sha256_compress(&mut state, &block);
        block = [0u8; 64];
    }
    let bit_len = (input.len() as u64).wrapping_mul(8);
    block[56..64].copy_from_slice(&bit_len.to_be_bytes());
    sha256_compress(&mut state, &block);

    let mut out = [0u8; 32];
    for (index, word) in state.iter().enumerate() {
        out[index * 4..index * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

fn sha256_compress(state: &mut [u32; 8], block: &[u8]) {
    let mut w = [0u32; 64];
    for index in 0..16 {
        let start = index * 4;
        w[index] = u32::from_be_bytes([
            block[start],
            block[start + 1],
            block[start + 2],
            block[start + 3],
        ]);
    }
    for index in 16..64 {
        let s0 =
            w[index - 15].rotate_right(7) ^ w[index - 15].rotate_right(18) ^ (w[index - 15] >> 3);
        let s1 =
            w[index - 2].rotate_right(17) ^ w[index - 2].rotate_right(19) ^ (w[index - 2] >> 10);
        w[index] = w[index - 16]
            .wrapping_add(s0)
            .wrapping_add(w[index - 7])
            .wrapping_add(s1);
    }

    let mut a = state[0];
    let mut b = state[1];
    let mut c = state[2];
    let mut d = state[3];
    let mut e = state[4];
    let mut f = state[5];
    let mut g = state[6];
    let mut h = state[7];

    for index in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = h
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(SHA256_K[index])
            .wrapping_add(w[index]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        h = g;
        g = f;
        f = e;
        e = d.wrapping_add(temp1);
        d = c;
        c = b;
        b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a);
    state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c);
    state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e);
    state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g);
    state[7] = state[7].wrapping_add(h);
}

fn busy_wait_ms(ms: u64) {
    let freq = read_cntfrq();
    let Some(ticks) = ms_to_ticks(ms, freq) else {
        return;
    };
    let start = read_cntpct();
    while read_cntpct().wrapping_sub(start) < ticks {
        core::hint::spin_loop();
    }
}

fn set_netload_boot_flag() {
    unsafe {
        write_volatile(
            NETLOAD_BOOT_MAGIC_ADDR as usize as *mut u64,
            NETLOAD_BOOT_MAGIC,
        );
    }
}

fn take_netload_boot_flag() -> bool {
    let value = unsafe { read_volatile(NETLOAD_BOOT_MAGIC_ADDR as usize as *const u64) };
    if value == NETLOAD_BOOT_MAGIC {
        unsafe {
            write_volatile(NETLOAD_BOOT_MAGIC_ADDR as usize as *mut u64, 0);
        }
        true
    } else {
        false
    }
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

    if ec == ESR_EC_DATA_ABORT_CURRENT && probe_flag_get(addr_of!(PROBE_ACTIVE)) {
        probe_flag_set(addr_of_mut!(PROBE_FAULTED), true);
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
