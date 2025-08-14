fn new() {
    let diffuse_texture = include_bytes!("../../happy-tree.png");
    let diffuse_image = image::load_from_memory(diffuse_texture).expect("load texture from memory");
}
