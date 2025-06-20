use crate::app::components::auth::authorization_components::perform_post_login_redirect;
use crate::app::middleware::global_settings::use_settings;
use crate::app::models::user::SessionUser;
use crate::app::server_functions::auth::login;
use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
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

    // Fix: Add missing method
    pub fn get_mapping_count(&self) -> usize {
        self.app_id_to_original.len()
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
    let (is_submitting, set_is_submitting) = create_signal(false);

    let set_current_user = use_context::<WriteSignal<Option<SessionUser>>>().unwrap();
    let redirect_after_login = perform_post_login_redirect();

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

        // Validate header names
        for (i, expected) in expected_headers.iter().enumerate() {
            if actual_headers.get(i).map(|h| h.to_lowercase()) != Some(expected.to_string()) {
                return Err(format!(
                    "Invalid header at position {}: expected '{}', found '{}'",
                    i + 1,
                    expected,
                    actual_headers.get(i).unwrap_or(&"missing")
                ));
            }
        }

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

            // Validate that IDs are positive
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

        // Check for duplicate app_ids or original_student_ids
        let mut seen_app_ids = std::collections::HashSet::new();
        let mut seen_original_ids = std::collections::HashSet::new();

        for mapping in &mappings {
            if !seen_app_ids.insert(mapping.app_id) {
                return Err(format!("Duplicate app_id found: {}", mapping.app_id));
            }
            if !seen_original_ids.insert(mapping.original_student_id) {
                return Err(format!(
                    "Duplicate original_student_id found: {}",
                    mapping.original_student_id
                ));
            }
        }

        Ok(StudentMappingData { mappings })
    };

    #[cfg(feature = "hydrate")]
    let handle_file_upload = move |ev: web_sys::Event| {
        let input = ev
            .target()
            .unwrap()
            .dyn_into::<web_sys::HtmlInputElement>()
            .unwrap();

        if let Some(files) = input.files() {
            if files.length() > 0 {
                let file = files.get(0).unwrap();

                // Validate file type
                if !file.name().ends_with(".csv") {
                    set_error.set(Some("Please select a CSV file".to_string()));
                    set_file_upload_status.set(Some("Invalid file type".to_string()));
                    return;
                }

                // Validate file size (e.g., max 10MB)
                if file.size() > 10_000_000.0 {
                    set_error.set(Some("File too large. Maximum size is 10MB".to_string()));
                    set_file_upload_status.set(Some("File too large".to_string()));
                    return;
                }

                let file_reader = web_sys::FileReader::new().unwrap();
                set_file_upload_status.set(Some("Loading file...".to_string()));
                set_error.set(None);

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
                                    Ok(mapping_data) => {
                                        set_student_mapping_file.set(Some(content));
                                        set_error.set(None);
                                        set_file_upload_status.set(Some(format!(
                                            "File loaded successfully ({} mappings)",
                                            mapping_data.mappings.len()
                                        )));
                                        logging::log!(
                                            "Student mapping file loaded with {} mappings",
                                            mapping_data.mappings.len()
                                        );
                                    }
                                    Err(e) => {
                                        set_error.set(Some(format!("Invalid CSV format: {}", e)));
                                        set_file_upload_status
                                            .set(Some("Failed to load file".to_string()));
                                        set_student_mapping_file.set(None);
                                        logging::log!("Invalid CSV in student mapping file: {}", e);
                                    }
                                }
                            }
                        } else {
                            set_error.set(Some("Failed to read file".to_string()));
                            set_file_upload_status.set(Some("Failed to read file".to_string()));
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
    #[cfg(not(feature = "hydrate"))]
    let handle_file_upload = move |_| {
        set_error.set(Some(
            "File upload is not supported in this environment".to_string(),
        ));
        set_file_upload_status.set(None);
    };

    let handle_submit = create_action(move |_: &()| {
        let username = username.get();
        let password = password.get();
        let mapping_data_content = student_mapping_file.get();

        async move {
            set_is_submitting.set(true);

            if username.trim().is_empty() || password.trim().is_empty() {
                set_error.set(Some("Username and password are required".to_string()));
                set_is_submitting.set(false);
                return;
            }

            logging::log!("Attempting login with username: {}", username);

            match login(username, password).await {
                Ok(response) => {
                    if response.success {
                        logging::log!("Login successful, setting user");
                        set_current_user.set(response.user.clone());

                        // Set up mapping service only if a mapping file was provided
                        if let Some(data_content) = mapping_data_content {
                            match parse_csv_content(data_content) {
                                Ok(mapping_data) => {
                                    let mapping_service =
                                        StudentMappingService::new(mapping_data.mappings);
                                    let mapping_count = mapping_service.get_mapping_count();
                                    set_student_mapping_service.set(Some(mapping_service));
                                    logging::log!(
                                        "Student mapping service initialized with {} mappings",
                                        mapping_count
                                    );
                                }
                                Err(e) => {
                                    logging::log!("Failed to parse student mapping data: {}", e);
                                    set_error.set(Some(format!(
                                        "Failed to initialize mapping service: {}",
                                        e
                                    )));
                                    set_is_submitting.set(false);
                                    return;
                                }
                            }
                        } else {
                            set_student_mapping_service.set(None);
                            logging::log!(
                                "No student mapping file provided - de-anonymization disabled"
                            );
                        }

                        set_error.set(None);

                        // Use the redirect function from AuthProvider
                        perform_post_login_redirect();
                    } else {
                        logging::log!("Login failed: {}", response.message);
                        set_error.set(Some(response.message));
                    }
                }
                Err(err) => {
                    logging::log!("Login error: {:?}", err);
                    set_error.set(Some(
                        "Login failed. Please check your credentials and try again.".to_string(),
                    ));
                }
            }

            set_is_submitting.set(false);
        }
    });

    view! {
        <div class="p-6 bg-white rounded-lg shadow-md max-w-md mx-auto">
            <h2 class="text-2xl font-bold mb-6 text-center text-gray-800">"Login"</h2>

            {move || {
                error.get().map(|err| {
                    view! {
                        <div class="mb-4 p-3 bg-red-100 border border-red-400 text-red-700 rounded-md">
                            <div class="flex items-center">
                                <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 20 20">
                                    <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7 4a1 1 0 11-2 0 1 1 0 012 0zm-1-9a1 1 0 00-1 1v4a1 1 0 102 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                </svg>
                                {err}
                            </div>
                        </div>
                    }
                })
            }}

            <form on:submit=move |ev| {
                ev.prevent_default();
                if !is_submitting.get() {
                    handle_submit.dispatch(());
                }
            }>
                <div class="mb-4">
                    <label class="block text-gray-700 text-sm font-medium mb-2" for="username">
                        "Username"
                    </label>
                    <input
                        id="username"
                        type="text"
                        class="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        prop:value=move || username.get()
                        on:input=move |ev| set_username.set(event_target_value(&ev))
                        prop:disabled=move || is_submitting.get()
                        placeholder="Enter your username"
                    />
                </div>

                <div class="mb-6">
                    <label class="block text-gray-700 text-sm font-medium mb-2" for="password">
                        "Password"
                    </label>
                    <input
                        id="password"
                        type="password"
                        class="w-full p-3 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                        prop:value=move || password.get()
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        prop:disabled=move || is_submitting.get()
                        placeholder="Enter your password"
                    />
                </div>

                {move || {
                    if anonymization_enabled() {
                        view! {
                            <div class="mb-6">
                                <label class="block text-gray-700 text-sm font-medium mb-2" for="student-mapping">
                                    "Student ID Mapping File"
                                    <span class="text-sm text-gray-500 font-normal">" (Optional)"</span>
                                </label>
                                <input
                                    id="student-mapping"
                                    type="file"
                                    accept=".csv"
                                    class="w-full p-2 border border-gray-300 rounded-md file:mr-4 file:py-2 file:px-4 file:rounded-md file:border-0 file:text-sm file:font-medium file:bg-blue-50 file:text-blue-700 hover:file:bg-blue-100"
                                    on:change=handle_file_upload
                                    prop:disabled=move || is_submitting.get()
                                />
                                <p class="text-sm text-gray-600 mt-2">
                                    "Upload a CSV file containing student ID mappings for de-anonymization."
                                </p>
                                {move || {
                                    if let Some(status) = file_upload_status.get() {
                                        if status.contains("successfully") {
                                            view! {
                                                <div class="flex items-center mt-2 text-sm text-green-600">
                                                    <svg class="w-4 h-4 mr-1" fill="currentColor" viewBox="0 0 20 20">
                                                        <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                                    </svg>
                                                    {status}" - De-anonymization enabled"
                                                </div>
                                            }.into_view()
                                        } else {
                                            view! {
                                                <p class="text-sm text-blue-600 mt-2">{status}</p>
                                            }.into_view()
                                        }
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

                <button
                    type="submit"
                    class="w-full p-3 bg-blue-600 text-white rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-gray-400 disabled:cursor-not-allowed transition-colors duration-200"
                    prop:disabled=move || is_submitting.get()
                >
                    {move || {
                        if is_submitting.get() {
                            view! {
                                <div class="flex items-center justify-center">
                                    <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                                    </svg>
                                    "Logging in..."
                                </div>
                            }.into_view()
                        } else {
                            view! { "Login" }.into_view()
                        }
                    }}
                </button>
            </form>

            {move || {
                if anonymization_enabled() {
                    view! {
                        <div class="mt-6 p-4 bg-gray-50 rounded-md">
                            <h3 class="text-sm font-semibold text-gray-700 mb-2">
                                "CSV Format Requirements:"
                            </h3>
                            <p class="text-xs text-gray-600 mb-2">
                                "For de-anonymization, upload a CSV file with this exact format:"
                            </p>
                            <div class="bg-gray-800 text-gray-100 p-3 rounded text-xs font-mono overflow-x-auto">
                                <pre>{"app_id,original_student_id,firstname,lastname,pin,created_at\n100000,12345,John,Doe,1234,2025-06-09 19:52:19.862183\n100001,52884,Thien,Le,1234,2025-06-09 19:52:19.862183"}</pre>
                            </div>
                            <p class="text-xs text-gray-500 mt-2">
                                "Without this file, students will be displayed with anonymized IDs."
                            </p>
                        </div>
                    }.into_view()
                } else {
                    view! { <span></span> }.into_view()
                }
            }}
        </div>
    }
}
