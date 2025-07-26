use crate::app::components::assessment_page::{
    assessment_form::AssessmentForm, assessment_list::AssessmentList,
    shared::hooks::use_assessment_form,
};
use crate::app::components::dashboard::dashboard_sidebar::{DashboardSidebar, SidebarSelected};
use crate::app::components::header::Header;
use crate::app::components::server_auth_components::ServerAuthGuard;
use crate::app::models::assessment::{Assessment, DeleteAssessmentRequest};
use crate::app::models::test::Test;
use crate::app::server_functions::assessments::{delete_assessment, get_assessments};
use crate::app::server_functions::courses::get_courses;
use crate::app::server_functions::tests::get_tests;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn AssessmentPage() -> impl IntoView {
    view! {
        <ServerAuthGuard page_path="/assessments">
            <AssessmentPageContent />
        </ServerAuthGuard>
    }
}

#[component]
pub fn AssessmentPageContent() -> impl IntoView {
    let (selected_view, set_selected_view) = create_signal(SidebarSelected::Assessments);
    let (show_modal, set_show_modal) = create_signal(false);

    // Resources
    let assessments_resource =
        create_local_resource(|| (), |_| async move { get_assessments().await });
    let tests_resource = create_local_resource(|| (), |_| async move { get_tests().await });
    let courses_resource = create_local_resource(|| (), |_| async move { get_courses().await });

    // Form state management
    let form_hook = use_assessment_form();

    // Delete action
    let delete_action = create_action(|id: &Uuid| {
        let id = *id;
        async move {
            let request = DeleteAssessmentRequest::new(1, id);
            delete_assessment(request).await
        }
    });

    // Handle successful form submission
    let handle_form_success = move || {
        assessments_resource.refetch();
    };

    // Handle edit assessment
    let handle_edit_assessment = move |assessment: Assessment| {
        form_hook.load_assessment.call(assessment); // FIXED: Added .call()
        set_show_modal.set(true);
    };

    // Handle delete assessment
    let handle_delete_assessment = move |id: Uuid| {
        delete_action.dispatch(id);
    };

    // Handle new assessment
    let handle_new_assessment = move || {
        form_hook.reset_form.call(()); // FIXED: Added .call(())
        set_show_modal.set(true);
    };

    // Effect to refetch assessments after successful deletion
    create_effect(move |_| {
        if let Some(Ok(_)) = delete_action.value().get() {
            assessments_resource.refetch();
        }
    });

    view! {
        <div class="min-h-screen bg-[#F9F9F8] text-[#2E3A59] font-sans">
            <Header />
            <DashboardSidebar
                selected_item=selected_view
                set_selected_item=set_selected_view
            />
            <div class="max-w-6xl mx-auto px-4 py-8">
                <AssessmentPageHeader on_new_assessment=handle_new_assessment />

                <AssessmentListSection
                    assessments_resource=assessments_resource
                    tests_resource=tests_resource
                    on_edit=handle_edit_assessment
                    on_delete=handle_delete_assessment
                />

                <AssessmentForm
                    show_modal=show_modal
                    set_show_modal=set_show_modal
                    form_hook=form_hook
                    tests_resource=tests_resource
                    courses_resource=courses_resource
                    on_success=handle_form_success
                />
            </div>
        </div>
    }
}

#[component]
fn AssessmentPageHeader(on_new_assessment: impl Fn() + 'static + Copy) -> impl IntoView {
    view! {
        <div class="flex justify-between">
            <h1 class="text-3xl font-medium mb-8 text-[#2E3A59]">"Assessments"</h1>
            <div class="mb-8">
                <button
                    class="bg-[#2E3A59] text-white px-4 py-2 rounded shadow-md hover:opacity-90 transition-opacity text-sm font-medium"
                    on:click=move |_| on_new_assessment()
                >
                    "Add New Assessment"
                </button>
            </div>
        </div>
    }
}

#[component]
fn AssessmentListSection(
    assessments_resource: Resource<(), Result<Vec<Assessment>, ServerFnError>>,
    tests_resource: Resource<(), Result<Vec<Test>, ServerFnError>>,
    on_edit: impl Fn(Assessment) + 'static + Copy,
    on_delete: impl Fn(Uuid) + 'static + Copy,
) -> impl IntoView {
    view! {
        {move || {
            match (assessments_resource.get(), tests_resource.get()) {
                (Some(Ok(assessments)), Some(Ok(tests))) => {
                    view! {
                        <AssessmentList
                            assessments=assessments
                            tests=tests
                            on_edit=on_edit
                            on_delete=on_delete
                        />
                    }.into_view()
                },
                (Some(Err(e)), _) | (_, Some(Err(e))) => {
                    view! {
                        <div class="bg-white rounded-lg shadow-sm mb-8 overflow-hidden">
                            <div class="p-6">
                                <div class="p-4 bg-red-50 text-red-700 rounded border border-red-200">
                                    "Error loading data: " {e.to_string()}
                                </div>
                            </div>
                        </div>
                    }.into_view()
                },
                _ => {
                    view! {
                        <div class="bg-white rounded-lg shadow-sm mb-8 overflow-hidden">
                            <div class="border-b border-[#DADADA] px-6 py-4">
                                <h2 class="text-xl font-medium text-[#2E3A59]">"All Assessments"</h2>
                            </div>
                            <div class="p-6">
                                <div class="flex items-center justify-center py-12">
                                    <div class="text-center">
                                        <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-[#2E3A59] mx-auto mb-4"></div>
                                        <p class="text-gray-500">"Loading assessments..."</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }.into_view()
                }
            }
        }}
    }
}
