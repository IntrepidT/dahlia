use crate::app::models::{Question, QuestionType};
use leptos::*;

const FIELD_TITLE: &str = "mt-5 font-base text-[#00356b] text-xl";
const INPUT: &str = "w-40 h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
const INPUT_QUESTION: &str = "w-full h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
const SELECTED_BUTTON: &str = "w-40 h-12 border-[#00356b] border pr-4 pl-6 py-4 text-[#00356b] rounded transition-all duration-1000 ease-in-out";
const UNSELECTED_BUTTON: &str = "w-40 h-12 bg-[#00356b] pr-4 pl-6 py-4 text-white rounded transition-all duration-1000 ease-in-out";
const INPUT_SELECTOR: &str ="w-45 h-12 border-[#00356b] border pr-4 pl-6 py-2 text-[#00356b] rounded transition-all duration-1000 ease-in-out";

#[component]
pub fn BuildingQuestion() -> impl IntoView {
    let (word_problem, set_word_problem) = create_signal(String::new());
    let (point_value, set_point_value) = create_signal(String::new());
    let (question_type, set_question_type) = create_signal(String::new());
    let (options, set_question_options) = create_signal(Vec::<String>::new());
    let (correct_answer, set_correct_answer) = create_signal(String::new());
    let (comments, set_comments) = create_signal(String::new());
    let (qnumber, set_qnumber) = create_signal(String::new());

    //im trying to get the actual question builder to work and this may be inefficient.
    let (options_number, set_options_number) = create_signal(3);
    let on_click_add = move |_| {
        set_options_number(options_number() + 1);
    };

    view! {
        <li class="">
            <h1 class=FIELD_TITLE>Question</h1>
            <input type="text" placeholder="Question" class=INPUT_QUESTION
                value=word_problem
                on:input= move |event| {
                    set_word_problem(event_target_value(&event));
                }
            />
            <h1 class=FIELD_TITLE>Point Value</h1>
            <input type="text" placeholder="Points" class=INPUT
                value=point_value
                on:input=move |event| {
                    set_point_value(event_target_value(&event));
                }
            />
            <h1 class=FIELD_TITLE>Question Type</h1>
            <select class=INPUT_SELECTOR
                value=question_type
                on:change=move |event| {
                    set_question_type(event_target_value(&event));
                }
            >
                <option value="">"Please Select a Value"</option>
                <option value="MultipleChoice">"Multiple Choice"</option>
                <option value="Written">"Written"</option>
                <option value="Selection">"Selection"</option>
                <option value="TrueFalse">"True-False"</option>
            </select>
            <Show when=move || {question_type() == "MultipleChoice"}>
                <MultipleChoice />
            </Show>
            <Show when=move || {question_type() == "Written"}>
               <p>"This is the Written Question Builder"</p>
            </Show>
            <Show when=move || {question_type() == "Selection"}>
                <button on:click=on_click_add class=INPUT_SELECTOR>
                    "Add Option"
                </button>
               <p>"This is the Selection Builder"</p>
            </Show>
            <Show when=move || {question_type() == "TrueFalse"}>
                <TrueFalse />
            </Show>
            <hr class="w-full mt-10" />
        </li>
    }
}

#[component]
pub fn MultipleChoice() -> impl IntoView {
    let (options_number, set_options_number) = create_signal(3);
    let on_click_add = move |_| {
        set_options_number(options_number() + 1);
    };
    view! {
        <button on:click=on_click_add class=INPUT_SELECTOR>
            "Add Option"
        </button>
        <p>"This is the Multiple Choice Builder"</p>
        <p>"This is the value of the option counter " {options_number}</p>
    }
}

#[component]
pub fn SelectionQuestion() -> impl IntoView {
    let (options_number, set_options_number) = create_signal(2);
    let on_click_add = move |_| {
        set_options_number(options_number() + 1);
    };
    view! {
        <button on:click=on_click_add class=INPUT_SELECTOR>
            "Add Selection Option"
        </button>
        <p>"This is the Selction Question Builder"</p>
        <p>"This is the value of the option counter " {options_number}</p>
    }
}

#[component]
pub fn TrueFalse() -> impl IntoView {
    let mut true_button_style = UNSELECTED_BUTTON;
    let mut false_button_style = UNSELECTED_BUTTON;
    let (answer, set_answer) = create_signal(false);
    let on_click_true = move |_| {
        set_answer(true);
        true_button_style = &SELECTED_BUTTON;
        false_button_style = &UNSELECTED_BUTTON;
    };
    let on_click_false = move |_| {
        set_answer(false);
        true_button_style = &UNSELECTED_BUTTON;
        false_button_style = &SELECTED_BUTTON;
    };

    view! {
        <h1 class=INPUT>Correct Answer</h1>
        <button on:click=on_click_true class=true_button_style>
            "True"
        </button>
        <button on:click=on_click_false class=false_button_style>
            "False"
        </button>
    }
}
