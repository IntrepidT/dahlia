use crate::app::components::header::Header;
use crate::app::components::question_builder::BuildingQuestion;
use crate::app::models::{CreateNewQuestionRequest, Question, QuestionType};
use crate::app::server_functions::questions::{add_question, get_questions};
use crate::app::server_functions::tests::score_overrider;
use leptos::prelude::*;
use leptos::*;
use leptos_router::*;

#[component]
pub fn TestBuilder() -> impl IntoView {
    const TAB_BUTTON_STYLE: &str =
        "bg-[#00356b] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out ml-2";
    const INPUT_STYLE: &str = "w-40 h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
    const INPUT_STYLE_BOX: &str = "mt-5 h-[30rem] w-full max-w-7xl rounded border-[#00356b] border text-wrap pl-4 align-text-top text-start text-[#00356b] align-top transition-all duration-1000 ease-in-out";

    //get the test_id from the URL parameters
    let params = use_params_map();
    //let test_id = create_memo(move |_| {
    //    params.with(|params| params.get("test_id").cloned().unwrap_or_default())
    //});
    let test_id = move || params().get("test_id").cloned().unwrap_or_default();

    let (selected_tab, set_selected_tab) = create_signal(0);
    let (test_title, set_test_title) = create_signal(String::new());

    //let (test_instructs, set_test_instructs) = create_signal(String::new());
    //let (question_number, set_question_number) = create_signal(0);

    let (questions, set_questions) = create_signal(Vec::<Question>::new());

    let mut score_counter = 0;

    let add_new_question = move |_| {
        set_questions.update(|qs| {
            let new_question = Question::new(
                String::new(),
                0,
                QuestionType::Selection,
                Vec::new(),
                String::new(),
                (qs.len() + 1) as i32,
                test_id(),
            );
            qs.push(new_question);
        });
    };

    let update_question = move |index: usize, updated_question: Question| {
        set_questions.update(|qs| {
            if index < qs.len() {
                qs[index] = updated_question;
            }
        });
    };

    let remove_question = move |index: usize| {
        set_questions.update(|qs| {
            qs.remove(index);

            for (i, q) in qs.iter_mut().enumerate() {
                q.qnumber = (i + 1) as i32;
            }
        });
    };

    let on_submit = move |_| {
        let test_id_value = test_id();

        if test_id_value.is_empty() {
            log::error!("Invalid test ID");
            return;
        }

        let current_questions = questions.get();

        let valid_questions: Vec<_> = current_questions
            .into_iter()
            .filter(|q| {
                let is_valid = match &q.question_type {
                    QuestionType::MultipleChoice => {
                        !q.word_problem.is_empty()
                            && q.point_value > 0
                            && !q.options.is_empty()
                            && !q.correct_answer.is_empty()
                            && q.options.contains(&q.correct_answer)
                    }
                    QuestionType::TrueFalse => {
                        !q.word_problem.is_empty()
                            && q.point_value > 0
                            && (q.correct_answer == "true" || q.correct_answer == "false")
                            && !q.correct_answer.is_empty()
                    }
                    _ => false,
                };
                if !is_valid {
                    log::warn!("Invalid question found: {:?}", q);
                }
                is_valid
            })
            .collect();

        if valid_questions.is_empty() {
            log::error!("No valid questions to submit");
            return;
        }

        let question_requests: Vec<CreateNewQuestionRequest> = valid_questions
            .into_iter()
            .map(|q| {
                let request = CreateNewQuestionRequest::new(
                    q.word_problem.clone(),
                    q.point_value,
                    q.question_type.clone(),
                    q.options.clone(),
                    q.correct_answer.clone(),
                    q.qnumber,
                    test_id(),
                );
                request
            })
            .collect();

        spawn_local(async move {
            for question in question_requests {
                score_counter += question.point_value;
                match add_question(test_id(), question.clone()).await {
                    Ok(_) => log::info!("Added question {}", question.qnumber),
                    Err(e) => log::error!("Failed to add question {}: {:?}", question.qnumber, e),
                }
            }
            log::info!("Total score counter: {}", score_counter);
            match score_overrider(test_id(), score_counter).await {
                Ok(test) => log::info!("Score updated successfully. New score: {}", test.score),
                Err(e) => log::error!("Failed to update score: {:?}", e),
            }

            let navigate = leptos_router::use_navigate();
            navigate("/mathtesting", Default::default());
        });
    };

    view! {
        <Header />
        <main class="z-auto w-full aspect-video max-w-7xl mx-auto mt-20">
            <div class="flex flex-row w-full">
                <h1 class="text-2xl font-bold leading-7 text-[#00356b]">
                    Test Builder
                </h1>
                <hr class="w-[60rem] max-w-5xl inline justify-center items-center ml-3 pl-4 pt-4 mt-4 mr-4 text-[#00356b]" />
            </div>
            <div class="tab-headers mt-2">
                <button
                    class=TAB_BUTTON_STYLE
                    on:click=move |_| set_selected_tab.set(0)
                >
                    Instructions for Test
                </button>
                <button
                    class=TAB_BUTTON_STYLE
                    on:click=move |_| set_selected_tab.set(1)
                >
                    Question Builder
                </button>
                <button
                    class=TAB_BUTTON_STYLE
                    on:click=move |_| set_selected_tab.set(2)
                >
                    Additional Options
                </button>
            </div>
            <div class="tab-content">
                {move || match selected_tab() {
                    0 => view!{
                        <div class="z-auto flex flex-col ml-2 h-full">
                            <h1 class="mt-10 font-base text-[#00356b] text-xl">Test Title</h1>
                            <input type="text" placeholder="Displayed Title" class=INPUT_STYLE
                                value=test_title
                                on:input=move |event| {
                                    set_test_title(event_target_value(&event));
                                }
                            />
                            <h1 class="mt-5 font-base text-[#00356b] text-xl">Test Instructions</h1>
                            //<TinyMCEEditor />
                            <textarea
                                placeholder="Instructions Before Exam"
                                class=INPUT_STYLE_BOX
                            ></textarea>
                        </div>
                    }.into_any(),
                    1 => view!{
                        <div>
                            <div class="flex flex-row ml-2 h-full">
                                //<h1 class="mt-5 font-base text-[#00356b] text-xl">Test Question Builder</h1>
                                <hr class="w-[60rem] max-w-7xl justify-center items-center ml-3 pl-4 pt-4 mt-4 mr-4 text-[#00356b]" />
                                <button
                                    class="bg-[#00356b] w-60 px-8 py-2 rounded text-white items-center transition-all duration-1000 inline ease-in-out"
                                    on:click=add_new_question
                                >
                                    Add New Question
                                </button>
                            </div>
                            <For
                                each=move || questions.get()
                                key=|question| question.qnumber
                                children=move |question: Question| {
                                    let index= questions.get().iter().position(|q| q.qnumber == question.qnumber).unwrap_or(0);
                                    view! {
                                        <BuildingQuestion
                                            initial_question=question.clone()
                                            on_update=Callback::new(move |updated_q| update_question(index, updated_q))
                                            on_remove=Callback::new(move |_| remove_question(index))
                                        />
                                    }
                                }
                            />
                        </div>
                    }.into_any(),
                    2 => view!{
                        <div>
                            <button
                                class="bg-[#00356b] mt-20 px-8 py-2 rounded text-white items-center transition-all duration-1000 inline ease-in-out"
                                on:click=on_submit
                            >
                                Submit All Questions
                            </button>
                        </div>
                    }.into_any(),
                    _ => view!{<p>This is the backup tab</p>}.into_any(),
                }}
            </div>
        </main>
    }
}

#[component]
fn TinyMCEEditor() -> impl IntoView {
    view! {
        <html lang="en">
        <head>
            <script
                type="text/javascript"
                src="/static/tinymce/tinymce.min.js"
                referrerpolicy="origin">
            </script>
            <script type="text/javascript">
            tinymce.init({
                selector: "#myTextarea",
                width: 1200,
                height: 500,
                plugins: [
                    "advlist", "autosave", "autolink", "link", "image", "lists", "charmap", "preview", "anchor", "pagebreak",
                    "wordcount", "visualblocks", "visualchars", "insertdatetime",
                    "media", "table", "save"
                ],
                toolbar: "undo redo | styles | bold italic | alignleft aligncenter alignright alignjustify | " +
                    "bullist numlist outdent indent | link image | " +
                    "forecolor backcolor save",
                autosave_ask_before_unload: true,
                menubar: "file edit view insert format tools table",
                autosave_interval: "10s",
                autosave_restore_when_empty: true,
                license_key: "gpl",
            });
            </script>
        </head>

        <body>
            <textarea id="myTextarea" placeholder="Please Write Your Instructions Here" class="border-[#00356b] border"></textarea>
        </body>
        </html>
    }
}
