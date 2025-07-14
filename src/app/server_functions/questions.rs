use crate::app::db::question_database;
use crate::app::errors::{question_errors, ErrorMessageQuestion, ResponseErrorTraitQuestion};
use crate::app::models::{
    question::{Question, QuestionType},
    CreateNewQuestionRequest, DeleteQuestionRequest, UpdateQuestionRequest,
};
use leptos::*;
#[cfg(feature = "ssr")]
use {
    crate::app::db::database, actix_web::web, rand::seq::SliceRandom, rand::thread_rng,
    sqlx::PgPool, std::error::Error, uuid::Uuid,
};

#[server(GetQuestions, "/api")]
pub async fn get_questions(test_id: String) -> Result<Vec<Question>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;
        log::info!("Attempting to retrieve all tests from database");

        match question_database::get_all_questions(test_id, &pool).await {
            Ok(questions) => {
                log::info!("Successfully retrieved all tests from database");
                Ok(questions)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(DeleteQuestions, "/api")]
pub async fn delete_questions(test_id: String) -> Result<Vec<Question>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;
        log::info!("Attempting to retrieve all tests from database");

        match question_database::delete_all_questions(test_id, &pool).await {
            Ok(questions) => {
                log::info!("Successfully deleted all questions related to test from database");
                Ok(questions)
            }
            Err(e) => {
                log::error!("Database error: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(AddQuestion, "/api")]
pub async fn add_question(
    test_id: String,
    add_question_request: CreateNewQuestionRequest,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to add new question to the database");

        let buffer_question = Question::new(
            add_question_request.word_problem,
            add_question_request.point_value,
            add_question_request.question_type,
            add_question_request.options,
            add_question_request.correct_answer,
            0, //this value is technically the qnumber but qnumber is determined by the backend
            test_id.clone(),
        );

        match question_database::add_question(&buffer_question, &pool).await {
            Ok(created_question) => {
                log::info!(
                    "Successfully created question with ID: {}",
                    created_question.testlinker
                );
                Ok(created_question)
            }
            Err(e) => {
                log::info!("Failed to create question: {:?}", e);
                Err(ServerFnError::new(format!(
                    "The question created was not a question"
                )))
            }
        }
    }
}

#[server(DeleteQuestion, "/api")]
pub async fn delete_question(
    delete_question_request: DeleteQuestionRequest,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to delete question from the database");

        match question_database::delete_question(
            delete_question_request.qnumber,
            delete_question_request.testlinker,
            &pool,
        )
        .await
        {
            Ok(deleted) => Ok(deleted),
            Err(_) => Err(ServerFnError::new(
                "Failed to delete question from the database",
            )),
        }
    }
}

#[server(EditQuestion, "/api")]
pub async fn edit_question(
    test_id: String,
    edit_question_request: UpdateQuestionRequest,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to update question from the database");

        let buffer_question = Question::new(
            edit_question_request.word_problem,
            edit_question_request.point_value,
            edit_question_request.question_type,
            edit_question_request.options,
            edit_question_request.correct_answer,
            edit_question_request.qnumber,
            edit_question_request.testlinker,
        );

        match question_database::update_question(&buffer_question, &pool).await {
            Ok(Some(updated_student)) => Ok(updated_student),
            Ok(None) => Err(ServerFnError::new(format!(
                "Failed to correctly existing student in the database"
            ))),
            Err(e) => Err(ServerFnError::new(format!(
                "Failed to update student: {}",
                e
            ))),
        }
    }
}

#[server(DuplicateAndRandomizeQuestions, "/api")]
pub async fn duplicate_and_randomize_questions(
    source_test_id: String,
    target_test_id: String,
) -> Result<Vec<Question>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to duplicate and randomize questions from database");

        // First, get all questions from the source test
        match question_database::get_all_questions(source_test_id.clone(), &pool).await {
            Ok(source_questions) => {
                if source_questions.is_empty() {
                    return Err(ServerFnError::new("No questions found in source test"));
                }

                // Create randomized versions of the questions
                let mut randomized_questions = Vec::new();
                let mut rng = thread_rng();

                // Create a shuffled order for the questions
                let mut question_indices: Vec<usize> = (0..source_questions.len()).collect();
                question_indices.shuffle(&mut rng);

                for (new_index, &original_index) in question_indices.iter().enumerate() {
                    let source_question = &source_questions[original_index];
                    let mut new_question = source_question.clone();

                    // Update the question number to maintain sequential order in the new test
                    new_question.qnumber = (new_index + 1) as i32;
                    new_question.testlinker = target_test_id.clone();

                    // Randomize the answer options for multiple choice questions
                    if source_question.question_type == QuestionType::MultipleChoice
                        && !source_question.options.is_empty()
                    {
                        let mut shuffled_options = source_question.options.clone();
                        shuffled_options.shuffle(&mut rng);
                        new_question.options = shuffled_options;
                    }

                    randomized_questions.push(new_question);
                }

                // Insert the randomized questions into the database
                let mut created_questions = Vec::new();

                for question in randomized_questions {
                    match question_database::add_question(&question, &pool).await {
                        Ok(created_question) => {
                            created_questions.push(created_question);
                        }
                        Err(e) => {
                            log::error!("Failed to create randomized question: {:?}", e);
                            return Err(ServerFnError::new(format!(
                                "Failed to create randomized question: {}",
                                e
                            )));
                        }
                    }
                }

                log::info!(
                    "Successfully created {} randomized questions for test {}",
                    created_questions.len(),
                    target_test_id
                );

                Ok(created_questions)
            }
            Err(e) => {
                log::error!("Database error fetching source questions: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(GenerateRandomizedTest, "/api")]
pub async fn generate_randomized_test(
    base_test_id: String,
    variation_name: String,
    shuffle_questions: bool,
    shuffle_options: bool,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::models::test::{CreateNewTestRequest, Test};
        use crate::app::server_functions::tests::{add_test, get_test};

        log::info!("Attempting to generate randomized test");

        // Get the base test
        let base_test = match get_test(base_test_id.clone()).await {
            Ok(test) => test,
            Err(e) => {
                return Err(ServerFnError::new(format!("Base test not found: {}", e)));
            }
        };

        // Create the variation test
        let variation_test_request = CreateNewTestRequest::new(
            variation_name,
            base_test.score,
            format!("Randomized variation of {}", base_test.name),
            base_test.testarea,
            base_test.school_year,
            base_test.benchmark_categories,
            base_test.test_variant + 100, // Randomized variant offset
            base_test.grade_level,
            base_test.scope,
            base_test.course_id,
        );

        let new_test = match add_test(variation_test_request).await {
            Ok(test) => test,
            Err(e) => {
                return Err(ServerFnError::new(format!(
                    "Failed to create variation test: {}",
                    e
                )));
            }
        };

        // Generate randomized questions
        match duplicate_and_randomize_questions(base_test_id, new_test.test_id.clone()).await {
            Ok(_) => Ok(new_test.test_id),
            Err(e) => Err(e),
        }
    }
}

#[server(ShuffleQuestionOptions, "/api")]
pub async fn shuffle_question_options(
    qnumber: i32,
    test_id: String,
) -> Result<Question, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to shuffle question options");

        // Get the specific question first
        match question_database::get_single_question(qnumber, test_id.clone(), &pool).await {
            Ok(question) => {
                // Only shuffle if it's a multiple choice question with options
                if question.question_type == QuestionType::MultipleChoice
                    && !question.options.is_empty()
                {
                    let mut rng = thread_rng();
                    let mut shuffled_options = question.options.clone();
                    shuffled_options.shuffle(&mut rng);

                    // Update the question options in the database
                    match question_database::update_question_options(
                        qnumber,
                        test_id,
                        shuffled_options,
                        &pool,
                    )
                    .await
                    {
                        Ok(updated_question) => Ok(updated_question),
                        Err(e) => Err(ServerFnError::new(format!(
                            "Failed to update question options: {}",
                            e
                        ))),
                    }
                } else {
                    Ok(question)
                }
            }
            Err(e) => {
                log::error!("Database error fetching question: {}", e);
                Err(ServerFnError::new(format!("Database error: {}", e)))
            }
        }
    }
}

#[server(ValidateTestForRandomization, "/api")]
pub async fn validate_test_for_randomization(
    test_id: String,
) -> Result<(bool, String), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use actix_web::web;
        use leptos_actix::extract;
        let pool = extract::<web::Data<PgPool>>()
            .await
            .map_err(|e| ServerFnError::new(format!("Failed to extract pool: {}", e)))?;

        log::info!("Attempting to validate test for randomization");

        // Get question count
        let question_count =
            match question_database::count_questions_by_test(test_id.clone(), &pool).await {
                Ok(count) => count,
                Err(e) => {
                    log::error!("Database error counting questions: {}", e);
                    return Err(ServerFnError::new(format!("Database error: {}", e)));
                }
            };

        if question_count == 0 {
            return Ok((false, "Test has no questions to randomize".to_string()));
        }

        // Get multiple choice question count
        let mc_count = match question_database::count_multiple_choice_questions(
            test_id.clone(),
            &pool,
        )
        .await
        {
            Ok(count) => count,
            Err(e) => {
                log::error!("Database error counting MC questions: {}", e);
                return Err(ServerFnError::new(format!("Database error: {}", e)));
            }
        };

        if mc_count == 0 {
            return Ok((true, format!("Test has {} questions but no multiple choice questions. Only question order will be randomized.", question_count)));
        }

        Ok((true, format!("Test has {} questions ({} multiple choice). Both question order and answer choices will be randomized.", question_count, mc_count)))
    }
}

