.p2align 2
.globl _loom_runtime_restore_context_arm64_darwin
_loom_runtime_restore_context_arm64_darwin:
    mov x15,  x0

    ldr x16, [x15, #248] ;; sp
    ldr x17, [x15, #256] ;; pc

    ldr x0,  [x15, #0]
    ldr x1,  [x15, #8]
    ldr x2,  [x15, #16]
    ldr x3,  [x15, #24]
    ldr x4,  [x15, #32]
    ldr x5,  [x15, #40]
    ldr x6,  [x15, #48]
    ldr x7,  [x15, #56]
    ldr x8,  [x15, #64]
    ldr x9,  [x15, #72]
    ldr x10, [x15, #80]
    ldr x11, [x15, #88]
    ldr x12, [x15, #96]
    ldr x13, [x15, #104]
    ldr x14, [x15, #112]

    ;;ldr x15, [x15, #120]
    ;;ldr x16, [x15, #128]
    ;;ldr x17, [x15, #136]

    ldr x18, [x15, #144]
    ldr x19, [x15, #152]
    ldr x20, [x15, #160]
    ldr x21, [x15, #168]
    ldr x22, [x15, #176]
    ldr x23, [x15, #184]
    ldr x24, [x15, #192]
    ldr x25, [x15, #200]
    ldr x26, [x15, #208]
    ldr x27, [x15, #216]
    ldr x28, [x15, #224]
    ldr x29, [x15, #232]
    ldr x30, [x15, #240]

    mov sp, x16
    ldr x16, [x15, #128]
    ldr x15, [x15, 120]

    br x17