use std::collections::HashMap;

struct Character {
    tex_coords: [f32; 4],
    rect: [[f32; 2]; 4],
}

struct Font {
    characters: HashMap<char, Character>
}

impl Font {
    pub fn new(file: &str) {
        
    }
}