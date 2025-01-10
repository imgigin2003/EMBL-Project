use std::fs::File; // Import for handling file operations.
use std::io::{self, BufReader}; // Import for input/output utilities.
use serde::Deserialize; // Import for deserializing JSON data into Rust structs.
use csv::Writer; // Import for writing CSV files.

// Struct representing a node in the JSON file.
// #[derive(Deserialize)] allows serde to parse JSON data into this struct.
#[derive(Debug, Deserialize)]
//allowing dead code to avoid warnings for unused fields
#[allow(dead_code)]
struct Node {
    id: String, // ID of the node, e.g., "HG315671".
    labels: Vec<String>, // Labels associated with the node, e.g., ["Contig"].
    properties: Option<Properties>, // Optional properties of the node (might be null).
    #[serde(rename = "type")] // Rename "type" to avoid conflict with Rust's type keyword.
    node_type: String, // The type of the node, e.g., "node".
}

// Struct representing a relationship in the JSON file.
#[derive(Debug, Deserialize)]
//allowing dead code to avoid warnings for unused fields
#[allow(dead_code)]
struct Relationship {
    id: String, // ID of the relationship.
    label: String, // Label of the relationship, e.g., "NEXT".
    start: EntityRef, // Reference to the starting node of the relationship.
    end: EntityRef, // Reference to the ending node of the relationship.
    #[serde(rename = "type")] // Rename "type" to avoid conflict.
    relationship_type: String, // The type of the relationship.
}

// Helper struct for representing references to nodes in relationships.
#[derive(Debug, Deserialize)]
//allowing dead code to avoid warnings for unused fields
#[allow(dead_code)]
struct EntityRef {
    id: String, // ID of the referenced node.
    labels: Vec<String>, // Labels of the referenced node.
}

// Struct representing the properties of a node.
// It uses serde_json::Value to capture any extra unstructured JSON data.
#[derive(Debug, Deserialize)]
//allowing dead code to avoid warnings for unused fields
#[allow(dead_code)]
struct Properties {
    name: Option<String>, // Optional name property of the node.
    organism: Option<String>, // Optional organism property of the node.
    #[serde(flatten)] // Include any other JSON fields in this value.
    extra: serde_json::Value,
}

// Main public function to convert JSON data to a CSV file.
pub fn update_csv(input_json: &str, output_csv: &str) -> io::Result<()> {
    // Open the input JSON file for reading.
    let input_file = File::open(input_json)?;
    // Wrap the file in a BufReader for efficient reading.
    let reader = BufReader::new(input_file);

    // Parse the JSON data into a serde_json::Value.
    let json_data: serde_json::Value = serde_json::from_reader(reader)?;

    // Create a CSV writer to write data into the output CSV file.
    let mut writer = Writer::from_path(output_csv)?;

    // Write the CSV header row with column names.
    writer.write_record(&["id", "labels", "properties", "type"])?;

    // Check if the parsed JSON is an array of objects.
    if let Some(array) = json_data.as_array() {
        // Iterate over each entry in the JSON array.
        for entry in array {
            // Try to deserialize the entry as a Node.
            if let Ok(node) = serde_json::from_value::<Node>(entry.clone()) {
                // Write the node's data to the CSV file.
                writer.write_record(&[
                    node.id.clone(), // ID of the node.
                    format!("{:?}", node.labels), // Labels as a stringified vector.
                    format!("{:?}", node.properties), // Properties as a stringified object.
                    node.node_type.clone(), // Node type.
                ])?;
            // Try to deserialize the entry as a Relationship.
            } else if let Ok(rel) = serde_json::from_value::<Relationship>(entry.clone()) {
                // Write the relationship's data to the CSV file.
                writer.write_record(&[
                    rel.id.clone(), // ID of the relationship.
                    format!("{} -> {}", rel.start.id, rel.end.id), // Relationship's start and end nodes.
                    format!("{:?}", rel.label), // Relationship label.
                    rel.relationship_type.clone(), // Relationship type.
                ])?;
            }
        }
    }

    // Ensure all data is written to the CSV file.
    writer.flush()?;
    // Return success.
    Ok(())
}
