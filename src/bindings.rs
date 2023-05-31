// Copyright 2019. The Tari Project
//
// Redistribution and use in source and binary forms, with or without modification, are permitted provided that the
// following conditions are met:
//
// 1. Redistributions of source code must retain the above copyright notice, this list of conditions and the following
// disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the
// following disclaimer in the documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote
// products derived from this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES,
// INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY,
// WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE
// USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use libc::{c_uint, c_ulong, c_void};

#[repr(C)]
pub struct randomx_dataset {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct randomx_cache {
    _unused: [u8; 0],
}

#[repr(C)]
pub struct randomx_vm {
    _unused: [u8; 0],
}

extern "C" {
    pub fn randomx_alloc_cache(flags: c_uint) -> *mut randomx_cache;
    pub fn randomx_init_cache(cache: *mut randomx_cache, key: *const c_void, keySize: usize);
    pub fn randomx_release_cache(cache: *mut randomx_cache);
    pub fn randomx_alloc_dataset(flags: c_uint) -> *mut randomx_dataset;
    pub fn randomx_dataset_item_count() -> c_ulong;
    pub fn randomx_init_dataset(
        dataset: *mut randomx_dataset,
        cache: *mut randomx_cache,
        start_item: c_ulong,
        item_count: c_ulong,
    );
    pub fn randomx_get_dataset_memory(dataset: *mut randomx_dataset) -> *mut c_void;
    pub fn randomx_release_dataset(dataset: *mut randomx_dataset);
    pub fn randomx_create_vm(
        flags: c_uint,
        cache: *mut randomx_cache,
        dataset: *mut randomx_dataset,
    ) -> *mut randomx_vm;
    pub fn randomx_vm_set_cache(machine: *mut randomx_vm, cache: *mut randomx_cache);
    pub fn randomx_vm_set_dataset(machine: *mut randomx_vm, dataset: *mut randomx_dataset);
    pub fn randomx_destroy_vm(machine: *mut randomx_vm);
    pub fn randomx_calculate_numeric(machine: *mut randomx_vm, input: *const c_void, input_size: usize) -> u64;
    pub fn randomx_calculate_hash_first(machine: *mut randomx_vm, input: *const c_void, input_size: usize);
    pub fn randomx_calculate_numeric_next(
        machine: *mut randomx_vm,
        input_next: *const c_void,
        input_size_next: usize,
    ) -> u64;
    pub fn randomx_calculate_numeric_last(machine: *mut randomx_vm) -> u64;
    pub fn randomx_get_flags() -> c_uint;
}

#[cfg(test)]
mod tests {
    use std::{ffi::CString, mem, ptr};

    use libc::{c_char, c_uint, c_void};

    use super::*;

    #[test]
    fn alloc_cache() {
        let key = "Key";
        let c_key = CString::new(key).unwrap();
        let c_key_ptr = c_key.as_bytes().as_ptr() as *mut c_void;
        let flag: c_uint = 0;
        let cache = unsafe { randomx_alloc_cache(flag) };
        let size_key = c_key.as_bytes().len() * mem::size_of::<*const c_char>();

        unsafe {
            randomx_init_cache(cache, c_key_ptr, size_key);
        }
        assert!(!cache.is_null(), "Failed to init cache");
        unsafe {
            randomx_release_cache(cache);
        }
    }

    #[test]
    fn alloc_dataset() {
        let key = "Key";
        let c_key = CString::new(key).unwrap();
        let c_key_ptr = c_key.as_bytes().as_ptr() as *mut c_void;
        let flag: c_uint = 0;
        let cache = unsafe { randomx_alloc_cache(flag) };
        let size_key = c_key.as_bytes().len() * mem::size_of::<*const c_char>();

        unsafe {
            randomx_init_cache(cache, c_key_ptr, size_key);
        }

        let dataset = unsafe { randomx_alloc_dataset(flag) };
        let item_count = unsafe { randomx_dataset_item_count() };
        assert_ne!(0, item_count);

        unsafe { randomx_init_dataset(dataset, cache, 0, item_count) }

        unsafe {
            randomx_release_dataset(dataset);
            randomx_release_cache(cache);
        }
    }

