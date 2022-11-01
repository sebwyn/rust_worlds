#[derive(Debug)]
pub struct TileView {
    width: u32,
    height: u32,

    rows: u32,
    cols: u32,

    row: u32,
    col: u32,
}

impl TileView {
    pub fn new(path: &str, rows: u32, cols: u32, row: u32, col: u32) -> Self {
        //get width and height from a texture
        let (width, height) = image::image_dimensions(path).expect("Invalid image path");

        Self {
            width,
            height,
            rows,
            cols,
            row,
            col
        }
    }

    pub fn set_position(&mut self, row: u32, col: u32) {
        self.row = row; self.col = col;
    }

    pub fn tex_coords(&self) -> [[f32; 2]; 4] {
        let x_step = (self.width as f32 / self.cols as f32) / self.width as f32;
        let y_step = (self.height as f32 / self.rows as f32) / self.height as f32;

        let x_start = self.col as f32 * x_step;
        let y_start = 1f32 - self.row as f32 * y_step;

        //what I thought the text coords are rotated clockwise twice, because the image was flipped
        let tex_coords = [[x_start + x_step, y_start], [x_start, y_start], [x_start, y_start - y_step], [x_start + x_step, y_start - y_step]];
        tex_coords
    }
}