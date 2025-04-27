use generate_wgsl_oil::generate_from_entrypoints;

fn main() {
    let dir = std::path::Path::new("res/shaders/");
    let mut shader_paths = vec![];
    for res in dir.read_dir().expect("failed to open res/shaders/") {
        if let Ok(entry) = res {
            shader_paths.push(entry.path().to_string_lossy().into_owned());
        }
    }
    let result = generate_from_entrypoints(&shader_paths);
    std::fs::write("src/renderer/shaders.rs", result).unwrap();
}
