use crate::app::components::data_processing::student_results_summary::StudentResultsSummary;
use leptos::*;
use std::collections::HashMap;

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

#[server(GetStudentResultsBatch, "/api")]
pub async fn get_student_results_batch(
    student_ids: Vec<i32>,
) -> Result<HashMap<i32, StudentResultsSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::components::data_processing::student_results_summary::get_student_results;
        let mut results_map: HashMap<i32, StudentResultsSummary> = HashMap::new();

        const CHUNK_SIZE: usize = 30;

        for chunk in student_ids.chunks(CHUNK_SIZE) {
            let mut futures = Vec::with_capacity(chunk.len());

            for &student_id in chunk {
                futures.push(get_student_results(student_id));
            }

            let chunk_results = futures::future::join_all(futures).await;

            for (i, result) in chunk_results.into_iter().enumerate() {
                let student_id = chunk[i];
                match result {
                    Ok(student_result) => {
                        results_map.insert(student_id, student_result);
                    }
                    Err(e) => {
                        log::error!("Failed to fetch result for student {}: {}", student_id, e);
                    }
                }
            }
        }
        Ok(results_map)
    }
}
