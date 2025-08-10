use leptos::prelude::*;
let handle_csv_upload = move |file: File| {
    let reader = FileReader::new().unwrap();
    let reader_clone = reader.clone();
    
    set_is_uploading(true);
    set_upload_status("Reading file...".to_string());
    set_show_status(true);
    set_status_type("info");
    
    // Handle file load completion
    let on_load = Closure::wrap(Box::new(move |_event: web_sys::Event| {
        if let Ok(result) = reader_clone.result() {
            if let Some(text) = result.as_string() {
                // Process CSV data
                match parse_csv_to_add_requests(&text) {
                    Ok(requests) => {
                        if requests.is_empty() {
                            set_upload_status("No valid student records found in CSV".to_string());
                            set_status_type("warning");
                            set_is_uploading(false);
                            return;
                        }
                        
                        set_upload_status(format!("Processing {} students...", requests.len()));
                        
                        // Call the bulk import API
                        spawn_local(async move {
                            match bulk_import_students(requests).await {
                                Ok(result) => {
                                    let message = format!(
                                        "Import completed: {} of {} students added successfully",
                                        result.success_count, result.total
                                    );
                                    set_upload_status(message);
                                    set_status_type(if result.error_count > 0 { "warning" } else { "success" });
                                    set_import_result(Some(result));
                                    
                                    // Refresh the student list if we have a refresh trigger
                                    if let Some(refresh) = refresh_trigger {
                                        refresh.update(|count| *count += 1);
                                    }
                                },
                                Err(err) => {
                                    set_upload_status(format!("Import failed: {}", err));
                                    set_status_type("error");
                                }
                            }
                            set_is_uploading(false);
                        });
                    },
                    Err(err) => {
                        set_upload_status(format!("Error parsing CSV: {}", err));
                        set_status_type("error");
                        set_is_uploading(false);
                    }
                }
            }
        }
    }) as Box<dyn FnMut(_)>);
    
    reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
    reader.read_as_text(&file).unwrap();
    
    // Keep closure alive
    on_load.forget();
};

// Handle file selection from input
let handle_file_select = move |ev: web_sys::Event| {
    let input: HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
    if let Some(file_list) = input.files() {
        if let Some(file) = file_list.get(0) {
            handle_csv_upload(file);
        }
    }
};
