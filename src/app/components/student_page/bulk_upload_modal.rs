use crate::app::server_functions::bulk_students::test_bulk_function;
use crate::app::server_functions::bulk_students::upload_students_bulk;
use js_sys::Array;
use leptos::ev::Event;
use leptos::*;
use std::sync::mpsc;
use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{FileList, HtmlInputElement};

#[component]
pub fn BulkUploadModal(
    set_show_modal: WriteSignal<bool>,
    set_refresh_trigger: WriteSignal<i32>,
) -> impl IntoView {
    let (file, set_file) = create_signal::<Option<web_sys::File>>(None);
    let (upload_status, set_upload_status) = create_signal(String::new());
    let (is_uploading, set_is_uploading) = create_signal(false);
    let (imported_count, set_imported_count) = create_signal(0);

    let on_file_change = move |ev: Event| {
        let target = ev.target();
        let input_element = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

        if let Some(input) = input_element {
            let files = input.files();
            if let Some(files) = files {
                if files.length() > 0 {
                    if let Some(first_file) = files.item(0) {
                        set_file(Some(first_file));
                    }
                }
            }
        }
    };

    let handle_upload = move |_| {
        set_is_uploading(true);
        set_upload_status(String::new());
        set_imported_count(0);

        if let Some(selected_file) = file() {
            spawn_local(async move {
                match upload_file(selected_file).await {
                    Ok(count) => {
                        set_upload_status(format!("Successfully imported {} students", count));
                        set_imported_count(count);
                        set_refresh_trigger.update(|count| *count += 1);
                        set_is_uploading(false);
                        set_show_modal(false);
                    }
                    Err(e) => {
                        set_upload_status(format!("Upload failed: {}", e));
                        set_is_uploading(false);
                    }
                }
            });
        } else {
            set_upload_status("Please select a file first".to_string());
            set_is_uploading(false);
        }
    };

    let download_template = move |_| {
        let template_content = include_str!("student_csv_template.csv");
        let blob = web_sys::Blob::new_with_str_sequence(&Array::of1(&template_content.into()))
            .unwrap_or_else(|_| web_sys::Blob::new().unwrap());

        let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();

        if let Some(window) = web_sys::window() {
            if let Some(document) = window.document() {
                if let Ok(a) = document.create_element("a") {
                    let _ = a.set_attribute("href", &url);
                    let _ = a.set_attribute("download", "student_template.csv");

                    if let Some(html_element) = a.dyn_ref::<web_sys::HtmlElement>() {
                        html_element.click();
                    }
                }
            }
        }
    };

    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white p-6 rounded-lg shadow-xl max-w-md w-full">
                <h3 class="text-xl font-bold mb-4">"Bulk Student Upload"</h3>

                <input
                    type="file"
                    accept=".csv"
                    on:change=on_file_change
                    class="w-full p-2 border rounded mb-4"
                />

                <div class="text-sm text-gray-600 mb-4 flex justify-between items-center">
                    <span>"Expected CSV format with student details"</span>
                    <button
                        class="text-blue-500 hover:underline"
                        on:click=download_template
                    >
                        "Download Template"
                    </button>
                </div>

                {move || {
                    if !upload_status().is_empty() {
                        let status_class = if upload_status().contains("failed") {
                            "text-red-500"
                        } else {
                            "text-green-500"
                        };

                        Some(view! {
                            <div class={format!("mt-2 {}", status_class)}>
                                {upload_status()}
                                {move || if imported_count() > 0 {
                                    format!(" ({} students)", imported_count())
                                } else {
                                    "".to_string()
                                }}
                            </div>
                        })
                    } else {
                        None
                    }
                }}

                <div class="flex justify-end gap-2 mt-4">
                    <button
                        type="button"
                        class="px-4 py-2 bg-gray-200 rounded hover:bg-gray-300"
                        on:click=move |_| set_show_modal(false)
                    >
                        "Cancel"
                    </button>
                    <button
                        type="button"
                        class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                        on:click=move |_| test_server_function()
                    >
                        "Test Server Function"
                    </button>
                    <button
                        type="button"
                        class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
                        disabled=move || file().is_none() || is_uploading()
                        on:click=handle_upload
                    >
                        {move || if is_uploading() { "Uploading..." } else { "Upload" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

async fn upload_file(file: web_sys::File) -> Result<usize, String> {
    // Create a future that resolves when the file is read
    let file_content_future =
        wasm_bindgen_futures::JsFuture::from(js_sys::Promise::new(&mut |resolve, reject| {
            let reader = web_sys::FileReader::new().unwrap();
            let reader_clone = reader.clone();

            let onload_callback = Closure::once(move |_event: web_sys::ProgressEvent| {
                if let Ok(result) = reader_clone.result() {
                    if let Some(text) = result.as_string() {
                        resolve
                            .call1(&wasm_bindgen::JsValue::NULL, &text.into())
                            .unwrap();
                    } else {
                        reject
                            .call1(
                                &wasm_bindgen::JsValue::NULL,
                                &"Failed to get file content as string".into(),
                            )
                            .unwrap();
                    }
                } else {
                    reject
                        .call1(
                            &wasm_bindgen::JsValue::NULL,
                            &"Failed to get file content".into(),
                        )
                        .unwrap();
                }
            });

            reader.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
            let _ = reader.read_as_text(&file);
            onload_callback.forget();
        }))
        .await
        .map_err(|e| format!("Error reading file: {:?}", e))?;

    // Extract the file content as a string
    let file_contents = file_content_future
        .as_string()
        .ok_or_else(|| "Failed to convert file content to string".to_string())?;

    // Call the server function with the file contents
    upload_students_bulk(file_contents)
        .await
        .map_err(|e| e.to_string())
}

fn test_server_function() {
    spawn_local(async move {
        match test_bulk_function().await {
            Ok(result) => {
                // Success - print to console or update UI
                log::info!("Test function result: {}", result);
                // Or use web_sys to show an alert
                if let Some(window) = web_sys::window() {
                    let _ = window.alert_with_message(&format!("Success: {}", result));
                }
            }
            Err(e) => {
                // Handle error
                log::error!("Test function failed: {}", e);
                if let Some(window) = web_sys::window() {
                    let _ = window.alert_with_message(&format!("Error: {}", e));
                }
            }
        }
    });
}
