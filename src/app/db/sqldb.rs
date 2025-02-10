cfg_if::cfg_if!{

  if #[cfg(feature = "ssr")]{

    use crate::app::models::{Student, Test, test_type, QuestionType, Question};
    use crate::app::errors::{ErrorMessage, StudentError, TestError, ErrorMessageTest, QuestionError, ErrorMessageQuestion};
    use sqlx::{migrate::MigrateDatabase, FromRow, Row, Sqlite, SqlitePool, SqliteConnection};

    let pool = SqlitePool::connect(&env::var("data")?).await?;
    
  }
}
