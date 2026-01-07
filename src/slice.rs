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

// Assume buffer initialized
// @todo: Replace with BufRead.filled()
// @todo: Replace when `maybe_uninit_slice` feature
// will be stabilized
#[inline(always)]
pub(crate) fn slice_assume_init_ref(slice: &[MaybeUninit<u8>]) -> &[u8] {
    //MaybeUninit::slice_assume_init_ref(&self.buf[self.proto.ip_header_size..size]);
    unsafe { &*(slice as *const [MaybeUninit<u8>] as *const [u8]) }
}

// @todo: Replace with MaybeUninit::slice_assume_init_mut
// when `maybe_uninit_slice` feature will be stabilized
#[inline(always)]
pub(crate) fn slice_assume_init_mut(slice: &mut [MaybeUninit<u8>]) -> &mut [u8] {
    unsafe { &mut *(slice as *mut [MaybeUninit<u8>] as *mut [u8]) }
}
