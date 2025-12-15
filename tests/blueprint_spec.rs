//! Blueprint Starlark Specification Compliance Tests
//!
//! These tests verify that Blueprint's Starlark implementation is compliant
//! with the Bazel Starlark specification.

use blueprint_interpreter::{eval_plan, SchemaGenerator, PlanGenerator};
use blueprint_common::ExecutionContext;

fn run_star_code(code: &str) -> Result<String, String> {
    let schema = SchemaGenerator::generate_for_eval(code, "eval.star")
        .map_err(|e| format!("Generation error: {}", e))?;

    let ctx = ExecutionContext::from_current_env();
    let plan_gen = PlanGenerator::new(&ctx);
    let plan = plan_gen.generate(&schema)
        .map_err(|e| format!("Plan generation error: {}", e))?;

    eval_plan(&plan)
        .map_err(|e| format!("Execution error: {}", e))
}

fn assert_eval(code: &str, expected: &str) {
    let result = run_star_code(code).expect("evaluation failed");
    assert_eq!(result.trim_matches('"'), expected, "Code: {}", code);
}

fn assert_eval_int(code: &str, expected: i64) {
    let result = run_star_code(code).expect("evaluation failed");
    let parsed: i64 = result.parse().expect(&format!("Expected int, got: {}", result));
    assert_eq!(parsed, expected, "Code: {}", code);
}

fn assert_eval_bool(code: &str, expected: bool) {
    let result = run_star_code(code).expect("evaluation failed");
    let expected_str = if expected { "True" } else { "False" };
    assert_eq!(result, expected_str, "Code: {}", code);
}

fn assert_eval_float(code: &str, expected: f64, tolerance: f64) {
    let result = run_star_code(code).expect("evaluation failed");
    let parsed: f64 = result.parse().expect(&format!("Expected float, got: {}", result));
    assert!((parsed - expected).abs() < tolerance,
        "Expected ~{}, got {} for code: {}", expected, parsed, code);
}

fn assert_eval_error(code: &str) {
    assert!(run_star_code(code).is_err(), "Expected error for code: {}", code);
}

fn assert_eval_list_len(code: &str, expected_len: usize) {
    let result = run_star_code(&format!("len({})", code)).expect("evaluation failed");
    let len: usize = result.parse().expect(&format!("Expected int, got: {}", result));
    assert_eq!(len, expected_len, "Code: {}", code);
}

// ============================================================================
// Data Types - None
// ============================================================================

mod none_type {
    use super::*;

    #[test]
    fn literal() {
        assert_eval("None", "None");
    }

    #[test]
    fn is_falsy() {
        assert_eval_bool("bool(None)", false);
    }

    #[test]
    fn equals_itself() {
        assert_eval_bool("None == None", true);
    }

    #[test]
    fn not_equals_other_values() {
        assert_eval_bool("None != 0", true);
        assert_eval_bool("None != False", true);
        assert_eval_bool("None != \"\"", true);
    }
}

// ============================================================================
// Data Types - Booleans
// ============================================================================

mod booleans {
    use super::*;

    #[test]
    fn true_literal() {
        assert_eval_bool("True", true);
    }

    #[test]
    fn false_literal() {
        assert_eval_bool("False", false);
    }

    #[test]
    fn equality() {
        assert_eval_bool("True == True", true);
        assert_eval_bool("False == False", true);
        assert_eval_bool("True != False", true);
    }

    #[test]
    fn int_one_not_equals_true() {
        assert_eval_bool("1 == True", false);
    }

    #[test]
    fn bool_one_equals_true() {
        assert_eval_bool("bool(1) == True", true);
    }

    #[test]
    fn int_conversion() {
        assert_eval_int("int(True)", 1);
        assert_eval_int("int(False)", 0);
    }

    #[test]
    fn falsy_values() {
        assert_eval_bool("bool(False)", false);
        assert_eval_bool("bool(None)", false);
        assert_eval_bool("bool(0)", false);
        assert_eval_bool("bool(0.0)", false);
        assert_eval_bool("bool(\"\")", false);
        assert_eval_bool("bool([])", false);
        assert_eval_bool("bool({})", false);
    }

    #[test]
    fn truthy_values() {
        assert_eval_bool("bool(True)", true);
        assert_eval_bool("bool(1)", true);
        assert_eval_bool("bool(-1)", true);
        assert_eval_bool("bool(0.1)", true);
        assert_eval_bool("bool(\"hello\")", true);
        assert_eval_bool("bool([1])", true);
        assert_eval_bool("bool({\"a\": 1})", true);
    }
}

// ============================================================================
// Data Types - Integers
// ============================================================================

mod integers {
    use super::*;

    #[test]
    fn decimal_literal() {
        assert_eval_int("123", 123);
        assert_eval_int("0", 0);
    }

    #[test]
    fn hex_literal() {
        assert_eval_int("0x7f", 127);
        assert_eval_int("0xFF", 255);
        assert_eval_int("0xffff", 65535);
    }

    #[test]
    fn octal_literal() {
        assert_eval_int("0o755", 493);
        assert_eval_int("0O10", 8);
    }

    #[test]
    fn negative() {
        assert_eval_int("-42", -42);
    }

    #[test]
    fn addition() {
        assert_eval_int("1 + 2", 3);
        assert_eval_int("-5 + 10", 5);
    }

    #[test]
    fn subtraction() {
        assert_eval_int("10 - 3", 7);
        assert_eval_int("5 - 10", -5);
    }

    #[test]
    fn multiplication() {
        assert_eval_int("4 * 5", 20);
        assert_eval_int("-3 * 7", -21);
    }

    #[test]
    fn floor_division_positive() {
        assert_eval_int("10 // 3", 3);
        assert_eval_int("3 // 2", 1);
    }

    #[test]
    fn floor_division_negative() {
        assert_eval_int("-10 // 3", -4);
        assert_eval_int("10 // -3", -4);
    }

    #[test]
    fn modulo_positive() {
        assert_eval_int("10 % 3", 1);
    }

    #[test]
    fn modulo_negative() {
        assert_eval_int("-10 % 3", 2);
        assert_eval_int("10 % -3", -2);
    }

    #[test]
    fn bitwise_not() {
        assert_eval_int("~0", -1);
        assert_eval_int("~1", -2);
    }

    #[test]
    fn bitwise_or() {
        assert_eval_int("0b1010 | 0b0101", 0b1111);
    }

    #[test]
    fn bitwise_and() {
        assert_eval_int("0b1110 & 0b0111", 0b0110);
    }

    #[test]
    fn bitwise_xor() {
        assert_eval_int("0b1010 ^ 0b0110", 0b1100);
    }

    #[test]
    fn left_shift() {
        assert_eval_int("1 << 4", 16);
    }

    #[test]
    fn right_shift() {
        assert_eval_int("16 >> 2", 4);
    }
}

// ============================================================================
// Data Types - Floats
// ============================================================================

mod floats {
    use super::*;

    #[test]
    fn literal_decimal() {
        assert_eval_float("3.14", 3.14, 0.001);
    }

    #[test]
    fn literal_trailing_dot() {
        assert_eval_float("1.", 1.0, 0.001);
    }

    #[test]
    fn literal_leading_dot() {
        assert_eval_float(".5", 0.5, 0.001);
    }

    #[test]
    fn literal_exponent() {
        assert_eval_float("1e10", 1e10, 1e5);
    }

    #[test]
    fn addition() {
        assert_eval_float("1.5 + 2.5", 4.0, 0.001);
    }

    #[test]
    fn subtraction() {
        assert_eval_float("5.5 - 2.0", 3.5, 0.001);
    }

    #[test]
    fn multiplication() {
        assert_eval_float("2.5 * 4.0", 10.0, 0.001);
    }

    #[test]
    fn division() {
        assert_eval_float("10.0 / 4.0", 2.5, 0.001);
    }

    #[test]
    fn floor_division() {
        assert_eval_float("10.0 // 3.0", 3.0, 0.001);
    }

    #[test]
    fn int_float_mixed() {
        assert_eval_float("3.141 + 1", 4.141, 0.001);
    }

    #[test]
    fn comparison() {
        assert_eval_bool("1.5 < 2.0", true);
        assert_eval_bool("2.0 > 1.5", true);
        assert_eval_bool("1.5 == 1.5", true);
    }

    #[test]
    fn positive_and_negative_zero_equal() {
        assert_eval_bool("0.0 == -0.0", true);
    }
}

// ============================================================================
// Data Types - Strings
// ============================================================================

mod strings {
    use super::*;

    #[test]
    fn single_quoted() {
        assert_eval("'hello'", "hello");
    }

    #[test]
    fn double_quoted() {
        assert_eval("\"hello\"", "hello");
    }

    #[test]
    fn concatenation() {
        assert_eval("\"hello\" + \" world\"", "hello world");
    }

    #[test]
    fn repetition() {
        assert_eval("\"ab\" * 3", "ababab");
    }

    #[test]
    fn length() {
        assert_eval_int("len(\"hello\")", 5);
    }

    #[test]
    fn indexing() {
        assert_eval("\"hello\"[0]", "h");
        assert_eval("\"hello\"[4]", "o");
    }

    #[test]
    fn negative_indexing() {
        assert_eval("\"hello\"[-1]", "o");
        assert_eval("\"hello\"[-5]", "h");
    }

    #[test]
    fn slicing() {
        assert_eval("\"hello\"[1:4]", "ell");
        assert_eval("\"hello\"[:3]", "hel");
        assert_eval("\"hello\"[2:]", "llo");
    }

    #[test]
    fn membership() {
        assert_eval_bool("\"el\" in \"hello\"", true);
        assert_eval_bool("\"x\" in \"hello\"", false);
    }

    #[test]
    fn escape_newline() {
        assert_eval("\"a\\nb\"", "a\nb");
    }

    #[test]
    fn escape_tab() {
        assert_eval("\"a\\tb\"", "a\tb");
    }

    #[test]
    fn escape_backslash() {
        assert_eval("\"a\\\\b\"", "a\\b");
    }

    #[test]
    fn comparison() {
        assert_eval_bool("\"abc\" < \"abd\"", true);
        assert_eval_bool("\"abc\" == \"abc\"", true);
    }

    #[test]
    fn empty_is_falsy() {
        assert_eval_bool("bool(\"\")", false);
    }

    #[test]
    fn nonempty_is_truthy() {
        assert_eval_bool("bool(\"x\")", true);
    }

    #[test]
    fn method_upper() {
        assert_eval("\"hello\".upper()", "HELLO");
    }

    #[test]
    fn method_lower() {
        assert_eval("\"HELLO\".lower()", "hello");
    }

    #[test]
    fn method_strip() {
        assert_eval("\"  hello  \".strip()", "hello");
    }

    #[test]
    fn method_split() {
        assert_eval_list_len("\"a,b,c\".split(\",\")", 3);
    }

    #[test]
    fn method_join() {
        assert_eval("\",\".join([\"a\", \"b\", \"c\"])", "a,b,c");
    }

    #[test]
    fn method_startswith() {
        assert_eval_bool("\"hello\".startswith(\"hel\")", true);
    }

    #[test]
    fn method_endswith() {
        assert_eval_bool("\"hello\".endswith(\"lo\")", true);
    }

