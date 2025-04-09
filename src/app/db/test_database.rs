use leptos::ServerFnError;

use crate::app::models::student::GradeEnum;
use crate::app::models::test::BenchmarkCategory;
cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {
        use crate::app::models::{Test, TestType};
        use crate::app::errors::{ErrorMessageTest, TestError};
        use uuid::Uuid;
        use sqlx::PgPool;
        use sqlx::types::Json;
        use leptos::*;
        use sqlx::prelude::*;

        pub async fn get_all_tests(pool: &sqlx::PgPool) -> Result<Vec<Test>, ServerFnError>{
           let rows = sqlx::query("SELECT name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text FROM tests ORDER BY name DESC")
               .fetch_all(pool)
               .await?;

            let tests: Vec<Test> = rows
                .into_iter()
                .map(|row| {
                    let name: String = row.get("name");
                    let score: i32 = row.get("score");
                    let comments: String = row.get("comments");
                    let testarea: TestType = row.get("testarea");
                    let school_year: Option<String> = row.get("school_year");
                    let benchmark_categories: Option<Json<Vec<BenchmarkCategory>>> = row.get("benchmark_categories");
                    let test_variant: i32 = row.get("test_variant");
                    let grade_level: Option<GradeEnum> = row.get("grade_level");
                    let test_id  = row.get::<String,_>("test_id");

                    let categories = benchmark_categories.map(|categories| categories.0);

                    Test {
                        name,
                        score,
                        comments,
                        testarea,
                        school_year,
                        benchmark_categories: categories,
                        test_variant,
                        grade_level,
                        test_id
                    }
                })
                .collect();
            Ok(tests)
        }

        pub async fn get_test(test_id: String, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;
            let row = sqlx::query("SELECT name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text FROM tests WHERE test_id::text = $1")
                .bind(&test_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let benchmark_categories: Option<Json<Vec<BenchmarkCategory>>> = row.get("benchmark_categories");
            let categories = benchmark_categories.map(|categories| categories.0);

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories: categories,
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
            };

            Ok(test)
        }

        pub async fn add_test(new_test: &Test, pool: &PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&new_test.test_id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            log::info!("Benchmark categories before insert: {:?}", &new_test.benchmark_categories);

            let benchmark_json = Json(&new_test.benchmark_categories);

            let row = sqlx::query("INSERT INTO tests (name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id) VALUES($1, $2, $3, $4::testarea_enum, $5, $6, $7, $8, $9::uuid) RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text") .bind(&new_test.name).bind(&new_test.score).bind(&new_test.comments).bind(&new_test.testarea.to_string()).bind(&new_test.school_year).bind(benchmark_json).bind(&new_test.test_variant).bind(&new_test.grade_level).bind(&ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let test = Test {
                    name: row.get("name"),
                    score: row.get("score"),
                    comments: row.get("comments"),
                    testarea: row.get("testarea"),
                    school_year: row.get("school_year"),
                    benchmark_categories:{
                        let json: Option<Json<Vec<BenchmarkCategory>>> = row.get("benchmark_categories");
                        json.map(|categories| categories.0)
                    },
                    test_variant: row.get("test_variant"),
                    grade_level: row.get("grade_level"),
                    test_id: row.get("test_id"),
            };

            Ok(test)
        }

        pub async fn update_test(test: &Test, pool: &sqlx::PgPool) -> Result<Option<Test>, ServerFnError> {
            //let query = "UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum, school_year =$5, benchmark_categories=$6, test_variant=$7, grade_level=$8, questions =$9 WHERE test_id =$10";
            let ID = Uuid::parse_str(&test.test_id).expect("The UUID conversion did not occur correctly");
            let benchmark_json = Json(&test.benchmark_categories);

            let row = sqlx::query("UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum, school_year =$5, benchmark_categories=$6, test_variant=$7, grade_level=$8 WHERE test_id =$9 RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text")
                .bind(&test.name)
                .bind(&test.score)
                .bind(&test.comments)
                .bind(&test.testarea.to_string())
                .bind(&test.school_year)
                .bind(benchmark_json)
                .bind(&test.test_variant)
                .bind(&test.grade_level)
                .bind(ID)
                .fetch_one(pool)
                .await?;

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories: {
                    let json: Option<Json<Vec<BenchmarkCategory>>> = row.get("benchmark_categories");
                    json.map(|categories| categories.0)
                },
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
            };

            Ok(Some(test))
        }

        pub async fn delete_test(test_id: String, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("The test_id did not correctly become a UUID");
            let row = sqlx::query("DELETE FROM tests WHERE test_id = $1 RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text")
                .bind(ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;


            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories: {
                    let json: Option<Json<Vec<BenchmarkCategory>>> = row.get("benchmark_categories");
                    json.map(|categories| categories.0)
                },
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
            };

            Ok(test)
        }

        pub async fn score_override(test_id: String, score: i32, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("The test id was not correctly converted to UUID");

            let row = sqlx::query("UPDATE tests SET score = $1 WHERE test_id = $2 RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text")
                .bind(score)
                .bind(ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error occured: {}", e)))?;

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories: {
                    let json: Option<Json<Vec<(i32, i32, String)>>> = row.get("benchmark_categories");
                    json.map(|categories| {
                        categories.0.into_iter()
                            .map(|(min, max, label)| BenchmarkCategory {
                                min,
                                max,
                                label,
                            })
                            .collect()
                    })
                },
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
            };

            Ok(test)
        }

    }
}
