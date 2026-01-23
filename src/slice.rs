// ---------------------------------------------------------------------
// Gufo Ping: Slice utilities
// ---------------------------------------------------------------------
// Copyright (C) 2022-26, Gufo Labs
// ---------------------------------------------------------------------

use std::mem::MaybeUninit;

pub(crate) type BufType = [MaybeUninit<u8>; 4096];

#[inline(always)]
pub(crate) fn get_buffer_mut() -> BufType {
    unsafe { MaybeUninit::uninit().assume_init() }
}
