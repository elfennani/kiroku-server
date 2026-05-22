fn main() {
    cynic_codegen::register_schema("anilist")
        .from_sdl_file("schemes/anilist/schema.json")
        .unwrap()
        .as_default()
        .unwrap();
}