    #[test]
    fn method_replace() {
        assert_eval("\"hello\".replace(\"l\", \"L\")", "heLLo");
    }

    #[test]
    fn method_find() {
        assert_eval_int("\"hello\".find(\"l\")", 2);
        assert_eval_int("\"hello\".find(\"x\")", -1);
    }
}

// ============================================================================
// Data Types - Lists
// ============================================================================

mod lists {
    use super::*;

    #[test]
    fn literal() {
        assert_eval_list_len("[1, 2, 3]", 3);
    }

    #[test]
    fn empty() {
        assert_eval_list_len("[]", 0);
    }

    #[test]
    fn length() {
        assert_eval_int("len([1, 2, 3])", 3);
    }

    #[test]
    fn indexing() {
        assert_eval_int("[1, 2, 3][0]", 1);
        assert_eval_int("[1, 2, 3][2]", 3);
    }

    #[test]
    fn negative_indexing() {
        assert_eval_int("[1, 2, 3][-1]", 3);
        assert_eval_int("[1, 2, 3][-3]", 1);
    }

    #[test]
    fn slicing() {
        assert_eval_list_len("[1, 2, 3, 4][1:3]", 2);
    }

    #[test]
    fn concatenation() {
        assert_eval_list_len("[1, 2] + [3, 4]", 4);
    }

    #[test]
    fn repetition() {
        assert_eval_list_len("[1, 2] * 3", 6);
    }

    #[test]
    fn membership() {
        assert_eval_bool("1 in [1, 2, 3]", true);
        assert_eval_bool("4 in [1, 2, 3]", false);
    }

    #[test]
    fn not_membership() {
        assert_eval_bool("4 not in [1, 2, 3]", true);
    }

    #[test]
    fn empty_is_falsy() {
        assert_eval_bool("bool([])", false);
    }

    #[test]
    fn nonempty_is_truthy() {
        assert_eval_bool("bool([1])", true);
    }

