use crate::app::server_functions::bulk_enrollment::upload_bulk_enrollment;
use crate::app::server_functions::bulk_students::upload_students_bulk;
use leptos::ev::{Event, MouseEvent};
use leptos::prelude::*;
use leptos::task::spawn_local;

#[cfg(feature = "hydrate")]
use js_sys::Array;
#[cfg(feature = "hydrate")]
use std::sync::mpsc;
#[cfg(feature = "hydrate")]
use wasm_bindgen::{closure::Closure, JsCast};
#[cfg(feature = "hydrate")]
use web_sys::{FileList, HtmlInputElement};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImportType {
    Students,
    Enrollments,
}

impl ImportType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ImportType::Students => "Students",
            ImportType::Enrollments => "Enrollments",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ImportType::Students => {
                "Import student records with basic information (teacher names as text)"
            }
            ImportType::Enrollments => {
                "Import enrollment records with academic year and teacher IDs"
            }
        }
    }

    pub fn template_filename(&self) -> &'static str {
        match self {
            ImportType::Students => "student_template.csv",
            ImportType::Enrollments => "enrollment_template.csv",
        }
    }

    pub fn template_content(&self) -> &'static str {
        match self {
            ImportType::Students => include_str!("student_csv_template.csv"),
            ImportType::Enrollments => include_str!("enrollment_csv_template.csv"),
        }
    }
}

