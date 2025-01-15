pub mod student;
pub use student::AddStudentRequest;
pub use student::DeleteStudentRequest;
pub use student::EditStudentRequest;
pub use student::Student;

pub mod test;
pub use test::CreateNewTestRequest;
pub use test::DeleteTestRequest;
pub use test::EditTestRequest;
pub use test::{test_type, Test};

pub mod question;
pub use question::CreateNewQuestionRequest;
pub use question::DeleteQuestionRequest;
pub use question::EditQuestionRequest;
pub use question::{Question, QuestionType};
