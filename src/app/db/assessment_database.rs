use crate::app::models::assessment::RangeCategory;
use crate::app::models::assessment::ScopeEnum;
use crate::app::models::assessment_sequences::{SequenceBehavior, TestSequenceItem};
use crate::app::models::student::GradeEnum;
use leptos::ServerFnError;
use uuid::Uuid;
cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")]{
        use crate::app::models::assessment::{Assessment, SubjectEnum, RangeCategoriesWrapper};
        use sqlx::PgPool;
        use sqlx::types::Json;
        use leptos::*;
        use sqlx::prelude::*;

        pub async fn get_all_assessments(pool: &sqlx::PgPool) -> Result<Vec<Assessment>, ServerFnError> {
            let rows = sqlx::query("SELECT name, frequency, grade, version, id, tests, test_sequence, composite_score, risk_benchmarks, national_benchmarks, subject, scope, course_id FROM assessments ORDER BY name ASC")
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let assessments: Vec<Assessment> = rows
                .into_iter()
                .map(|row| {
                    let name: String = row.get("name");
                    let frequency: Option<i32> = row.get("frequency");
                    let grade: Option<GradeEnum> = row.get("grade");
                    let version: i32 = row.get("version");
                    let id: Uuid = row.get("id");
                    let tests: Vec<Uuid> = row.get("tests");
                    let composite_score: Option<i32> = row.get("composite_score");

                    // ADD: Parse test_sequence
                    let test_sequence: Option<Vec<TestSequenceItem>> = match row.try_get::<Option<Json<Vec<TestSequenceItem>>>, _>("test_sequence") {
                        Ok(Some(json)) => Some(json.0),
                        _ => None,
                    };

                    let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                        Ok(Some(json)) => Some(json.0),
                        _ => None,
                    };
                    let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                        Ok(Some(json)) => Some(json.0),
                        _ => None,
                    };
                    let subject: Option<SubjectEnum> = row.get("subject");
                    let scope: Option<ScopeEnum> = row.get("scope");
                    let course_id: Option<i32> = row.get("course_id");

                    Assessment {
                        name,
                        frequency,
                        grade,
                        version,
                        id,
                        tests,
                        composite_score,
                        risk_benchmarks,
                        national_benchmarks,
                        subject,
                        scope,
                        course_id,
                        test_sequence,
                    }
                })
                .collect();

            Ok(assessments)
        }

    pub async fn get_assessment(id: String, pool: &sqlx::PgPool) -> Result<Assessment, ServerFnError> {
        let uuid = Uuid::parse_str(&id)
            .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

        let row = sqlx::query("SELECT name, frequency, grade, version, id, tests, test_sequence, composite_score, risk_benchmarks, national_benchmarks, subject, scope, course_id FROM assessments WHERE id = $1")
            .bind(&uuid)
            .fetch_one(pool)
            .await
            .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

        // ADD: Parse test_sequence
        let test_sequence: Option<Vec<TestSequenceItem>> = match row.try_get::<Option<Json<Vec<TestSequenceItem>>>, _>("test_sequence") {
            Ok(Some(json)) => Some(json.0),
            _ => None,
        };

        let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
            Ok(Some(json)) => Some(json.0),
            _ => None,
        };

        let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
            Ok(Some(json)) => Some(json.0),
            _ => None,
        };

        let assessment = Assessment {
            name: row.get("name"),
            frequency: row.get("frequency"),
            grade: row.get("grade"),
            version: row.get("version"),
            id: row.get("id"),
            tests: row.get("tests"),
            composite_score: row.get("composite_score"),
            risk_benchmarks,
            national_benchmarks,
            subject: row.get("subject"),
            scope: row.get("scope"),
            course_id: row.get("course_id"),
            test_sequence, // ADD THIS
        };

        Ok(assessment)
    }


        pub async fn add_assessment(new_assessment: &Assessment, pool: &PgPool) -> Result<Assessment, ServerFnError> {
            let risk_benchmarks = match &new_assessment.risk_benchmarks {
                Some(categories) if !categories.is_empty() => Json(categories.clone()),
                _ => Json(Vec::<RangeCategory>::new()),
            };
            let national_benchmarks = match &new_assessment.national_benchmarks {
                Some(categories) if !categories.is_empty() => Json(categories.clone()),
                _ => Json(Vec::<RangeCategory>::new()),
            };

            // ADD: Handle test_sequence
            let test_sequence = match &new_assessment.test_sequence {
                Some(sequence) if !sequence.is_empty() => Json(sequence.clone()),
                _ => Json(Vec::<TestSequenceItem>::new()),
            };

            let row = sqlx::query("INSERT INTO assessments (name, frequency, grade, version, id, tests, test_sequence, composite_score, risk_benchmarks, national_benchmarks, subject, scope, course_id) VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) RETURNING name, frequency, grade, version, id, tests, test_sequence, composite_score, risk_benchmarks, national_benchmarks, subject, scope, course_id")
                .bind(&new_assessment.name)
                .bind(&new_assessment.frequency)
                .bind(&new_assessment.grade)
                .bind(&new_assessment.version)
                .bind(&new_assessment.id)
                .bind(&new_assessment.tests)
                .bind(&test_sequence) // ADD THIS BIND
                .bind(&new_assessment.composite_score)
                .bind(&risk_benchmarks)
                .bind(&national_benchmarks)
                .bind(&new_assessment.subject)
                .bind(&new_assessment.scope)
                .bind(&new_assessment.course_id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            // Fixed parsing logic to handle empty arrays
            let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Json<Vec<RangeCategory>>, _>("risk_benchmarks") {
                Ok(json) => {
                    if json.0.is_empty() {
                        None
                    } else {
                        Some(json.0)
                    }
                },
                Err(_) => None,
            };

            let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Json<Vec<RangeCategory>>, _>("national_benchmarks") {
                Ok(json) => {
                    if json.0.is_empty() {
                        None
                    } else {
                        Some(json.0)
                    }
                },
                Err(_) => None,
            };

            // ADD: Parse test_sequence from result
            let test_sequence: Option<Vec<TestSequenceItem>> = match row.try_get::<Json<Vec<TestSequenceItem>>, _>("test_sequence") {
                Ok(json) => {
                    if json.0.is_empty() {
                        None
                    } else {
                        Some(json.0)
                    }
                },
                Err(_) => None,
            };

            let id: Uuid = row.get("id");

            let assessment = Assessment {
                name: row.get("name"),
                frequency: row.get("frequency"),
                grade: row.get("grade"),
                version: row.get("version"),
                id,
                tests: row.get("tests"),
                composite_score: row.get("composite_score"),
                risk_benchmarks,
                national_benchmarks,
                subject: row.get("subject"),
                scope: row.get("scope"),
                course_id: row.get("course_id"),
                test_sequence, // ADD THI
            };
            Ok(assessment)
        }

        pub async fn update_all_assessments_referencing_test(test_id: &String, pool: &sqlx::PgPool) -> Result<(), ServerFnError> {
            // Convert the string to a UUID
            let test_uuid = Uuid::parse_str(test_id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID format: {}", e)))?;

            // Find all assessments that reference this test
            let assessment_ids: Vec<Uuid> = sqlx::query_as::<_, (Uuid,)>("SELECT id FROM assessments WHERE $1 = ANY(tests)")
                .bind(test_uuid)
                .fetch_all(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?
                .into_iter()
                .map(|(id,)| id)
                .collect();

            // Update each assessment's composite score
            for assessment_id in assessment_ids {
                // Get the test IDs for this assessment
                let test_ids: Vec<Uuid> = sqlx::query_scalar::<_, Uuid>("SELECT unnest(tests) FROM assessments WHERE id = $1")
                    .bind(assessment_id)
                    .fetch_all(pool)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

                // Get scores for all tests and calculate the sum
                let scores: Vec<i32> = sqlx::query_scalar::<_, i32>("SELECT score FROM tests WHERE test_id = ANY($1)")
                    .bind(&test_ids)
                    .fetch_all(pool)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Issue retrieving score values for tests: {}", e)))?;

                let total: i32 = scores.iter().sum();

                // Update the assessment with the new composite score
                sqlx::query("UPDATE assessments SET composite_score = $1 WHERE id = $2")
                    .bind(total)
                    .bind(assessment_id)
                    .execute(pool)
                    .await
                    .map_err(|e| ServerFnError::new(format!("Database error updating assessment: {}", e)))?;
            }

            Ok(())
        }

        pub async fn update_assessment(assessment: &Assessment, pool: &sqlx::PgPool) -> Result<Assessment, ServerFnError> {
            let risk_benchmarks = match &assessment.risk_benchmarks {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };
            let national_benchmarks = match &assessment.national_benchmarks {
                Some(categories) => Json(categories.clone()),
                None => Json(Vec::new()),
            };

            // ADD: Handle test_sequence
            let test_sequence = match &assessment.test_sequence {
                Some(sequence) => Json(sequence.clone()),
                None => Json(Vec::new()),
            };

            let row = sqlx::query("UPDATE assessments SET name = $1, frequency = $2, grade = $3, version = $4, tests = $5, test_sequence = $6, composite_score = $7, risk_benchmarks = $8, national_benchmarks = $9, subject = $10, scope = $11, course_id = $12 WHERE id = $13 RETURNING name, frequency, grade, version, id, tests, test_sequence, composite_score, risk_benchmarks, national_benchmarks, subject, scope, course_id")
                .bind(&assessment.name)
                .bind(&assessment.frequency)
                .bind(&assessment.grade)
                .bind(&assessment.version)
                .bind(&assessment.tests)
                .bind(&test_sequence) // ADD THIS BIND
                .bind(&assessment.composite_score)
                .bind(&risk_benchmarks)
                .bind(&national_benchmarks)
                .bind(&assessment.subject)
                .bind(&assessment.scope)
                .bind(&assessment.course_id)
                .bind(&assessment.id)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };
            let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            // ADD: Parse test_sequence from result
            let test_sequence: Option<Vec<TestSequenceItem>> = match row.try_get::<Option<Json<Vec<TestSequenceItem>>>, _>("test_sequence") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let id: Uuid = row.get("id");

            let assessment = Assessment {
                name: row.get("name"),
                frequency: row.get("frequency"),
                grade: row.get("grade"),
                version: row.get("version"),
                id,
                tests: row.get("tests"),
                composite_score: row.get("composite_score"),
                risk_benchmarks,
                national_benchmarks,
                subject: row.get("subject"),
                scope: row.get("scope"),
                course_id: row.get("course_id"),
                test_sequence, // ADD
            };

            Ok(assessment)
        }


        pub async fn delete_assessment(id: String, pool: &sqlx::PgPool) -> Result<Assessment, ServerFnError> {
            let uuid = Uuid::parse_str(&id)
                .map_err(|e| ServerFnError::new(format!("Invalid UUID: {}", e)))?;

            let row = sqlx::query("DELETE from assessments WHERE id = $1 RETURNING name, frequency, grade, version, id, tests, test_sequence, composite_score, risk_benchmarks, national_benchmarks, subject, scope, course_id")
                .bind(&uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| ServerFnError::new(format!("Database error: {}", e)))?;

            let risk_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("risk_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };
            let national_benchmarks: Option<Vec<RangeCategory>> = match row.try_get::<Option<Json<Vec<RangeCategory>>>, _>("national_benchmarks") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            // ADD: Parse test_sequence from result
            let test_sequence: Option<Vec<TestSequenceItem>> = match row.try_get::<Option<Json<Vec<TestSequenceItem>>>, _>("test_sequence") {
                Ok(Some(json)) => Some(json.0),
                _ => None,
            };

            let id: Uuid = row.get("id");

            let assessment = Assessment {
                name: row.get("name"),
                frequency: row.get("frequency"),
                grade: row.get("grade"),
                version: row.get("version"),
                id,
                tests: row.get("tests"),
                composite_score: row.get("composite_score"),
                risk_benchmarks,
                national_benchmarks,
                subject: row.get("subject"),
                scope: row.get("scope"),
                course_id: row.get("course_id"),
                test_sequence,
            };

            Ok(assessment)
        }

        pub async fn get_test_sequence(assessment_id: &String, pool: &sqlx::PgPool) -> Result<Vec<(String, String)>, ServerFnError> {

            let uuid = Uuid::parse_str(assessment_id)
                .map_err(|e| {
                    log::error!("UUID parsing failed for '{}': {}", assessment_id, e);
                    ServerFnError::new(format!("Invalid UUID: {}", e))
                })?;

            let row = sqlx::query("SELECT test_sequence FROM assessments WHERE id = $1")
                .bind(&uuid)
                .fetch_one(pool)
                .await
                .map_err(|e| {
                    log::error!("Database query failed for assessment {}: {}", uuid, e);
                    ServerFnError::new(format!("Database error: {}", e))
                })?;

            // Parse the test_sequence from the row
            let test_sequence: Option<Vec<TestSequenceItem>> = match row.try_get::<Option<Json<Vec<TestSequenceItem>>>, _>("test_sequence") {
                Ok(Some(json)) => {
                    log::info!("test_sequence JSON parsed successfully, {} items", json.0.len());
                    Some(json.0)
                },
                Ok(None) => {
                    log::warn!("test_sequence is NULL for assessment {}", uuid);
                    None
                },
                Err(e) => {
                    log::error!("Failed to parse test_sequence JSON for assessment {}: {}", uuid, e);
                    return Err(ServerFnError::new(format!("JSON parsing error: {}", e)));
                }
            };

            // Convert TestSequenceItem to (String, String) pairs
            let sequence_items = test_sequence.unwrap_or_default();

            if sequence_items.is_empty() {
                log::warn!("No test sequence items found for assessment {}", uuid);
                return Ok(vec![]);
            }

            // Get test names from the tests table for each test_id in the sequence
            let mut result = Vec::new();

            for (index, item) in sequence_items.iter().enumerate() {
                let test_name: Option<String> = sqlx::query_scalar("SELECT name FROM tests WHERE test_id = $1")
                    .bind(&item.test_id)  // Bind the UUID directly, not as string
                    .fetch_optional(pool)
                    .await
                    .map_err(|e| {
                        log::error!("Database error fetching test name for test_id {}: {}", item.test_id, e);
                        ServerFnError::new(format!("Database error fetching test name: {}", e))
                    })?;

                let name = test_name.unwrap_or_else(|| {
                    log::warn!("Test name not found for test_id {}, using fallback", item.test_id);
                    format!("Test {}", item.test_id)
                });

                // Return the UUID as string in the result tuple
                result.push((item.test_id.to_string(), name));
            }

            Ok(result)
        }
    }
}
