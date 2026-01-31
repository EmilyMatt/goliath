use tinyvec::ArrayVec;

pub(crate) struct ScreenSpace {
    width: u32,
    height: u32,
    page_count: u32,                      // Number of pages in the display
    buffer: ArrayVec<[u8; 1024]>,         // Maximum buffer for 128x128 display
    pages_to_update: ArrayVec<[bool; 8]>, // Pages that need to be updated
}

impl ScreenSpace {
    pub(crate) fn new(width: u32, height: u32) -> Self {
        let page_count = height / 8; // Each page is 8 pixels tall
        let buffer = ArrayVec::from_array_len([0; 1024], (width * page_count) as usize);

        // Initially, everything needs to be updated
        let pages_to_update = ArrayVec::from_array_len([true; 8], page_count as usize);

        ScreenSpace {
            width,
            height,
            page_count,
            buffer,
            pages_to_update,
        }
    }

    pub(crate) fn clear(&mut self) {
        let buffer_len = self.buffer.len();
        self.buffer[0..buffer_len].fill(0);
        self.pages_to_update[0..].fill(true)
    }

    #[allow(unused)]
    pub(crate) fn update(&mut self, pos: usize, data: &[u8]) {
        if data.is_empty() {
            return;
        }

        let screen_width = self.width as usize;
        let pos_page_start = pos % screen_width;
        let start_height = pos_page_start / screen_width;
        let start_page_idx = start_height / 8;

        let page_count = data.len().div_ceil(128);

        self.buffer[pos..pos + data.len()].copy_from_slice(data);
        self.pages_to_update[start_page_idx..start_page_idx + page_count].fill(true);
    }

    pub(crate) fn take_pages_to_update(&mut self) -> ArrayVec<[bool; 8]> {
        let pages_to_update = self.pages_to_update;
        self.pages_to_update.as_mut_slice().fill(false);

        pages_to_update
    }

    pub(crate) fn width(&self) -> u32 {
        self.width
    }

    pub(crate) fn height(&self) -> u32 {
        self.height
    }

    #[allow(unused)]
    pub(crate) fn page_count(&self) -> u32 {
        self.page_count
    }

    pub(crate) fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    #[allow(unused)]
    pub(crate) fn buffer_size(&self) -> usize {
        self.buffer.len()
    }
}
