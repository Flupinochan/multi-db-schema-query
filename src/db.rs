use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use futures::stream::{self, StreamExt};
use sqlx::TypeInfo as _;
use sqlx::ValueRef as _;
use sqlx::mysql::{MySqlPool, MySqlRow};
use sqlx::{Column, Row};
use std::fs;
use std::path::Path;
use tracing::info;

/// スキーマを横断してクエリを並行実行
pub async fn query_schemas(
    pool: &MySqlPool,
    schemas: &[&str],
    sql: &str,
    concurrency: usize,
) -> Result<Vec<(String, Vec<MySqlRow>)>> {
    let results: Vec<_> = stream::iter(schemas)
        .map(|schema| {
            let pool = pool.clone();
            let query = sql.replace("FROM ", &format!("FROM {}.", schema));
            let schema = schema.to_string();
            async move {
                let rows: Vec<MySqlRow> =
                    sqlx::query(&query).fetch_all(&pool).await?;
                Ok::<_, sqlx::Error>((schema, rows))
            }
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;

    results
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// MySqlRowの各カラムを変換
///
/// 型マッピング表
/// https://docs.rs/sqlx/latest/sqlx/mysql/types/index.html
fn row_to_values(row: &MySqlRow) -> Vec<String> {
    row.columns()
        .iter()
        .map(|col| {
            let idx = col.ordinal();

            if row.try_get_raw(idx).map(|r| r.is_null()).unwrap_or(true) {
                return String::new();
            }

            let type_name = col.type_info().name();

            match type_name {
                // 整数型 (signed)
                "TINYINT" => row
                    .try_get::<i8, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "SMALLINT" => row
                    .try_get::<i16, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "INT" => row
                    .try_get::<i32, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "BIGINT" => row
                    .try_get::<i64, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),

                // 整数型 (unsigned)
                "TINYINT UNSIGNED" => row
                    .try_get::<u8, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "SMALLINT UNSIGNED" => row
                    .try_get::<u16, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "INT UNSIGNED" => row
                    .try_get::<u32, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "BIGINT UNSIGNED" => row
                    .try_get::<u64, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),

                // 浮動小数点
                "FLOAT" => row
                    .try_get::<f32, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "DOUBLE" => row
                    .try_get::<f64, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),

                // DECIMAL
                "DECIMAL" => row
                    .try_get::<rust_decimal::Decimal, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),

                // 日付・時刻
                "TIMESTAMP" => row
                    .try_get::<DateTime<Utc>, _>(idx)
                    .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
                "DATETIME" => row
                    .try_get::<NaiveDateTime, _>(idx)
                    .map(|v| v.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_default(),
                "DATE" => row
                    .try_get::<NaiveDate, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
                "TIME" => row
                    .try_get::<NaiveTime, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),

                // バイナリ
                "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY"
                | "VARBINARY" => row
                    .try_get::<Vec<u8>, _>(idx)
                    .map(|v| format!("[{} bytes]", v.len()))
                    .unwrap_or_default(),

                // JSON
                "JSON" => row
                    .try_get::<serde_json::Value, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),

                // BOOLEAN
                "BOOLEAN" | "BOOL" => row
                    .try_get::<bool, _>(idx)
                    .map(|v| v.to_string())
                    .unwrap_or_default(),

                // 文字列型
                _ => row.try_get::<String, _>(idx).unwrap_or_default(),
            }
        })
        .collect()
}

/// クエリ結果をCSV出力
pub fn write_csv(
    results: &[(String, Vec<MySqlRow>)],
    output: &Path,
) -> Result<()> {
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut wtr = csv::Writer::from_path(output)?;
    let mut header_done = false;

    for (schema, rows) in results {
        for row in rows {
            if !header_done {
                let mut cols = vec!["schema".to_string()];
                cols.extend(row.columns().iter().map(|c| c.name().to_string()));
                wtr.write_record(&cols)?;
                header_done = true;
            }

            let mut vals = vec![schema.clone()];
            vals.extend(row_to_values(row));
            wtr.write_record(&vals)?;
        }
    }

    wtr.flush()?;
    info!("CSV出力完了: {:?}", output);
    Ok(())
}

/// クエリ結果をログ出力
pub fn print_rows(results: &[(String, Vec<MySqlRow>)]) {
    for (schema, rows) in results {
        for row in rows {
            let values = row_to_values(row);
            let cols: Vec<String> = row
                .columns()
                .iter()
                .enumerate()
                .map(|(i, col)| format!("{}={}", col.name(), &values[i]))
                .collect();
            info!("[{}] {}", schema, cols.join(", "));
        }
    }
}

/// テスト用のスキーマとテーブルを作成（全型の動作確認用）
pub async fn setup_test_schemas(pool: &MySqlPool) -> Result<()> {
    let schemas = ["schema_a", "schema_b", "schema_c"];

    for schema in schemas {
        sqlx::query(&format!("CREATE DATABASE IF NOT EXISTS {}", schema))
            .execute(pool)
            .await
            .with_context(|| format!("スキーマ {} の作成に失敗", schema))?;

        // 全型テスト用テーブル
        sqlx::query(&format!(
            "CREATE TABLE IF NOT EXISTS {}.type_test (
                -- 整数型 (signed)
                col_tinyint TINYINT,
                col_smallint SMALLINT,
                col_int INT,
                col_bigint BIGINT,
                -- 整数型 (unsigned)
                col_tinyint_unsigned TINYINT UNSIGNED,
                col_smallint_unsigned SMALLINT UNSIGNED,
                col_int_unsigned INT UNSIGNED,
                col_bigint_unsigned BIGINT UNSIGNED,
                -- 浮動小数点
                col_float FLOAT,
                col_double DOUBLE,
                -- DECIMAL
                col_decimal DECIMAL(10, 2),
                -- 日付・時刻
                col_timestamp TIMESTAMP NULL,
                col_datetime DATETIME,
                col_date DATE,
                col_time TIME,
                -- バイナリ
                col_blob BLOB,
                col_binary BINARY(16),
                -- JSON
                col_json JSON,
                -- BOOLEAN
                col_boolean BOOLEAN,
                -- 文字列
                col_varchar VARCHAR(100),
                col_char CHAR(10),
                col_text TEXT
            )",
            schema
        ))
        .execute(pool)
        .await
        .with_context(|| {
            format!("{}.type_test テーブルの作成に失敗", schema)
        })?;

        // テストデータ投入
        sqlx::query(&format!(
            r#"INSERT IGNORE INTO {}.type_test VALUES (
                -- 整数型 (signed)
                -128, -32768, -2147483648, -9223372036854775808,
                -- 整数型 (unsigned)
                255, 65535, 4294967295, 18446744073709551615,
                -- 浮動小数点
                3.14, 3.141592653589793,
                -- DECIMAL
                12345.67,
                -- 日付・時刻
                '2025-12-29 12:34:56', '2025-12-29 12:34:56', '2025-12-29', '12:34:56',
                -- バイナリ
                X'48454C4C4F', X'00112233445566778899AABBCCDDEEFF',
                -- JSON
                '{{"key": "value", "number": 123}}',
                -- BOOLEAN
                TRUE,
                -- 文字列
                'varchar_from_{}', 'char_val', 'text_from_{}'
            )"#,
            schema, schema, schema
        ))
        .execute(pool)
        .await?;

        info!("スキーマ {} のセットアップ完了", schema);
    }

    Ok(())
}
