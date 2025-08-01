use {
    crate::*,
    gluesql_core::{
        error::{EvaluateError, TranslateError},
        prelude::{Payload, Value::*},
    },
};

test_case!(abs, {
    let g = get_tester!();

    g.named_test(
        "ABS should convert integer to absolute value",
        "SELECT ABS(1) AS ABS1, 
        ABS(-1) AS ABS2, 
        ABS(+1) AS ABS3",
        Ok(select!(
            "ABS1" | "ABS2" | "ABS3";
            I64    | I64    | I64;
            1        1        1
        )),
    )
    .await;

    g.named_test(
        "ABS should convert float to absolute value",
        "SELECT ABS(1.5) AS ABS1, 
        ABS(-1.5) AS ABS2, 
        ABS(+1.5) AS ABS3;",
        Ok(select!(
            "ABS1" | "ABS2" | "ABS3";
            F64    | F64    | F64;
            1.5      1.5      1.5
        )),
    )
    .await;

    g.named_test(
        "ABS should handle zero values correctly",
        "SELECT ABS(0) AS ABS1, 
        ABS(-0) AS ABS2, 
        ABS(+0) AS ABS3;",
        Ok(select!(
            "ABS1" | "ABS2" | "ABS3";
            I64    | I64    | I64;
            0        0        0
        )),
    )
    .await;

    g.run("CREATE TABLE SingleItem (id integer, int8 int8, dec decimal)")
        .await;

    g.run(r#"INSERT INTO SingleItem VALUES (0, -1, -2)"#).await;

    g.named_test(
        "ABS should convert integer, int8, and decimal to absolute value",
        "SELECT ABS(id) AS ABS1, 
                ABS(int8) AS ABS2, 
                ABS(dec) AS ABS3 
        FROM SingleItem",
        Ok(select!(
            "ABS1"  | "ABS2" | "ABS3";
            I64     | I8     |  Decimal;
            0         1         2.into()
        )),
    )
    .await;

    g.named_test(
        "ABS should throw evaluate error when given a string",
        "SELECT ABS('string') AS ABS FROM SingleItem",
        Err(EvaluateError::FunctionRequiresFloatValue(String::from("ABS")).into()),
    )
    .await;

    g.named_test(
        "ABS should return NULL if it gets NULL",
        "SELECT ABS(NULL) AS ABS;",
        Ok(select_with_null!(ABS; Null)),
    )
    .await;

    g.named_test(
        "ABS should throw evaluate error when given boolean TRUE",
        "SELECT ABS(TRUE) AS ABS FROM SingleItem",
        Err(EvaluateError::FunctionRequiresFloatValue(String::from("ABS")).into()),
    )
    .await;

    g.named_test(
        "ABS should throw evaluate error when given boolean FALSE",
        "SELECT ABS(FALSE) AS ABS FROM SingleItem",
        Err(EvaluateError::FunctionRequiresFloatValue(String::from("ABS")).into()),
    )
    .await;

    g.named_test(
        "ABS should throw translate error when given more than one argument",
        "SELECT ABS('string', 'string2') AS ABS",
        Err(TranslateError::FunctionArgsLengthNotMatching {
            name: "ABS".to_owned(),
            expected: 1,
            found: 2,
        }
        .into()),
    )
    .await;
});
