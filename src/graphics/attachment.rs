use super::Texture;

use std::rc::Rc;

//an attachment is a render target
pub enum Attachment {
    Swapchain,
    Texture(Rc<Texture>),
}

pub struct AttachmentAccess {
    pub clear_color: Option<([f64; 4])>,
    pub attachment: Attachment,
}
