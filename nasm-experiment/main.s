bits 64

section .data

msg: db `Hello, world!\n`, 0

section .text

global _start
_start:
    mov rsi, msg
    call printz

    mov rdi, 0
    call exit

; inputs:
;   rdi = exit code
exit:
    mov rax, 60
    syscall

; inputs:
;   rsi = null-terminated char*
; returns:
;   rcx = length, not including null
; clobbers:
;   al
strlen:
    mov rcx, 0
.loop:
    mov al,[rsi]
    inc rsi
    inc rcx
    test al,al
    jne .loop
    ret

; inputs:
;   rsi = null-terminated char*
; clobbers:
;   rcx
;   al
printz:
    ; save pointer
    push rsi
    call strlen
    ; length
    mov rdx,rcx
    ; 1 = write
    mov rax,1
    ; 1 = stdout
    mov rdi,1
    ; restore pointer
    pop rsi
    syscall
    ret

