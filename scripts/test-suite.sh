#!/usr/bin/env bash
set -euo pipefail
# =============================================================================
# OpenCode Test Suite — compares rtc against rustc for teaching compiler validation
# Usage: bash scripts/test-suite.sh              # run all tests
#        bash scripts/test-suite.sh --positive    # positive tests only
#        bash scripts/test-suite.sh --negative    # negative tests only
#        bash scripts/test-suite.sh --ir-compare  # IR comparison tests
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

echo ""
echo "=========================================="
echo -e "Results: ${GREEN}$PASS passed${NC}, ${RED}$FAIL failed${NC}, ${YELLOW}$SKIP skipped${NC}"
echo "=========================================="

if [ $FAIL -gt 0 ]; then
    exit 1
fi
