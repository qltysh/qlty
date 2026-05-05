use qlty_types::analysis::v1::Language;
use qlty_types::language_enum_from_name;

#[test]
fn scala_maps_to_scala_language_enum() {
    assert_eq!(language_enum_from_name("scala"), Language::Scala);
}

#[test]
fn vbnet_maps_to_vbdotnet() {
    assert_eq!(language_enum_from_name("vbnet"), Language::Vbdotnet);
}
