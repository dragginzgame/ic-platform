use super::*;
#[test]
fn inventory_tracks_actual_cli_actions_and_embeds_the_live_dictionary() {
    let manual = manual();
    let rows = manual["command_inventory"]["profile"].as_array().unwrap();
    assert_eq!(rows.len(), ProfileAction::value_variants().len());
    for action in ProfileAction::value_variants() {
        let name = action.to_possible_value().unwrap();
        assert!(rows.iter().any(|row| row["name"] == name.get_name()));
    }
    assert_eq!(
        manual["system_profile"]["built_in_reference"],
        crate::profiling::backend_description::explain()
    );
    assert!(
        manual["rust_system_example"]
            .as_str()
            .unwrap()
            .contains("start_system_profile")
    );
    assert!(
        manual["server_options"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["flag"] == "--instruction-profile-max-origin-ranges RANGES")
    );
}
