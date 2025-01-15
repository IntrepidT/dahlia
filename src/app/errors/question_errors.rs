use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuestionError {
    #[error("question not found")]
    QuestionNotFound,
    #[error("failed to update question")]
    QuestionUpdateFailure,
    #[error("failed to create question")]
    QuestionCreationFailure,
    #[error("failed to delete question")]
    QuestionDeleteFailure,
}

pub type ErrorMessageQuestion = String;

pub trait ResponseErrorTraitQuestion {
    fn create(question_error: QuestionError) -> ErrorMessageQuestion;
}

impl ResponseErrorTraitQuestion for ErrorMessageQuestion {
    fn create(question_error: QuestionError) -> ErrorMessageQuestion {
        match question_error {
            QuestionError::QuestionNotFound => ErrorMessageQuestion::from("question not found"),
            QuestionError::QuestionUpdateFailure => {
                ErrorMessageQuestion::from("failed to update question")
            }
            QuestionError::QuestionDeleteFailure => {
                ErrorMessageQuestion::from("failed to delete question")
            }
            QuestionError::QuestionCreationFailure => {
                ErrorMessageQuestion::from("failed to create question")
            }
        }
    }
}
