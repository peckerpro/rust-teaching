#!/usr/bin/env bash
set -euo pipefail
# =============================================================================
# OpenCode Test Suite — compares rtc against rustc for teaching compiler validation
# Usage: bash scripts/test-suite.sh                  # all tests (rtc only)
#        bash scripts/test-suite.sh --positive        # positive tests only
#        bash scripts/test-suite.sh --negative        # negative tests only
#        bash scripts/test-suite.sh --ir-compare      # IR comparison tests
#        bash scripts/test-suite.sh --rustc-verify    # cross-verify against rustc
# =============================================================================

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; NC='\033[0m'
RTC="./target/debug/rtc"
RUSTC="rustc"
PASS=0; FAIL=0; SKIP=0
TESTS_DIR="tests"

run_test() {
    local name="$1"
    local file="$TESTS_DIR/$2"
    local mode="$3"
    local expect_pass="$4"

    if [ ! -f "$file" ]; then
        echo -e "  ${YELLOW}SKIP${NC} $name (missing $file)"
        SKIP=$((SKIP + 1))
        return
    fi

    case "$mode" in
        positive)
            if timeout 10 "$RTC" --check -S "$file" > /dev/null 2>&1; then
                echo -e "  ${GREEN}PASS${NC} $name"
                PASS=$((PASS + 1))
            else
                echo -e "  ${RED}FAIL${NC} $name (rtc rejected valid code)"
                FAIL=$((FAIL + 1))
            fi
            ;;
        negative)
            if timeout 10 "$RTC" --check "$file" > /dev/null 2>&1; then
                echo -e "  ${RED}FAIL${NC} $name (rtc accepted invalid code)"
                FAIL=$((FAIL + 1))
            else
                echo -e "  ${GREEN}PASS${NC} $name"
                PASS=$((PASS + 1))
            fi
            ;;
        ir-compare)
            local rtc_ir="/tmp/rtc_test_$$.ll"
            local rustc_ir="/tmp/rustc_test_$$.ll"
            local rtc_ok=0 rustc_ok=0

            timeout 10 "$RTC" -S -o "$rtc_ir" "$file" > /dev/null 2>&1 && rtc_ok=1 || true
            if timeout 10 "$RUSTC" --emit=llvm-ir -o "$rustc_ir" "$file" > /dev/null 2>&1; then
                rustc_ok=1
            fi

            if [ $rtc_ok -eq 1 ] && [ $rustc_ok -eq 1 ]; then
                echo -e "  ${GREEN}PASS${NC} $name (both compiled)"
                PASS=$((PASS + 1))
            elif [ $rtc_ok -ne $rustc_ok ]; then
                echo -e "  ${RED}FAIL${NC} $name (rtc=$rtc_ok rustc=$rustc_ok)"
                FAIL=$((FAIL + 1))
            else
                echo -e "  ${YELLOW}SKIP${NC} $name (neither compiled)"
                SKIP=$((SKIP + 1))
            fi
            rm -f "$rtc_ir" "$rustc_ir"
            ;;
    esac
}

echo "=========================================="
echo " OpenCode Test Suite"
echo "=========================================="

# ---- Positive Tests ----
if [ "${1:-}" != "--negative" ] && [ "${1:-}" != "--ir-compare" ]; then
echo ""
echo "[Positive Tests — valid code that should compile]"
echo "------------------------------------------"