#[component]
pub fn BulkUploadModal(
    set_show_modal: WriteSignal<bool>,
    set_refresh_trigger: WriteSignal<i32>,
) -> impl IntoView {
    let (upload_status, set_upload_status) = signal(String::new());
    let (is_uploading, set_is_uploading) = signal(false);
    let (imported_count, set_imported_count) = signal(0);
    let (import_type, set_import_type) = signal(ImportType::Students);

    #[cfg(feature = "hydrate")]
    let (file, set_file) = signal_local(None::<web_sys::File>);

    let on_file_change = move |ev: Event| {
        #[cfg(feature = "hydrate")]
        {
            let target = ev.target();
            let input_element = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());

            if let Some(input) = input_element {
                let files = input.files();
                if let Some(files) = files {
                    if files.length() > 0 {
                        if let Some(first_file) = files.item(0) {
                            set_file.set(Some(first_file));
                        }
                    }
                }
            }
        }
    };

    #[cfg(feature = "hydrate")]
    let handle_upload = move |ev: MouseEvent| {
        set_is_uploading.set(true);
        set_upload_status.set(String::new());
        set_imported_count.set(0);

        #[cfg(feature = "hydrate")]
        {
            if let Some(selected_file) = file.get() {
                let current_import_type = import_type.get();
                spawn_local(async move {
                    match upload_file(selected_file, current_import_type).await {
                        Ok(count) => {
                            set_upload_status.set(format!(
                                "Successfully imported {} {}",
                                count,
                                if count == 1 {
                                    current_import_type.display_name().trim_end_matches('s')
                                } else {
                                    current_import_type.display_name()
                                }
                            ));
                            set_imported_count.set(count);
                            set_refresh_trigger.update(|count| *count += 1);
                            set_is_uploading.set(false);

                            // Auto-close modal after successful upload
                            set_timeout(
                                move || set_show_modal.set(false),
                                std::time::Duration::from_millis(2000),
                            );
                        }
                        Err(e) => {
                            set_upload_status.set(format!("Upload failed: {}", e));
                            set_is_uploading.set(false);
                        }
                    }
                });
            } else {
                set_upload_status.set("Please select a file first".to_string());
                set_is_uploading.set(false);
            }
        }
    };

    let download_template = move |_| {
        #[cfg(feature = "hydrate")]
        {
            let current_type = import_type.get();
            let template_content = current_type.template_content();
            let filename = current_type.template_filename();

            let blob = web_sys::Blob::new_with_str_sequence(&Array::of1(&template_content.into()))
                .unwrap_or_else(|_| web_sys::Blob::new().unwrap());

            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();

            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    if let Ok(a) = document.create_element("a") {
                        let _ = a.set_attribute("href", &url);
                        let _ = a.set_attribute("download", filename);

                        if let Some(html_element) = a.dyn_ref::<web_sys::HtmlElement>() {
                            html_element.click();
                        }
                    }
                }
            }
        }
    };

    // Reset file when import type changes
    Effect::new(move |_| {
        import_type.get();
        #[cfg(feature = "hydrate")]
        set_file.set(None);
        set_upload_status.set(String::new());
    });

    view! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-[#F9F9F8] p-6 rounded-lg shadow-xl max-w-lg w-full">
                <h3 class="text-xl font-bold mb-4">"Bulk Data Upload"</h3>

                // Import type selection
                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        "Import Type"
                    </label>
                    <select
                        class="w-full p-2 border rounded"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            match value.as_str() {
                                "students" => set_import_type.set(ImportType::Students),
                                "enrollments" => set_import_type.set(ImportType::Enrollments),
                                _ => {}
                            }
                        }
                    >
                        <option value="students" selected=move || import_type.get() == ImportType::Students>
                            "Students"
                        </option>
                        <option value="enrollments" selected=move || import_type.get() == ImportType::Enrollments>
                            "Enrollments"
                        </option>
                    </select>
                </div>

                // Description
                <div class="mb-4 p-3 bg-blue-50 rounded border-l-4 border-blue-400">
                    <p class="text-sm text-blue-800">
                        {move || import_type.get().description()}
                    </p>
                </div>

                // File input
                <input
                    type="file"
                    accept=".csv"
                    on:change=on_file_change
                    class="w-full p-2 border rounded mb-4"
                />

                // Template download section
                <div class="text-sm text-gray-600 mb-4 flex justify-between items-center">
                    <span>
                        {move || format!("Expected CSV format for {} import", import_type.get().display_name().to_lowercase())}
                    </span>
                    <button
                        class="text-blue-500 hover:underline"
                        on:click=download_template
                    >
                        "Download Template"
                    </button>
                </div>

                // Important notes based on import type
                <div class="mb-4 text-xs text-gray-600 bg-gray-50 p-2 rounded">
                    {move || match import_type.get() {
                        ImportType::Students => view! {
                            <div>
                                <strong>"Student Import Notes:"</strong>
                                <ul class="list-disc list-inside mt-1">
                                    <li>"Teacher field uses teacher names (text)"</li>
                                    <li>"All boolean fields: true/false"</li>
                                    <li>"Date format: YYYY-MM-DD"</li>
                                    <li>"Intervention: None, Literacy, Math, or 'Literacy and Math'"</li>
                                </ul>
                            </div>
                        },
                        ImportType::Enrollments => view! {
                            <div>
                                <strong>"Enrollment Import Notes:"</strong>
                                <ul class="list-disc list-inside mt-1">
                                    <li>"teacher_id field uses numeric IDs from employees table"</li>
                                    <li>"student_id must exist in students table"</li>
                                    <li>"Academic year format: 2024-2025"</li>
                                    <li>"Status/dates are auto-set (Active, current date)"</li>
                                </ul>
                            </div>
                        }
                    }}
                </div>

                // Status message
                {move || {
                    if !upload_status.get().is_empty() {
                        let status_class = if upload_status.get().contains("failed") || upload_status.get().contains("error") {
                            "text-red-500"
                        } else {
                            "text-green-500"
                        };

                        Some(view! {
                            <div class={format!("mt-2 {}", status_class)}>
                                {upload_status.get()}
                                {move || if imported_count.get() > 0 {
                                    format!(" ({} records)", imported_count.get())
                                } else {
                                    "".to_string()
                                }}
                            </div>
                        })
                    } else {
                        None
                    }
                }}

                // Action buttons
                <div class="flex justify-end gap-2 mt-4">
                    <button
                        type="button"
                        class="px-4 py-2 text-white bg-[#F44336] rounded hover:bg-[#D32F2F]"
                        on:click=move |_| set_show_modal.set(false)
                    >
                        "Cancel"
                    </button>

                    {
                        #[cfg(feature = "hydrate")]
                        {
                            view! {
                                <button
                                    type="button"
                                    class="px-4 py-2 bg-[#4CAF50] text-white rounded hover:bg-[#388E3C] disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || file.get().is_none() || is_uploading.get()
                                    on:click=handle_upload
                                >
                                    {move || if is_uploading.get() {
                                        format!("Uploading {}...", import_type.get().display_name())
                                    } else {
                                        format!("Upload {}", import_type.get().display_name())
                                    }}
                                </button>
                            }
                        }
                    }
                </div>
            </div>
        </div>
    }
}

#[cfg(feature = "hydrate")]
async fn upload_file(file: web_sys::File, import_type: ImportType) -> Result<usize, String> {
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

    // Call the appropriate server function based on import type
    match import_type {
        ImportType::Students => {
            crate::app::server_functions::bulk_students::upload_students_bulk(file_contents)
                .await
                .map_err(|e| e.to_string())
        }
        ImportType::Enrollments => {
            crate::app::server_functions::bulk_enrollment::upload_bulk_enrollment(file_contents)
                .await
                .map_err(|e| e.to_string())
        }
    }
}
