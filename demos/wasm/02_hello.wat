;; Demo 2: Hello World with Host Functions
;; Purpose: Tests host function calls (env.print)
;; Tests: Host imports, function calls across boundary

(module
  ;; Import the print function from host
  (import "env" "print" (func $print (param i32)))

  ;; Export a main function that prints numbers
  (func $main (export "main")
    ;; Print 42
    i32.const 42
    call $print

    ;; Print 100
    i32.const 100
    call $print

    ;; Print 255
    i32.const 255
    call $print
  )

  ;; Export a function that prints a range
  (func $print_range (export "print_range") (param $start i32) (param $end i32)
    (local $i i32)
    local.get $start
    local.set $i

    (block $break
      (loop $continue
        ;; Check if i >= end
        local.get $i
        local.get $end
        i32.ge_s
        br_if $break

        ;; Print current value
        local.get $i
        call $print

        ;; Increment i
        local.get $i
        i32.const 1
        i32.add
        local.set $i

        br $continue
      )
    )
  )
)