    #[test]
    fn method_append() {
        let code = r#"
x = [1, 2]
x.append(3)
len(x)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn method_extend() {
        let code = r#"
x = [1, 2]
x.extend([3, 4])
len(x)
"#;
        assert_eval_int(code, 4);
    }

    #[test]
    fn method_pop() {
        let code = r#"
x = [1, 2, 3]
x.pop()
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn method_index() {
        assert_eval_int("[1, 2, 3].index(2)", 1);
    }
}

// ============================================================================
// Data Types - Dictionaries
// ============================================================================

mod dictionaries {
    use super::*;

    #[test]
    fn literal() {
        assert_eval_list_len("{\"a\": 1, \"b\": 2}.keys()", 2);
    }

    #[test]
    fn empty() {
        assert_eval_list_len("{}.keys()", 0);
    }

    #[test]
    fn length() {
        assert_eval_int("len({\"a\": 1, \"b\": 2})", 2);
    }

    #[test]
    fn indexing() {
        assert_eval_int("{\"a\": 1}[\"a\"]", 1);
    }

    #[test]
    fn membership() {
        assert_eval_bool("\"a\" in {\"a\": 1}", true);
        assert_eval_bool("\"b\" in {\"a\": 1}", false);
    }

    #[test]
    fn empty_is_falsy() {
        assert_eval_bool("bool({})", false);
    }

    #[test]
    fn nonempty_is_truthy() {
        assert_eval_bool("bool({\"a\": 1})", true);
    }

    #[test]
    fn method_get() {
        assert_eval_int("{\"a\": 1}.get(\"a\")", 1);
    }

    #[test]
    fn method_get_default() {
        assert_eval_int("{\"a\": 1}.get(\"b\", 42)", 42);
    }

    #[test]
    fn method_keys() {
        assert_eval_list_len("{\"a\": 1, \"b\": 2}.keys()", 2);
    }

    #[test]
    fn method_values() {
        assert_eval_list_len("{\"a\": 1, \"b\": 2}.values()", 2);
    }
}

// ============================================================================
// Data Types - Bytes (starlark-rust limitation)
// ============================================================================

mod bytes_type {
    use super::*;

    #[test]
    #[ignore = "starlark-rust does not support bytes literals at AST level"]
    fn bytes_literal() {
        assert_eval("b\"hello\"", "b\"hello\"");
    }

    #[test]
    #[ignore = "starlark-rust does not support bytes literals at AST level"]
    fn bytes_escape_sequences() {
        assert_eval("b\"\\x00\\xff\"", "b\"\\x00\\xff\"");
    }

    #[test]
    #[ignore = "starlark-rust does not support bytes literals at AST level"]
    fn bytes_len() {
        assert_eval_int("len(b\"hello\")", 5);
    }

    #[test]
    #[ignore = "starlark-rust does not support bytes literals at AST level"]
    fn bytes_indexing() {
        assert_eval_int("b\"hello\"[0]", 104);
    }

    #[test]
    #[ignore = "starlark-rust does not support bytes literals at AST level"]
    fn bytes_slicing() {
        assert_eval("b\"hello\"[1:4]", "b\"ell\"");
    }

    #[test]
    #[ignore = "starlark-rust does not support bytes literals at AST level"]
    fn bytes_concatenation() {
        assert_eval("b\"hello\" + b\" world\"", "b\"hello world\"");
    }

    #[test]
    #[ignore = "starlark-rust does not support bytes literals at AST level"]
    fn bytes_elems() {
        assert_eval("list(b\"abc\".elems())", "[97, 98, 99]");
    }

    #[test]
    fn bytes_from_string() {
        assert_eval_int("len(bytes(\"hello\"))", 5);
    }

    #[test]
    fn bytes_from_list() {
        assert_eval_int("len(bytes([65, 66, 67]))", 3);
    }

    #[test]
    fn bytes_elems_from_constructor() {
        assert_eval("bytes(\"abc\").elems()", "[97, 98, 99]");
    }

    #[test]
    fn bytes_empty() {
        assert_eval_int("len(bytes())", 0);
    }
}

// ============================================================================
// Data Types - Set (starlark-rust limitation)
// ============================================================================

mod set_type {
    use super::*;

    #[test]
    fn set_from_list() {
        run_star_code("s = set([1, 2, 2, 3])").expect("set creation failed");
    }

    #[test]
    fn set_len() {
        assert_eval_int("len(set([1, 2, 2, 3]))", 3);
    }

    #[test]
    fn set_membership() {
        assert_eval_bool("2 in set([1, 2, 3])", true);
        assert_eval_bool("4 in set([1, 2, 3])", false);
    }

    #[test]
    fn set_union() {
        assert_eval_int("len(set([1, 2]) | set([2, 3]))", 3);
    }

    #[test]
    fn set_intersection() {
        assert_eval_int("len(set([1, 2, 3]) & set([2, 3, 4]))", 2);
    }

    #[test]
    fn set_difference() {
        assert_eval_int("len(set([1, 2, 3]) - set([2]))", 2);
    }

    #[test]
    fn set_symmetric_difference() {
        assert_eval_int("len(set([1, 2, 3]) ^ set([2, 3, 4]))", 2);
    }
}

// ============================================================================
// Built-in Functions
// ============================================================================

mod builtins {
    use super::*;

    #[test]
    fn abs_int() {
        assert_eval_int("abs(-5)", 5);
        assert_eval_int("abs(5)", 5);
    }

    #[test]
    fn abs_float() {
        assert_eval_float("abs(-3.14)", 3.14, 0.001);
    }

    #[test]
    fn all_true() {
        assert_eval_bool("all([True, True, True])", true);
    }

    #[test]
    fn all_false() {
        assert_eval_bool("all([True, False, True])", false);
    }

    #[test]
    fn all_empty() {
        assert_eval_bool("all([])", true);
    }

    #[test]
    fn any_true() {
        assert_eval_bool("any([False, True, False])", true);
    }

    #[test]
    fn any_false() {
        assert_eval_bool("any([False, False, False])", false);
    }

    #[test]
    fn any_empty() {
        assert_eval_bool("any([])", false);
    }

    #[test]
    fn bool_conversion() {
        assert_eval_bool("bool(1)", true);
        assert_eval_bool("bool(0)", false);
        assert_eval_bool("bool(\"hello\")", true);
        assert_eval_bool("bool(\"\")", false);
    }

    #[test]
    fn int_from_float() {
        assert_eval_int("int(3.14)", 3);
        assert_eval_int("int(-3.14)", -3);
    }

    #[test]
    fn int_from_string() {
        assert_eval_int("int(\"42\")", 42);
    }

    #[test]
    fn int_from_bool() {
        assert_eval_int("int(True)", 1);
        assert_eval_int("int(False)", 0);
    }

    #[test]
    fn float_from_int() {
        assert_eval_float("float(42)", 42.0, 0.001);
    }

    #[test]
    fn float_from_string() {
        assert_eval_float("float(\"3.14\")", 3.14, 0.001);
    }

    #[test]
    fn str_from_int() {
        assert_eval("str(42)", "42");
    }

    #[test]
    fn str_from_bool() {
        assert_eval("str(True)", "True");
    }

    #[test]
    fn str_from_none() {
        assert_eval("str(None)", "None");
    }

    #[test]
    fn len_string() {
        assert_eval_int("len(\"hello\")", 5);
    }

    #[test]
    fn len_list() {
        assert_eval_int("len([1, 2, 3])", 3);
    }

    #[test]
    fn len_dict() {
        assert_eval_int("len({\"a\": 1})", 1);
    }

    #[test]
    fn range_single_arg() {
        assert_eval_int("len(range(5))", 5);
    }

    #[test]
    fn range_two_args() {
        assert_eval_int("len(range(2, 5))", 3);
    }

    #[test]
    fn range_three_args() {
        assert_eval_int("len(range(0, 10, 2))", 5);
    }

    #[test]
    fn range_negative_step() {
        assert_eval_int("len(range(10, 0, -2))", 5);
    }

    #[test]
    fn range_zero_step_error() {
        assert_eval_error("range(10, 0, 0)");
    }

    #[test]
    fn list_from_string() {
        assert_eval_list_len("list(\"abc\")", 3);
    }

    #[test]
    fn list_from_range() {
        assert_eval_list_len("list(range(5))", 5);
    }

    #[test]
    fn min_list() {
        assert_eval_int("min([5, 1, 3])", 1);
    }

    #[test]
    fn min_args() {
        assert_eval_int("min(5, 1, 3)", 1);
    }

    #[test]
    fn max_list() {
        assert_eval_int("max([1, 5, 3])", 5);
    }

    #[test]
    fn max_args() {
        assert_eval_int("max(1, 5, 3)", 5);
    }

    #[test]
    fn sum() {
        assert_eval_int("sum([1, 2, 3])", 6);
        assert_eval_int("sum(range(5))", 10);
    }

    #[test]
    fn sorted_list() {
        assert_eval_int("sorted([3, 1, 2])[0]", 1);
    }

    #[test]
    fn reversed_list() {
        assert_eval_int("list(reversed([1, 2, 3]))[0]", 3);
    }

    #[test]
    fn enumerate_list() {
        assert_eval_list_len("list(enumerate([\"a\", \"b\"]))", 2);
    }

    #[test]
    fn zip_lists() {
        assert_eval_list_len("list(zip([1, 2], [\"a\", \"b\"]))", 2);
    }

    #[test]
    fn type_of_int() {
        assert_eval("type(42)", "int");
    }

    #[test]
    fn type_of_float() {
        assert_eval("type(3.14)", "float");
    }

    #[test]
    fn type_of_string() {
        assert_eval("type(\"hello\")", "string");
    }

    #[test]
    fn type_of_list() {
        assert_eval("type([])", "list");
    }

    #[test]
    fn type_of_dict() {
        assert_eval("type({})", "dict");
    }

    #[test]
    fn type_of_bool() {
        assert_eval("type(True)", "bool");
    }

    #[test]
    #[ignore = "starlark-rust print requires extra configuration"]
    fn print_basic() {
        assert!(run_star_code("print(\"hello\")").is_ok());
    }
}

// ============================================================================
// Expressions
// ============================================================================

mod expressions {
    use super::*;

    #[test]
    fn conditional_true() {
        assert_eval_int("1 if True else 2", 1);
    }

    #[test]
    fn conditional_false() {
        assert_eval_int("1 if False else 2", 2);
    }

    #[test]
    fn operator_precedence() {
        assert_eval_int("2 + 3 * 4", 14);
        assert_eval_int("(2 + 3) * 4", 20);
    }

    #[test]
    fn logical_and() {
        assert_eval_bool("True and True", true);
        assert_eval_bool("True and False", false);
    }

    #[test]
    fn logical_or() {
        assert_eval_bool("False or True", true);
        assert_eval_bool("False or False", false);
    }

    #[test]
    fn logical_not() {
        assert_eval_bool("not True", false);
        assert_eval_bool("not False", true);
    }

    #[test]
    fn logical_and_returns_value() {
        assert_eval_int("1 and 2", 2);
        assert_eval_int("0 and 2", 0);
    }

    #[test]
    fn logical_or_returns_value() {
        assert_eval_int("0 or 2", 2);
        assert_eval_int("1 or 2", 1);
    }

    #[test]
    #[ignore = "comparison chaining not supported in starlark-rust"]
    fn comparison_chain() {
        assert_eval_bool("1 < 2 < 3", true);
        assert_eval_bool("1 < 2 > 3", false);
    }
}

// ============================================================================
// Comprehensions
// ============================================================================

mod comprehensions {
    use super::*;

    #[test]
    fn list_basic() {
        assert_eval_int("[x * 2 for x in [1, 2, 3]][0]", 2);
        assert_eval_list_len("[x * 2 for x in [1, 2, 3]]", 3);
    }

    #[test]
    fn list_with_condition() {
        assert_eval_list_len("[x for x in [1, 2, 3, 4] if x % 2 == 0]", 2);
    }

    #[test]
    fn list_nested() {
        assert_eval_list_len("[x + y for x in [1, 2] for y in [10, 20]]", 4);
    }

    #[test]
    fn dict_basic() {
        assert_eval_int("len({str(x): x * 2 for x in [1, 2, 3]})", 3);
    }
}

// ============================================================================
// Statements
// ============================================================================

mod statements {
    use super::*;

    #[test]
    fn if_true() {
        let code = r#"
def test():
    x = 0
    if True:
        x = 1
    return x
test()
"#;
        assert_eval_int(code, 1);
    }

    #[test]
    fn if_false() {
        let code = r#"
def test():
    x = 0
    if False:
        x = 1
    return x
test()
"#;
        assert_eval_int(code, 0);
    }

    #[test]
    fn if_else() {
        let code = r#"
def test():
    x = 0
    if False:
        x = 1
    else:
        x = 2
    return x
test()
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn if_elif() {
        let code = r#"
def test():
    x = 0
    if False:
        x = 1
    elif True:
        x = 2
    else:
        x = 3
    return x
test()
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn for_loop() {
        let code = r#"
def test():
    total = 0
    for i in [1, 2, 3]:
        total += i
    return total
test()
"#;
        assert_eval_int(code, 6);
    }

    #[test]
    fn for_loop_range() {
        let code = r#"
def test():
    total = 0
    for i in range(5):
        total += i
    return total
test()
"#;
        assert_eval_int(code, 10);
    }

    #[test]
    fn for_tuple_unpacking() {
        let code = r#"
def test():
    total = 0
    for k, v in [("a", 1), ("b", 2)]:
        total += v
    return total
test()
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn break_statement() {
        let code = r#"
def test():
    total = 0
    for i in range(10):
        if i == 5:
            break
        total += i
    return total
test()
"#;
        assert_eval_int(code, 10);
    }

    #[test]
    fn continue_statement() {
        let code = r#"
def test():
    total = 0
    for i in range(5):
        if i == 2:
            continue
        total += i
    return total
test()
"#;
        assert_eval_int(code, 8);
    }

    #[test]
    fn simple_assignment() {
        let code = r#"
x = 42
x
"#;
        assert_eval_int(code, 42);
    }

    #[test]
    fn tuple_unpacking() {
        let code = r#"
a, b = (1, 2)
a + b
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn augmented_add() {
        let code = r#"
x = 1
x += 2
x
"#;
        assert_eval_int(code, 3);
    }
}

// ============================================================================
// Functions
// ============================================================================

mod functions {
    use super::*;

    #[test]
    fn definition() {
        let code = r#"
def add(a, b):
    return a + b
add(1, 2)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn default_args() {
        let code = r#"
def greet(name, greeting="Hello"):
    return greeting + " " + name
greet("World")
"#;
        assert_eval(code, "Hello World");
    }

    #[test]
    fn keyword_args() {
        let code = r#"
def greet(name, greeting):
    return greeting + " " + name
greet(greeting="Hi", name="World")
"#;
        assert_eval(code, "Hi World");
    }

    #[test]
    fn no_return() {
        let code = r#"
def f():
    x = 1
f()
"#;
        assert_eval(code, "None");
    }

    #[test]
    fn early_return() {
        let code = r#"
def f(x):
    if x < 0:
        return -1
    return 1
f(-5)
"#;
        assert_eval_int(code, -1);
    }

    #[test]
    fn lambda_expression() {
        let code = r#"
f = lambda x: x * 2
f(5)
"#;
        assert_eval_int(code, 10);
    }

    #[test]
    fn lambda_multiple_args() {
        let code = r#"
add = lambda a, b: a + b
add(3, 4)
"#;
        assert_eval_int(code, 7);
    }

    #[test]
    fn closure() {
        let code = r#"
def make_adder(n):
    def adder(x):
        return x + n
    return adder
add5 = make_adder(5)
add5(10)
"#;
        assert_eval_int(code, 15);
    }
}

// ============================================================================
// Indexing and Slicing
// ============================================================================

mod indexing {
    use super::*;

    #[test]
    fn list_negative() {
        assert_eval_int("[1, 2, 3, 4, 5][-1]", 5);
    }

    #[test]
    fn string_negative() {
        assert_eval("\"hello\"[-1]", "o");
    }

    #[test]
    fn slice_start_only() {
        assert_eval_list_len("[1, 2, 3, 4, 5][2:]", 3);
    }

    #[test]
    fn slice_end_only() {
        assert_eval_list_len("[1, 2, 3, 4, 5][:3]", 3);
    }

    #[test]
    fn slice_with_step() {
        assert_eval_list_len("[1, 2, 3, 4, 5][::2]", 3);
    }

    #[test]
    fn slice_negative_step() {
        assert_eval_list_len("[1, 2, 3, 4, 5][::-1]", 5);
    }

    #[test]
    fn slice_negative_indices() {
        assert_eval_list_len("[1, 2, 3, 4, 5][-3:-1]", 2);
    }

    #[test]
    fn index_out_of_bounds() {
        assert_eval_error("[1, 2, 3][10]");
    }

    #[test]
    fn slice_out_of_bounds_ok() {
        assert_eval_list_len("[1, 2, 3][1:100]", 2);
    }
}

// ============================================================================
// Data Types - Tuples
// ============================================================================

mod tuples {
    use super::*;

    #[test]
    fn literal() {
        assert_eval_int("len((1, 2, 3))", 3);
    }

    #[test]
    fn empty() {
        assert_eval_int("len(())", 0);
    }

    #[test]
    fn singleton_with_comma() {
        assert_eval_int("len((1,))", 1);
    }

    #[test]
    fn indexing() {
        assert_eval_int("(1, 2, 3)[0]", 1);
        assert_eval_int("(1, 2, 3)[2]", 3);
    }

    #[test]
    fn negative_indexing() {
        assert_eval_int("(1, 2, 3)[-1]", 3);
        assert_eval_int("(1, 2, 3)[-3]", 1);
    }

    #[test]
    fn slicing() {
        assert_eval_int("len((1, 2, 3, 4)[1:3])", 2);
    }

    #[test]
    fn concatenation() {
        assert_eval_int("len((1, 2) + (3, 4))", 4);
    }

    #[test]
    fn repetition() {
        assert_eval_int("len((1, 2) * 3)", 6);
    }

    #[test]
    fn membership() {
        assert_eval_bool("1 in (1, 2, 3)", true);
        assert_eval_bool("4 in (1, 2, 3)", false);
    }

    #[test]
    fn empty_is_falsy() {
        assert_eval_bool("bool(())", false);
    }

    #[test]
    fn nonempty_is_truthy() {
        assert_eval_bool("bool((1,))", true);
    }

    #[test]
    fn comparison() {
        assert_eval_bool("(1, 2) == (1, 2)", true);
        assert_eval_bool("(1, 2) != (1, 3)", true);
        assert_eval_bool("(1, 2) < (1, 3)", true);
    }

    #[test]
    fn tuple_function() {
        assert_eval_int("len(tuple([1, 2, 3]))", 3);
    }

    #[test]
    fn tuple_from_range() {
        assert_eval_int("len(tuple(range(5)))", 5);
    }

    #[test]
    fn tuple_empty() {
        assert_eval_int("len(tuple())", 0);
    }

    #[test]
    fn type_is_tuple() {
        assert_eval("type((1, 2))", "tuple");
    }

    #[test]
    fn immutable() {
        let code = r#"
x = (1, 2, 3)
x[0] = 10
"#;
        assert_eval_error(code);
    }
}

// ============================================================================
// Additional Built-in Functions
// ============================================================================

mod additional_builtins {
    use super::*;

    #[test]
    fn dir_on_string() {
        let code = "len(dir(\"hello\"))";
        let result = run_star_code(code).expect("evaluation failed");
        let len: i64 = result.parse().expect("expected int");
        assert!(len > 0, "dir should return non-empty list for string");
    }

    #[test]
    fn dir_on_list() {
        let code = "len(dir([]))";
        let result = run_star_code(code).expect("evaluation failed");
        let len: i64 = result.parse().expect("expected int");
        assert!(len > 0, "dir should return non-empty list for list");
    }

    #[test]
    fn dir_on_dict() {
        let code = "len(dir({}))";
        let result = run_star_code(code).expect("evaluation failed");
        let len: i64 = result.parse().expect("expected int");
        assert!(len > 0, "dir should return non-empty list for dict");
    }

    #[test]
    fn hasattr_true() {
        assert_eval_bool("hasattr(\"hello\", \"upper\")", true);
    }

    #[test]
    fn hasattr_false() {
        assert_eval_bool("hasattr(\"hello\", \"nonexistent\")", false);
    }

    #[test]
    fn getattr_method() {
        assert_eval("getattr(\"hello\", \"upper\")()", "HELLO");
    }

    #[test]
    fn getattr_default() {
        assert_eval_int("getattr(\"hello\", \"nonexistent\", 42)", 42);
    }

    #[test]
    fn getattr_missing_error() {
        assert_eval_error("getattr(\"hello\", \"nonexistent\")");
    }

    #[test]
    fn hash_string() {
        let code = "hash(\"hello\")";
        let result = run_star_code(code);
        assert!(result.is_ok(), "hash should work on strings");
    }

    #[test]
    fn hash_same_strings_equal() {
        assert_eval_bool("hash(\"abc\") == hash(\"abc\")", true);
    }

    #[test]
    fn hash_different_strings_differ() {
        assert_eval_bool("hash(\"abc\") != hash(\"xyz\")", true);
    }

    #[test]
    fn repr_int() {
        assert_eval("repr(42)", "42");
    }

    #[test]
    #[ignore = "starlark-rust repr does not quote strings"]
    fn repr_string() {
        assert_eval("repr(\"hello\")", "\"hello\"");
    }

    #[test]
    fn repr_list() {
        assert_eval("repr([1, 2])", "[1, 2]");
    }

    #[test]
    fn repr_bool() {
        assert_eval("repr(True)", "True");
    }

    #[test]
    fn dict_constructor_empty() {
        assert_eval_int("len(dict())", 0);
    }

    #[test]
    fn dict_constructor_from_pairs() {
        assert_eval_int("len(dict([(\"a\", 1), (\"b\", 2)]))", 2);
    }

    #[test]
    fn dict_constructor_kwargs() {
        assert_eval_int("dict(a=1, b=2)[\"a\"]", 1);
    }

    #[test]
    fn dict_constructor_combined() {
        let code = r#"
d = dict([("x", 1)], y=2)
d["x"] + d["y"]
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn fail_causes_error() {
        assert_eval_error("fail(\"oops\")");
    }

    #[test]
    fn fail_multiple_args() {
        assert_eval_error("fail(\"error\", 1, 2)");
    }

    #[test]
    fn int_with_base() {
        assert_eval_int("int(\"ff\", 16)", 255);
    }

    #[test]
    fn int_with_base_zero() {
        assert_eval_int("int(\"0xff\", 0)", 255);
    }

    #[test]
    fn int_with_base_binary() {
        assert_eval_int("int(\"1010\", 2)", 10);
    }

    #[test]
    fn enumerate_with_start() {
        let code = "list(enumerate([\"a\", \"b\"], 1))[0][0]";
        assert_eval_int(code, 1);
    }

    #[test]
    fn max_with_key() {
        assert_eval("max([\"a\", \"bb\", \"ccc\"], key=len)", "ccc");
    }

    #[test]
    fn min_with_key() {
        assert_eval("min([\"aaa\", \"bb\", \"c\"], key=len)", "c");
    }

    #[test]
    fn sorted_reverse() {
        assert_eval_int("sorted([1, 3, 2], reverse=True)[0]", 3);
    }

    #[test]
    fn sorted_with_key() {
        assert_eval("sorted([\"bb\", \"a\", \"ccc\"], key=len)[0]", "a");
    }
}

// ============================================================================
// Additional String Methods
// ============================================================================

mod additional_string_methods {
    use super::*;

    #[test]
    fn capitalize() {
        assert_eval("\"hello world\".capitalize()", "Hello world");
    }

    #[test]
    fn count() {
        assert_eval_int("\"banana\".count(\"a\")", 3);
    }

    #[test]
    fn count_with_range() {
        assert_eval_int("\"banana\".count(\"a\", 2)", 2);
    }

    #[test]
    fn index() {
        assert_eval_int("\"hello\".index(\"l\")", 2);
    }

    #[test]
    fn index_not_found_error() {
        assert_eval_error("\"hello\".index(\"x\")");
    }

    #[test]
    fn isalnum_true() {
        assert_eval_bool("\"abc123\".isalnum()", true);
    }

    #[test]
    fn isalnum_false() {
        assert_eval_bool("\"abc 123\".isalnum()", false);
    }

    #[test]
    fn isalpha_true() {
        assert_eval_bool("\"hello\".isalpha()", true);
    }

    #[test]
    fn isalpha_false() {
        assert_eval_bool("\"hello1\".isalpha()", false);
    }

    #[test]
    fn isdigit_true() {
        assert_eval_bool("\"123\".isdigit()", true);
    }

    #[test]
    fn isdigit_false() {
        assert_eval_bool("\"12a\".isdigit()", false);
    }

    #[test]
    fn islower_true() {
        assert_eval_bool("\"hello\".islower()", true);
    }

    #[test]
    fn islower_false() {
        assert_eval_bool("\"Hello\".islower()", false);
    }

    #[test]
    fn isupper_true() {
        assert_eval_bool("\"HELLO\".isupper()", true);
    }

    #[test]
    fn isupper_false() {
        assert_eval_bool("\"Hello\".isupper()", false);
    }

    #[test]
    fn isspace_true() {
        assert_eval_bool("\"   \".isspace()", true);
    }

    #[test]
    fn isspace_false() {
        assert_eval_bool("\" a \".isspace()", false);
    }

    #[test]
    fn istitle_true() {
        assert_eval_bool("\"Hello World\".istitle()", true);
    }

    #[test]
    fn istitle_false() {
        assert_eval_bool("\"hello world\".istitle()", false);
    }

    #[test]
    fn title() {
        assert_eval("\"hello world\".title()", "Hello World");
    }

    #[test]
    fn lstrip() {
        assert_eval("\"  hello\".lstrip()", "hello");
    }

    #[test]
    fn rstrip() {
        assert_eval("\"hello  \".rstrip()", "hello");
    }

    #[test]
    fn lstrip_chars() {
        assert_eval("\"xxyhello\".lstrip(\"xy\")", "hello");
    }

    #[test]
    fn rstrip_chars() {
        assert_eval("\"helloxyy\".rstrip(\"xy\")", "hello");
    }

    #[test]
    fn partition() {
        let code = "\"hello-world\".partition(\"-\")[0]";
        assert_eval(code, "hello");
    }

    #[test]
    fn partition_middle() {
        let code = "\"hello-world\".partition(\"-\")[1]";
        assert_eval(code, "-");
    }

    #[test]
    fn partition_end() {
        let code = "\"hello-world\".partition(\"-\")[2]";
        assert_eval(code, "world");
    }

    #[test]
    fn rpartition() {
        let code = "\"hello-world-foo\".rpartition(\"-\")[0]";
        assert_eval(code, "hello-world");
    }

    #[test]
    fn rfind() {
        assert_eval_int("\"hello\".rfind(\"l\")", 3);
    }

    #[test]
    fn rfind_not_found() {
        assert_eval_int("\"hello\".rfind(\"x\")", -1);
    }

    #[test]
    fn rindex() {
        assert_eval_int("\"hello\".rindex(\"l\")", 3);
    }

    #[test]
    fn rindex_not_found_error() {
        assert_eval_error("\"hello\".rindex(\"x\")");
    }

    #[test]
    fn rsplit() {
        assert_eval_int("len(\"a-b-c\".rsplit(\"-\"))", 3);
    }

    #[test]
    fn rsplit_maxsplit() {
        assert_eval_int("len(\"a-b-c\".rsplit(\"-\", 1))", 2);
    }

    #[test]
    fn split_maxsplit() {
        assert_eval_int("len(\"a-b-c\".split(\"-\", 1))", 2);
    }

    #[test]
    fn splitlines() {
        assert_eval_int("len(\"a\\nb\\nc\".splitlines())", 3);
    }

    #[test]
    fn removeprefix() {
        assert_eval("\"hello world\".removeprefix(\"hello \")", "world");
    }

    #[test]
    fn removeprefix_no_match() {
        assert_eval("\"hello world\".removeprefix(\"foo\")", "hello world");
    }

    #[test]
    fn removesuffix() {
        assert_eval("\"hello world\".removesuffix(\" world\")", "hello");
    }

    #[test]
    fn removesuffix_no_match() {
        assert_eval("\"hello world\".removesuffix(\"foo\")", "hello world");
    }

    #[test]
    fn format_basic() {
        assert_eval("\"{} {}\".format(\"hello\", \"world\")", "hello world");
    }

    #[test]
    fn format_positional() {
        assert_eval("\"{0} {1}\".format(\"hello\", \"world\")", "hello world");
    }

    #[test]
    fn format_named() {
        assert_eval("\"{name}\".format(name=\"world\")", "world");
    }

    #[test]
    fn elems() {
        assert_eval_int("len(list(\"abc\".elems()))", 3);
    }
}

// ============================================================================
// Additional Dict Methods
// ============================================================================

mod additional_dict_methods {
    use super::*;

    #[test]
    fn clear() {
        let code = r#"
d = {"a": 1, "b": 2}
d.clear()
len(d)
"#;
        assert_eval_int(code, 0);
    }

    #[test]
    fn items() {
        assert_eval_int("len({\"a\": 1, \"b\": 2}.items())", 2);
    }

    #[test]
    fn pop_existing() {
        let code = r#"
d = {"a": 1, "b": 2}
d.pop("a")
"#;
        assert_eval_int(code, 1);
    }

    #[test]
    fn pop_with_default() {
        let code = r#"
d = {"a": 1}
d.pop("b", 42)
"#;
        assert_eval_int(code, 42);
    }

    #[test]
    fn pop_missing_error() {
        let code = r#"
d = {"a": 1}
d.pop("b")
"#;
        assert_eval_error(code);
    }

    #[test]
    fn popitem() {
        let code = r#"
d = {"a": 1}
len(d.popitem())
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn popitem_empty_error() {
        let code = r#"
d = {}
d.popitem()
"#;
        assert_eval_error(code);
    }

    #[test]
    fn setdefault_missing() {
        let code = r#"
d = {"a": 1}
d.setdefault("b", 2)
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn setdefault_existing() {
        let code = r#"
d = {"a": 1}
d.setdefault("a", 2)
"#;
        assert_eval_int(code, 1);
    }

    #[test]
    fn setdefault_inserts() {
        let code = r#"
d = {"a": 1}
d.setdefault("b", 2)
d["b"]
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn update_from_dict() {
        let code = r#"
d = {"a": 1}
d.update({"b": 2})
len(d)
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn update_from_pairs() {
        let code = r#"
d = {"a": 1}
d.update([("b", 2)])
len(d)
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn update_kwargs() {
        let code = r#"
d = {"a": 1}
d.update(b=2)
len(d)
"#;
        assert_eval_int(code, 2);
    }
}

// ============================================================================
// Additional List Methods
// ============================================================================

mod additional_list_methods {
    use super::*;

    #[test]
    fn clear() {
        let code = r#"
x = [1, 2, 3]
x.clear()
len(x)
"#;
        assert_eval_int(code, 0);
    }

    #[test]
    fn insert() {
        let code = r#"
x = [1, 3]
x.insert(1, 2)
x[1]
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn insert_at_end() {
        let code = r#"
x = [1, 2]
x.insert(100, 3)
x[-1]
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn remove() {
        let code = r#"
x = [1, 2, 3, 2]
x.remove(2)
len(x)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn remove_first_occurrence() {
        let code = r#"
x = [1, 2, 3, 2]
x.remove(2)
x[1]
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn remove_not_found_error() {
        let code = r#"
x = [1, 2, 3]
x.remove(4)
"#;
        assert_eval_error(code);
    }

    #[test]
    fn pop_at_index() {
        let code = r#"
x = [1, 2, 3]
x.pop(0)
"#;
        assert_eval_int(code, 1);
    }

    #[test]
    fn pop_negative_index() {
        let code = r#"
x = [1, 2, 3]
x.pop(-2)
"#;
        assert_eval_int(code, 2);
    }
}

// ============================================================================
// String Interpolation (%)
// ============================================================================

mod string_interpolation {
    use super::*;

    #[test]
    fn simple_string() {
        assert_eval("\"Hello %s\" % \"World\"", "Hello World");
    }

    #[test]
    fn multiple_values() {
        assert_eval("\"%s %s\" % (\"hello\", \"world\")", "hello world");
    }

    #[test]
    fn integer_d() {
        assert_eval("\"value: %d\" % 42", "value: 42");
    }

    #[test]
    #[ignore = "starlark-rust %r does not quote strings"]
    fn repr_r() {
        assert_eval("\"%r\" % \"hello\"", "\"hello\"");
    }

    #[test]
    fn hex_x() {
        assert_eval("\"%x\" % 255", "ff");
    }

    #[test]
    fn hex_upper_x() {
        assert_eval("\"%X\" % 255", "FF");
    }

    #[test]
    fn octal_o() {
        assert_eval("\"%o\" % 8", "10");
    }

    #[test]
    #[ignore = "starlark-rust does not process %% escape in string literals"]
    fn percent_escape() {
        assert_eval("\"100%%\"", "100%");
    }

    #[test]
    fn float_f() {
        let code = "\"%f\" % 3.14";
        let result = run_star_code(code).expect("evaluation failed");
        assert!(result.contains("3.14"), "Expected 3.14 in {}", result);
    }

    #[test]
    fn float_e() {
        let code = "\"%e\" % 1000.0";
        let result = run_star_code(code).expect("evaluation failed");
        assert!(result.to_lowercase().contains("e"), "Expected scientific notation in {}", result);
    }
}

// ============================================================================
// Variadic Functions
// ============================================================================

mod variadic_functions {
    use super::*;

    #[test]
    fn args_basic() {
        let code = r#"
def f(*args):
    return len(args)
f(1, 2, 3)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn args_empty() {
        let code = r#"
def f(*args):
    return len(args)
f()
"#;
        assert_eval_int(code, 0);
    }

    #[test]
    fn args_with_regular() {
        let code = r#"
def f(x, *args):
    return x + len(args)
f(10, 1, 2, 3)
"#;
        assert_eval_int(code, 13);
    }

    #[test]
    fn kwargs_basic() {
        let code = r#"
def f(**kwargs):
    return len(kwargs)
f(a=1, b=2)
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn kwargs_empty() {
        let code = r#"
def f(**kwargs):
    return len(kwargs)
f()
"#;
        assert_eval_int(code, 0);
    }

    #[test]
    fn kwargs_with_regular() {
        let code = r#"
def f(x, **kwargs):
    return x + len(kwargs)
f(10, a=1, b=2)
"#;
        assert_eval_int(code, 12);
    }

    #[test]
    fn args_and_kwargs() {
        let code = r#"
def f(*args, **kwargs):
    return len(args) + len(kwargs)
f(1, 2, a=3, b=4)
"#;
        assert_eval_int(code, 4);
    }

    #[test]
    fn splat_call() {
        let code = r#"
def f(a, b, c):
    return a + b + c
f(*[1, 2, 3])
"#;
        assert_eval_int(code, 6);
    }

    #[test]
    fn double_splat_call() {
        let code = r#"
def f(a, b):
    return a + b
f(**{"a": 1, "b": 2})
"#;
        assert_eval_int(code, 3);
    }
}

// ============================================================================
// Blueprint Extended Dialect Features
// ============================================================================

mod extended_features {
    use super::*;

    #[test]
    fn struct_creation() {
        let code = r#"
s = struct(x=1, y=2)
s.x + s.y
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn struct_type() {
        let code = "type(struct(a=1))";
        let result = run_star_code(code).expect("evaluation failed");
        assert!(result.contains("struct"), "Expected struct type, got: {}", result);
    }

    #[test]
    #[ignore = "starlark-rust enum requires different syntax"]
    fn enum_type_definition() {
        let code = r#"
Color = enum("Color", "RED", "GREEN", "BLUE")
Color.RED
"#;
        let result = run_star_code(code);
        assert!(result.is_ok(), "enum should work: {:?}", result);
    }

    #[test]
    fn map_function() {
        let code = r#"
list(map(lambda x: x * 2, [1, 2, 3]))[0]
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    #[ignore = "starlark-rust map does not support multiple iterables like Python"]
    fn map_with_multiple_iterables() {
        let code = r#"
list(map(lambda x, y: x + y, [1, 2], [10, 20]))[0]
"#;
        assert_eval_int(code, 11);
    }

    #[test]
    fn filter_function() {
        let code = r#"
len(list(filter(lambda x: x > 2, [1, 2, 3, 4])))
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn filter_returns_matching() {
        let code = r#"
list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4]))[0]
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn partial_function() {
        let code = r#"
def add(a, b):
    return a + b
add5 = partial(add, 5)
add5(3)
"#;
        assert_eval_int(code, 8);
    }

    #[test]
    fn partial_with_kwargs() {
        let code = r#"
def greet(name, greeting):
    return greeting + " " + name
say_hello = partial(greet, greeting="Hello")
say_hello("World")
"#;
        assert_eval(code, "Hello World");
    }

    #[test]
    fn json_encode() {
        let code = r#"json.encode({"a": 1})"#;
        let result = run_star_code(code).expect("evaluation failed");
        assert!(result.contains("a") && result.contains("1"), "Expected JSON, got: {}", result);
    }

    #[test]
    fn json_decode() {
        let code = r#"json.decode("{\"x\": 42}")["x"]"#;
        assert_eval_int(code, 42);
    }

    #[test]
    fn json_roundtrip() {
        let code = r#"
data = {"key": "value", "num": 123}
decoded = json.decode(json.encode(data))
decoded["num"]
"#;
        assert_eval_int(code, 123);
    }
}

// ============================================================================
// Pass Statement
// ============================================================================

mod pass_statement {
    use super::*;

    #[test]
    fn pass_in_function() {
        let code = r#"
def noop():
    pass
noop()
"#;
        assert_eval(code, "None");
    }

    #[test]
    fn pass_in_if() {
        let code = r#"
def f(x):
    if x > 0:
        pass
    else:
        return -1
    return 1
f(5)
"#;
        assert_eval_int(code, 1);
    }

    #[test]
    fn pass_in_for() {
        let code = r#"
def count_positive(items):
    count = 0
    for x in items:
        if x <= 0:
            pass
        else:
            count += 1
    return count
count_positive([1, -2, 3, -4, 5])
"#;
        assert_eval_int(code, 3);
    }
}

mod algorithms {
    use super::*;

    // ==================== SORTING ALGORITHMS ====================
    // Note: Starlark has no `while` loops, only `for` with range()
    // Dict keys must be strings

    #[test]
    fn bubble_sort() {
        let code = r#"
def bubble_sort(arr):
    n = len(arr)
    for i in range(n):
        for j in range(0, n - i - 1):
            if arr[j] > arr[j + 1]:
                temp = arr[j]
                arr[j] = arr[j + 1]
                arr[j + 1] = temp
    return arr
bubble_sort([5, 3, 1, 4, 2])
"#;
        assert_eval(code, "[1, 2, 3, 4, 5]");
    }

    #[test]
    fn selection_sort() {
        let code = r#"
def selection_sort(arr):
    n = len(arr)
    for i in range(n):
        min_idx = i
        for j in range(i + 1, n):
            if arr[j] < arr[min_idx]:
                min_idx = j
        temp = arr[i]
        arr[i] = arr[min_idx]
        arr[min_idx] = temp
    return arr
selection_sort([5, 3, 1, 4, 2])
"#;
        assert_eval(code, "[1, 2, 3, 4, 5]");
    }

    #[test]
    fn insertion_sort() {
        let code = r#"
def insertion_sort(arr):
    for i in range(1, len(arr)):
        key = arr[i]
        j = i - 1
        for _ in range(i):
            if j >= 0 and arr[j] > key:
                arr[j + 1] = arr[j]
                j -= 1
            else:
                break
        arr[j + 1] = key
    return arr
insertion_sort([5, 3, 1, 4, 2])
"#;
        assert_eval(code, "[1, 2, 3, 4, 5]");
    }

    #[test]
    fn merge_sort() {
        let code = r#"
def merge_sort(arr):
    if len(arr) <= 1:
        return arr
    mid = len(arr) // 2
    left = merge_sort(arr[:mid])
    right = merge_sort(arr[mid:])
    return merge(left, right)

def merge(left, right):
    result = []
    i = 0
    j = 0
    for _ in range(len(left) + len(right)):
        if i < len(left) and j < len(right):
            if left[i] <= right[j]:
                result.append(left[i])
                i += 1
            else:
                result.append(right[j])
                j += 1
        elif i < len(left):
            result.append(left[i])
            i += 1
        elif j < len(right):
            result.append(right[j])
            j += 1
    return result

merge_sort([5, 3, 1, 4, 2])
"#;
        assert_eval(code, "[1, 2, 3, 4, 5]");
    }

    #[test]
    fn quick_sort() {
        let code = r#"
def quick_sort(arr):
    if len(arr) <= 1:
        return arr
    pivot = arr[len(arr) // 2]
    left = [x for x in arr if x < pivot]
    middle = [x for x in arr if x == pivot]
    right = [x for x in arr if x > pivot]
    return quick_sort(left) + middle + quick_sort(right)

quick_sort([5, 3, 1, 4, 2])
"#;
        assert_eval(code, "[1, 2, 3, 4, 5]");
    }

    #[test]
    fn counting_sort() {
        let code = r#"
def counting_sort(arr):
    if len(arr) == 0:
        return arr
    max_val = arr[0]
    for x in arr:
        if x > max_val:
            max_val = x
    count = [0] * (max_val + 1)
    for x in arr:
        count[x] += 1
    result = []
    for i in range(len(count)):
        for j in range(count[i]):
            result.append(i)
    return result

counting_sort([4, 2, 1, 3, 1])
"#;
        assert_eval(code, "[1, 1, 2, 3, 4]");
    }

    // ==================== SEARCHING ALGORITHMS ====================

    #[test]
    fn linear_search() {
        let code = r#"
def linear_search(arr, target):
    for i in range(len(arr)):
        if arr[i] == target:
            return i
    return -1

linear_search([10, 20, 30, 40, 50], 30)
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn binary_search() {
        let code = r#"
def binary_search(arr, target):
    left = 0
    right = len(arr) - 1
    result = -1
    for _ in range(len(arr) + 1):
        if left > right:
            break
        mid = (left + right) // 2
        if arr[mid] == target:
            result = mid
            break
        elif arr[mid] < target:
            left = mid + 1
        else:
            right = mid - 1
    return result

binary_search([10, 20, 30, 40, 50], 40)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn binary_search_recursive() {
        let code = r#"
def binary_search_rec(arr, target, left, right):
    if left > right:
        return -1
    mid = (left + right) // 2
    if arr[mid] == target:
        return mid
    elif arr[mid] < target:
        return binary_search_rec(arr, target, mid + 1, right)
    else:
        return binary_search_rec(arr, target, left, mid - 1)

arr = [10, 20, 30, 40, 50]
binary_search_rec(arr, 40, 0, len(arr) - 1)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn find_min_max() {
        let code = r#"
def find_min_max(arr):
    if len(arr) == 0:
        return [0, 0]
    min_val = arr[0]
    max_val = arr[0]
    for x in arr:
        if x < min_val:
            min_val = x
        if x > max_val:
            max_val = x
    return [min_val, max_val]

find_min_max([3, 1, 9, 4, 7, 2])
"#;
        assert_eval(code, "[1, 9]");
    }

    // ==================== DYNAMIC PROGRAMMING ====================

    #[test]
    fn fibonacci_dp() {
        let code = r#"
def fibonacci(n):
    if n <= 1:
        return n
    dp = [0] * (n + 1)
    dp[1] = 1
    for i in range(2, n + 1):
        dp[i] = dp[i - 1] + dp[i - 2]
    return dp[n]

fibonacci(10)
"#;
        assert_eval_int(code, 55);
    }

    #[test]
    fn factorial_iterative() {
        let code = r#"
def factorial(n):
    result = 1
    for i in range(2, n + 1):
        result *= i
    return result

factorial(5)
"#;
        assert_eval_int(code, 120);
    }

    #[test]
    fn longest_common_subsequence() {
        let code = r#"
def lcs(s1, s2):
    m = len(s1)
    n = len(s2)
    dp = []
    for i in range(m + 1):
        row = [0] * (n + 1)
        dp.append(row)
    for i in range(1, m + 1):
        for j in range(1, n + 1):
            if s1[i - 1] == s2[j - 1]:
                dp[i][j] = dp[i - 1][j - 1] + 1
            else:
                dp[i][j] = dp[i - 1][j] if dp[i - 1][j] > dp[i][j - 1] else dp[i][j - 1]
    return dp[m][n]

lcs("ABCDGH", "AEDFHR")
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn coin_change() {
        let code = r#"
def coin_change(coins, amount):
    dp = [amount + 1] * (amount + 1)
    dp[0] = 0
    for i in range(1, amount + 1):
        for coin in coins:
            if coin <= i:
                if dp[i - coin] + 1 < dp[i]:
                    dp[i] = dp[i - coin] + 1
    if dp[amount] > amount:
        return -1
    return dp[amount]

coin_change([1, 2, 5], 11)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn knapsack_01() {
        let code = r#"
def knapsack(weights, values, capacity):
    n = len(weights)
    dp = []
    for i in range(n + 1):
        row = [0] * (capacity + 1)
        dp.append(row)
    for i in range(1, n + 1):
        for w in range(capacity + 1):
            if weights[i - 1] <= w:
                include = values[i - 1] + dp[i - 1][w - weights[i - 1]]
                exclude = dp[i - 1][w]
                dp[i][w] = include if include > exclude else exclude
            else:
                dp[i][w] = dp[i - 1][w]
    return dp[n][capacity]

knapsack([10, 20, 30], [60, 100, 120], 50)
"#;
        assert_eval_int(code, 220);
    }

    #[test]
    fn edit_distance() {
        let code = r#"
def edit_distance(s1, s2):
    m = len(s1)
    n = len(s2)
    dp = []
    for i in range(m + 1):
        row = [0] * (n + 1)
        dp.append(row)
    for i in range(m + 1):
        dp[i][0] = i
    for j in range(n + 1):
        dp[0][j] = j
    for i in range(1, m + 1):
        for j in range(1, n + 1):
            if s1[i - 1] == s2[j - 1]:
                dp[i][j] = dp[i - 1][j - 1]
            else:
                insert = dp[i][j - 1] + 1
                delete = dp[i - 1][j] + 1
                replace = dp[i - 1][j - 1] + 1
                min_val = insert
                if delete < min_val:
                    min_val = delete
                if replace < min_val:
                    min_val = replace
                dp[i][j] = min_val
    return dp[m][n]

edit_distance("kitten", "sitting")
"#;
        assert_eval_int(code, 3);
    }

    // ==================== GRAPH ALGORITHMS ====================

    #[test]
    fn bfs_traversal() {
        let code = r#"
def bfs(graph, start):
    visited = []
    queue = [start]
    for _ in range(100):
        if len(queue) == 0:
            break
        node = queue[0]
        queue = queue[1:]
        if node not in visited:
            visited.append(node)
            for neighbor in graph[node]:
                if neighbor not in visited:
                    queue.append(neighbor)
    return visited

graph = {"a": ["b", "c"], "b": ["a", "d", "e"], "c": ["a", "e"], "d": ["b"], "e": ["b", "c"]}
bfs(graph, "a")
"#;
        assert_eval(code, r#"["a", "b", "c", "d", "e"]"#);
    }

    #[test]
    fn dfs_traversal() {
        let code = r#"
def dfs(graph, start, visited):
    if start in visited:
        return visited
    visited.append(start)
    for neighbor in graph[start]:
        dfs(graph, neighbor, visited)
    return visited

graph = {"a": ["b", "c"], "b": ["a", "d"], "c": ["a"], "d": ["b"]}
dfs(graph, "a", [])
"#;
        assert_eval(code, r#"["a", "b", "d", "c"]"#);
    }

    #[test]
    fn detect_cycle() {
        let code = r#"
def has_cycle_dfs(graph, node, visited, rec_stack):
    visited[node] = True
    rec_stack[node] = True
    for neighbor in graph[node]:
        if not visited[neighbor]:
            if has_cycle_dfs(graph, neighbor, visited, rec_stack):
                return True
        elif rec_stack[neighbor]:
            return True
    rec_stack[node] = False
    return False

def has_cycle(graph, nodes):
    visited = {}
    rec_stack = {}
    for n in nodes:
        visited[n] = False
        rec_stack[n] = False
    for n in nodes:
        if not visited[n]:
            if has_cycle_dfs(graph, n, visited, rec_stack):
                return True
    return False

graph = {"a": ["b"], "b": ["c"], "c": ["a"]}
has_cycle(graph, ["a", "b", "c"])
"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn topological_sort() {
        let code = r#"
def topological_sort(graph, nodes):
    in_degree = {}
    for n in nodes:
        in_degree[n] = 0
    for node in graph:
        for neighbor in graph[node]:
            in_degree[neighbor] += 1

    queue = []
    for n in nodes:
        if in_degree[n] == 0:
            queue.append(n)

    result = []
    for _ in range(len(nodes)):
        if len(queue) == 0:
            break
        node = queue[0]
        queue = queue[1:]
        result.append(node)
        for neighbor in graph[node]:
            in_degree[neighbor] -= 1
            if in_degree[neighbor] == 0:
                queue.append(neighbor)

    return result

graph = {"a": ["b", "c"], "b": ["d"], "c": ["d"], "d": []}
topological_sort(graph, ["a", "b", "c", "d"])
"#;
        assert_eval(code, r#"["a", "b", "c", "d"]"#);
    }

    #[test]
    fn shortest_path_dijkstra() {
        let code = r#"
def dijkstra(graph, start, nodes):
    dist = {}
    visited = {}
    for n in nodes:
        dist[n] = 999999
        visited[n] = False
    dist[start] = 0

    for _ in range(len(nodes)):
        u = ""
        min_dist = 999999
        for n in nodes:
            if not visited[n] and dist[n] < min_dist:
                min_dist = dist[n]
                u = n

        if u == "":
            break

        visited[u] = True
        for edge in graph[u]:
            v = edge[0]
            weight = edge[1]
            if dist[u] + weight < dist[v]:
                dist[v] = dist[u] + weight

    return [dist["a"], dist["b"], dist["c"]]

graph = {"a": [["b", 1], ["c", 4]], "b": [["c", 2]], "c": []}
dijkstra(graph, "a", ["a", "b", "c"])
"#;
        assert_eval(code, "[0, 1, 3]");
    }

    // ==================== DATA STRUCTURE OPERATIONS ====================

    #[test]
    fn stack_operations() {
        let code = r#"
def test_stack():
    stack = []
    stack.append(1)
    stack.append(2)
    stack.append(3)
    top = stack.pop()
    stack.append(4)
    return stack[len(stack) - 1]

test_stack()
"#;
        assert_eval_int(code, 4);
    }

    #[test]
    fn queue_operations() {
        let code = r#"
def test_queue():
    queue = []
    queue.append(1)
    queue.append(2)
    queue.append(3)
    first = queue[0]
    queue = queue[1:]
    queue.append(4)
    return queue[0]

test_queue()
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn priority_queue_min_heap() {
        let code = r#"
def heapify_up(heap, idx):
    for _ in range(len(heap)):
        if idx <= 0:
            break
        parent = (idx - 1) // 2
        if heap[idx] < heap[parent]:
            temp = heap[idx]
            heap[idx] = heap[parent]
            heap[parent] = temp
            idx = parent
        else:
            break

def heap_push(heap, val):
    heap.append(val)
    heapify_up(heap, len(heap) - 1)

def heapify_down(heap, idx):
    n = len(heap)
    for _ in range(n):
        smallest = idx
        left = 2 * idx + 1
        right = 2 * idx + 2
        if left < n and heap[left] < heap[smallest]:
            smallest = left
        if right < n and heap[right] < heap[smallest]:
            smallest = right
        if smallest != idx:
            temp = heap[idx]
            heap[idx] = heap[smallest]
            heap[smallest] = temp
            idx = smallest
        else:
            break

def heap_pop(heap):
    if len(heap) == 0:
        return None
    val = heap[0]
    heap[0] = heap[len(heap) - 1]
    heap.pop()
    if len(heap) > 0:
        heapify_down(heap, 0)
    return val

heap = []
heap_push(heap, 3)
heap_push(heap, 1)
heap_push(heap, 2)
result = []
result.append(heap_pop(heap))
result.append(heap_pop(heap))
result.append(heap_pop(heap))
result
"#;
        assert_eval(code, "[1, 2, 3]");
    }

    #[test]
    fn linked_list_reverse() {
        let code = r#"
def reverse_list(arr):
    result = []
    for i in range(len(arr) - 1, -1, -1):
        result.append(arr[i])
    return result

reverse_list([1, 2, 3, 4, 5])
"#;
        assert_eval(code, "[5, 4, 3, 2, 1]");
    }

    #[test]
    fn binary_tree_inorder() {
        let code = r#"
def inorder(tree, node, result):
    if node == "":
        return
    left = tree[node][0]
    right = tree[node][1]
    val = tree[node][2]
    inorder(tree, left, result)
    result.append(val)
    inorder(tree, right, result)

tree = {
    "root": ["left", "right", 4],
    "left": ["ll", "lr", 2],
    "right": ["", "", 5],
    "ll": ["", "", 1],
    "lr": ["", "", 3]
}
result = []
inorder(tree, "root", result)
result
"#;
        assert_eval(code, "[1, 2, 3, 4, 5]");
    }

    // ==================== STRING ALGORITHMS ====================

    #[test]
    fn is_palindrome() {
        let code = r#"
def is_palindrome(s):
    for i in range(len(s) // 2):
        if s[i] != s[len(s) - 1 - i]:
            return False
    return True

is_palindrome("racecar")
"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn anagram_check() {
        let code = r#"
def is_anagram(s1, s2):
    if len(s1) != len(s2):
        return False
    count = {}
    for c in s1:
        if c in count:
            count[c] += 1
        else:
            count[c] = 1
    for c in s2:
        if c not in count:
            return False
        count[c] -= 1
        if count[c] < 0:
            return False
    return True

is_anagram("listen", "silent")
"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn longest_palindrome_substring() {
        let code = r#"
def expand(s, left, right):
    for _ in range(len(s)):
        if left >= 0 and right < len(s) and s[left] == s[right]:
            left -= 1
            right += 1
        else:
            break
    return s[left + 1:right]

def longest_palindrome(s):
    if len(s) == 0:
        return ""

    longest = ""
    for i in range(len(s)):
        odd = expand(s, i, i)
        even = expand(s, i, i + 1)
        if len(odd) > len(longest):
            longest = odd
        if len(even) > len(longest):
            longest = even
    return longest

longest_palindrome("babad")
"#;
        let result = run_star_code(code).expect("evaluation failed");
        assert!(result == "bab" || result == "aba", "Expected 'bab' or 'aba', got: {}", result);
    }

    #[test]
    fn string_compression() {
        let code = r#"
def compress(s):
    if len(s) == 0:
        return ""
    result = ""
    count = 1
    for i in range(1, len(s)):
        if s[i] == s[i - 1]:
            count += 1
        else:
            result += s[i - 1] + str(count)
            count = 1
    result += s[len(s) - 1] + str(count)
    return result

compress("aaabbc")
"#;
        assert_eval(code, "a3b2c1");
    }

    // ==================== MATH/NUMBER ALGORITHMS ====================

    #[test]
    fn gcd_euclidean() {
        let code = r#"
def gcd(a, b):
    if b == 0:
        return a
    return gcd(b, a % b)

gcd(48, 18)
"#;
        assert_eval_int(code, 6);
    }

    #[test]
    fn lcm_from_gcd() {
        let code = r#"
def gcd(a, b):
    if b == 0:
        return a
    return gcd(b, a % b)

def lcm(a, b):
    return (a * b) // gcd(a, b)

lcm(4, 6)
"#;
        assert_eval_int(code, 12);
    }

    #[test]
    fn sieve_of_eratosthenes() {
        let code = r#"
def sieve(n):
    is_prime = [True] * (n + 1)
    is_prime[0] = False
    is_prime[1] = False
    for p in range(2, n + 1):
        if p * p > n:
            break
        if is_prime[p]:
            for i in range(p * p, n + 1, p):
                is_prime[i] = False
    primes = []
    for i in range(n + 1):
        if is_prime[i]:
            primes.append(i)
    return primes

sieve(10)
"#;
        assert_eval(code, "[2, 3, 5, 7]");
    }

    #[test]
    fn is_prime_test() {
        let code = r#"
def is_prime(n):
    if n < 2:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False
    for i in range(3, n, 2):
        if i * i > n:
            break
        if n % i == 0:
            return False
    return True

is_prime(17)
"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn power_fast() {
        let code = r#"
def power(base, exp):
    if exp == 0:
        return 1
    if exp % 2 == 0:
        half = power(base, exp // 2)
        return half * half
    else:
        return base * power(base, exp - 1)

power(2, 10)
"#;
        assert_eval_int(code, 1024);
    }

    // ==================== BIT MANIPULATION ====================

    #[test]
    fn count_set_bits() {
        let code = r#"
def count_bits(n):
    count = 0
    for _ in range(64):
        if n == 0:
            break
        count += n & 1
        n = n >> 1
    return count

count_bits(11)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn is_power_of_two() {
        let code = r#"
def is_power_of_two(n):
    if n <= 0:
        return False
    return (n & (n - 1)) == 0

is_power_of_two(16)
"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn single_number_xor() {
        let code = r#"
def single_number(nums):
    result = 0
    for n in nums:
        result = result ^ n
    return result

single_number([2, 1, 4, 1, 2])
"#;
        assert_eval_int(code, 4);
    }

    #[test]
    fn reverse_bits() {
        let code = r#"
def reverse_bits(n, bits):
    result = 0
    for i in range(bits):
        bit = (n >> i) & 1
        result = result | (bit << (bits - 1 - i))
    return result

reverse_bits(11, 8)
"#;
        assert_eval_int(code, 208);
    }

    #[test]
    fn hamming_distance() {
        let code = r#"
def hamming(x, y):
    xor = x ^ y
    count = 0
    for _ in range(64):
        if xor == 0:
            break
        count += xor & 1
        xor = xor >> 1
    return count

hamming(1, 4)
"#;
        assert_eval_int(code, 2);
    }

    #[test]
    fn get_bit() {
        let code = r#"
def get_bit(n, pos):
    return (n >> pos) & 1

get_bit(5, 2)
"#;
        assert_eval_int(code, 1);
    }

    #[test]
    fn set_bit() {
        let code = r#"
def set_bit(n, pos):
    return n | (1 << pos)

set_bit(5, 1)
"#;
        assert_eval_int(code, 7);
    }

    #[test]
    fn clear_bit() {
        let code = r#"
def clear_bit(n, pos):
    mask = 1 << pos
    inverted = 0
    for i in range(32):
        if ((mask >> i) & 1) == 0:
            inverted = inverted | (1 << i)
    return n & inverted

clear_bit(7, 1)
"#;
        assert_eval_int(code, 5);
    }

    // ==================== BACKTRACKING ====================

    #[test]
    fn permutations() {
        let code = r#"
def permute(arr):
    if len(arr) <= 1:
        return [arr]
    result = []
    for i in range(len(arr)):
        rest = arr[:i] + arr[i+1:]
        for p in permute(rest):
            result.append([arr[i]] + p)
    return result

len(permute([1, 2, 3]))
"#;
        assert_eval_int(code, 6);
    }

    #[test]
    fn combinations() {
        let code = r#"
def combine(n, k):
    result = []
    def backtrack(start, path):
        if len(path) == k:
            result.append(path[:])
            return
        for i in range(start, n + 1):
            path.append(i)
            backtrack(i + 1, path)
            path.pop()
    backtrack(1, [])
    return result

combine(4, 2)
"#;
        assert_eval(code, "[[1, 2], [1, 3], [1, 4], [2, 3], [2, 4], [3, 4]]");
    }

    #[test]
    fn subsets_power_set() {
        let code = r#"
def subsets(nums):
    result = [[]]
    for num in nums:
        new_subsets = []
        for subset in result:
            new_subsets.append(subset + [num])
        for s in new_subsets:
            result.append(s)
    return result

len(subsets([1, 2, 3]))
"#;
        assert_eval_int(code, 8);
    }

    #[test]
    fn generate_parentheses() {
        let code = r#"
def gen_parens(n):
    result = []
    def backtrack(s, open_count, close_count):
        if len(s) == 2 * n:
            result.append(s)
            return
        if open_count < n:
            backtrack(s + "(", open_count + 1, close_count)
        if close_count < open_count:
            backtrack(s + ")", open_count, close_count + 1)
    backtrack("", 0, 0)
    return result

gen_parens(3)
"#;
        assert_eval(code, r#"["((()))", "(()())", "(())()", "()(())", "()()()"]"#);
    }

    #[test]
    fn letter_combinations_phone() {
        let code = r#"
def letter_combos(digits):
    if len(digits) == 0:
        return []
    mapping = {
        "2": "abc", "3": "def", "4": "ghi", "5": "jkl",
        "6": "mno", "7": "pqrs", "8": "tuv", "9": "wxyz"
    }
    result = []
    def backtrack(idx, path):
        if idx == len(digits):
            result.append(path)
            return
        for c in mapping[digits[idx]]:
            backtrack(idx + 1, path + c)
    backtrack(0, "")
    return result

letter_combos("23")
"#;
        assert_eval(code, r#"["ad", "ae", "af", "bd", "be", "bf", "cd", "ce", "cf"]"#);
    }

    #[test]
    fn combination_sum() {
        let code = r#"
def combo_sum(candidates, target):
    result = []
    def backtrack(remaining, combo, start):
        if remaining == 0:
            result.append(combo[:])
            return
        if remaining < 0:
            return
        for i in range(start, len(candidates)):
            combo.append(candidates[i])
            backtrack(remaining - candidates[i], combo, i)
            combo.pop()
    backtrack(target, [], 0)
    return result

combo_sum([2, 3, 6, 7], 7)
"#;
        assert_eval(code, "[[2, 2, 3], [7]]");
    }

    #[test]
    fn word_search_exists() {
        let code = r#"
def exist(board, word):
    rows = len(board)
    cols = len(board[0])

    def dfs(r, c, idx, visited):
        if idx == len(word):
            return True
        if r < 0 or r >= rows or c < 0 or c >= cols:
            return False
        key = str(r) + "," + str(c)
        if key in visited:
            return False
        if board[r][c] != word[idx]:
            return False
        visited[key] = True
        found = dfs(r+1, c, idx+1, visited) or dfs(r-1, c, idx+1, visited) or dfs(r, c+1, idx+1, visited) or dfs(r, c-1, idx+1, visited)
        visited[key] = False
        return found

    for r in range(rows):
        for c in range(cols):
            if dfs(r, c, 0, {}):
                return True
    return False

board = [["A","B","C","E"],["S","F","C","S"],["A","D","E","E"]]
exist(board, "ABCCED")
"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn n_queens_count() {
        let code = r#"
def n_queens(n):
    count = [0]

    def is_safe(board, row, col):
        for i in range(row):
            if board[i] == col:
                return False
            if board[i] - i == col - row:
                return False
            if board[i] + i == col + row:
                return False
        return True

    def solve(board, row):
        if row == n:
            count[0] += 1
            return
        for col in range(n):
            if is_safe(board, row, col):
                board[row] = col
                solve(board, row + 1)
                board[row] = -1

    board = [-1] * n
    solve(board, 0)
    return count[0]

n_queens(4)
"#;
        assert_eval_int(code, 2);
    }

    // ==================== TWO POINTERS / SLIDING WINDOW ====================

    #[test]
    fn two_sum_sorted() {
        let code = r#"
def two_sum_sorted(nums, target):
    left = 0
    right = len(nums) - 1
    for _ in range(len(nums)):
        if left >= right:
            break
        s = nums[left] + nums[right]
        if s == target:
            return [left, right]
        elif s < target:
            left += 1
        else:
            right -= 1
    return [-1, -1]

two_sum_sorted([2, 7, 11, 15], 9)
"#;
        assert_eval(code, "[0, 1]");
    }

    #[test]
    fn three_sum_zero() {
        let code = r#"
def three_sum(nums):
    nums = sorted(nums)
    result = []
    n = len(nums)
    for i in range(n - 2):
        if i > 0 and nums[i] == nums[i-1]:
            continue
        left = i + 1
        right = n - 1
        for _ in range(n):
            if left >= right:
                break
            s = nums[i] + nums[left] + nums[right]
            if s == 0:
                result.append([nums[i], nums[left], nums[right]])
                left += 1
                for _ in range(n):
                    if left < right and nums[left] == nums[left-1]:
                        left += 1
                    else:
                        break
            elif s < 0:
                left += 1
            else:
                right -= 1
    return result

three_sum([-1, 0, 1, 2, -1, -4])
"#;
        assert_eval(code, "[[-1, -1, 2], [-1, 0, 1]]");
    }

    #[test]
    fn container_with_most_water() {
        let code = r#"
def max_area(height):
    left = 0
    right = len(height) - 1
    max_water = 0
    for _ in range(len(height)):
        if left >= right:
            break
        h = height[left] if height[left] < height[right] else height[right]
        area = h * (right - left)
        if area > max_water:
            max_water = area
        if height[left] < height[right]:
            left += 1
        else:
            right -= 1
    return max_water

max_area([1,8,6,2,5,4,8,3,7])
"#;
        assert_eval_int(code, 49);
    }

    #[test]
    fn remove_duplicates_sorted() {
        let code = r#"
def remove_dups(nums):
    if len(nums) == 0:
        return 0
    k = 1
    for i in range(1, len(nums)):
        if nums[i] != nums[i-1]:
            nums[k] = nums[i]
            k += 1
    return k

remove_dups([0,0,1,1,1,2,2,3,3,4])
"#;
        assert_eval_int(code, 5);
    }

    #[test]
    fn max_subarray_kadane() {
        let code = r#"
def max_subarray(nums):
    max_sum = nums[0]
    current_sum = nums[0]
    for i in range(1, len(nums)):
        if current_sum + nums[i] > nums[i]:
            current_sum = current_sum + nums[i]
        else:
            current_sum = nums[i]
        if current_sum > max_sum:
            max_sum = current_sum
    return max_sum

max_subarray([-2,1,-3,4,-1,2,1,-5,4])
"#;
        assert_eval_int(code, 6);
    }

    #[test]
    fn longest_substring_no_repeat() {
        let code = r#"
def length_of_longest(s):
    char_idx = {}
    max_len = 0
    start = 0
    for i in range(len(s)):
        c = s[i]
        if c in char_idx and char_idx[c] >= start:
            start = char_idx[c] + 1
        char_idx[c] = i
        if i - start + 1 > max_len:
            max_len = i - start + 1
    return max_len

length_of_longest("abcabcbb")
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn min_window_substring() {
        let code = r#"
def min_window(s, t):
    if len(s) == 0 or len(t) == 0:
        return ""

    t_count = {}
    for c in t:
        if c in t_count:
            t_count[c] += 1
        else:
            t_count[c] = 1

    required = len(t_count)
    formed = 0
    window = {}
    left = 0
    min_len = len(s) + 1
    min_left = 0

    for right in range(len(s)):
        c = s[right]
        if c in window:
            window[c] += 1
        else:
            window[c] = 1

        if c in t_count and window[c] == t_count[c]:
            formed += 1

        for _ in range(len(s)):
            if formed < required:
                break
            if right - left + 1 < min_len:
                min_len = right - left + 1
                min_left = left
            lc = s[left]
            window[lc] -= 1
            if lc in t_count and window[lc] < t_count[lc]:
                formed -= 1
            left += 1

    if min_len > len(s):
        return ""
    return s[min_left:min_left + min_len]

min_window("ADOBECODEBANC", "ABC")
"#;
        assert_eval(code, "BANC");
    }

    #[test]
    fn trap_rainwater() {
        let code = r#"
def trap(height):
    if len(height) == 0:
        return 0

    n = len(height)
    left_max = [0] * n
    right_max = [0] * n

    left_max[0] = height[0]
    for i in range(1, n):
        if height[i] > left_max[i-1]:
            left_max[i] = height[i]
        else:
            left_max[i] = left_max[i-1]

    right_max[n-1] = height[n-1]
    for i in range(n - 2, -1, -1):
        if height[i] > right_max[i+1]:
            right_max[i] = height[i]
        else:
            right_max[i] = right_max[i+1]

    water = 0
    for i in range(n):
        min_height = left_max[i] if left_max[i] < right_max[i] else right_max[i]
        water += min_height - height[i]
    return water

trap([0,1,0,2,1,0,1,3,2,1,2,1])
"#;
        assert_eval_int(code, 6);
    }

    // ==================== MATRIX OPERATIONS ====================

    #[test]
    fn rotate_matrix_90() {
        let code = r#"
def rotate(matrix):
    n = len(matrix)
    for i in range(n):
        for j in range(i, n):
            temp = matrix[i][j]
            matrix[i][j] = matrix[j][i]
            matrix[j][i] = temp
    for i in range(n):
        left = 0
        right = n - 1
        for _ in range(n // 2):
            temp = matrix[i][left]
            matrix[i][left] = matrix[i][right]
            matrix[i][right] = temp
            left += 1
            right -= 1
    return matrix

rotate([[1,2,3],[4,5,6],[7,8,9]])
"#;
        assert_eval(code, "[[7, 4, 1], [8, 5, 2], [9, 6, 3]]");
    }

    #[test]
    fn spiral_order() {
        let code = r#"
def spiral(matrix):
    if len(matrix) == 0:
        return []
    result = []
    top = 0
    bottom = len(matrix) - 1
    left = 0
    right = len(matrix[0]) - 1

    for _ in range(len(matrix) * len(matrix[0])):
        if top > bottom or left > right:
            break
        for i in range(left, right + 1):
            result.append(matrix[top][i])
        top += 1

        for i in range(top, bottom + 1):
            result.append(matrix[i][right])
        right -= 1

        if top <= bottom:
            for i in range(right, left - 1, -1):
                result.append(matrix[bottom][i])
            bottom -= 1

        if left <= right:
            for i in range(bottom, top - 1, -1):
                result.append(matrix[i][left])
            left += 1

    return result

spiral([[1,2,3],[4,5,6],[7,8,9]])
"#;
        assert_eval(code, "[1, 2, 3, 6, 9, 8, 7, 4, 5]");
    }

    #[test]
    fn set_matrix_zeroes() {
        let code = r#"
def set_zeroes(matrix):
    m = len(matrix)
    n = len(matrix[0])
    zero_rows = []
    zero_cols = []

    for i in range(m):
        for j in range(n):
            if matrix[i][j] == 0:
                if i not in zero_rows:
                    zero_rows.append(i)
                if j not in zero_cols:
                    zero_cols.append(j)

    for r in zero_rows:
        for j in range(n):
            matrix[r][j] = 0

    for c in zero_cols:
        for i in range(m):
            matrix[i][c] = 0

    return matrix

set_zeroes([[1,1,1],[1,0,1],[1,1,1]])
"#;
        assert_eval(code, "[[1, 0, 1], [0, 0, 0], [1, 0, 1]]");
    }

    #[test]
    fn matrix_transpose() {
        let code = r#"
def transpose(matrix):
    m = len(matrix)
    n = len(matrix[0])
    result = []
    for j in range(n):
        row = []
        for i in range(m):
            row.append(matrix[i][j])
        result.append(row)
    return result

transpose([[1,2,3],[4,5,6]])
"#;
        assert_eval(code, "[[1, 4], [2, 5], [3, 6]]");
    }

    #[test]
    fn count_islands() {
        let code = r#"
def num_islands(grid):
    if len(grid) == 0:
        return 0

    m = len(grid)
    n = len(grid[0])
    count = 0

    def dfs(i, j):
        if i < 0 or i >= m or j < 0 or j >= n:
            return
        if grid[i][j] != 1:
            return
        grid[i][j] = 0
        dfs(i+1, j)
        dfs(i-1, j)
        dfs(i, j+1)
        dfs(i, j-1)

    for i in range(m):
        for j in range(n):
            if grid[i][j] == 1:
                count += 1
                dfs(i, j)

    return count

grid = [[1,1,0,0,0],[1,1,0,0,0],[0,0,1,0,0],[0,0,0,1,1]]
num_islands(grid)
"#;
        assert_eval_int(code, 3);
    }

    #[test]
    fn matrix_search_2d() {
        let code = r#"
def search_matrix(matrix, target):
    if len(matrix) == 0 or len(matrix[0]) == 0:
        return False
    m = len(matrix)
    n = len(matrix[0])
    row = 0
    col = n - 1
    for _ in range(m + n):
        if row >= m or col < 0:
            break
        if matrix[row][col] == target:
            return True
        elif matrix[row][col] > target:
            col -= 1
        else:
            row += 1
    return False

matrix = [[1,4,7,11,15],[2,5,8,12,19],[3,6,9,16,22],[10,13,14,17,24],[18,21,23,26,30]]
search_matrix(matrix, 5)
"#;
        assert_eval_bool(code, true);
    }

    #[test]
    fn diagonal_traverse() {
        let code = r#"
def find_diagonal(matrix):
    if len(matrix) == 0:
        return []
    m = len(matrix)
    n = len(matrix[0])
    result = []

    for d in range(m + n - 1):
        if d % 2 == 0:
            r = d if d < m else m - 1
            c = 0 if d < m else d - m + 1
            for _ in range(m + n):
                if r < 0 or c >= n:
                    break
                result.append(matrix[r][c])
                r -= 1
                c += 1
        else:
            c = d if d < n else n - 1
            r = 0 if d < n else d - n + 1
            for _ in range(m + n):
                if c < 0 or r >= m:
                    break
                result.append(matrix[r][c])
                r += 1
                c -= 1
    return result

find_diagonal([[1,2,3],[4,5,6],[7,8,9]])
"#;
        assert_eval(code, "[1, 2, 4, 7, 5, 3, 6, 8, 9]");
    }

    #[test]
    fn game_of_life() {
        let code = r#"
def game_of_life(board):
    m = len(board)
    n = len(board[0])

    def count_neighbors(r, c):
        count = 0
        for dr in [-1, 0, 1]:
            for dc in [-1, 0, 1]:
                if dr == 0 and dc == 0:
                    continue
                nr = r + dr
                nc = c + dc
                if nr >= 0 and nr < m and nc >= 0 and nc < n:
                    if board[nr][nc] == 1 or board[nr][nc] == 2:
                        count += 1
        return count

    for r in range(m):
        for c in range(n):
            neighbors = count_neighbors(r, c)
            if board[r][c] == 1:
                if neighbors < 2 or neighbors > 3:
                    board[r][c] = 2
            else:
                if neighbors == 3:
                    board[r][c] = 3

    for r in range(m):
        for c in range(n):
            if board[r][c] == 2:
                board[r][c] = 0
            elif board[r][c] == 3:
                board[r][c] = 1

    return board

game_of_life([[0,1,0],[0,0,1],[1,1,1],[0,0,0]])
"#;
        assert_eval(code, "[[0, 0, 0], [1, 0, 1], [0, 1, 1], [0, 1, 0]]");
    }
}
