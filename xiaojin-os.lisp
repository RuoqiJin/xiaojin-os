(xiaojin-os
  :schema "xiaojin-os.ssot.v1"
  :status "thin-authoring-ssot"
  :runtime-rule "Kernel hot paths do not parse Lisp; generated Rust constants come later."

  (board nanopi-r5c
    :soc rk3568
    :arch aarch64
    :uart2-base 0xfe660000
    :uart-baud 1500000
    :load-addr 0x04080000
    :mmc-dev 1
    :mmc-part 1
    :image-name "xiaojin-os.bin")

  (memory-map
    (dram-bank bank0 :start 0x00200000 :end 0x08400000)
    (dram-bank bank1 :start 0x09400000 :end 0xf0000000)
    (region kernel-load :start 0x04080000)
    (region xload-slot0 :start 0x04080000 :end 0x04880000)
    (region xload-slot1 :start 0x05080000 :end 0x05880000)
    (region xload-trampoline :start 0x0587f000 :end 0x05880000)
    (mmio-region uart2
      :start 0xfe660000
      :end 0xfe660100
      :description "RK3568 debug UART2"))

  (pcie-map
    :raw-read-rule "PCIe DBI/config/outbound windows must not be read before controller init."
    (controller pcie3x1
      :domain 1
      :bus-range [0x10 0x1f]
      :num-lanes 1
      :reset-gpio "gpio0 A0 active-high"
      :dbi [0x00000003c0400000 0x00000003c0800000]
      :apb [0xfe270000 0xfe280000]
      :config [0xf2000000 0xf2100000]
      :io [0xf2100000 0xf2200000]
      :mem32 [0xf2200000 0xf4000000]
      :mem64 [0x0000000340000000 0x0000000380000000])
    (controller pcie3x2
      :domain 2
      :bus-range [0x20 0x2f]
      :num-lanes 1
      :reset-gpio "gpio0 B6 active-high"
      :dbi [0x00000003c0800000 0x00000003c0c00000]
      :apb [0xfe280000 0xfe290000]
      :config [0xf0000000 0xf0100000]
      :io [0xf0100000 0xf0200000]
      :mem32 [0xf0200000 0xf2000000]
      :mem64 [0x0000000380000000 0x00000003c0000000])
    (controller pcie2x1
      :reset-gpio "gpio3 C1 active-high"
      :role "enabled on R5C; likely not RJ45 path")
    (phy pcie30phy
      :base 0xfe8c0000
      :end 0xfe8e0000
      :grf-base 0xfdcb8000
      :grf-end 0xfdcc8000
      :data-lanes [1 2]))

  (network-bringup
    :target "wired-loader"
    :strategy "PCIe first, RTL8125 second, UDP xload third"
    :rescue-channel "UART xload"
    (stage facts
      :status implemented
      :commands [net facts pcie facts mmio read32]
      :purpose "Expose R5C network and PCIe facts from the board profile.")
    (stage pcie-enumeration
      :status next
      :purpose "Reset, power, PHY, link training, and config-space enumeration.")
    (stage rtl8125-driver
      :status planned
      :purpose "Map BAR, initialize RX/TX rings, and poll packets.")
    (stage udp-xload
      :status planned
      :purpose "Receive Xiaojin OS images over Ethernet from xjctl."))

  (kernel-shell
    :prompt "xj> "
    (command help
      :purpose "List shell commands.")
    (command caps
      :purpose "Print machine-readable feature flags for xjctl.")
    (command about
      :purpose "Show kernel and board identity.")
    (command mem
      :purpose "Print board memory facts and linker-owned regions.")
    (command regs
      :purpose "Print selected AArch64 system registers.")
    (command time
      :purpose "Print generic timer registers and uptime.")
    (command uptime
      :purpose "Print elapsed kernel time from boot CNTPCT.")
    (command sleep
      :args [ms]
      :max-ms 10000
      :purpose "Busy-wait using the ARM generic timer.")
    (command bench
      :args [iters]
      :default-iters 1000000
      :max-iters 20000000
      :purpose "Run timer-backed CPU microbenchmarks.")
    (command net
      :subcommands [facts]
      :purpose "Show wired-network bring-up facts and staging.")
    (command pcie
      :subcommands [facts]
      :purpose "Show R5C PCIe controller facts needed for Ethernet bring-up.")
    (command xload
      :args [addr-or-auto size crc32 go]
      :default-target auto
      :max-image-bytes 4194304
      :reload-mode "copy-down-trampoline"
      :purpose "Receive a raw serial-loaded image, validate CRC32, and optionally reload to the static link address.")
    (command hexdump
      :alias hd
      :args [addr len]
      :purpose "Dump known DRAM bytes with a conservative max length.")
    (command peek32
      :args [addr]
      :purpose "Read one safe 32-bit word from known DRAM or whitelisted MMIO.")
    (command mmio
      :subcommands [list read32]
      :purpose "List and read whitelisted MMIO regions.")
    (command last-exception
      :alias lastx
      :purpose "Print the latest captured exception snapshot.")
    (command fault
      :purpose "Trigger BRK #0x123 to verify the EL2 exception vector path.")
    (command abort
      :args [confirm]
      :purpose "Trigger a fatal data abort test; requires explicit confirm.")
    (command reboot
      :purpose "Request PSCI SYSTEM_RESET.")))