run_test "integer literal"               "positive/int_literal.rs"       positive true
run_test "float literal"                 "positive/float_literal.rs"     positive true
run_test "bool literal"                  "positive/bool_literal.rs"      positive true
run_test "let binding"                   "positive/let_binding.rs"       positive true
run_test "arithmetic (add)"             "positive/arith_add.rs"         positive true
run_test "arithmetic (mul)"             "positive/arith_mul.rs"         positive true
run_test "arithmetic (div)"             "positive/arith_div.rs"         positive true
run_test "comparison (eq)"              "positive/cmp_eq.rs"            positive true
run_test "comparison (lt)"              "positive/cmp_lt.rs"            positive true
run_test "logical not"                   "positive/logical_not.rs"       positive true
run_test "if/else"                       "positive/if_else.rs"          positive true
run_test "return value"                  "positive/return_value.rs"     positive true
run_test "return void"                   "positive/return_void.rs"      positive true
run_test "nested blocks"                 "positive/nested_block.rs"     positive true
run_test "unary neg"                     "positive/unary_neg.rs"        positive true
run_test "function call (no args)"       "positive/fn_call.rs"          positive true
run_test "function call (with args)"     "positive/fn_call_args.rs"     positive true
run_test "function call (nested)"        "positive/fn_call_nested.rs"   positive true
run_test "multiple functions"            "positive/multi_fn.rs"         positive true
run_test "fibonacci"                     "positive/fibonacci.rs"        positive true
run_test "mutual recursion"              "positive/mutual_rec.rs"       positive true
echo "  [Phase A1 — extended integral types]"
run_test "type i8"                       "phase_a1/type_i8.rs"           positive true
run_test "type i16"                      "phase_a1/type_i16.rs"          positive true
run_test "type i32"                      "phase_a1/type_i32.rs"          positive true
run_test "type i64"                      "phase_a1/type_i64.rs"          positive true
run_test "type u8"                       "phase_a1/type_u8.rs"           positive true
run_test "type u16"                      "phase_a1/type_u16.rs"          positive true
run_test "type u32"                      "phase_a1/type_u32.rs"          positive true
run_test "type u64"                      "phase_a1/type_u64.rs"          positive true
run_test "type f32"                      "phase_a1/type_f32.rs"          positive true
run_test "type f64"                      "phase_a1/type_f64.rs"          positive true
run_test "type isize"                    "phase_a1/type_isize.rs"        positive true
run_test "type usize"                    "phase_a1/type_usize.rs"        positive true
run_test "fn with i8"                    "phase_a1/fn_i8.rs"             positive true
run_test "fn with i64"                   "phase_a1/fn_i64.rs"            positive true
run_test "fn with f32"                   "phase_a1/fn_f32.rs"            positive true
run_test "fn with f64"                   "phase_a1/fn_f64.rs"            positive true
run_test "all types"                     "phase_a1/all_types.rs"         positive true
echo "  [Phase B — control flow]"
run_test "loop break"                    "phase_b/loop_break.rs"          positive true
run_test "loop counter"                  "phase_b/loop_counter.rs"        positive true
run_test "while simple"                  "phase_b/while_simple.rs"        positive true
run_test "while continue"                "phase_b/while_continue.rs"      positive true
run_test "while break"                   "phase_b/while_break.rs"         positive true
run_test "nested loop"                   "phase_b/nested_loop.rs"         positive true
run_test "mixed loop/while"              "phase_b/mixed_loop_while.rs"    positive true
run_test "for simple"                    "phase_b/for_simple.rs"          positive true
run_test "while large"                   "phase_b/while_large.rs"         positive true
echo "  [Phase C — compound data types]"
run_test "struct basic"                  "phase_c/struct_basic.rs"        positive true
run_test "struct field access"           "phase_c/struct_field_access.rs" positive true
run_test "struct literal"                "phase_c/struct_literal.rs"      positive true
run_test "tuple basic"                   "phase_c/tuple_basic.rs"         positive true
run_test "tuple mixed"                   "phase_c/tuple_mixed.rs"         positive true
run_test "tuple unit"                    "phase_c/tuple_unit.rs"          positive true
run_test "match literal"                 "phase_c/match_literal.rs"       positive true
run_test "match wildcard"                "phase_c/match_wildcard.rs"      positive true
run_test "match single arm"              "phase_c/match_single_arm.rs"    positive true
echo "  [Phase D — ownership]"
run_test "copy ok"                       "phase_d/copy_ok.rs"             positive true
run_test "move struct"                   "phase_d/move_struct.rs"         positive true
run_test "move into fn"                  "phase_d/move_into_fn.rs"        positive true
run_test "borrow shared"                 "phase_d/borrow_shared.rs"       positive true
run_test "borrow multi shared"           "phase_d/borrow_multi_shared.rs" positive true
fi

# ---- Negative Tests ----
if [ "${1:-}" != "--positive" ] && [ "${1:-}" != "--ir-compare" ]; then
echo ""
echo "[Negative Tests — invalid code that should be rejected]"
echo "------------------------------------------"