/*cfg_if::cfg_if! {
    if #[cfg(feature = "ssr")] {

        use crate::app::db::database;
        use crate::app::errors::QuestionError;
        use sqlx::PgPool;

        pub async fn retrieve_all_questions(test_id: String, pool: &sqlx::PgPool) -> Vec<Question> {

            let get_all_question_results = database::get_all_questions(test_id.clone(), pool).await;

            get_all_question_results.expect("There was a problem gathering all the questions for this test.")
        }

        pub async fn add_new_question<T> (word_problem: T, point_value: i32, question_type: QuestionType, options: Vec<String>, correct_answer: T, qnumber: i64, test_id: T, pool: &sqlx::PgPool) -> Result<Question, ServerFnErro> where T: Into<String> {
            let new_question = Question::new(
                word_problem.into(),
                point_value,
                question_type,
                options,
                correct_answer.into(),
                qnumber,
                test_id.into(),
            );

            database::add_question(&new_question, pool).await
        }
        pub async fn delete_certain_question(qnumber: i64, test_id: String, pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
            database::delete_question(qnumber, test_id, pool).await
        }

        pub async fn edit_certain_question<T>(word_problem: T, point_value: i32, question_type: QuestionType, options:Vec<String>, correct_answer: T, qnumber: i64, test_id: T, pool: &sqlx::PgPool) -> Result<Option<Question>, sqlx::Error> where T: Into<String> {
            let updated_question = Question::new(
                word_problem.into(),
                point_value,
                question_type,
                options,
                correct_answer.into(),
                qnumber,
                test_id.into(),
            );
            database::update_question(&updated_question, pool).await
        }
    }
}*/
