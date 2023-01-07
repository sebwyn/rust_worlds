use std::{collections::HashMap, path::Path, fs};

#[derive(Debug, Clone)]
pub(super) struct Character {
    pub tex_width: f32, pub tex_height: f32,
    pub tex_x: f32, pub tex_y: f32,
    pub width: u32, pub height: u32,
}

#[derive(Debug)]
pub(super) struct Font {
    characters: HashMap<char, Character>,
    pub image_path: String,
}

impl Font {
    pub fn get_character(&self, c: char) -> Option<Character> { self.characters.get(&c).cloned() }
}

impl Font {
    pub fn new(file: &str) -> Result<Self, String> {
        let file = Path::new(file);
        //load the font file
        let is_correct_extension = file.extension().map_or(false, |v| v.to_str().map_or(false, |s| String::from(s) == "fnt"));
        assert!(is_correct_extension, "Trying to load a font of an unsupported type: only '.fnt' is supported!");

        let fnt_file = fs::read_to_string(file).map_err(|e| "File does not exist")?;

        let char_reg = regex::Regex::new(
            r"char id=(\d+)\s+x=(\d+)\s+y=(\d+)\s+width=(\d+)\s+height=(\d+)\s+xoffset=(-?\d+)\s+yoffset=(-?\d+)\s+xadvance=(-?\d+)\s+page=(\d+)\s+chnl=(\d+)"
        ).unwrap();
        
        let captures = char_reg.captures_iter(&fnt_file);

        let page_reg = regex::Regex::new("page id=\\d+\\s+file=\"(.*)\"").unwrap();
        let image_file_name = &page_reg.captures(&fnt_file).unwrap()[1];
        let image_path = file.parent().unwrap().join(image_file_name);

        let (image_width, image_height) = image::image_dimensions(&image_path).map_err(|e| "File does not exist")?;

        let mut characters = HashMap::new();
        for capture in captures {
            let id = char::from(capture[1].parse::<u8>().unwrap());

            let x = capture[2].parse::<u32>().unwrap();
            let y = capture[3].parse::<u32>().unwrap();

            let width = capture[4].parse::<u32>().unwrap();
            let height = capture[5].parse::<u32>().unwrap();
            /*
            let _xoffset = capture[1].parse::<i32>().unwrap();
            let _yoffset = capture[1].parse::<i32>().unwrap();
            let _xadvance = capture[1].parse::<i32>().unwrap();
            let _page = capture[1].parse::<i32>().unwrap();
            let _chnl = capture[1].parse::<i32>().unwrap();
            */
            let tex_x = x as f32 / image_width as f32;
            let tex_y = y as f32 / image_height as f32;

            let tex_width = width as f32 / image_width as f32;
            let tex_height = height as f32 / image_height as f32;

            let character = Character { width, height, tex_x, tex_y, tex_width, tex_height};
            characters.insert(id, character);
        }

        Ok(Font {
            characters,
            image_path: String::from(image_path.to_str().unwrap()),
        })
    }
}