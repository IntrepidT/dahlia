use leptos::prelude::*;
pub mod student_errors;
pub use student_errors::ErrorMessage;
pub use student_errors::ResponseErrorTrait;
pub use student_errors::StudentError;

pub mod test_errors;
pub use test_errors::ErrorMessageTest;
pub use test_errors::ResponseErrorTraitTest;
pub use test_errors::TestError;

pub mod question_errors;
pub use question_errors::ErrorMessageQuestion;
pub use question_errors::QuestionError;
pub use question_errors::ResponseErrorTraitQuestion;
