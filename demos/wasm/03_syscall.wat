;; Demo 3: Syscall and Capability Test
;; Purpose: Tests syscall interface and capability checking
;; Tests: Syscall bridge, capability validation, security isolation

(module
  ;; Import syscall function from host
  ;; syscall(syscall_num, arg1, arg2, arg3) -> result
  (import "env" "syscall" (func $syscall (param i32 i32 i32 i32) (result i32)))

  ;; Import print for debugging
  (import "env" "print" (func $print (param i32)))

  ;; Syscall numbers (should match kernel definitions)
  (global $SYS_READ i32 (i32.const 0))
  (global $SYS_WRITE i32 (i32.const 1))
  (global $SYS_ALLOCATE i32 (i32.const 2))

  ;; Test basic syscall
  (func $test_syscall (export "test_syscall")
    ;; Call sys_write(1, 0x1000, 10) - write 10 bytes to fd 1
    global.get $SYS_WRITE
    i32.const 1      ;; fd = 1 (stdout)
    i32.const 0x1000 ;; buffer address
    i32.const 10     ;; length
    call $syscall

    ;; Print result
    call $print
  )

  ;; Test memory allocation (should require capability)
  (func $test_allocate (export "test_allocate") (param $size i32) (result i32)
    ;; Call sys_allocate(size, 0, 0)
    global.get $SYS_ALLOCATE
    local.get $size
    i32.const 0
    i32.const 0
    call $syscall
  )

  ;; Test unauthorized access (should fail without capability)
  (func $test_unauthorized (export "test_unauthorized") (result i32)
    ;; Try to read from protected resource (fd 99)
    global.get $SYS_READ
    i32.const 99     ;; protected fd
    i32.const 0x2000 ;; buffer
    i32.const 100    ;; length
    call $syscall
  )
)
