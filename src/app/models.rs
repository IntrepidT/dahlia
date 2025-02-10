pub mod student;
pub use student::AddStudentRequest;
pub use student::DeleteStudentRequest;
pub use student::Student;
pub use student::UpdateStudentRequest;

pub mod test;
pub use test::CreateNewTestRequest;
pub use test::DeleteTestRequest;
pub use test::UpdateTestRequest;
pub use test::{Test, TestType};

pub mod question;
pub use question::CreateNewQuestionRequest;
pub use question::DeleteQuestionRequest;
pub use question::UpdateQuestionRequest;
pub use question::{Question, QuestionType};
