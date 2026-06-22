(xiaojin-os
  :schema "xiaojin-os.ssot.v1"
  :status "thin-authoring-ssot"
  :runtime-rule "Kernel hot paths do not parse Lisp; generated Rust constants come later."

  (board nanopi-r5c
    :soc rk3568
    :arch aarch64
    :uart2-base 0xfe660000
    :uart-baud 1500000
    :load-addr 0x02080000
    :mmc-dev 1
    :mmc-part 1
    :image-name "xiaojin-os.bin"
    (wireless-module wlan0
      :chip rtl8822ce
      :form-factor "M.2 E-Key"
      :wifi-bus "candidate-pcie2x1"
      :bt-bus "sideband-unverified"
      :station-mode planned
      :ap-mode planned))

  (target x86_64-amd8845hs-dev
    :arch x86_64
    :boot uefi
    :status planned
    :first-console "uefi-serial-or-framebuffer"
    :timer "tsc-hpet-apic"
    :interrupt-controller "apic-or-x2apic"
    :storage "nvme-sata-usb-block"
    :network "pcie-nic-driver-selected-by-hardware"
    :shared-control-protocols [xjnl xjq1 xjev xjdh ota-ab]
    :purpose "Prepare Xiaojin OS for AMD 8845HS-class PCs without binding the kernel architecture to the RK3568/R5C boot chain.")

  (memory-map
    (dram-bank bank0 :start 0x00200000 :end 0x08400000)
    (dram-bank bank1 :start 0x09400000 :end 0xf0000000)
    (region kernel-load :start 0x02080000)
    (region xload-slot0 :start 0x02080000 :end 0x02880000)
    (region xload-slot1 :start 0x05080000 :end 0x05880000)
    (region xload-trampoline :start 0x0587f000 :end 0x05880000)
    (region wifi-firmware-staging :start 0x06080000 :end 0x06480000)
    (mmio-region uart2
      :start 0xfe660000
      :end 0xfe660100
      :description "RK3568 debug UART2")
    (mmio-region gpio0
      :start 0xfdd60000
      :end 0xfdd60100
      :description "RK3568 GPIO bank 0")
    (mmio-region gpio3
      :start 0xfe760000
      :end 0xfe760100
      :description "RK3568 GPIO bank 3"))

  (pcie-map
    :raw-read-rule "PCIe DBI/config/outbound windows must not be read before controller init."
    (controller pcie3x1
      :domain 1
      :bus-range [0x10 0x1f]
      :num-lanes 1
      :reset-gpio "gpio0 A0 release-high-observed"
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
      :reset-gpio "gpio0 B6 release-high-observed"
      :dbi [0x00000003c0800000 0x00000003c0c00000]
      :apb [0xfe280000 0xfe290000]
      :config [0xf0000000 0xf0100000]
      :io [0xf0100000 0xf0200000]
      :mem32 [0xf0200000 0xf2000000]
      :mem64 [0x0000000380000000 0x00000003c0000000])
    (controller pcie2x1
      :domain 0
      :bus-range [0x00 0x0f]
      :num-lanes 1
      :reset-gpio "gpio3 C1 active-high"
      :dbi [0x00000003c0000000 0x00000003c0400000]
      :apb [0xfe260000 0xfe270000]
      :config [0xf4000000 0xf4100000]
      :io [0xf4100000 0xf4200000]
      :mem32 [0xf4200000 0xf6000000]
      :mem64 [0x0000000300000000 0x0000000340000000]
      :role "candidate R5C M.2 E-Key WiFi path; keep out of wired RTL8125 loops until separately probed")
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
      :status implemented
      :implemented [reset-gpio-read reset-gpio-deassert-confirm phy-grf-status phy-grf-init cru-clock-reset-status cru-clock-reset-enable apb-ltssm-status rc-mode-init ltssm-control link-start cfg-probe iatu-probe cfg-scan bar-scan]
      :purpose "Reset, power, PHY, link training, and config-space enumeration.")
    (stage rtl8125-driver
      :status in-progress
      :implemented [bar-map rtl-mmio-probe rtl-facts rx-tx-ring-init tx-ring-cursor promiscuous-rx raw-tx-test rx-poll arp-announcement ipv4-udp-hello ipv4-udp-ping-handler macos-bpf-raw-sender]
      :observed [pcie3x2-raw-tx-descriptor-complete macos-receives-xjhello-via-nc macos-raw-xjping-receives-xjpong]
      :next [polling-stability interrupt-mode]
      :purpose "Map BAR, initialize RX/TX rings, poll packets, send board-to-host UDP hello, and answer ARP/UDP smoke-test probes.")
    (stage udp-xload
      :status implemented
      :implemented [xjnl-begin xjnl-data-ack xjnl-commit-crc32 copy-down-reload net-listen-autostart routed-udp-host-transport runtime-ip-config udp-ip-config]
      :observed [sudo-xjctl-net-dev-ready-data-ok-reload xjctl-net-dev-transport-udp-reload]
      :purpose "Receive Xiaojin OS images over Ethernet from xjctl and reload through the copy-down trampoline.")
    (stage dhcp-client
      :status implemented-manual
      :implemented [dhcp-discover dhcp-offer-parse dhcp-request dhcp-ack-parse runtime-lease-apply udp-xjdh-control]
      :next [cold-boot-dhcp-fallback]
      :purpose "Learn runtime IP, subnet mask, gateway, and DNS facts from a LAN DHCP server without making DHCP mandatory for cold boot; allow xjctl to trigger DHCP without UART.")
    (stage dns-client
      :status implemented-probe
      :implemented [runtime-dns-server route-next-hop arp-resolve dns-a-query dns-a-response-parse udp-xjq-control]
      :next [tcp-stream-state tls-client]
      :purpose "Resolve one A record through the runtime DHCP/static DNS and route state.")
    (stage tcp-client
      :status implemented-probe
      :implemented [route-next-hop arp-resolve tcp-syn tcp-synack-parse tcp-final-ack tcp-reset-close udp-xjq-control]
      :next [tcp-stream-state tls-client]
      :purpose "Prove outbound TCP reachability with a SYN/SYN-ACK/ACK connect probe before building HTTP/TLS.")
    (stage http-client
      :status implemented-probe
      :implemented [http10-get request-build tcp-psh-ack sequential-payload-ack status-code-parse response-sample-cap udp-xjq-control]
      :next [tcp-reassembly chunked-body host-auth tls-client]
      :purpose "Prove Xiaojin OS can actively fetch an HTTP endpoint over its own NIC before implementing the cloud maintenance poller.")
    (stage tls-first-flight
      :status implemented-probe
      :implemented [clienthello-build sni-extension sha256-cipher-suite-filter supported-versions supported-groups signature-algorithms x25519-keyshare serverhello-parse transcript-hash hkdf-handshake-traffic-secret-fingerprint record-key-iv-fingerprint chacha20poly1305-first-record-decrypt tls-record-parse first-certificate-sha256 first-certificate-sha256-pin udp-xjq-control]
      :next [certificate-chain-validation encrypted-handshake-stream record-decrypt-complete https-client]
      :purpose "Prove Xiaojin OS can move from DNS/TCP to the TLS key-agreement, record-key schedule, and first encrypted handshake record layer without claiming full TLS yet.")
    (stage maintainer-poll-client
      :status implemented-narrow-signed-exec
      :implemented [maint-poll maint-status maint-audit maint-verify maint-verify-hmac maint-local-hmac-key maint-pull-exec maint-loop-bounded maint-loop-xjq-compact maint-config-persistent maint-config-run maint-config-tls-pin-persistent maint-cold-boot-tick maint-ack-intent public-poll-text-parse command-digest-check hmac-command-signature-check xiaojindb-audit-event http-post-event]
      :next [scheduled-cold-boot-maint-loop signed-artifact-fetch https-poll tls-client]
      :purpose "Let Xiaojin OS poll the GCP maintainer, verify a narrow signed public xj_query command, execute it locally, write XiaojinDB audit events, persist a bounded maintainer endpoint config, and run bounded device-side loops from UART, compact Ethernet XJQ control, saved config, or one listener-start maintenance tick before scheduled autonomous maintenance is enabled.")
    (stage udp-query-control
      :status implemented-readonly
      :implemented [xjq-caps-query xjq-net-ips-query xjq-net-dns-query xjq-net-tcp-query xjq-net-http-query xjq-maint-status-query rtl-dma-rx-watchdog xjq-wifi-facts-query xjq-wifi-status-query xjq-sd-facts-query xjq-sd-regs-query xjq-sd-read0-query xjq-sd-bpb-query xjq-sd-root-query xjq-sd-slots-query xjq-db-page-store-query xjq-db-table-query xjq-db-btree-query xjq-db-event-log-query]
      :next [ab-bootstate-writer extlinux-default-switch rollback-confirmation]
      :purpose "Let xjctl ask short board-control questions over Ethernet, so storage/OTA/DB bring-up no longer requires the UART shell for every probe.")
    (stage net-persist
      :status implemented-stage-only
      :implemented [xjps-last-net-image-state sd-cmd24-single-block-write fat32-slot-chain-follow inactive-slot-b-write readback-crc-verify xjctl-net-dev-persist]
      :not-yet [bootstate-write extlinux-default-switch automatic-confirm rollback]
      :purpose "Persist the last uploaded image into a selected preallocated A/B slot over Ethernet, without changing the boot target yet."))

  (wifi-bringup
    :target "station-and-ap"
    :strategy "Keep wired net-dev/net-ota as rescue path while adding RTL8822CE facts, PCIe2 enumeration, firmware loading, station mode, then AP mode."
    :rescue-channel "wired RTL8125 UDP loader"
    (stage facts
      :status implemented
      :commands [wifi facts wifi status wifi roadmap wifi pcie wifi cru wifi apb wifi phy wifi power wifi firmware]
      :queries [xjq-wifi-facts xjq-wifi-status xjq-wifi-pcie xjq-wifi-cru xjq-wifi-apb xjq-wifi-phy xjq-wifi-power xjq-wifi-firmware]
      :purpose "Expose WiFi hardware assumptions and current driver state over serial and Ethernet before touching WiFi registers.")
    (stage pcie2x1-cru-status
      :status implemented
      :commands [wifi cru]
      :queries [xjq-wifi-cru]
      :safety "Read-only CRU/GPIO status; no pcie2 clock, reset, DBI, or config writes.")
    (stage pcie2x1-apb-status
      :status implemented
      :commands [wifi apb]
      :queries [xjq-wifi-apb]
      :safety "Read-only controller-local APB status; no DBI/config/window reads.")
    (stage pcie2x1-combphy-status
      :status implemented
      :commands [wifi phy]
      :queries [xjq-wifi-phy]
      :safety "Read-only RK3568 combphy2, pipe-grf, pipe-phy-grf, and reset status.")
    (stage pcie2-power-status
      :status implemented
      :commands [wifi power]
      :queries [xjq-wifi-power]
      :safety "Read-only vcc3v3_pcie regulator GPIO status; init path only drives it active-high enabled.")
    (stage pcie2x1-enumeration
      :status in-progress
      :commands [wifi pcie-init wifi pcie-init-dt]
      :queries [xjq-wifi-pcie-init xjq-wifi-pcie-init-dt]
      :requires [safe-reset-gpio-pc1 pcie2-clock-map pcie2-combphy-map pcie2-config-window rtl8822ce-vendor-id]
      :safety "Requires explicit confirm; writes only PCIe2/combphy2 WiFi path while wired RTL8125 OTA remains active."
      :purpose "Enumerate the RTL8822CE path without changing the proven wired PCIe3 controllers.")
    (stage firmware-sd-probe
      :status implemented-readonly
      :commands [wifi firmware-sd wifi firmware-crc]
      :queries [xjq-wifi-firmware-sd xjq-wifi-firmware-crc]
      :requires [fat32-root-reader realtek-firmware-catalog]
      :implemented [fat32-metadata-presence fat-chain-byte-read crc32-verification]
      :safety "Reads SD FAT32 metadata and staged firmware bytes only; does not load firmware or write WiFi MMIO."
      :purpose "Verify that Realtek firmware blobs are staged and readable from the Xiaojin OS boot SD before implementing the firmware loader.")
    (stage firmware-ram-staging
      :status implemented-no-mmio
      :commands [wifi firmware-load]
      :queries [xjq-wifi-firmware-load]
      :requires [firmware-sd-probe wifi-firmware-staging]
      :implemented [fat-chain-copy-to-dram ram-crc32-verify safe-range-check]
      :safety "Copies one staged firmware blob into reserved DRAM only; no WiFi BAR/MMIO/register writes."
      :purpose "Make the Realtek firmware bytes available in RAM for the next RTL8822CE loader step.")
    (stage wifi-mmio-snapshot
      :status implemented-readonly-after-bar-map
      :commands [wifi mmio-snapshot wifi mmio-fingerprint wifi driver-gate wifi rtw88-profile]
      :queries [xjq-wifi-mmio-snapshot xjq-wifi-mmio-fingerprint xjq-wifi-driver-gate xjq-wifi-rtw88-profile]
      :requires [pcie2x1-enumeration firmware-ram-staging]
      :implemented [pcie2-bar2-map pci-command-memory-master fixed-mmio-read-snapshot safe-4k-mmio-fingerprint driver-ready-gate rtw88-readonly-register-profile]
      :safety "Writes only PCIe config/iATU state for the WiFi endpoint BAR2 mapping; reads WiFi MMIO registers without writing them."
      :purpose "Collect first BAR2 register evidence, a stable first-4KiB MMIO fingerprint, a machine-readable driver readiness gate, and Linux rtw88-aligned read-only register evidence needed before implementing the Realtek firmware/register loader.")
    (stage rtl8822ce-driver
      :status planned
      :requires [wifi-mmio-snapshot firmware-loader mmio-map dma-rings interrupts-or-polling regulatory-domain]
      :purpose "Bring up the WiFi MAC/PHY and enough packet IO to support 802.11 station and AP work.")
    (stage station-mode
      :status planned
      :requires [scan auth assoc wpa2-psk dhcp-client default-route dns-client]
      :purpose "Allow R5C to join an external WiFi network and keep cloud maintenance reachable without wired Ethernet.")
    (stage ap-mode
      :status planned
      :requires [softap beacon probe-response auth assoc wpa-authenticator dhcp-server bridge-or-nat]
      :purpose "Allow R5C to act as a WiFi access point for local control and offline operation."))

  (maintenance-plane
    :local-priority true
    :local-protocols [xjnl xjq1 xjev xjdh ota-ab]
    :cloud-runtime gcp
    :cloud-role "outbound device command broker and artifact inventory"
    :deploy-center-surfaces [monitor-center resource-center placement-leases]
    :secret-rule "Device credentials live in Secret Store or device-provisioned keys, never in board profiles or tracked docs."
    (stage identity-and-access
      :status planned
      :requires [persistent-keystore cryptographic-rng password-or-public-key-auth role-policy audit-log secure-clock]
      :next [operator-device-provisioning signed-command-authz native-secure-admin-channel ssh-compatible-server]
      :purpose "Give Xiaojin OS real user/account control before accepting remote administrative commands: identities, roles, credentials, session authorization, and durable audit events.")
    (stage signed-command-authz
      :status planned
      :requires [identity-and-access maintainer-poll-client command-digest-ed25519-or-equivalent xiaojindb-audit-log replay-window]
      :purpose "Verify GCP-maintainer commands against a canonical command digest, operator identity, role policy, and replay window before any board-side execution.")
    (stage xiaojindb-audit-log
      :status planned
      :requires [xiaojindb-event-log identity-and-access monotonic-sequence]
      :purpose "Record every accepted, rejected, failed, and rolled-back remote administrative action in XiaojinDB.")
    (stage remote-admin-shell
      :status planned
      :requires [identity-and-access signed-command-authz tcp-stream-state tls-or-ssh-crypto command-authorizer pty-or-command-session xiaojindb-audit-log]
      :compatibility-targets [openssh-client native-xjctl-secure-shell]
      :purpose "Provide an SSH-like or SSH-compatible secure management surface for operators, while keeping the local LAN hot-update and recovery path available.")
    (stage local-lan-hot-update
      :status implemented
      :tools [xjctl-net-dev xjctl-net-ota xjctl-net-command xjctl-net-event xjctl-net-db-smoke]
      :purpose "Keep same-LAN development and recovery independent from cloud reachability.")
    (stage gcp-maintenance-service
      :status implemented-host-bridge
      :apis [device-hello device-list command-poll command-enqueue command-ack artifact-catalog event-ingest]
      :tools [xiaojin-maintainer xjctl-gcp-bridge]
      :implemented [persistent-json-registry whitelist-command-executor net-db-smoke-command net-ota-confirm-command]
      :not-yet [gcp-production-deploy device-auth signed-artifacts kernel-native-maintenance-poller tls-client]
      :purpose "Let a GCP-hosted maintainer broker durable commands while a reachable host bridge executes the current LAN UDP control protocol.")
    (stage kernel-outbound-client
      :status planned
      :requires [dns tcp tls artifact-signature-verification wifi-station-optional]
      :purpose "Move cloud maintenance from a host bridge into Xiaojin OS itself.")
    (stage postgres-backup-compatibility
      :status implemented-host-backup-set
      :implemented [xiaojindb-blob-store host-pg-dump-importer pg-restore-list-check restore-smoke backup-set-tarball maintainer-restore-smoked-registry]
      :next [row-level-import logical-replay-format backup-diff-verifier scheduled-gcp-backup]
      :purpose "Make XiaojinDB able to store and restore-smoke a complete backup copy of PostgreSQL databases running on the GCP VM.")
    (stage postgres-serving-compatibility
      :status implemented-host-bridge-partial
      :implemented [http-kv-bridge postgres-wire-simple-query postgres-wire-extended-query protected-password-auth system-psql-smoke read-only-health-smoke vm-local-systemd-template]
      :next [schema-catalog transaction-layer query-planner driver-compatibility-matrix load-test diff-test]
      :purpose "Replace selected GCP VM PostgreSQL serving paths with XiaojinDB after host bridge compatibility, backup/replay/diff parity, and load tests are proven."))

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
    (command rng
      :subcommands [status sample]
      :confirmed-write-commands [sample]
      :grade "development-hmac-drbg"
      :hardware-trng false
      :purpose "Expose a timer/identity mixed HMAC-SHA256 DRBG for nonce/session bring-up before a hardware TRNG is wired.")
    (command net
      :subcommands [facts ips ip dhcp dns tcp http tls loop listen sniff]
      :write-commands [ip dhcp loop listen sniff]
      :confirmed-write-commands [ip dhcp loop listen sniff]
      :loop-duration-ms 120000
      :udp-port 5510
      :test-ips [192.168.80.251 192.168.80.250]
      :default-netmask 255.255.255.0
      :cold-boot-auto-listen true
      :serial-escape-ms 2000
      :runtime-ip-config true
      :udp-ip-config true
      :manual-dhcp-client true
      :udp-dhcp-control true
      :dns-client true
      :tcp-connect-probe true
      :http-get-probe true
      :udp-query-control true
      :purpose "Show wired-network bring-up facts, configure runtime static/DHCP IPs, run the ARP/IPv4/UDP responder smoke test, and host the XJNL UDP image loader.")
    (command wifi
      :subcommands [facts status roadmap scan join ap]
      :implemented-readonly [facts status roadmap]
      :reserved-write-commands [scan join ap]
      :confirmed-write-commands [scan join ap]
      :current-stage facts-only
      :chip rtl8822ce
      :purpose "Expose R5C WiFi hardware facts and planned station/AP surfaces while keeping the RTL8125 wired loader as the rescue path.")
    (command sd
      :subcommands [facts regs read0 bpb root slots]
      :confirmed-read-commands [regs read0 bpb root slots]
      :controller "dwmmc@fe2b0000"
      :uboot-mmc-dev 1
      :write-enabled "inactive-slot-image-only-via-XJPS"
      :purpose "Show SD/MMC OTA storage facts, read FAT32 metadata, verify A/B slot FAT chains, and support controlled inactive-slot image writes.")
    (command pcie
      :subcommands [facts reset-pins status phy-status phy-init apb-status cfg-probe iatu-probe cfg-scan bar-scan bar-map rtl-probe rtl-facts rtl-init rtl-tx-test rtl-rx-poll cru-status cru-enable rc-init ltssm-enable ltssm-disable link-start init reset-deassert reset-assert]
      :init-mode "read-only-preflight"
      :write-commands [phy-init cru-enable rc-init ltssm-enable ltssm-disable link-start reset-deassert reset-assert bar-map rtl-init rtl-tx-test]
      :confirmed-read-commands [apb-status cfg-probe iatu-probe cfg-scan bar-scan rtl-probe rtl-facts rtl-rx-poll]
      :write-scope "PCIe3 PHY GRF, PCIe3 CRU clock/reset bits, pcie3x1/pcie3x2 APB RC/LTSSM mode, pcie3x1/pcie3x2 reset GPIOs, DWC iATU windows, RTL8125 PCI command/BAR registers, and RTL8125 polling descriptor rings only; requires confirm"
      :purpose "Show R5C PCIe controller facts, inspect PHY/APB status, and control RJ45 PCIe reset GPIOs.")
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