run_test "undefined variable"            "negative/undefined_var.rs"     negative false
run_test "type mismatch (int vs bool)"   "negative/type_mismatch.rs"    negative false
run_test "double declaration"            "negative/double_decl.rs"      negative false
run_test "if condition not bool"         "negative/if_not_bool.rs"      negative false
run_test "return type mismatch"          "negative/return_mismatch.rs"  negative false
echo "  [Phase A1 — negative]"
run_test "float literal → i32 var"       "phase_a1/neg_float_to_int.rs"   negative false
run_test "int literal → f64 var"         "phase_a1/neg_int_to_float.rs"   negative false
run_test "i64 literal → i16 var"         "phase_a1/neg_i64_to_i16.rs"     negative false
run_test "i8 literal → i32 var"          "phase_a1/type_mismatch_width.rs" negative false
echo "  [Phase B — negative]"
run_test "break outside loop"            "phase_b/neg_break_outside_loop.rs"   negative false
run_test "continue outside loop"         "phase_b/neg_continue_outside_loop.rs" negative false
echo "  [Phase C — negative]"
run_test "field type mismatch"           "phase_c/neg_field_type_mismatch.rs"       negative false
run_test "struct literal type mismatch"  "phase_c/neg_struct_literal_mismatch.rs"   negative false
run_test "tuple type mismatch"           "phase_c/neg_tuple_type_mismatch.rs"       negative false
echo "  [Phase D — negative]"
run_test "use after move"                "phase_d/neg_use_after_move.rs"          negative false
run_test "move into fn (use after)"      "phase_d/neg_move_into_fn.rs"            negative false
run_test "shared + mut borrow"           "phase_d/neg_shared_and_mut.rs"          negative false
run_test "use while mut borrowed"        "phase_d/neg_use_while_mut_borrowed.rs"  negative false
fi

# ---- IR Compare Tests ----
if [ "${1:-}" == "--ir-compare" ] || [ -z "${1:-}" ]; then
echo ""
echo "[IR Compare Tests — compile with both rtc and rustc]"
echo "------------------------------------------"

run_test "simple main"                   "ir_compare/simple_main.rs"    ir-compare true
run_test "fn with params"                "ir_compare/fn_params.rs"      ir-compare true
run_test "arithmetic"                    "ir_compare/arithmetic.rs"     ir-compare true
fi

# ---- Rustc cross-verification ----
if [ "${1:-}" == "--rustc-verify" ]; then
echo ""
echo "[Rustc Cross-Verification — positive tests must also pass rustc]"
echo "------------------------------------------"

rustc_verify() {
    local name="$1" file="$TESTS_DIR/$2"
    if [ ! -f "$file" ]; then echo -e "  ${YELLOW}SKIP${NC} $name"; SKIP=$((SKIP+1)); return; fi
    if timeout 10 "$RUSTC" --edition 2024 --crate-type bin -o /tmp/rustc_test_out_$$ "$file" > /dev/null 2>&1; then
        rm -f /tmp/rustc_test_out_$$
        echo -e "  ${GREEN}PASS${NC} $name (rtc+rustc agree)"
        PASS=$((PASS + 1))
    else
        echo -e "  ${RED}FAIL${NC} $name (rustc rejected)"
        FAIL=$((FAIL + 1))
    fi
}

for f in "$TESTS_DIR"/positive/*.rs "$TESTS_DIR"/phase_a1/type_i8.rs "$TESTS_DIR"/phase_a1/type_i16.rs "$TESTS_DIR"/phase_a1/type_i32.rs "$TESTS_DIR"/phase_a1/type_i64.rs "$TESTS_DIR"/phase_a1/type_u8.rs "$TESTS_DIR"/phase_a1/type_u16.rs "$TESTS_DIR"/phase_a1/type_u32.rs "$TESTS_DIR"/phase_a1/type_u64.rs "$TESTS_DIR"/phase_a1/type_f32.rs "$TESTS_DIR"/phase_a1/type_f64.rs "$TESTS_DIR"/phase_a1/type_isize.rs "$TESTS_DIR"/phase_a1/type_usize.rs "$TESTS_DIR"/phase_a1/fn_*.rs "$TESTS_DIR"/phase_a1/all_types.rs; do
    [ -f "$f" ] || continue
    name="$(basename "$f" .rs)"
    rustc_verify "rustc: $name" "${f#$TESTS_DIR/}"
done
fi

echo ""
echo "=========================================="
echo -e "Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, ${YELLOW}$SKIP skipped${NC}"
echo "=========================================="

if [ $FAIL -gt 0 ]; then
    exit 1
fi
