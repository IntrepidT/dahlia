use crate::app::middleware::global_settings::use_settings;
use crate::app::models::user::UserJwt;
use crate::app::server_functions::auth::login;
use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use wasm_bindgen::{closure::Closure, JsCast};

#[derive(Debug, Clone)]
pub struct StudentMappingService {
    app_id_to_original: HashMap<i32, StudentMapping>,
    original_to_app_id: HashMap<i32, StudentMapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentMapping {
    pub app_id: i32,
    pub original_student_id: i32,
    pub firstname: String,
    pub lastname: String,
    pub pin: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudentMappingData {
    pub mappings: Vec<StudentMapping>,
}

impl StudentMappingService {
    pub fn new(mappings: Vec<StudentMapping>) -> Self {
        let mut app_id_to_original = HashMap::new();
        let mut original_to_app_id = HashMap::new();

        for mapping in mappings {
            app_id_to_original.insert(mapping.app_id, mapping.clone());
            original_to_app_id.insert(mapping.original_student_id, mapping);
        }

        Self {
            app_id_to_original,
            original_to_app_id,
        }
    }

    // De-anonymize: Convert app_id back to original student info
    pub fn de_anonymize_student_id(&self, app_id: i32) -> Option<i32> {
        self.app_id_to_original
            .get(&app_id)
            .map(|m| m.original_student_id)
    }

    // Get full student info for de-anonymization
    pub fn get_original_student_info(&self, app_id: i32) -> Option<&StudentMapping> {
        self.app_id_to_original.get(&app_id)
    }

    // Anonymize: Convert original student_id to app_id (if needed)
    pub fn anonymize_student_id(&self, original_student_id: i32) -> Option<i32> {
        self.original_to_app_id
            .get(&original_student_id)
            .map(|m| m.app_id)
    }

    pub fn get_app_id_info(&self, original_student_id: i32) -> Option<&StudentMapping> {
        self.original_to_app_id.get(&original_student_id)
    }

    pub fn has_mapping_for_app_id(&self, app_id: i32) -> bool {
        self.app_id_to_original.contains_key(&app_id)
    }

    // Batch de-anonymization for performance
    pub fn de_anonymize_batch(&self, app_ids: &[i32]) -> HashMap<i32, StudentMapping> {
        app_ids
            .iter()
            .filter_map(|&app_id| {
                self.app_id_to_original
                    .get(&app_id)
                    .map(|mapping| (app_id, mapping.clone()))
            })
            .collect()
    }
}

// Add a context provider for the mapping service
pub fn provide_student_mapping_service() -> (
    ReadSignal<Option<StudentMappingService>>,
    WriteSignal<Option<StudentMappingService>>,
) {
    create_signal(None)
}

// Hook to use the mapping service
pub fn use_student_mapping_service() -> (
    ReadSignal<Option<StudentMappingService>>,
    WriteSignal<Option<StudentMappingService>>,
) {
    use_context::<(
        ReadSignal<Option<StudentMappingService>>,
        WriteSignal<Option<StudentMappingService>>,
    )>()
    .expect("StudentMappingService context not found. Make sure to provide it in your app.")
}

// Enhanced Student struct with de-anonymization support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeAnonymizedStudent {
    pub app_id: i32,
    pub original_student_id: Option<i32>,
    pub display_name: String,
    pub display_id: String,
}

impl DeAnonymizedStudent {
    pub fn from_student_with_mapping(
        student: &crate::app::models::student::Student,
        mapping_service: Option<&StudentMappingService>,
    ) -> Self {
        if let Some(service) = mapping_service {
            if let Some(mapping) = service.get_original_student_info(student.student_id) {
                return Self {
                    app_id: student.student_id,
                    original_student_id: Some(mapping.original_student_id),
                    display_name: format!("{} {}", mapping.firstname, mapping.lastname),
                    display_id: mapping.original_student_id.to_string(),
                };
            }
        }

        // Fallback to anonymized display
        Self {
            app_id: student.student_id,
            original_student_id: None,
            display_name: format!(
                "{} {}",
                student.firstname.as_deref().unwrap_or("Student"),
                student
                    .lastname
                    .as_deref()
                    .unwrap_or(&format!("#{}", student.student_id))
            ),
            display_id: student.student_id.to_string(),
        }
    }
}

// Updated login form component
#[component]
pub fn EnhancedLoginForm() -> impl IntoView {
    let (username, set_username) = create_signal("".to_string());
    let (password, set_password) = create_signal("".to_string());
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (student_mapping_file, set_student_mapping_file) = create_signal::<Option<String>>(None);
    let (file_upload_status, set_file_upload_status) = create_signal::<Option<String>>(None);
    let set_current_user = use_context::<WriteSignal<Option<UserJwt>>>().unwrap();

    // Get the mapping service context
    let (_, set_student_mapping_service) = use_student_mapping_service();

    // Get settings to check if anonymization is enabled
    let (settings, _) = use_settings();
    let anonymization_enabled = move || settings.get().student_protections;

    let parse_csv_content = move |content: String| -> Result<StudentMappingData, String> {
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return Err("Empty file".to_string());
        }

        // Validate header line
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

        // Skip header line and parse data
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

            let app_id = parts[0]
                .trim()
                .parse::<i32>()
                .map_err(|_| format!("Invalid app_id at line {}: '{}'", line_num + 2, parts[0]))?;

            let original_student_id = parts[1].trim().parse::<i32>().map_err(|_| {
                format!(
                    "Invalid original_student_id at line {}: '{}'",
                    line_num + 2,
                    parts[1]
                )
            })?;

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

    let handle_file_upload = move |ev: web_sys::Event| {
        let input = ev
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        if let Some(files) = input.files() {
            if files.length() > 0 {
                let file = files.get(0).unwrap();
                let file_reader = web_sys::FileReader::new().unwrap();

                set_file_upload_status.set(Some("Loading file...".to_string()));

                let onload = Closure::wrap(Box::new({
                    let set_student_mapping_file = set_student_mapping_file.clone();
                    let set_error = set_error.clone();
                    let set_file_upload_status = set_file_upload_status.clone();
                    let parse_csv_content = parse_csv_content.clone();

                    move |event: web_sys::Event| {
                        let file_reader = event
                            .target()
                            .unwrap()
                            .dyn_into::<web_sys::FileReader>()
                            .unwrap();

                        if let Ok(result) = file_reader.result() {
                            if let Some(content) = result.as_string() {
                                match parse_csv_content(content.clone()) {
                                    Ok(_) => {
                                        set_student_mapping_file.set(Some(content));
                                        set_error.set(None);
                                        set_file_upload_status
                                            .set(Some("File loaded successfully".to_string()));
                                        logging::log!("Student mapping file loaded successfully");
                                    }
                                    Err(e) => {
                                        set_error.set(Some(format!("Invalid CSV format: {}", e)));
                                        set_file_upload_status
                                            .set(Some("Failed to load file".to_string()));
                                        logging::log!("Invalid CSV in student mapping file: {}", e);
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
        } else {
            set_student_mapping_file.set(None);
            set_file_upload_status.set(None);
        }
    };

    let handle_submit = create_action(move |_: &()| {
        let username = username.get();
        let password = password.get();
        let mapping_data_content = student_mapping_file.get();

        async move {
            if username.trim().is_empty() || password.trim().is_empty() {
                set_error.set(Some("Username and password are required".to_string()));
                return;
            }

            logging::log!("Attempting login with username: {}", username);

            match login(username, password).await {
                Ok(response) => {
                    if response.success {
                        logging::log!("Login successful, setting user");
                        set_current_user.set(response.user);

                        // Set up mapping service only if a mapping file was provided
                        if let Some(data_content) = mapping_data_content {
                            match parse_csv_content(data_content) {
                                Ok(mapping_data) => {
                                    let mapping_service =
                                        StudentMappingService::new(mapping_data.mappings);
                                    set_student_mapping_service.set(Some(mapping_service));
                                    logging::log!(
                                        "Student mapping service initialized successfully"
                                    );
                                }
                                Err(e) => {
                                    logging::log!("Failed to parse student mapping data: {}", e);
                                    set_error.set(Some(format!(
                                        "Failed to initialize mapping service: {}",
                                        e
                                    )));
                                    return;
                                }
                            }
                        } else {
                            // No mapping file provided - clear any existing mapping service
                            set_student_mapping_service.set(None);
                            logging::log!(
                                "No student mapping file provided - de-anonymization disabled"
                            );
                        }

                        set_error.set(None);
                    } else {
                        logging::log!("Login failed: {}", response.message);
                        set_error.set(Some(response.message));
                    }
                }
                Err(err) => {
                    logging::log!("Login error: {:?}", err);
                    set_error.set(Some(format!("Error: {:?}", err)));
                }
            }
        }
    });

    view! {
            <div class="p-4 bg-white rounded shadow-md">
                <h2 class="text-2xl font-bold mb-4">"Login"</h2>

                {move || {
                    error.get().map(|err| {
                        view! {
                            <div class="mb-4 p-2 bg-red-100 text-red-700 rounded">{err}</div>
                        }
                    })
                }}

                <form on:submit=move |ev| {
                    ev.prevent_default();
                    handle_submit.dispatch(());
                }>
                    <div class="mb-4">
                        <label class="block text-gray-700 mb-2" for="username">"Username"</label>
                        <input
                            id="username"
                            type="text"
                            class="w-full p-2 border rounded"
                            prop:value=move || username.get()
                            on:input=move |ev| set_username.set(event_target_value(&ev))
                        />
                    </div>

                    <div class="mb-4">
                        <label class="block text-gray-700 mb-2" for="password">"Password"</label>
                        <input
                            id="password"
                            type="password"
                            class="w-full p-2 border rounded"
                            prop:value=move || password.get()
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                        />
                    </div>

                    {move || {
                        if anonymization_enabled() {
                            view! {
                                <div class="mb-4">
                                    <label class="block text-gray-700 mb-2" for="student-mapping">
                                        "Student ID Mapping File (CSV)"
                                        <span class="text-sm text-gray-500">" (Optional)"</span>
                                    </label>
                                    <input
                                        id="student-mapping"
                                        type="file"
                                        accept=".csv"
                                        class="w-full p-2 border rounded"
                                        on:change=handle_file_upload
                                    />
                                    <p class="text-sm text-gray-500 mt-1">
                                        "Upload a CSV file containing student ID mappings for de-anonymization. If not provided, student data will remain anonymized."
                                    </p>
                                    {move || {
                                        if let Some(status) = file_upload_status.get() {
                                            if status == "File loaded successfully" {
                                                view! {
                                                    <p class="text-sm text-green-600 mt-1">
                                                        "✓ "{status}" - De-anonymization will be enabled"
                                                    </p>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <p class="text-sm text-blue-600 mt-1">
                                                        {status}
                                                    </p>
                                                }.into_any()
                                            }
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }
                                    }}
                                </div>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}

                    <button
                        type="submit"
                        class="w-full p-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:bg-gray-400"
                        prop:disabled=move || handle_submit.pending().get()
                    >
                        {move || {
                            if handle_submit.pending().get() {
                                "Logging in..."
                            } else {
                                "Login"
                            }
                        }}
                    </button>
                </form>

                {move || {
                    if anonymization_enabled() {
                        view! {
                            <div class="mt-4 p-3 bg-gray-50 rounded">
                                <h3 class="text-sm font-semibold text-gray-700 mb-2">"CSV Format Information:"</h3>
                                <p class="text-xs text-gray-600 mb-2">
                                    "If you want to enable de-anonymization, upload a CSV file with the following format:"
                                </p>
                                <pre class="text-xs text-gray-600 overflow-x-auto">
    {r#"app_id,original_student_id,firstname,lastname,pin,created_at
100000,12345,John,Doe,1234,2025-06-09 19:52:19.862183
100001,52884,Thien,Le,1234,2025-06-09 19:52:19.862183
100002,67890,Jane,Smith,6789,2025-06-09 19:52:19.862183"#}
                                </pre>
                                <p class="text-xs text-gray-500 mt-2">
                                    "Without this file, students will be displayed with their anonymized IDs."
                                </p>
                            </div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }
                }}
            </div>
        }
}
