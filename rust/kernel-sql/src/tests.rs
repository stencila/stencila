use super::*;
use kernel::{
    common::{itertools::Itertools, tokio},
    stencila_schema::{
        ArrayValidator, BooleanValidator, DatatableColumn, IntegerValidator, Number,
        NumberValidator, Primitive, StringValidator, ValidatorTypes,
    },
    KernelTrait,
};
use test_utils::{assert_json_eq, skip_ci};

/// Test against SQLite
#[tokio::test]
async fn test_sqlite() -> Result<()> {
    test("sqlite://:memory:").await
}

/// Test against Postgres
///
/// Requires some manual setup:
///   > docker run --rm -it -p5432:5432 -e POSTGRES_PASSWORD=postgres postgres
///   > psql --host localhost --user postgres
///   postgres=# CREATE DATABASE testdb;
#[tokio::test]
async fn test_postgres() -> Result<()> {
    if skip_ci("Not yet setup to work on CI") {
        return Ok(());
    };
    test("postgres://postgres:postgres@localhost:5432/testdb").await
}

/// General integration test
async fn test(config: &str) -> Result<()> {
    let mut kernel = SqlKernel::new(&KernelSelector {
        config: Some(config.to_string()),
        ..Default::default()
    });

    kernel.exec("DROP TABLE IF EXISTS table_a", None).await?;

    if let Ok(..) = kernel.get("table_a").await {
        bail!("Expected an error because table not yet created")
    };

    match kernel.set("table_a", Node::String("A".to_string())).await {
        Ok(..) => bail!("Expected an error"),
        Err(error) => assert!(error
            .to_string()
            .contains("Only Datatables can be set as symbols")),
    };

    let rows = 5;
    let col_1 = DatatableColumn {
        name: "col_1".to_string(),
        validator: Some(Box::new(ArrayValidator {
            items_validator: Some(Box::new(ValidatorTypes::BooleanValidator(
                BooleanValidator::default(),
            ))),
            ..Default::default()
        })),
        values: vec![Node::Boolean(true); rows],
        ..Default::default()
    };
    let col_2 = DatatableColumn {
        name: "col_2".to_string(),
        validator: Some(Box::new(ArrayValidator {
            items_validator: Some(Box::new(ValidatorTypes::IntegerValidator(
                IntegerValidator::default(),
            ))),
            ..Default::default()
        })),
        values: (0..rows)
            .map(|index| Node::Integer(index as i64))
            .collect_vec(),
        ..Default::default()
    };
    let col_3 = DatatableColumn {
        name: "col_3".to_string(),
        validator: Some(Box::new(ArrayValidator {
            items_validator: Some(Box::new(ValidatorTypes::NumberValidator(
                NumberValidator::default(),
            ))),
            ..Default::default()
        })),
        values: (0..rows)
            .map(|index| Node::Number(Number(index as f64)))
            .collect_vec(),
        ..Default::default()
    };
    let col_4 = DatatableColumn {
        name: "col_4".to_string(),
        validator: Some(Box::new(ArrayValidator {
            items_validator: Some(Box::new(ValidatorTypes::StringValidator(
                StringValidator::default(),
            ))),
            ..Default::default()
        })),
        values: (0..rows)
            .map(|index| Node::String(format!("string-{}", index)))
            .collect_vec(),
        ..Default::default()
    };
    let col_5 = DatatableColumn {
        name: "col_5".to_string(),
        validator: None,
        values: (0..rows)
            .map(|index| Node::Array(vec![Primitive::Integer(index as i64)]))
            .collect_vec(),
        ..Default::default()
    };
    let datatable_a = Datatable {
        columns: vec![col_1, col_2, col_3, col_4, col_5],
        ..Default::default()
    };
    kernel
        .set("table_a", Node::Datatable(datatable_a.clone()))
        .await?;

    let table_a = kernel.get("table_a").await?;
    assert!(matches!(table_a, Node::Datatable(..)));
    assert_json_eq!(table_a, datatable_a);

    Ok(())
}
