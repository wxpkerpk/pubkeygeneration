use anchor_lang::prelude::*;

#[account]
#[derive(Default)]
pub struct CreatorMemecoinCounter {
    pub count: u32,
}

impl CreatorMemecoinCounter {
    pub const LEN: usize = 8 + 4;

    pub fn increment(
        &mut self,
    ) {
        self.count += 1;
    }
}
