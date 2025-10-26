// ndnm-core/tests/config_sections_test.rs
use ndnm_core::{NodeConfig, SectionBehavior, ConnectionMode};

#[test]
fn test_deserialize_advanced_config() {
    let yaml = r#"
node_id_hash: "hash_sha256_test"
label: "📂 Test Node"
node_type: "filesystem"

sections:
  - section_name: "copy_here"
    section_label: "Copiar Para Cá"
    behavior: "auto_increment"
    slot_template:
      input:
        name: "copy_input"
        label: "Novo Arquivo"
        type: "FILE_CONTENT"
        connections: 1
      output:
        name: "copied_output"
        label: "Arquivo Copiado"
        type: "FILE_CONTENT"
        connections: "n"

  - section_name: "internal_files"
    section_label: "Arquivos na Pasta"
    behavior: "dynamic_per_file"
    slot_template:
      input:
        name: "internal_input"
        label: "Substituir {filename}"
        type: "FILE_CONTENT"
        connections: 1
      output:
        name: "internal_output"
        label: "{filename}"
        type: "FILE_CONTENT"
        connections: "n"

input_fields:
  - name: "target_directory"
    type: "text"
  - name: "refresh_button"
    type: "button"
"#;

    let config: NodeConfig = serde_yaml::from_str(yaml).expect("Failed to deserialize config");

    // Verifica campos básicos
    assert_eq!(config.node_id_hash, Some("hash_sha256_test".to_string()));
    assert_eq!(config.label, Some("📂 Test Node".to_string()));
    assert_eq!(config.node_type, Some("filesystem".to_string()));

    // Verifica sections
    assert_eq!(config.sections.len(), 2);

    // Verifica seção 1 (auto_increment)
    let section1 = &config.sections[0];
    assert_eq!(section1.section_name, "copy_here");
    assert_eq!(section1.section_label, Some("Copiar Para Cá".to_string()));
    assert_eq!(section1.behavior, SectionBehavior::AutoIncrement);

    let template1 = section1.slot_template.as_ref().unwrap();
    let input1 = template1.input.as_ref().unwrap();
    assert_eq!(input1.name, "copy_input");
    assert_eq!(input1.r#type, "FILE_CONTENT");
    assert_eq!(input1.connections, ConnectionMode::Single);

    let output1 = template1.output.as_ref().unwrap();
    assert_eq!(output1.name, "copied_output");
    assert_eq!(output1.connections, ConnectionMode::Multiple);

    // Verifica seção 2 (dynamic_per_file)
    let section2 = &config.sections[1];
    assert_eq!(section2.section_name, "internal_files");
    assert_eq!(section2.behavior, SectionBehavior::DynamicPerFile);

    // Verifica input_fields
    assert_eq!(config.input_fields.len(), 2);
    assert_eq!(config.input_fields[0].name, "target_directory");
    assert_eq!(config.input_fields[1].name, "refresh_button");
}

#[test]
fn test_legacy_config_compatibility() {
    let yaml = r#"
port: 3000
label: "➕ Somar"
node_type: "add"
inputs_mode: "n"
initial_inputs_count: 1
outputs_mode: "1"
initial_outputs_count: 1
"#;

    let config: NodeConfig = serde_yaml::from_str(yaml).expect("Failed to deserialize legacy config");

    assert_eq!(config.port, 3000);
    assert_eq!(config.label, Some("➕ Somar".to_string()));
    assert_eq!(config.node_type, Some("add".to_string()));
    assert_eq!(config.inputs_mode, Some("n".to_string()));
    assert_eq!(config.sections.len(), 0); // Sem sections no formato legado
}
