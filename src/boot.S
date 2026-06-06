.section .text.boot
.global _start
.type _start, %function

_start:
    msr daifset, #0xf

    adrp x0, __bss_start
    add  x0, x0, :lo12:__bss_start
    adrp x1, __bss_end
    add  x1, x1, :lo12:__bss_end

1:
    cmp x0, x1
    b.hs 2f
    str xzr, [x0], #8
    b 1b

2:
    adrp x0, _boot_stack_top
    add  x0, x0, :lo12:_boot_stack_top
    mov sp, x0

    adrp x0, __exception_vectors
    add  x0, x0, :lo12:__exception_vectors
    mrs  x1, CurrentEL
    lsr  x1, x1, #2
    cmp  x1, #2
    b.ne 4f
    msr  VBAR_EL2, x0
    b    5f
4:
    cmp  x1, #1
    b.ne 5f
    msr  VBAR_EL1, x0
5:
    isb

    bl kernel_main

3:
    wfe
    b 3b

.macro VECTOR_ENTRY label
    b \label
    .balign 128
.endm

.macro EXCEPTION_ENTRY label, kind
\label:
    sub sp, sp, #256
    stp x0,  x1,  [sp, #0]
    stp x2,  x3,  [sp, #16]
    stp x4,  x5,  [sp, #32]
    stp x6,  x7,  [sp, #48]
    stp x8,  x9,  [sp, #64]
    stp x10, x11, [sp, #80]
    stp x12, x13, [sp, #96]
    stp x14, x15, [sp, #112]
    stp x16, x17, [sp, #128]
    stp x18, x19, [sp, #144]
    stp x20, x21, [sp, #160]
    stp x22, x23, [sp, #176]
    stp x24, x25, [sp, #192]
    stp x26, x27, [sp, #208]
    stp x28, x29, [sp, #224]
    str x30, [sp, #240]

    mov x0, #\kind
    mov x1, sp
    bl exception_handler

    ldp x0,  x1,  [sp, #0]
    ldp x2,  x3,  [sp, #16]
    ldp x4,  x5,  [sp, #32]
    ldp x6,  x7,  [sp, #48]
    ldp x8,  x9,  [sp, #64]
    ldp x10, x11, [sp, #80]
    ldp x12, x13, [sp, #96]
    ldp x14, x15, [sp, #112]
    ldp x16, x17, [sp, #128]
    ldp x18, x19, [sp, #144]
    ldp x20, x21, [sp, #160]
    ldp x22, x23, [sp, #176]
    ldp x24, x25, [sp, #192]
    ldp x26, x27, [sp, #208]
    ldp x28, x29, [sp, #224]
    ldr x30, [sp, #240]
    add sp, sp, #256
    eret
.endm

.section .text.exceptions, "ax"
.balign 2048
.global __exception_vectors
.type __exception_vectors, %function
__exception_vectors:
    VECTOR_ENTRY __exception_current_sp0_sync
    VECTOR_ENTRY __exception_current_sp0_irq
    VECTOR_ENTRY __exception_current_sp0_fiq
    VECTOR_ENTRY __exception_current_sp0_serror
    VECTOR_ENTRY __exception_current_spx_sync
    VECTOR_ENTRY __exception_current_spx_irq
    VECTOR_ENTRY __exception_current_spx_fiq
    VECTOR_ENTRY __exception_current_spx_serror
    VECTOR_ENTRY __exception_lower_a64_sync
    VECTOR_ENTRY __exception_lower_a64_irq
    VECTOR_ENTRY __exception_lower_a64_fiq
    VECTOR_ENTRY __exception_lower_a64_serror
    VECTOR_ENTRY __exception_lower_a32_sync
    VECTOR_ENTRY __exception_lower_a32_irq
    VECTOR_ENTRY __exception_lower_a32_fiq
    VECTOR_ENTRY __exception_lower_a32_serror

EXCEPTION_ENTRY __exception_current_sp0_sync, 0
EXCEPTION_ENTRY __exception_current_sp0_irq, 1
EXCEPTION_ENTRY __exception_current_sp0_fiq, 2
EXCEPTION_ENTRY __exception_current_sp0_serror, 3
EXCEPTION_ENTRY __exception_current_spx_sync, 4
EXCEPTION_ENTRY __exception_current_spx_irq, 5
EXCEPTION_ENTRY __exception_current_spx_fiq, 6
EXCEPTION_ENTRY __exception_current_spx_serror, 7
EXCEPTION_ENTRY __exception_lower_a64_sync, 8
EXCEPTION_ENTRY __exception_lower_a64_irq, 9
EXCEPTION_ENTRY __exception_lower_a64_fiq, 10
EXCEPTION_ENTRY __exception_lower_a64_serror, 11
EXCEPTION_ENTRY __exception_lower_a32_sync, 12
EXCEPTION_ENTRY __exception_lower_a32_irq, 13
EXCEPTION_ENTRY __exception_lower_a32_fiq, 14
EXCEPTION_ENTRY __exception_lower_a32_serror, 15

.section .text.trampoline, "ax"
.balign 64
.global reload_trampoline_start
.global reload_trampoline_end
.type reload_trampoline_start, %function

reload_trampoline_start:
    // x0 = dst, x1 = src, x2 = size, x3 = entry
    mov x4, x0
    mov x5, x1
    mov x6, x2

1:
    cbz x6, 2f
    ldrb w7, [x5], #1
    strb w7, [x4], #1
    subs x6, x6, #1
    b.ne 1b

2:
    mov x4, x0
    add x5, x0, x2
    bic x4, x4, #63

3:
    cmp x4, x5
    b.hs 4f
    dc cvau, x4
    add x4, x4, #64
    b 3b

4:
    dsb ish
    mov x4, x0
    add x5, x0, x2
    bic x4, x4, #63

5:
    cmp x4, x5
    b.hs 6f
    ic ivau, x4
    add x4, x4, #64
    b 5b

6:
    dsb ish
    isb
    br x3

reload_trampoline_end:
