use crate::graphics::shader::reflector::reflect_shader;

use std::path::Path;

#[test]
fn test_reflection() {
    let file = Path::new("shaders/basic.wgsl");
    let shader_source = std::fs::read_to_string(file).unwrap_or_else(|_| {
        panic!("Can't read shader file: basic.wgsl")
    });

    let reflection = reflect_shader(&shader_source);
    println!("{:?}", reflection)
}
