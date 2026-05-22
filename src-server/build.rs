fn main() {
    cynic_codegen::register_schema("anilist")
        .from_sdl_file("schemes/anilist/schema.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}