use std::fs::File;
use std::io::{self, BufReader, Write};
use serde_json::Value;

pub fn update_fasta(input_json: &str, output_fasta: &str) -> io::Result<()> {
    let file = File::open(input_json)?;
    let reader = BufReader::new(file);

    let json_data: Value = serde_json::from_reader(reader)?;

    let mut fasta_file = File::create(output_fasta)?;

    if let Some(entries) = json_data.as_array() {
        for entry in entries {
            if let Some(id) = entry.get("id").and_then(|v| v.as_str()) {
                let name = entry
                    .get("properties")
                    .and_then(|props| props.get("name"))
                    .and_then(|name| name.as_str())
                    .unwrap_or_else(|| entry.get("labels").and_then(|labels| labels[0].as_str()).unwrap_or("UNKNOWN"));
                let organism = entry
                    .get("properties")
                    .and_then(|props| props.get("organism"))
                    .and_then(|org| org.as_str())
                    .unwrap_or("UNKNOWN");
                let taxonomy = entry
                    .get("properties")
                    .and_then(|props| props.get("taxonomy"))
                    .and_then(|tax| tax.as_str())
                    .unwrap_or("UNKNOWN");
                let entry_type = entry
                    .get("type")
                    .and_then(|t| t.as_str())
                    .unwrap_or("UNKNOWN");

                // Construct header
                let header = format!(
                    ">{} | id={} | name={} | organism={} | taxonomy={}",
                    id, id, name, organism, taxonomy
                );

                // Write header
                writeln!(fasta_file, "{}", header)?;

                // Extract and write sequence
                if let Some(sequence) = entry.get("properties").and_then(|props| props.get("type")).and_then(|seq| seq.as_str()) {
                    writeln!(fasta_file, "{}", sequence)?;
                } else {
                    writeln!(fasta_file, "UNKNOWN")?;
                }
            }
        }
    }

    Ok(())
}
