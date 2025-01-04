use std::fs::File;
use std::io::{BufReader,BufWriter, Write};
use serde_json::Value;

pub fn convert_json(input_json: &str, output_embl: &str) {
    let file = File::open(input_json).expect("Failed to read JSON file.");
    let reader = BufReader::new(file);

    let output_file = File::create(output_embl).expect("Failed to creat EMBL file.");
    let mut writer = BufWriter::new(output_file);


    let json_content: Vec<Value> = serde_json::from_reader(reader).expect("Failed to parse JSON.");
    
    for record in json_content {
        let properties = record.get("properties").expect("missing 'properties' field.");
        let annotations = properties.get("annotations").unwrap_or(&Value::Null);
        let references = properties.get("references").unwrap_or(&Value::Null);
        let taxonomy = properties.get("taxonomy").unwrap_or(&Value::Null);

        if let Some(id) = properties.get("id") {
            writeln!(writer, "ID    {}; linear;", id.as_str().unwrap()).unwrap();
        }

        if let Some(accession) = annotations.get("accession") {
            writeln!(writer, "AC    {};", accession.as_str().unwrap()).unwrap();
        }

        if let Some(project) = annotations.get("project") {
            writeln!(writer, "PR    Project: {};", project.as_str().unwrap()).unwrap();
        }

        if let Some(date) = annotations.get("dates") {
            writeln!(writer, "DT    {};", date.as_str().unwrap()).unwrap();
        }

        if let Some(description) = annotations.get("description") {
            writeln!(writer, "DE    {};", description.as_str().unwrap()).unwrap();
        }

        if let Some(keyword) = annotations.get("keywords") {
            writeln!(writer, "KW    {};", keyword.as_str().unwrap()).unwrap();
        }

        if let Some(orgasism) = properties.get("organism") {
            writeln!(writer, "OS    {};", orgasism.as_str().unwrap()).unwrap();
        }

        if let Some(taxanomy_str) =  taxonomy.as_str() {
            writeln!(writer, "OC    {};", taxanomy_str).unwrap();
        }

        if let Some(refs) = references.as_array() {
            for reference in refs {
                if let Some(rn) = reference.get("Reference_Number") {
                    writeln!(writer, "RN    [{}];", rn.as_str().unwrap()).unwrap();
                }
                if let Some(rp) = reference.get("Reference_Position") {
                    writeln!(writer, "RP    {};", rp.as_str().unwrap()).unwrap();
                }
            }
        }

        writeln!(writer, "XX").unwrap();

        if let Some(features) = record.get("features") {
            for feature in features.as_array().unwrap_or(&vec![]) {
                if let Some(feature_type) = feature.get("type") {
                    writeln!(writer, "FT    {}                 {}", feature_type.as_str().unwrap(), feature.get("location").unwrap_or(&Value::Null)).unwrap();
                }

                if let Some(qualifiers) = feature.get("qualifiers").unwrap_or(&Value::Null).as_object() {
                    for (key, value) in  qualifiers {
                        writeln!(writer, "FT                    /{}={}", key, value.as_str().unwrap_or("")).unwrap();
                    }
                }
            }
        }

        if let Some(sequence) = record.get("sequence") {
            writeln!(writer, "SQ    Sequence {};", sequence.as_str().unwrap().len()).unwrap();
            let seq_str = sequence.as_str().unwrap();
            for chunk in seq_str.as_bytes().chunks(60) {
                writeln!(writer, "         {}", String::from_utf8_lossy(chunk)).unwrap();
            }
        }
        writeln!(writer, "//").unwrap();
    }

}