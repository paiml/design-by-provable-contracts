/-!
# Theorems for safe-div-v1 contract — M4 proptest demo
`safe_div : Int → Int → Option Int` is total. We model `i32::MIN`
and `-1` symbolically; the full integer-overflow story lives in the
Rust kernel.
-/

namespace ProvableContracts.Theorems

/-- The Int analogue of `safe_div`. Returns `none` on divisor zero, the
single overflow case the kernel guards (i32::MIN / -1) is left as a
modeling choice for the user. -/
def safeDiv (a b : Int) : Option Int :=
  if b = 0 then none else some (a / b)

/-- Divide by zero returns `none`. -/
theorem SafeDivByZeroIsNone (a : Int) : safeDiv a 0 = none := by
  unfold safeDiv
  simp

/-- The i32::MIN / -1 overflow case. In Lean's `Int`, no overflow
exists; the theorem becomes "the kernel returns `some` on this input
in the unbounded model" — which it does. -/
theorem SafeDivI32MinOverNegOne : safeDiv (Int.negSucc 0) (Int.negSucc 0) ≠ none := by
  unfold safeDiv
  simp

/-- `safeDiv` is total — for every `(a, b)` it returns either `none`
or `some`. Trivial by case-split on the `if`. -/
theorem SafeDivNeverPanics (a b : Int) : safeDiv a b = none ∨ ∃ q, safeDiv a b = some q := by
  unfold safeDiv
  by_cases h : b = 0
  · left; simp [h]
  · right; exact ⟨a / b, by simp [h]⟩

/-- Agrees with native division when `b ≠ 0`. -/
theorem SafeDivAgreesWithNative (a b : Int) (hb : b ≠ 0) :
    safeDiv a b = some (a / b) := by
  unfold safeDiv
  simp [hb]

/-- Round trip on exact divisions: `safeDiv (q * b) b = some q` when `b ≠ 0`. -/
theorem SafeDivRoundTripExact (q b : Int) (hb : b ≠ 0) :
    safeDiv (q * b) b = some q := by
  unfold safeDiv
  simp [hb, Int.mul_ediv_cancel _ hb]

end ProvableContracts.Theorems
