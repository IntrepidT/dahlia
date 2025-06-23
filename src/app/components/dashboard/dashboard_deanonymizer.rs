use crate::app::components::auth::enhanced_login_form::{
    StudentMappingService, use_student_mapping_service, StudentMappingData, StudentMapping
};
use crate::app::middleware::global_settings::use_settings;
use leptos::*;

#[cfg(feature = "hydrate")]
use wasm_bindgen::{closure::Closure, JsCast};

#[component]
pub fn DashboardDeanonymizer() -> impl IntoView {
    let (student_mapping_file, set_student_mapping_file) = create_signal::<Option<String>>(None);
    let (file_upload_status, set_file_upload_status) = create_signal::<Option<String>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (is_expanded, set_is_expanded) = create_signal(false);

    // Get settings and mapping service
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;
    let (mapping_service_signal, set_student_mapping_service) = use_student_mapping_service();

    // Check if mapping service is already active
    let has_active_mapping = move || mapping_service_signal.get().is_some();
    let mapping_count = move || {
        mapping_service_signal.get()
            .map(|service| service.get_mapping_count())
            .unwrap_or(0)
    };

    // CSV parsing (copy from enhanced_login_form.rs)
    let parse_csv_content = move |content: String| -> Result<StudentMappingData, String> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err("Empty file".to_string());
        }

        if lines.len() < 2 {
            return Err("File must contain header and at least one data row".to_string());
        }

        let header = lines[0];
        let expected_headers = [
            "app_id",
            "original_student_id", 
            "firstname",
            "lastname",
            "pin",
            "created_at",
        ];
        let actual_headers: Vec<&str> = header.split(',').map(|h| h.trim()).collect();

        if actual_headers.len() != expected_headers.len() {
            return Err(format!(
                "Invalid header format: expected {} columns, found {}",
                expected_headers.len(),
                actual_headers.len()
            ));
        }

        // Parse data rows
        let data_lines = &lines[1..];
        let mut mappings = Vec::new();

        for (line_num, line) in data_lines.iter().enumerate() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() != 6 {
                return Err(format!(
                    "Invalid CSV format at line {}: expected 6 columns, found {}",
                    line_num + 2,
                    parts.len()
                ));
            }

            let app_id = parts[0].trim().parse::<i32>()
                .map_err(|_| format!("Invalid app_id at line {}: '{}'", line_num + 2, parts[0]))?;

            let original_student_id = parts[1].trim().parse::<i32>()
                .map_err(|_| format!("Invalid original_student_id at line {}: '{}'", line_num + 2, parts[1]))?;

            if app_id <= 0 || original_student_id <= 0 {
                return Err(format!(
                    "Invalid ID values at line {}: IDs must be positive integers",
                    line_num + 2
                ));
            }

            mappings.push(StudentMapping {
                app_id,
                original_student_id,
                firstname: parts[2].trim().to_string(),
                lastname: parts[3].trim().to_string(),
                pin: parts[4].trim().to_string(),
                created_at: parts[5].trim().to_string(),
            });
        }

        if mappings.is_empty() {
            return Err("No valid data rows found in CSV file".to_string());
        }

        Ok(StudentMappingData { mappings })
    };

    // File upload handler
    #[cfg(feature = "hydrate")]
    let handle_file_upload = move |ev: web_sys::Event| {
        let input = ev.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        if let Some(files) = input.files() {
            if files.length() > 0 {
                let file = files.get(0).unwrap();

                if !file.name().ends_with(".csv") {
                    set_error.set(Some("Please select a CSV file".to_string()));
                    return;
                }

                if file.size() > 10_000_000.0 {
                    set_error.set(Some("File too large. Maximum size is 10MB".to_string()));
                    return;
                }

                let file_reader = web_sys::FileReader::new().unwrap();
                set_file_upload_status.set(Some("Processing file...".to_string()));
                set_error.set(None);

                let onload = Closure::wrap(Box::new({
                    let set_error = set_error.clone();
                    let set_file_upload_status = set_file_upload_status.clone();
                    let set_student_mapping_service = set_student_mapping_service.clone();
                    let parse_csv_content = parse_csv_content.clone();

                    move |event: web_sys::Event| {
                        let file_reader = event.target().unwrap()
                            .dyn_into::<web_sys::FileReader>().unwrap();

                        if let Ok(result) = file_reader.result() {
                            if let Some(content) = result.as_string() {
                                match parse_csv_content(content) {
                                    Ok(mapping_data) => {
                                        let mapping_service = StudentMappingService::new(mapping_data.mappings);
                                        let count = mapping_service.get_mapping_count();
                                        set_student_mapping_service.set(Some(mapping_service));
                                        set_error.set(None);
                                        set_file_upload_status.set(Some(format!(
                                            "✓ De-anonymization active ({} mappings loaded)",
                                            count
                                        )));
                                        logging::log!("Student mapping service activated with {} mappings", count);
                                    }
                                    Err(e) => {
                                        set_error.set(Some(format!("Invalid CSV: {}", e)));
                                        set_file_upload_status.set(None);
                                    }
                                }
                            }
                        }
                    }
                }) as Box<dyn Fn(web_sys::Event)>);

                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
                let _ = file_reader.read_as_text(&file);
            }
        }
    };

    #[cfg(not(feature = "hydrate"))]
    let handle_file_upload = move |_| {
        set_error.set(Some("File upload not supported in this environment".to_string()));
    };

    let clear_mapping = move |_| {
        set_student_mapping_service.set(None);
        set_file_upload_status.set(None);
        set_error.set(None);
        logging::log!("Student mapping service cleared");
    };

    view! {
        {move || {
            if anonymization_enabled() {
                view! {
                    <div class="bg-white rounded-lg shadow-md border border-gray-200 mb-6">
                        <div class="p-4 border-b border-gray-200">
                            <button
                                class="flex items-center justify-between w-full text-left"
                                on:click=move |_| set_is_expanded.set(!is_expanded.get())
                            >
                                <div class="flex items-center">
                                    <svg class="w-5 h-5 mr-2 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-3a1 1 0 011-1h2.586l6.414-6.414a6 6 0 015.743-7.743z"/>
                                    </svg>
                                    <h3 class="text-lg font-semibold text-gray-800">
                                        "Student De-anonymization"
                                    </h3>
                                </div>
                                <div class="flex items-center">
                                    {move || {
                                        if has_active_mapping() {
                                            view! {
                                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-green-100 text-green-800 mr-2">
                                                    "Active (" {mapping_count()} " mappings)"
                                                </span>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gray-100 text-gray-800 mr-2">
                                                    "Inactive"
                                                </span>
                                            }.into_view()
                                        }
                                    }}
                                    <svg 
                                        class=move || if is_expanded.get() { "w-5 h-5 transform rotate-180 transition-transform" } else { "w-5 h-5 transition-transform" }
                                        fill="none" stroke="currentColor" viewBox="0 0 24 24"
                                    >
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                                    </svg>
                                </div>
                            </button>
                        </div>

                        {move || {
                            if is_expanded.get() {
                                view! {
                                    <div class="p-4">
                                        {move || {
                                            error.get().map(|err| {
                                                view! {
                                                    <div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded-md text-sm">
                                                        {err}
                                                    </div>
                                                }
                                            })
                                        }}

                                        {move || {
                                            if has_active_mapping() {
                                                view! {
                                                    <div class="bg-green-50 border border-green-200 rounded-md p-4 mb-4">
                                                        <div class="flex items-center justify-between">
                                                            <div>
                                                                <h4 class="text-sm font-medium text-green-800">
                                                                    "De-anonymization Active"
                                                                </h4>
                                                                <p class="text-sm text-green-700 mt-1">
                                                                    "Student data will show real names and IDs (" {mapping_count()} " mappings loaded)"
                                                                </p>
                                                            </div>
                                                            <button
                                                                class="text-sm bg-red-600 text-white px-3 py-1 rounded hover:bg-red-700 transition-colors"
                                                                on:click=clear_mapping
                                                            >
                                                                "Disable"
                                                            </button>
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            } else {
                                                view! {
                                                    <div class="space-y-4">
                                                        <div class="bg-blue-50 border border-blue-200 rounded-md p-4">
                                                            <h4 class="text-sm font-medium text-blue-800 mb-2">
                                                                "Upload De-anonymization File"
                                                            </h4>
                                                            <p class="text-sm text-blue-700 mb-3">
                                                                "Upload a CSV file to convert anonymized student IDs back to real names and original IDs."
                                                            </p>
                                                            
                                                            <input
                                                                type="file"
                                                                accept=".csv"
                                                                class="w-full p-2 border border-gray-300 rounded-md file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100"
                                                                on:change=handle_file_upload
                                                            />
                                                            
                                                            {move || {
                                                                if let Some(status) = file_upload_status.get() {
                                                                    view! {
                                                                        <p class="text-sm text-blue-600 mt-2">{status}</p>
                                                                    }.into_view()
                                                                } else {
                                                                    view! { <span></span> }.into_view()
                                                                }
                                                            }}
                                                        </div>

                                                        <div class="bg-gray-50 rounded-md p-4">
                                                            <h4 class="text-sm font-semibold text-gray-700 mb-2">
                                                                "Required CSV Format:"
                                                            </h4>
                                                            <div class="bg-gray-800 text-gray-100 p-3 rounded text-xs font-mono overflow-x-auto">
                                                                <pre>{"app_id,original_student_id,firstname,lastname,pin,created_at\n100000,12345,John,Doe,1234,2025-06-09 19:52:19.862183\n100001,52884,Thien,Le,1234,2025-06-09 19:52:19.862183"}</pre>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }.into_view()
                                            }
                                        }}
                                    </div>
                                }.into_view()
                            } else {
                                view! { <span></span> }.into_view()
                            }
                        }}
                    </div>
                }.into_view()
            } else {
                view! { <span></span> }.into_view()
            }
        }}
    }
}
