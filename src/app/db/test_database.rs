cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {

        use leptos::ServerFnError;
        use crate::app::models::assessment::ScopeEnum;
        use crate::app::models::student::GradeEnum;
        use crate::app::models::test::BenchmarkCategory;
        use crate::app::models::{Test, TestType};
        use crate::app::errors::{ErrorMessageTest, TestError};
        use uuid::Uuid;
        use sqlx::PgPool;
        use sqlx::types::Json;
        use leptos::*;
        use sqlx::prelude::*;

        pub async fn get_all_tests(pool: &sqlx::PgPool) -> Result<Vec<Test>, ServerFnError>{
           let rows = sqlx::query("SELECT name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text, scope, course_id FROM tests ORDER BY name DESC")
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
                    let test_variant: i32 = row.get("test_variant");
                    let grade_level: Option<GradeEnum> = row.get("grade_level");
                    let test_id  = row.get::<String,_>("test_id");
                    let scope: Option<ScopeEnum> = row.get("scope");
                    let course_id: Option<i32> = row.get("course_id");

                    let benchmark_categories: Option<Vec<BenchmarkCategory>> = match row.try_get::<Option<Json<Vec<BenchmarkCategory>>>, _>("benchmark_categories") {
                        Ok(Some(json)) => Some(json.0),
                        _ => None,
                    };

                    Test {
                        name,
                        score,
                        comments,
                        testarea,
                        school_year,
                        benchmark_categories,
                        test_variant,
                        grade_level,
                        test_id,
                        scope,
                        course_id,
                    }
                })
                .collect();
            Ok(tests)
        }

        pub async fn get_tests_batch(test_ids: Vec<Uuid>, pool: &sqlx::PgPool) -> Result<Vec<Test>, ServerFnError> {
            let rows = sqlx::query("SELECT name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text, scope, course_id FROM tests WHERE test_id = ANY($1)")
                .bind(&test_ids)
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
                    let test_variant: i32 = row.get("test_variant");
                    let grade_level: Option<GradeEnum> = row.get("grade_level");
                    let test_id  = row.get::<String,_>("test_id");
                    let scope: Option<ScopeEnum> = row.get("scope");
                    let course_id: Option<i32> = row.get("course_id");

                    let benchmark_categories: Option<Vec<BenchmarkCategory>> = match row.try_get::<Option<Json<Vec<BenchmarkCategory>>>, _>("benchmark_categories") {
                        Ok(Some(json)) => Some(json.0),
                        _ => None,
                    };

                    Test {
                        name,
                        score,
                        comments,
                        testarea,
                        school_year,
                        benchmark_categories,
                        test_variant,
                        grade_level,
                        test_id,
                        scope,
                        course_id,
                    }
                })
                .collect();
            Ok(tests)
        }

        pub async fn get_test(test_id: String, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;
            let row = sqlx::query("SELECT name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text, scope, course_id FROM tests WHERE test_id::text = $1")
                .bind(&test_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let benchmark_categories: Option<Vec<BenchmarkCategory>> = match row.try_get::<Option<Json<Vec<BenchmarkCategory>>>, _>("benchmark_categories") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories,
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
                scope: row.get("scope"),
                course_id: row.get("course_id"),
            };

            Ok(test)
        }

        pub async fn add_test(new_test: &Test, pool: &PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&new_test.test_id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            log::info!("Benchmark categories before insert: {:?}", &new_test.benchmark_categories);

            let benchmark_json = match &new_test.benchmark_categories {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };

            let row = sqlx::query("INSERT INTO tests (name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id, scope, course_id) VALUES($1, $2, $3, $4::testarea_enum, $5, $6, $7, $8, $9::uuid, $10, $11) RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text, scope, course_id") .bind(&new_test.name).bind(&new_test.score).bind(&new_test.comments).bind(&new_test.testarea.to_string()).bind(&new_test.school_year).bind(benchmark_json).bind(&new_test.test_variant).bind(&new_test.grade_level).bind(&ID).bind(&new_test.scope).bind(&new_test.course_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let benchmark_categories: Option<Vec<BenchmarkCategory>> = match row.try_get::<Option<Json<Vec<BenchmarkCategory>>>, _>("benchmark_categories") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let test = Test {
                    name: row.get("name"),
                    score: row.get("score"),
                    comments: row.get("comments"),
                    testarea: row.get("testarea"),
                    school_year: row.get("school_year"),
                    benchmark_categories,
                    test_variant: row.get("test_variant"),
                    grade_level: row.get("grade_level"),
                    test_id: row.get("test_id"),
                    scope: row.get("scope"),
                    course_id: row.get("course_id"),
            };

            Ok(test)
        }

        pub async fn update_test(test: &Test, pool: &sqlx::PgPool) -> Result<Option<Test>, ServerFnError> {
            //let query = "UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum, school_year =$5, benchmark_categories=$6, test_variant=$7, grade_level=$8, questions =$9 WHERE test_id =$10";
            let ID = Uuid::parse_str(&test.test_id).expect("The UUID conversion did not occur correctly");
            let benchmark_json = match &test.benchmark_categories {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };

            let row = sqlx::query("UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum, school_year =$5, benchmark_categories=$6, test_variant=$7, grade_level=$8, scope =$9, course_id=$10 WHERE test_id =$11 RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text, scope, course_id")
                .bind(&test.name)
                .bind(&test.score)
                .bind(&test.comments)
                .bind(&test.testarea.to_string())
                .bind(&test.school_year)
                .bind(benchmark_json)
                .bind(&test.test_variant)
                .bind(&test.grade_level)
                .bind(&test.scope)
                .bind(&test.course_id)
                .bind(ID)
                .fetch_one(pool)
                .await?;



            let benchmark_categories: Option<Vec<BenchmarkCategory>> = match row.try_get::<Option<Json<Vec<BenchmarkCategory>>>, _>("benchmark_categories") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories,
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
                scope: row.get("scope"),
                course_id: row.get("course_id"),
            };

            Ok(Some(test))
        }

        pub async fn delete_test(test_id: String, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("The test_id did not correctly become a UUID");
            let row = sqlx::query("DELETE FROM tests WHERE test_id = $1 RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text, scope, course_id")
                .bind(ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let benchmark_categories: Option<Vec<BenchmarkCategory>> = match row.try_get::<Option<Json<Vec<BenchmarkCategory>>>, _>("benchmark_categories") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };


            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories,
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
                scope: row.get("scope"),
                course_id: row.get("course_id"),
            };

            Ok(test)
        }

        pub async fn score_override(test_id: String, score: i32, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("The test id was not correctly converted to UUID");

            let row = sqlx::query("UPDATE tests SET score = $1 WHERE test_id = $2 RETURNING name, score, comments, testarea, school_year, benchmark_categories, test_variant, grade_level, test_id::text, scope, course_id")
                .bind(score)
                .bind(ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error occured: {}", e)))?;

            let benchmark_categories: Option<Vec<BenchmarkCategory>> = match row.try_get::<Option<Json<Vec<BenchmarkCategory>>>, _>("benchmark_categories") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                school_year: row.get("school_year"),
                benchmark_categories,
                test_variant: row.get("test_variant"),
                grade_level: row.get("grade_level"),
                test_id: row.get("test_id"),
                scope: row.get("scope"),
                course_id: row.get("course_id"),
            };

            Ok(test)
        }

    }
}
