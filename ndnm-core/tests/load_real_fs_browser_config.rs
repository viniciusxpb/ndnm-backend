// ndnm-core/tests/load_real_fs_browser_config.rs
use ndnm_core::{load_config, SectionBehavior};

#[test]
fn test_load_real_fs_browser_config() {
    // Tenta carregar o config.yaml real do node-fs-browser
    let config_path = "../node-fs-browser/config.yaml";
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let result = load_config(config_path, manifest_dir);

    match result {
        Ok((config, _path)) => {
            println!("✅ Config carregado com sucesso!");
            println!("node_id_hash: {:?}", config.node_id_hash);
            println!("label: {:?}", config.label);
            println!("node_type: {:?}", config.node_type);
            println!("sections count: {}", config.sections.len());

            // Verifica que tem sections
            assert!(!config.sections.is_empty(), "Deveria ter pelo menos uma section");

            // Verifica seção 1 - copy_here
            let section1 = &config.sections[0];
            assert_eq!(section1.section_name, "copy_here");
            assert_eq!(section1.behavior, SectionBehavior::AutoIncrement);
            assert!(section1.slot_template.is_some());

            // Verifica seção 2 - internal_files
            let section2 = &config.sections[1];
            assert_eq!(section2.section_name, "internal_files");
            assert_eq!(section2.behavior, SectionBehavior::DynamicPerFile);
            assert!(section2.slot_template.is_some());

            // Verifica input_fields
            assert_eq!(config.input_fields.len(), 2);
            assert_eq!(config.input_fields[0].name, "target_directory");
            assert_eq!(config.input_fields[1].name, "refresh_button");

            println!("✅ Todas as validações passaram!");
        }
        Err(e) => {
            panic!("❌ Falha ao carregar config: {}", e);
        }
    }
}
