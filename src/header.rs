use bytemuck::{Pod, Zeroable};

#[derive(Pod, Clone, Copy, Zeroable)]
#[repr(C)]
pub struct Header {
    file_content_length: u64,
    file_name_length: u32,
    _padding: u32,
}

impl Header {
    pub fn new(file_content_length: usize, file_name_length: usize) -> Self {
        Self {
            file_content_length: file_content_length as u64,
            file_name_length: file_name_length as u32,
            _padding: 0,
        }
    }

    pub fn file_content_length(&self) -> usize {
        self.file_content_length as usize
    }

    pub fn sub_path_length(&self) -> usize {
        self.file_name_length as usize
    }
}
