cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {
        use crate::app::models::{Test, TestType};
        use crate::app::errors::{ErrorMessageTest, TestError};
        use uuid::Uuid;
        use sqlx::PgPool;
        use leptos::*;
        use sqlx::prelude::*;

        pub async fn get_all_tests(pool: &sqlx::PgPool) -> Result<Vec<Test>, ServerFnError>{
           let rows = sqlx::query("SELECT name, score, comments, testarea, test_id::text FROM tests ORDER BY name DESC")
               .fetch_all(pool)
               .await?;

            let tests: Vec<Test> = rows
                .into_iter()
                .map(|row| {
                    let name: String = row.get("name");
                    let score: i32 = row.get("score");
                    let comments: String = row.get("comments");
                    let testarea: TestType = row.get("testarea");
                    let test_id  = row.get::<String,_>("test_id");

                    Test {
                        name,
                        score,
                        comments,
                        testarea,
                        test_id
                    }
                })
                .collect();
            Ok(tests)
        }

        pub async fn add_test(new_test: &Test, pool: &PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&new_test.test_id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("INSERT INTO tests (name, score, comments, testarea, test_id) VALUES($1, $2, $3, $4::testarea_enum, $5::uuid) RETURNING name, score, comments, testarea, test_id::text") .bind(&new_test.name).bind(&new_test.score).bind(&new_test.comments).bind(&new_test.testarea).bind(&ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let test = Test {
                    name: row.get("name"),
                    score: row.get("score"),
                    comments: row.get("comments"),
                    testarea: row.get("testarea"),
                    test_id: row.get("test_id"),
            };

            Ok(test)
        }

        pub async fn update_test(test: &Test, pool: &sqlx::PgPool) -> Result<Option<Test>, ServerFnError> {
            let query = "UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum, questions =$5 WHERE test_id =$6";
            let ID = Uuid::parse_str(&test.test_id).expect("The UUID conversion did not occur correctly");

            let row = sqlx::query("UPDATE tests SET name =$1, score =$2, comments =$3, testarea =$4::testarea_enum WHERE test_id =$5 RETURNING name, score, comments, testarea, test_id::text")
                .bind(&test.name)
                .bind(&test.score)
                .bind(&test.comments)
                .bind(&test.testarea.to_string())
                .bind(ID)
                .fetch_one(pool)
                .await?;

            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                test_id: row.get("test_id"),
            };

            Ok(Some(test))
        }

        pub async fn delete_test(test_id: String, pool: &sqlx::PgPool) -> Result<Test, ServerFnError> {
            let ID = Uuid::parse_str(&test_id).expect("The test_id did not correctly become a UUID");
            let row = sqlx::query("DELETE FROM tests WHERE test_id = $1 RETURNING name, score, comments, testarea, test_id::text")
                .bind(ID)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;


            let test = Test {
                name: row.get("name"),
                score: row.get("score"),
                comments: row.get("comments"),
                testarea: row.get("testarea"),
                test_id: row.get("test_id"),
            };

            Ok(test)
        }

    }
}
