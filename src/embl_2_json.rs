use std::fs::File; // Import File for file operations
use std::io::{BufRead, BufReader, Write}; // Import BufRead, BufReader for reading files line-by-line, and Write for writing
use serde_json::{json, Value}; // Import JSON manipulation tools from serde_json
use std::collections::HashMap; // Import HashMap for key-value data storage

// Main function to process EMBL file and generate JSON output
pub fn process_embl(input_embl: &str, output_json: &str) {
    // Open the EMBL input file
    let file = File::open(input_embl).expect("Failed to open EMBL file");
    let reader = BufReader::new(file);

    // Vector to store all JSON output content
    let mut content = Vec::new();
    // Counter for assigning unique IDs to relationships
    let mut rid: u64 = 0;

    // Variables for EMBL parsing
    let mut seq_record_id = String::new(); // Stores the ID of the sequence
    let mut annotations: HashMap<String, String> = HashMap::new(); // Stores annotations
    let mut organism = String::new(); // Stores organism name
    let mut taxonomy = String::new(); // Stores taxonomy hierarchy
    let mut references: Vec<HashMap<String, String>> = Vec::new(); // Stores reference information
    let mut in_feature = false; // Flag for feature parsing
    let mut current_feature: HashMap<String, Value> = HashMap::new(); // Temporary storage for current feature
    let mut features: Vec<HashMap<String, Value>> = Vec::new(); // List of all parsed features
    let mut current_reference: HashMap<String, String> = HashMap::new(); // Temporary storage for references

    // Read each line from the input file
    for line in reader.lines() {
        let line = line.expect("Failed to read lines from EMBL file");

        //extracts the ID(Identifier)
        if line.starts_with("ID") {
            seq_record_id = line.split_whitespace().nth(1).unwrap_or_default().replace(";", "").to_string();
        
        //extracts the AC (Accession)
        } else if line.starts_with("AC") {
            annotations.insert("accession".to_string(), line.replace("AC   ", "").replace(";", "").trim().to_string());
        
        //extracts the PR(Project Reference)
        } else if line.starts_with("PR") {
            annotations.insert("project".to_string(), line.replace("PR   Project:", "").trim().to_string());
        
        //extracts the DT(DateTime)
        } else if line.starts_with("DT") {
            annotations.entry("dates".to_string()).or_insert_with(String::new).push_str(&format!("{} ", line.replace("DT   ", "").trim()));
        
        //extracts the DE(Description)
        } else if line.starts_with("DE") {
            annotations.insert("description".to_string(), line.replace("DE   ", "").trim().to_string());
        
        //extracts the KW(KeyWord)
        } else if line.starts_with("KW") {
            annotations.entry("keywords".to_string()).or_insert_with(String::new).push_str(&line.replace("KW   ", "").trim());
        
        //extracts the OC(Organism Specie)
        } else if line.starts_with("OS") {
            organism = line.replace("OS   ", "").trim().to_string();

        //extracts the OC(Organism Classification)
        } else if line.starts_with("OC") {
            taxonomy.push_str(&line.replace("OC   ", "").trim());
        
        //extracts the RN(Reference Number)
        } else if line.starts_with("RN") {
            //if anything exists in the current feature variable
            if !current_reference.is_empty() {
                //clone a copy of it and add it in the references
                references.push(current_reference.clone());
                //clear the variable 
                current_reference.clear();
            }
            //add RN into the current_reference variable
            current_reference.insert("reference_number".to_string(), line.replace("RN   [", "").replace("]", "").trim().to_string());
        
        //extracts the RP(Reference Position)
        } else if line.starts_with("RP") {
            current_reference.insert("reference_position".to_string(), line.replace("RP   ", "").trim().to_string());
        
        //extracts the FT(Features)
        } else if line.starts_with("FT") {
            if in_feature {
                features.push(current_feature.clone());
                current_feature.clear();
            }
            in_feature = true;

            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() > 1 {
                let feature_type = parts[1].trim().split_whitespace().next().unwrap_or("").to_string();
                current_feature.insert("type".to_string(), Value::String(feature_type));
            }
        
        } else if line.starts_with("  ") && in_feature {
            let detail = line.trim().splitn(2, ' ').collect::<Vec<&str>>();
            if detail.len() == 2 {
                current_feature.insert(detail[0].to_string(), Value::String(detail[1].to_string()));
            }
        
        } else if line.starts_with("XX") {
            if in_feature {
                features.push(current_feature.clone());
                current_feature.clear();
                in_feature = false;
            } else if !current_reference.is_empty() {
                references.push(current_reference.clone());
                current_reference.clear();
            }
        
        } else if line.starts_with("//") {
            if in_feature {
                features.push(current_feature.clone());
                current_feature.clear();
                in_feature = false;
            }
            if !current_reference.is_empty() {
                references.push(current_reference.clone());
                current_reference.clear();
            }
        }
    }

    // Create a JSON node for the contig (main record)
    let contig_node = json!({
        "type": "node",
        "id": seq_record_id,
        "labels": ["Contig"],
        "properties": {
            "id": seq_record_id,
            "name": organism,
            "organism": organism,
            "annotations": annotations,
            "taxonomy": taxonomy,
            "references": references
        }
    });

    content.push(contig_node);

    // Iterate over each feature to generate nodes and relationships
    for (index, feature) in features.iter().enumerate() {
        let node_type = if index == 0 { "Lead" } else { "Gene" };
        let node_name = feature.get("type").and_then(|v| v.as_str()).unwrap_or("Unnamed");

        //creating a json object for feature nodes
        let feature_node = json!({
            "type": "node",
            "id": format!("{}_{}", seq_record_id, index),
            "labels": [node_type],
            "properties": {
                "type": feature.get("type").unwrap_or(&Value::String("Unknown".to_string())),
                "organism": organism,
                "name": node_name
            }
        });

        content.push(feature_node);

        let relationship_label = if index == 0 { "OWNS" } else { "NEXT" };
        let start_id = if index == 0 { seq_record_id.clone() } else { format!("{}_{}", seq_record_id, index - 1) };
        let end_id = format!("{}_{}", seq_record_id, index);

        //creating a json object for relationships
        let relationship = json!({
            "id": format!("{}_r_{}", seq_record_id, rid),
            "type": "relationship",
            "label": relationship_label,
            "start": { "id": start_id, "labels": [if index == 0 { "Contig" } else { "Gene" }] },
            "end": { "id": end_id, "labels": [node_type] }
        });

        rid += 1;
        content.push(relationship);
    }

    //creating a new file of the output parameter
    let mut output_file = File::create(output_json).expect("Failed to create JSON file");
    //writing all the json content into the output file
    output_file.write_all(serde_json::to_string_pretty(&content).unwrap().as_bytes()).expect("Failed to write JSON");

    // Generate a DOT graph from the JSON output
    generate_graph(output_json, "graph/graph.dot");
}

