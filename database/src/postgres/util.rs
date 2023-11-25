#![allow(dead_code)]

/// Generate SQL to call a stored procedure with the given amount of parameters.
pub fn sql_call(name: &str, count: usize) -> String {
    format!(
        "CALL {name}({})",
        (1..=count)
            .map(|c| format!("${c}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

#[test]
fn sql_call_works() {
    assert_eq!(sql_call("cleanup", 0), "CALL cleanup()");
    assert_eq!(sql_call("user_create", 3), "CALL user_create($1, $2, $3)");
}
