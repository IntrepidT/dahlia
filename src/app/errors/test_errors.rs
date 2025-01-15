use thiserror::Error;

#[derive(Error, Debug)]
pub enum TestError {
    #[error("student not found")]
    TestNotFound,
    #[error("failed to update student")]
    TestUpdateFailure,
    #[error("failed to create student")]
    TestCreationFailure,
    #[error("failed to delete student")]
    TestDeleteFailure,
}

pub type ErrorMessageTest = String;

pub trait ResponseErrorTraitTest {
    fn create(test_error: TestError) -> ErrorMessageTest;
}

impl ResponseErrorTraitTest for ErrorMessageTest {
    fn create(test_error: TestError) -> ErrorMessageTest {
        match test_error {
            TestError::TestNotFound => ErrorMessageTest::from("test not found"),
            TestError::TestUpdateFailure => ErrorMessageTest::from("failed to update test"),
            TestError::TestCreationFailure => ErrorMessageTest::from("failed to create test"),
            TestError::TestDeleteFailure => ErrorMessageTest::from("failed to delete test"),
        }
    }
}