// Function to generate a DOT file from the JSON input file
fn generate_graph(input_json: &str, output_dot: &str) {
    // Open the JSON file for reading
    let file = File::open(input_json).expect("Failed to open JSON file.");
    let reader = BufReader::new(file);

    let json_content: Value = serde_json::from_reader(reader).expect("Failed to parse JSON.");

    // Initialize the DOT file content with the graph declaration
    let mut dot_content = String::from("digraph G {\n");

    if let Some(array) = json_content.as_array() {
        for value in array {
            // Match the type of the JSON object (node or relationship)
            match value.get("type").and_then(|v| v.as_str()) {
                // Handle node representation
                Some("node") => {
                    // Extract node ID and name for DOT representation
                    let id = value["id"].as_str().unwrap();
                    // Replace double quotes with escaped double quotes for DOT compatibility
                    let name = value["properties"]["name"].as_str().unwrap_or("unknown").replace("\"", "\\\"");
                    // Append the node representation to the DOT content
                    dot_content.push_str(&format!("    \"{}\" [label=\"{}\"]\n", id, name));
                }
                // Handle relationship representation
                Some("relationship") => {
                    // Extract relationship start ID, end ID, and label for DOT representation
                    let start_id = value["start"]["id"].as_str().unwrap();
                    // Replace double quotes with escaped double quotes for DOT compatibility
                    let end_id = value["end"]["id"].as_str().unwrap();
                    // Replace double quotes with escaped double quotes for DOT compatibility
                    let label = value["label"].as_str().unwrap_or("unknown").replace("\"", "\\\"");
                    // Append the relationship representation to the DOT content
                    dot_content.push_str(&format!("    \"{}\" -> \"{}\" [label=\"{}\"]\n", start_id, end_id, label));
                }
                // Ignore other types of JSON objects
                _ => {}
            }
        }
    }

    // Close the graph declaration in the DOT file
    dot_content.push_str("}\n");

    // Write the DOT content to the output file
    let mut output_file = File::create(output_dot).expect("Failed to create DOT file.");
    output_file.write_all(dot_content.as_bytes()).expect("Failed to write to DOT file.");
}