    #[test]
    fn alloc_vm() {
        let key = "Key";
        let flag: c_uint = 0;

        let c_key = CString::new(key).unwrap();
        let c_key_ptr = c_key.as_bytes().as_ptr() as *mut c_void;

        let cache = unsafe { randomx_alloc_cache(flag) };
        let size_key = c_key.as_bytes().len() * mem::size_of::<*const c_char>();

        unsafe {
            randomx_init_cache(cache, c_key_ptr, size_key);
        }
        let mut vm = unsafe { randomx_create_vm(flag, cache, ptr::null_mut()) };
        if vm.is_null() {
            panic!("Failed to init vm with cache");
        }
        unsafe {
            randomx_vm_set_cache(vm, cache);
        }
        if vm.is_null() {
            panic!("Failed to re-init vm with new cache");
        }
        let dataset = unsafe { randomx_alloc_dataset(flag) };
        let item_count = unsafe { randomx_dataset_item_count() };
        unsafe { randomx_init_dataset(dataset, cache, 0, item_count) }
        vm = unsafe { randomx_create_vm(flag, cache, dataset) };
        if vm.is_null() {
            panic!("Failed to init vm with dataset");
        }
        unsafe {
            randomx_vm_set_dataset(vm, dataset);
        }
        if vm.is_null() {
            panic!("Failed to re-init vm with new dataset");
        }
        unsafe {
            randomx_release_dataset(dataset);
            randomx_release_cache(cache);
            randomx_destroy_vm(vm);
        }
    }

    #[test]
    fn calculate_hash() {
        let key = "Key";
        let input = "Input";

        let flag: c_uint = 0;

        let c_key = CString::new(key).unwrap();
        let c_input = CString::new(input).unwrap();
        let c_key_ptr = c_key.as_bytes().as_ptr() as *mut c_void;
        let c_input_ptr = c_input.as_bytes().as_ptr() as *mut c_void;

        let cache = unsafe { randomx_alloc_cache(flag) };
        let size_key = c_key.as_bytes().len() * mem::size_of::<*const c_char>();
        let size_input = c_input.as_bytes().len() * mem::size_of::<*const c_char>();

        unsafe {
            randomx_init_cache(cache, c_key_ptr, size_key);
        }

        let vm = unsafe { randomx_create_vm(flag, cache, ptr::null_mut()) };

        let hash = unsafe { randomx_calculate_numeric(vm, c_input_ptr, size_input) };
        assert_ne!(hash, 0);

        unsafe {
            randomx_destroy_vm(vm);
            randomx_release_cache(cache);
        }
    }

    #[allow(clippy::cast_sign_loss)]
    #[test]
    fn calculate_hash_set() {
        let key = "Key";
        let input = "Input";
        let input2 = "Input 2";
        let input3 = "Input 3";

        let flag: c_uint = 0;

        let c_key = CString::new(key).unwrap();
        let c_input = CString::new(input).unwrap();
        let c_input2 = CString::new(input2).unwrap();
        let c_input3 = CString::new(input3).unwrap();
        let c_key_ptr = c_key.as_bytes().as_ptr() as *mut c_void;
        let c_input_ptr = c_input.as_bytes().as_ptr() as *mut c_void;
        let c_input_ptr2 = c_input2.as_bytes().as_ptr() as *mut c_void;
        let c_input_ptr3 = c_input3.as_bytes().as_ptr() as *mut c_void;

        let cache = unsafe { randomx_alloc_cache(flag) };
        let size_key = c_key.as_bytes().len() * mem::size_of::<*const c_char>();
        let size_input = c_input.as_bytes().len() * mem::size_of::<*const c_char>();
        let size_input2 = c_input2.as_bytes().len() * mem::size_of::<*const c_char>();
        let size_input3 = c_input3.as_bytes().len() * mem::size_of::<*const c_char>();

        unsafe {
            randomx_init_cache(cache, c_key_ptr, size_key);
        }

        let vm = unsafe { randomx_create_vm(flag, cache, ptr::null_mut()) };

        unsafe {
            randomx_calculate_hash_first(vm, c_input_ptr, size_input);
        }

        let hash = unsafe { randomx_calculate_numeric_next(vm, c_input_ptr2, size_input2) };
        assert_ne!(0, hash);

        let hash2 = unsafe { randomx_calculate_numeric_next(vm, c_input_ptr3, size_input3) };
        assert_ne!(0, hash2);
        assert_ne!(hash, hash2);

        let hash3 = unsafe { randomx_calculate_numeric_last(vm) };
        assert_ne!(0, hash3);
        assert_ne!(hash2, hash3);

        unsafe {
            randomx_destroy_vm(vm);
            randomx_release_cache(cache);
        }
    }
}
