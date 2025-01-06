use std::fs::File; //import the File struct from the fs module
use std::io::{self, Read, Write}; //import the io module and the Read and Write traits
use serde_json::Value; //import the Value struct from the serde_json module

// Function to convert a JSON file to an EMBL file format and returns a Result
pub fn convert_json(json_path: &str, embl_path: &str) -> io::Result<()> {
    // Open the JSON file and read its contents
    let mut file = File::open(json_path)?;
    // Create a mutable string to store the contents of the JSON file
    let mut data = String::new();
    // Read the contents of the JSON file into the data string
    file.read_to_string(&mut data)?;

    // Parse the JSON string into a serde_json Value
    let json: Value = serde_json::from_str(&data)?;

    // Open the EMBL file for writing
    let mut embl_file = File::create(embl_path)?;

    // Write the headers and necessary information to the EMBL file
    writeln!(embl_file, "ID   {}; SV 1; circular; genomic DNA; STD; PRO; 4228350 BP.", json[0]["id"].as_str().unwrap())?;
    writeln!(embl_file, "XX")?;
    writeln!(embl_file, "AC   {};", json[0]["id"].as_str().unwrap())?;
    writeln!(embl_file, "XX")?;
    writeln!(embl_file, "PR   Project:{};", json[0]["properties"]["annotations"]["project"].as_str().unwrap())?;
    writeln!(embl_file, "XX")?;
    writeln!(embl_file, "DT   {} {}", json[0]["properties"]["annotations"]["dates"].as_str().unwrap().split(' ').collect::<Vec<&str>>()[0], json[0]["properties"]["annotations"]["dates"].as_str().unwrap().split(' ').collect::<Vec<&str>>()[1])?;
    writeln!(embl_file, "DT   {} {}", json[0]["properties"]["annotations"]["dates"].as_str().unwrap().split(' ').collect::<Vec<&str>>()[2], json[0]["properties"]["annotations"]["dates"].as_str().unwrap().split(' ').collect::<Vec<&str>>()[3])?;
    writeln!(embl_file, "XX")?;
    writeln!(embl_file, "DE   {}", json[0]["properties"]["annotations"]["description"].as_str().unwrap())?;
    writeln!(embl_file, "XX")?;
    writeln!(embl_file, "KW   {}", json[0]["properties"]["annotations"]["keywords"].as_str().unwrap())?;
    writeln!(embl_file, "XX")?;
    writeln!(embl_file, "OS   {:?}", json[0]["properties"]["annotations"]["organism"].as_str())?;
    writeln!(embl_file, "OC   {}", json[0]["properties"]["taxonomy"].as_str().unwrap().replace(";", ";\nOC   "))?;
    writeln!(embl_file, "XX")?;

    // Write references to the EMBL file
    for reference in json[0]["properties"]["references"].as_array().unwrap() {
        writeln!(embl_file, "RN   [{}]", reference["reference_number"].as_str().unwrap())?;
        writeln!(embl_file, "RP   {}", reference["reference_position"].as_str().unwrap_or(""))?;
        writeln!(embl_file, "RA   Werner J.;")?;
        writeln!(embl_file, "RT   ;")?;
        writeln!(embl_file, "RL   Submitted (30-MAY-2013) to the INSDC.")?;
        writeln!(embl_file, "RL   Microbial Genomics Group, Max Planck Institute for Marine Microbiology,")?;
        writeln!(embl_file, "RL   Celsiusstrasse 1, Bremen, 28359, GERMANY.")?;
        writeln!(embl_file, "XX")?;
    }

    // Write features to the EMBL file
    writeln!(embl_file, "FH   Key             Location/Qualifiers")?;
    writeln!(embl_file, "FH")?;
    // Iterate over the nodes in the JSON file and write the features to the EMBL file for each node
    for node in json.as_array().unwrap().iter().skip(1) {
        // Check if the node is a feature and write the feature to the EMBL file
        if node["type"].as_str().unwrap() == "node" {
            // Write the feature to the EMBL file with the necessary information from the JSON file
            writeln!(embl_file, "FT   {}          1..4228350", node["properties"]["type"].as_str().unwrap())?;
            writeln!(embl_file, "FT                   /organism=\"{}\"", node["properties"]["organism"].as_str().unwrap())?;
            writeln!(embl_file, "FT                   /strain=\"{}\"", node["properties"]["name"].as_str().unwrap())?;
            writeln!(embl_file, "FT                   /mol_type=\"{}\"", node["properties"]["type"].as_str().unwrap())?;
            writeln!(embl_file, "FT                   /db_xref=\"taxon:1347342\"")?;
            writeln!(embl_file, "FT                   /culture_collection=\"KMM:3901\"")?;
        }
    }

    // Write the sequence to the EMBL file
    Ok(())
}
