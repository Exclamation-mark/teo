use sqlx::{AnyPool, Connection, Database, Executor, MySqlPool, Pool, Row};
use crate::connectors::sql::query_builder::dialect::SQLDialect;
use crate::connectors::sql::query_builder::traits::to_sql_string::ToSQLString;
use crate::connectors::sql::query_builder::integration::column::MySQLColumn;
use crate::connectors::sql::query_builder::stmt::create::table::SQLCreateTableStatement;
use crate::connectors::sql::query_builder::stmt::SQL;
use crate::connectors::sql::query_builder::structs::column::SQLColumn;
use crate::core::model::Model;

pub async fn migrate(dialect: SQLDialect, pool: &mut AnyPool, models: &Vec<Model>) {
    // compare each table and do migration
    for model in models {
        if model.r#virtual() {
            continue
        }
        let name = model.table_name();
        let show_table = SQL::show().tables().like(name).to_string(dialect);
        let result = pool.fetch_one(&*show_table).await;
        if result.is_err() {
            // table not exist, create table
            let stmt: SQLCreateTableStatement = model.into();
            let stmt_string = stmt.to_string(dialect);
            println!("EXECUTE SQL for create table: {}", stmt_string);
            pool.execute(&*stmt_string).await.unwrap();
        } else {
            // table exist, migrate
            let table_name = model.table_name();
            let desc = SQL::describe(table_name).to_string(SQLDialect::MySQL);
            let mut reviewed_columns: Vec<String> = Vec::new();
            let db_table_columns = pool.fetch_all(&*desc).await.unwrap();
            for db_table_column in db_table_columns {
                let db_column = MySQLColumn::from(db_table_column);
                let schema_field = model.field_with_column_name(&db_column.field);
                if schema_field.is_none() {
                    // remove this column
                    let stmt = SQL::alter_table(table_name).drop_column(db_column.field.clone()).to_string(dialect);
                    println!("EXECUTE SQL for remove column: {}", &stmt);
                    pool.execute(&*stmt).await.unwrap();
                } else {
                    // compare column definition
                    let sql_column_def: SQLColumn = schema_field.unwrap().into();
                    let schema_column: MySQLColumn = (&sql_column_def).into();
                    if schema_column != db_column {
                        // this column is different, alter it
                        let alter = SQL::alter_table(table_name).modify(sql_column_def).to_string(SQLDialect::MySQL);
                        println!("EXECUTE SQL for alter column: {}", &alter);
                        pool.execute(&*alter).await.unwrap();
                    }
                    reviewed_columns.push(db_column.field.clone());
                }
            }
            for field in model.fields() {
                if !reviewed_columns.contains(&field.column_name().to_string()) {
                    let sql_column_def: SQLColumn = field.into();
                    // add this column
                    let add = SQL::alter_table(table_name).add(sql_column_def).to_string(SQLDialect::MySQL);
                    println!("EXECUTE SQL for add column: {}", &add);
                    pool.execute(&*add).await.unwrap();
                }
            }
        }
    }
}