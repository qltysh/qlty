use qlty_types::analysis::v1::Language;
use qlty_types::language_enum_from_name;

#[test]
fn scala_maps_to_scala_language_enum() {
    assert_eq!(language_enum_from_name("scala"), Language::Scala);
}
