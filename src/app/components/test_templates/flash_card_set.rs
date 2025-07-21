use crate::app::components::flash_cards::*;
use crate::app::models::score::CreateScoreRequest;
use crate::app::models::user::SessionUser;
use crate::app::server_functions::{
    questions::get_questions, scores::add_score, tests::get_tests, users::get_user,
};
use leptos::*;
use leptos_router::*;
use log;
use std::collections::HashMap;

#[component]
pub fn FlashCardSet() -> impl IntoView {
    // Get test_id from URL parameters
    let params = use_params_map();
    let test_id = move || params.with(|params| params.get("test_id").cloned().unwrap_or_default());
    let user = use_context::<ReadSignal<Option<SessionUser>>>().expect("AuthProvider not Found");

    // Create resource to fetch test details
    let test_details = create_resource(test_id.clone(), move |tid| async move {
        if tid.is_empty() {
            log::warn!("No test ID provided in URL");
            return None;
        }
        match get_tests().await {
            Ok(tests) => tests.into_iter().find(|test| test.test_id == tid),
            Err(e) => {
                log::error!("Failed to fetch test details: {}", e);
                None
            }
        }
    });

    // Create resource that depends on the test_id from URL
    let questions = create_resource(test_id, move |tid| async move {
        if tid.is_empty() {
            log::warn!("No test ID provided in URL");
            return Vec::new();
        }
        match get_questions(tid).await {
            Ok(mut questions) => {
                // Sort questions by qnumber to ensure consistent ordering
                questions.sort_by_key(|q| q.qnumber);
                questions
            }
            Err(e) => {
                log::error!("Failed to fetch questions: {}", e);
                Vec::new()
            }
        }
    });

    // Get evaluator ID
    let evaluator_id = create_memo(move |_| match user.get() {
        Some(user_data) => user_data.id.to_string(),
        None => "0".to_string(),
    });

    // Create submission action using the helper
    let submission_action = create_submission_action(
        Signal::derive(test_id),
        Signal::derive(move || questions.get()),
        Signal::derive(evaluator_id),
    );

    view! {
        <Suspense
            fallback=move || view! {
                <div class="flex items-center justify-center h-96">
                    <div class="flex flex-col items-center gap-4">
                        <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin"></div>
                        <p class="text-gray-500 text-sm">"Loading questions..."</p>
                    </div>
                </div>
            }
        >
            {move || match (questions.get(), test_details.get()) {
                (None, _) => view! {
                    <div class="flex items-center justify-center h-96">
                        <div class="text-center">
                            <div class="w-8 h-8 border-2 border-blue-500 border-t-transparent rounded-full animate-spin mx-auto mb-4"></div>
                            <p class="text-gray-500">"Loading..."</p>
                        </div>
                    </div>
                }.into_view(),
                (Some(questions_vec), _) if questions_vec.is_empty() => {
                    view! {
                        <div class="flex items-center justify-center h-96">
                            <div class="text-center">
                                <div class="w-16 h-16 bg-red-50 rounded-full flex items-center justify-center mx-auto mb-4">
                                    <svg class="w-8 h-8 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.732-.833-2.5 0L4.268 18.5c-.77.833.192 2.5 1.732 2.5z"></path>
                                    </svg>
                                </div>
                                <p class="text-gray-500">"No questions found for this test."</p>
                            </div>
                        </div>
                    }.into_view()
                },
                (Some(questions_vec), test_data) => {
                    view! {
                        <FlashCardContainer
                            questions=questions_vec
                            test_details=test_data.flatten()
                            user=user.get()
                            on_submit=Callback::new(move |data| {
                                submission_action.dispatch(data);
                            })
                        />
                    }.into_view()
                }
            }}
        </Suspense>
    }
}
