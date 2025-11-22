;; Demo 1: Add Two Numbers
;; Purpose: Simplest possible WASM module - pure computation, no host calls
;; Tests: Basic WASM execution, parameter passing, return values

(module
  ;; Export a function that adds two i32 numbers
  (func $add (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Export a function that multiplies two numbers
  (func $mul (export "mul") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; Export a function that computes factorial (recursive test)
  (func $factorial (export "factorial") (param $n i32) (result i32)
    (if (result i32)
      (i32.le_s (local.get $n) (i32.const 1))
      (then (i32.const 1))
      (else
        (i32.mul
          (local.get $n)
          (call $factorial
            (i32.sub (local.get $n) (i32.const 1))
          )
        )
      )
    )
  )
)
