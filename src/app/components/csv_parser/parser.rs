use leptos::prelude::*;
fn parse_csv_to_add_requests(csv_content: &str) -> Result<Vec<AddStudentRequest>, String> {
    let mut lines = csv_content.lines();
    
    // Parse header row to identify column positions
    let header = match lines.next() {
        Some(header) => header,
        None => return Err("CSV file is empty".to_string()),
    };
    
    let headers: Vec<&str> = header.split(',').map(|s| s.trim()).collect();
    let mut column_map = std::collections::HashMap::new();
    
    for (i, col_name) in headers.iter().enumerate() {
        column_map.insert(col_name.to_lowercase(), i);
    }
    
    // Check required columns
    let required_columns = ["firstname", "lastname", "student_id", "grade", "teacher"];
    for &col in required_columns.iter() {
        if !column_map.contains_key(col) {
            return Err(format!("Required column '{}' not found in CSV", col));
        }
    }
    
    // Process data rows
    let mut requests = Vec::new();
    for (line_num, line) in lines.enumerate() {
        let line_num = line_num + 2; // Account for header and 1-indexing
        let fields: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        
        if fields.len() != headers.len() {
            return Err(format!("Line {} has incorrect number of columns", line_num));
        }
        
        // Helper function to get a field by column name
        let get_field = |name: &str| -> &str {
            if let Some(&index) = column_map.get(name) {
                if index < fields.len() {
                    return fields[index];
                }
            }
            ""
        };
        
        // Helper function to parse boolean fields
        let parse_bool = |name: &str| -> bool {
            let value = get_field(name).to_lowercase();
            match value.as_str() {
                "true" | "yes" | "1" => true,
                _ => false,
            }
        };
        
        // Parse required fields
        let firstname = get_field("firstname").to_string();
        let lastname = get_field("lastname").to_string();
        
        let student_id = match get_field("student_id").parse::<i32>() {
            Ok(id) => id,
            Err(_) => return Err(format!("Invalid student ID on line {}", line_num)),
        };
        
        let grade = match get_field("grade").parse::<i32>() {
            Ok(g) => g,
            Err(_) => return Err(format!("Invalid grade on line {}", line_num)),
        };
        
        let teacher = get_field("teacher").to_string();
        
        // Parse optional fields
        let preferred = if column_map.contains_key("preferred") {
            Some(get_field("preferred").to_string())
        } else {
            None
        };
        
        let gender = if column_map.contains_key("gender") {
            Some(get_field("gender").to_string())
        } else {
            None
        };
        
        let date_of_birth = if column_map.contains_key("date_of_birth") {
            let dob_str = get_field("date_of_birth");
            if !dob_str.is_empty() {
                match NaiveDate::parse_from_str(dob_str, "%Y-%m-%d") {
                    Ok(date) => Some(date),
                    Err(_) => return Err(format!("Invalid date of birth on line {}", line_num)),
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Create AddStudentRequest
        let request = AddStudentRequest {
            firstname,
            lastname,
            preferred,
            gender,
            date_of_birth,
            student_id,
            esl: parse_bool("esl"),
            grade,
            teacher,
            iep: parse_bool("iep"),
            bip: parse_bool("bip"),
            student_504: parse_bool("student_504"),
            readplan: parse_bool("readplan"),
            gt: parse_bool("gt"),
            intervention: parse_bool("intervention"),
            eye_glasses: parse_bool("eye_glasses"),
            notes: if column_map.contains_key("notes") {
                Some(get_field("notes").to_string())
            } else {
                None
            },
        };
        
        requests.push(request);
    }
    
    Ok(requests)
}
