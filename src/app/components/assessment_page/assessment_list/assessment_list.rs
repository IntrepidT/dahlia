use crate::app::components::assessment_page::assessment_list::assessment_card::AssessmentCard;
use crate::app::models::assessment::Assessment;
use crate::app::models::test::Test;
use leptos::*;
use uuid::Uuid;

#[component]
pub fn AssessmentList(
    assessments: Vec<Assessment>,
    tests: Vec<Test>,
    on_edit: impl Fn(Assessment) + 'static + Copy,
    on_delete: impl Fn(Uuid) + 'static + Copy,
) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow-sm mb-8 overflow-hidden">
            <div class="border-b border-[#DADADA] px-6 py-4">
                <h2 class="text-xl font-medium text-[#2E3A59]">"All Assessments"</h2>
            </div>
            <div class="p-6">
                {if assessments.is_empty() {
                    view! {
                        <div class="text-center py-12">
                            <div class="text-gray-400 text-6xl mb-4">"📋"</div>
                            <h3 class="text-lg font-medium text-gray-900 mb-2">"No assessments yet"</h3>
                            <p class="text-gray-500">"Create your first assessment to get started"</p>
                        </div>
                    }.into_view()
                } else {
                    view! {
                        <div class="grid grid-cols-1 gap-4">
                            {assessments.into_iter().map(|assessment| {
                                view! {
                                    <AssessmentCard
                                        assessment=assessment
                                        tests=tests.clone()
                                        on_edit=on_edit
                                        on_delete=on_delete
                                    />
                                }
                            }).collect_view()}
                        </div>
                    }.into_view()
                }}
            </div>
        </div>
    }
}
