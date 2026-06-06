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
    (mmio-region uart2
      :start 0xfe660000
      :end 0xfe660100
      :description "RK3568 debug UART2"))

  (kernel-shell
    :prompt "xj> "
    (command help
      :purpose "List shell commands.")
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
