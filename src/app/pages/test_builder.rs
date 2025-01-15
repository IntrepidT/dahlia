use crate::app::components::header::Header;
use crate::app::components::question_builder::BuildingQuestion;
use crate::app::models::CreateNewQuestionRequest;
use crate::app::server_functions::get_questions;
use leptos::prelude::*;
use leptos::*;

#[component]
pub fn TestBuilder() -> impl IntoView {
    const TAB_BUTTON_STYLE: &str =
        "bg-[#00356b] px-8 py-2 rounded text-white transition-all duration-1000 ease-in-out ml-2";
    const INPUT_STYLE: &str = "w-40 h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
    const INPUT_STYLE_BOX: &str = "mt-5 h-[30rem] w-full max-w-7xl rounded border-[#00356b] border text-wrap pl-4 align-text-top text-start text-[#00356b] align-top transition-all duration-1000 ease-in-out";

    let (selected_tab, set_selected_tab) = create_signal(0);
    let (test_title, set_test_title) = create_signal(String::new());
    let (test_instructs, set_test_instructs) = create_signal(String::new());
    let (question_number, set_question_number) = create_signal(0);

    //let on_submit = move |_| {
    //    let converging_to_question_type = question_type::from_str(&question_area()).clone().unwrap();
    //    let navigate = leptos_router::use_navigae();

    //    let add_question_request = CreateNewQuestionRequest::new(

    //    )
    //};

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
            <form>
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
                                <TinyMCEEditor />
                            //<input type="text" placeholder="Instructions Before Exam" class=INPUT_STYLE_BOX
                            //    value=test_instructs
                            //    on:input=move |event| {
                            //        set_test_instructs(event_target_value(&event));
                            //    }
                            //>
                            </div>
                        }.into_any(),
                        1 => view!{
                            <div>
                                <div class="flex flex-row ml-2 h-full">
                                    //<h1 class="mt-5 font-base text-[#00356b] text-xl">Test Question Builder</h1>
                                    <hr class="w-[60rem] max-w-7xl justify-center items-center ml-3 pl-4 pt-4 mt-4 mr-4 text-[#00356b]" />
                                    <button
                                        class="bg-[#00356b] w-60 px-8 py-2 rounded text-white items-center transition-all duration-1000 inline ease-in-out"
                                        on:click=move |_| {
                                            set_question_number(question_number() + 1);
                                    }>
                                            Add New Question
                                    </button>
                                </div>
                                <ul role="list" class="flex flex-col">
                                    {
                                        //for question in 0..question_number(){
                                        //    view!{<BuildingQuestion/>}
                                        //}.collect_view()
                                    }
                                </ul>
                            </div>
                        }.into_any(),
                        2 => view!{
                            <div>
                                <p>This is the third tab</p>
                                <input type="submit" class="bg-[#00356b] px-8 py-2 rounded text-white items-center transition-all duration-1000 inline ease-in-out"/>
                            </div>
                        }.into_any(),
                        _ => view!{
                            <p>This is the backup tab</p>
                    }.into_any(),
                    }}
                </div>
            </form>
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

//#[component]
//pub fn DynamicList(initial_length: usize) -> impl IntoView {
//    let mut next_counter_id = initial_length;

//    let initial_counters = (0..initial_length)
//        .map(|id| (id, ArcRwSignal::new(id + 1)))
//        .collect::<Vec<_>>();

//    let (counters, set_counters) = signal(initial_counters);

//    let add_counter = move |_| {
//        let sign = ArcRwSignal::new(next_counter_id + 1);

//        set_counters.update(move |counters| counters.push(next_counter_id, sign));

//        next_counter_id += 1;
//    };
//}
