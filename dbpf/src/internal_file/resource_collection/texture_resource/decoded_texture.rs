use std::cmp::max;

#[derive(Clone, Debug, Default)]
pub struct DecodedTexture {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ShrinkResult {
    /// can be / has been shrunk
    Ok,
    /// too small to be shrunk in this direction
    Small,
    /// dimension(s) are odd size; cannot be shrunk
    Unable,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum ShrinkDirection {
    #[default]
    Both,
    Horizontal,
    Vertical,
}

impl DecodedTexture {
    pub fn can_shrink_dimensions(width: usize, height: usize, shrink_direction: ShrinkDirection) -> ShrinkResult {
        match shrink_direction {
            ShrinkDirection::Both => {
                match (Self::can_shrink_dimensions(width, height, ShrinkDirection::Horizontal),
                       Self::can_shrink_dimensions(width, height, ShrinkDirection::Vertical)) {
                    (ShrinkResult::Unable, _) | (_, ShrinkResult::Unable) => ShrinkResult::Unable,
                    (ShrinkResult::Small, ShrinkResult::Small) => ShrinkResult::Small,
                    _ => ShrinkResult::Ok
                }
            }
            d => {
                let dim = if d == ShrinkDirection::Horizontal {
                    width
                } else {
                    height
                };
                if dim == 1 {
                    ShrinkResult::Small
                } else if dim % 2 == 1 {
                    ShrinkResult::Unable
                } else {
                    ShrinkResult::Ok
                }
            }
        }
    }

    pub fn can_shrink(&self, shrink_direction: ShrinkDirection) -> ShrinkResult {
        Self::can_shrink_dimensions(self.width, self.height, shrink_direction)
    }

    /// halves the image in both dimensions, combining the value of groups of four pixels into a single pixel
    pub fn shrink(&mut self, preserve_alpha_test: Option<u8>, shrink_direction: ShrinkDirection) -> ShrinkResult {
        let can_shrink = self.can_shrink(shrink_direction);
        if can_shrink != ShrinkResult::Ok {
            return can_shrink;
        }
        let pixel_offset = 4;
        let new_width = if shrink_direction == ShrinkDirection::Vertical {
            self.width
        } else {
            max(self.width / 2, 1)
        };
        let new_height = if shrink_direction == ShrinkDirection::Horizontal {
            self.height
        } else {
            max(self.height / 2, 1)
        };
        match (self.width, self.height) {
            (w, h) if w == 1 || h == 1 || shrink_direction != ShrinkDirection::Both => {
                for y in 0..new_height {
                    for x in 0..new_width {
                        let i = x + y * new_width;
                        let (orig_i, orig_offset) = if shrink_direction == ShrinkDirection::Vertical {
                            let row_offset = pixel_offset * self.width;
                            (x * pixel_offset + y * row_offset * 2, row_offset)
                        } else {
                            (i * pixel_offset * 2, pixel_offset)
                        };
                        let new_i = i * pixel_offset;

                        let a0 = self.data[3 + orig_i] as u32;
                        let a1 = self.data[3 + orig_i + orig_offset] as u32;
                        let a_total = a0 + a1;

                        for c in 0..3 {
                            let (a0, a1, a_total) = if a_total == 0 {
                                (1, 1, 4)
                            } else {
                                (a0, a1, a_total)
                            };

                            let new_c = ((self.data[c + orig_i] as u32 * a0) +
                                (self.data[c + orig_i + orig_offset] as u32 * a1))
                                / a_total;
                            self.data[c + new_i] = new_c as u8;
                        }

                        self.data[3 + new_i] = (a_total / 2) as u8;
                    }
                }
            }
            (w, h) => {
                for y in 0..new_height {
                    for x in 0..new_width {
                        let orig_row_offset = pixel_offset * self.width;
                        let orig_i = (x * pixel_offset * 2) + (y * orig_row_offset * 2);
                        let new_i = (x * pixel_offset) + (y * pixel_offset * new_width);

                        let a0 = self.data[3 + orig_i] as u32;
                        let a1 = self.data[3 + orig_i + pixel_offset] as u32;
                        let a2 = self.data[3 + orig_i + orig_row_offset] as u32;
                        let a3 = self.data[3 + orig_i + orig_row_offset + pixel_offset] as u32;
                        let a_total = a0 + a1 + a2 + a3;

                        for c in 0..3 {
                            let (a0, a1, a2, a3, a_total) = if a_total == 0 {
                                (1, 1, 1, 1, 4)
                            } else {
                                (a0, a1, a2, a3, a_total)
                            };
                            let o = c;
                            // makes a rainbow effect yayyy
                            /*let o = if c < 3 {
                                (c + 1) % 3
                            } else {
                                c
                            };*/
                            // weigh color by the alpha channel
                            let new_c = ((self.data[o + orig_i] as u32 * a0) +
                                (self.data[o + orig_i + pixel_offset] as u32 * a1) +
                                (self.data[o + orig_i + orig_row_offset] as u32 * a2) +
                                (self.data[o + orig_i + orig_row_offset + pixel_offset] as u32 * a3))
                                / a_total;
                            self.data[c + new_i] = new_c as u8;
                        }
                        // alpha
                        if let Some(preserve_alpha) = preserve_alpha_test {
                            let preserve_alpha = preserve_alpha as u32;
                            let preserve_alpha_inv = 255 - preserve_alpha;

                            let new_c = (
                                a0 * (a0 * preserve_alpha + preserve_alpha_inv * preserve_alpha_inv) +
                                    a1 * (a1 * preserve_alpha + preserve_alpha_inv * preserve_alpha_inv) +
                                    a2 * (a2 * preserve_alpha + preserve_alpha_inv * preserve_alpha_inv) +
                                    a3 * (a3 * preserve_alpha + preserve_alpha_inv * preserve_alpha_inv))
                                / max((a_total * preserve_alpha) + (preserve_alpha_inv * preserve_alpha_inv * 4), 1);
                            self.data[3 + new_i] = new_c as u8;
                        } else {
                            self.data[3 + new_i] = (a_total / 4) as u8;
                        }
                    }
                }
            }
        }
        self.data.truncate(new_width * new_height * pixel_offset);
        self.data.shrink_to_fit();
        self.width = new_width;
        self.height = new_height;
        ShrinkResult::Ok
    }
}
