use crate::app::components::data_processing::student_results_summary::StudentResultsSummary;
use leptos::prelude::*;
use std::collections::HashMap;

#[server]
pub async fn get_student_results_server(
    student_id: i32,
) -> Result<StudentResultsSummary, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::components::data_processing::student_results_summary::get_student_results;

        get_student_results(student_id)
            .await
            .map_err(|e| ServerFnError::new(format!("There was an issue compiling and processing data for the designated student. Error: {}", e)))
    }
    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::new(String::from(
            "SSR function called on the client",
        )))
    }
}

#[server]
pub async fn get_student_results_batch(
    student_ids: Vec<i32>,
) -> Result<HashMap<i32, StudentResultsSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::components::data_processing::student_results_summary::get_student_results;

        let mut results_map: HashMap<i32, StudentResultsSummary> =
            HashMap::with_capacity(student_ids.len());

        // OPTIMIZATION 1: Process in smaller concurrent batches for better memory usage
        const CHUNK_SIZE: usize = 8; // Reduced from 30 to 8 for better concurrency

        for chunk in student_ids.chunks(CHUNK_SIZE) {
            // Create futures for concurrent processing
            let mut futures = Vec::with_capacity(chunk.len());

            for &student_id in chunk {
                futures.push(get_student_results(student_id));
            }

            // OPTIMIZATION 2: Process chunk concurrently
            let chunk_results = futures::future::join_all(futures).await;

            // OPTIMIZATION 3: Process results efficiently
            for (i, result) in chunk_results.into_iter().enumerate() {
                let student_id = chunk[i];
                match result {
                    Ok(student_result) => {
                        results_map.insert(student_id, student_result);
                    }
                    Err(e) => {
                        log::error!("Failed to fetch result for student {}: {}", student_id, e);
                        // Continue processing other students instead of failing the entire batch
                    }
                }
            }

            // OPTIMIZATION 4: Add a small delay between chunks to prevent overwhelming the database
            if student_ids.len() > CHUNK_SIZE {
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        }

        Ok(results_map)
    }
}

// OPTIMIZATION 5: Add a new endpoint for streaming large datasets
#[server]
pub async fn get_student_results_stream(
    offset: usize,
    limit: usize,
) -> Result<(Vec<StudentResultsSummary>, bool), ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::server_functions::students::get_students;

        // Get all students first
        let all_students = get_students().await?;

        // Apply pagination
        let students_slice = if offset >= all_students.len() {
            Vec::new()
        } else {
            let end = std::cmp::min(offset + limit, all_students.len());
            all_students[offset..end].to_vec()
        };

        let has_more = offset + limit < all_students.len();

        if students_slice.is_empty() {
            return Ok((Vec::new(), false));
        }

        // Get student IDs for batch processing
        let student_ids: Vec<i32> = students_slice.iter().map(|s| s.student_id).collect();

        // Use the optimized batch function
        let results_map = get_student_results_batch(student_ids).await?;

        // Convert back to ordered list
        let results: Vec<StudentResultsSummary> = students_slice
            .into_iter()
            .filter_map(|student| results_map.get(&student.student_id).cloned())
            .collect();

        Ok((results, has_more))
    }
}

// OPTIMIZATION 6: Add caching for frequently accessed data
#[cfg(feature = "ssr")]
pub mod simple_cache {
    use chrono::{DateTime, Duration, Utc};
    use once_cell::sync::Lazy;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    pub struct CacheEntry<T> {
        pub data: T,
        pub expires_at: DateTime<Utc>,
    }

    impl<T> CacheEntry<T> {
        pub fn new(data: T, ttl_minutes: i64) -> Self {
            Self {
                data,
                expires_at: Utc::now() + Duration::minutes(ttl_minutes),
            }
        }

        pub fn is_expired(&self) -> bool {
            Utc::now() > self.expires_at
        }
    }

    type CacheMap<T> = Arc<RwLock<HashMap<String, CacheEntry<T>>>>;

    // Simple cache instances
    static STUDENT_RESULTS_CACHE: Lazy<
        CacheMap<
            crate::app::components::data_processing::student_results_summary::StudentResultsSummary,
        >,
    > = Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

    pub async fn get_cached_student_result(
        student_id: i32,
    ) -> Option<
        crate::app::components::data_processing::student_results_summary::StudentResultsSummary,
    > {
        let cache = STUDENT_RESULTS_CACHE.read().await;
        let key = student_id.to_string();

        if let Some(entry) = cache.get(&key) {
            if !entry.is_expired() {
                return Some(entry.data.clone());
            }
        }
        None
    }

    pub async fn cache_student_result(
        student_id: i32,
        result: crate::app::components::data_processing::student_results_summary::StudentResultsSummary,
    ) {
        let mut cache = STUDENT_RESULTS_CACHE.write().await;
        let key = student_id.to_string();
        cache.insert(key, CacheEntry::new(result, 5)); // Cache for 5 minutes
    }

    pub async fn clear_cache() {
        let mut cache = STUDENT_RESULTS_CACHE.write().await;
        cache.clear();
    }
}

// OPTIMIZATION 7: Enhanced batch function with caching
#[server]
pub async fn get_student_results_batch_cached(
    student_ids: Vec<i32>,
) -> Result<HashMap<i32, StudentResultsSummary>, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::app::components::data_processing::student_results_summary::get_student_results;
        use simple_cache::{cache_student_result, get_cached_student_result};

        let mut results_map: HashMap<i32, StudentResultsSummary> =
            HashMap::with_capacity(student_ids.len());
        let mut uncached_ids = Vec::new();

        // OPTIMIZATION: Check cache first
        for &student_id in &student_ids {
            if let Some(cached_result) = get_cached_student_result(student_id).await {
                results_map.insert(student_id, cached_result);
            } else {
                uncached_ids.push(student_id);
            }
        }

        // Process uncached students in batches
        if !uncached_ids.is_empty() {
            const CHUNK_SIZE: usize = 6; // Smaller chunks for cached version

            for chunk in uncached_ids.chunks(CHUNK_SIZE) {
                let mut futures = Vec::with_capacity(chunk.len());

                for &student_id in chunk {
                    futures.push(get_student_results(student_id));
                }

                let chunk_results = futures::future::join_all(futures).await;

                for (i, result) in chunk_results.into_iter().enumerate() {
                    let student_id = chunk[i];
                    match result {
                        Ok(student_result) => {
                            // Cache the result
                            cache_student_result(student_id, student_result.clone()).await;
                            results_map.insert(student_id, student_result);
                        }
                        Err(e) => {
                            log::error!("Failed to fetch result for student {}: {}", student_id, e);
                        }
                    }
                }

                // Small delay between chunks
                if uncached_ids.len() > CHUNK_SIZE {
                    tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
                }
            }
        }

        log::info!(
            "Batch processed {} students: {} from cache, {} fetched",
            student_ids.len(),
            student_ids.len() - uncached_ids.len(),
            uncached_ids.len()
        );

        Ok(results_map)
    }
}
