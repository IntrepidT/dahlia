use crate::app::components::data_processing::student_results_summary::StudentResultsSummary;
use leptos::*;

#[server(GetStudentResults, "/api", "GetJson")]
pub async fn get_student_results_server(
    student_id: i32,
) -> Result<StudentResultsSummary, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::components::data_processing::student_results_summary::get_student_results;

        get_student_results(student_id)
            .await
            .map_err(|e| ServerFnError::new(format!("Ther was an issue compiling and processing data for the designated student. Error:  {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(String::from(
            "SSR function called on the client",
        )))
    }
}
