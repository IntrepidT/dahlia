cfg_if::cfg_if! {

    if #[cfg(feature = "ssr")] {

        use crate::app::models::{Student, Test, test_type, QuestionType, Question};
        use crate::app::errors::{ErrorMessage, StudentError, TestError, ErrorMessageTest, QuestionError, ErrorMessageQuestion};
        use surrealdb::engine::remote::ws::{Client, Ws};
        use surrealdb::opt::auth::Root;
        use surrealdb::{Error, Surreal};
        use once_cell::sync::Lazy;

        static DB: Lazy<Surreal<Client>> = Lazy::new(Surreal::init);

        pub async fn open_db_connection() {

            let _ = DB.connect::<Ws>("127.0.0.1:8000").await;
            let _ = DB.signin(Root {
                username: "root",
                password: "root",
            })
            .await;
            let _ = DB.use_ns("surreal").use_db("store").await;
        }

        //pub async fn open_db_connection_test() {
        //    let _ = DB.connect::<Ws>("127.0.0.1:8000").await;
        //    let _ = DB.signin(Root {
        //        username: "root",
        //        password: "root",
        //    })
        //    .await;
        //    let _ = DB.use_ns("surreal").use_db("test").await;
        //}

        pub async fn get_all_students() -> Option<Vec<Student>> {

            open_db_connection().await;
            let get_all_students = DB.query("Select * FROM student ORDER BY joined_date DESC;").await;

            match get_all_students {

                Ok(mut res) => {
                    let found = res.take(0);
                    match found {
                        Ok(found_students) => Some(found_students),
                        Err(_) => None,
                    }
                },
                Err(_) => None,
            }
        }

        pub async fn get_all_test_questions(test_identifier: i64) -> Option<Vec<Question>> {
            open_db_connection().await;
            let get_all_questions = DB.query(format!("SELECT questions FROM test = {:?} ORDER BY qnumber;", test_identifier)).await;

            match get_all_questions {
                Ok(mut res) => {
                    let found = res.take(0);
                    match found {
                        Ok(found_questions) => Some(found_questions),
                        Err(_) => None,
                    }
                },
                Err(_) => None,
            }
        }

        pub async fn get_all_tests() -> Option<Vec<Test>> {
            open_db_connection().await;
            let get_all_tests = DB.query("Select * FROM test ORDER BY name;").await;

            match get_all_tests {

                Ok(mut res) => {
                    let found = res.take(0);
                    match found {
                        Ok(found_test) => Some(found_test),
                        Err(_) => None,
                    }
                },
                Err(_) => None,
            }
        }

        pub async fn add_student(new_student: Student) -> Option<Student> {

            open_db_connection().await;
            let results = DB.create(("student", new_student.uuid.clone()))
                .content(new_student)
                .await;
            let _ = DB.invalidate().await;

            match results {
                Ok(created_student) => created_student,
                Err(e) => {
                    println!("error in adding new student: {:?}", e);
                    None
                }
            }
        }

        pub async fn add_question_test(test_id: i64, new_question: Question) -> Option<Question> {

            open_db_connection().await;
            let results = DB.update(("test", test_id.clone()))
                //.create(("question", format!("{:?}", new_question.qnumber.clone())))
                .content(new_question)
                .await;
            let _ = DB.invalidate().await;

            match results {
                Ok(created_question) => created_question,
                Err(e) => {
                    println!("error in adding new question: {:?}", e);
                    None
                }
            }
        }

        pub async fn add_test(new_test: Test) -> Option<Test> {

            open_db_connection().await;
            let results = DB.create(("test", new_test.test_identifier.clone()))
                .content(new_test)
                .await;
            let _ = DB.invalidate().await;

            match results {
                Ok(created_test) => created_test,
                Err(e) => {
                    println!("error in adding new test: {:?}", e);
                    None
                }
            }
        }

        pub async fn delete_student(student_uuid: String) -> Result<Option<Student>, StudentError> {

            open_db_connection().await;
            let delete_results = DB.delete(("student", student_uuid)).await;
            let _ = DB.invalidate().await;

            match delete_results {
                Ok(deleted_student) => Ok(deleted_student),
                Err(_) => Err(StudentError::StudentDeleteFailure)
            }
        }

        pub async fn delete_question(question_number: i64) -> Result<Option<Question>, QuestionError> {

            open_db_connection().await;
            let delete_results = DB.delete(("question", question_number)).await;
            let _ = DB.invalidate().await;

            match delete_results {
                Ok(deleted_question) => Ok(deleted_question),
                Err(_) => Err(QuestionError::QuestionDeleteFailure)
            }
        }

        pub async fn delete_test(test_identifier: i64) -> Result<Option<Test>, TestError> {

            open_db_connection().await;
            let delete_results = DB.delete(("test", test_identifier)).await;
            let _ = DB.invalidate().await;

            match delete_results {
                Ok(deleted_test) => Ok(deleted_test),
                Err(_) => Err(TestError::TestDeleteFailure)
            }
        }

        pub async fn update_student(uuid: String, name: String, grade: String, student_id: i32)-> Result<Option<Student>, StudentError> {

            open_db_connection().await;

            let find_student: Result<Option<Student>, Error> =
                DB.select(("student", &uuid)).await;
            match find_student {

                Ok(found) => {

                    match found {
                        Some(found_student) => {

                            let updated_student: Result<Option<Student>, Error> =
                                DB.update(("student", &uuid))
                                .merge(Student::new(
                                        uuid,
                                        name,
                                        grade,
                                        student_id,
                                        found_student.joined_date
                                ))
                                .await;
                            let _ = DB.invalidate().await;
                            match updated_student {
                                Ok(returned_student) => Ok(returned_student),
                                Err(_) => Err(StudentError::StudentUpdateFailure)
                            }
                        },
                        None => Err(StudentError::StudentUpdateFailure)
                    }
                },
                Err(_) => {
                    let _ = DB.invalidate().await;
                    Err(StudentError::StudentNotFound)
                }
            }
        }

        pub async fn update_test(name: String, score: i32, comments: String, test_area: test_type, test_identifier: i64) -> Result<Option<Test>, TestError> {

            open_db_connection().await;

            let find_test: Result<Option<Test>, Error> =
                DB.select(("test", test_identifier)).await;
            match find_test {

                Ok(found) => {

                    match found {
                        Some(found_test) => {

                            let updated_test: Result<Option<Test>, Error> =
                                DB.update(("test", test_identifier))
                                .merge(Test::new(
                                        name,
                                        score,
                                        comments,
                                        test_area,
                                        test_identifier,
                                        found_test.date
                                ))
                                .await;
                            let _ = DB.invalidate().await;
                            match updated_test {
                                Ok(returned_test) => Ok(returned_test),
                                Err(_) => Err(TestError::TestUpdateFailure)
                            }
                        },
                        None => Err(TestError::TestUpdateFailure)
                    }
                },
                Err(_) => {
                    let _ = DB.invalidate().await;
                    Err(TestError::TestNotFound)
                }
            }
        }

        pub async fn update_question(test_identifier: i64, word_problem: String, point_value: i32, qtype: QuestionType, options: Vec<String>, correct_answer: String, comments: String, qnumber: i64) -> Result<Option<Question>, QuestionError> {
            open_db_connection().await;

            let find_question: Result<Option<Question>, Error> =
                DB.select(("test", test_identifier.clone())).await;
            match find_question {

                Ok(found) => {

                    match found {
                        Some(found_test) => {

                            let updated_question: Result<Option<Question>, Error> =
                                DB.update(("question", qnumber.clone()))
                                .merge(Question::new(
                                        word_problem,
                                        point_value,
                                        qtype,
                                        options,
                                        correct_answer,
                                        comments,
                                        qnumber,
                                ))
                                .await;
                            let _ = DB.invalidate().await;
                            match updated_question {
                                Ok(returned_question) => Ok(returned_question),
                                Err(_) => Err(QuestionError::QuestionUpdateFailure)
                            }
                        },
                        None => Err(QuestionError::QuestionUpdateFailure)
                    }
                },
                Err(_) => {
                    let _ = DB.invalidate().await;
                    Err(QuestionError::QuestionNotFound)
                }
            }
        }
    }
}
