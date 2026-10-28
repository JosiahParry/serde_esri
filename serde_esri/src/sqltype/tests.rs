use super::*;
use crate::features::Field;

#[test]
fn fields_read_and_write_sql_types() -> Result<(), serde_json::Error> {
    let json = r#"{"name":"t","type":"esriFieldTypeTimestampOffset","sqlType":"sqlTypeTimestampWithTimezone"}"#;
    let field = serde_json::from_str::<Field>(json)?;
    assert_eq!(field.sql_type, Some(SqlType::TimestampWithTimezone));
    assert_eq!(serde_json::to_string(&field)?, json);
    Ok(())
}

#[test]
fn missing_and_unknown_sql_types() -> Result<(), serde_json::Error> {
    let field = serde_json::from_str::<Field>(r#"{"name":"a","type":"esriFieldTypeString"}"#)?;
    assert_eq!(field.sql_type, None);
    let field = serde_json::from_str::<Field>(
        r#"{"name":"a","type":"esriFieldTypeString","sqlType":"sqlTypeSomethingNew"}"#,
    )?;
    assert_eq!(field.sql_type, Some(SqlType::Other));
    Ok(())
}
