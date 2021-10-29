#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use core::{
    intrinsics::transmute,
    ptr::{null, slice_from_raw_parts_mut},
};

use crate::common::{pb_byte_t, pb_istream_s, pb_istream_t, pb_msgdesc_t, pb_wire_type_t, size_t};

include!("bindings/pb_decode.rs");

extern crate alloc;
use alloc::vec::Vec;

use crate::pb::Error;

pub trait rx_context {
    fn read(&mut self, buff: &mut [u8]) -> Result<usize, ()>;
}

impl rx_context for u8 {
    fn read(&mut self, _buff: &mut [u8]) -> Result<usize, ()> {
        unreachable!();
    }
}

pub struct IStream<T: rx_context> {
    ctx: pb_istream_t,
    reader: Option<T>,
}

impl<T: rx_context> IStream<T> {
    pub fn from_buffer(buf: &[u8]) -> Self {
        Self {
            ctx: unsafe { pb_istream_from_buffer(buf.as_ptr(), buf.len()) },
            reader: None,
        }
    }

    pub fn from_callback(rx_ctx: T, bytes_left: Option<usize>) -> Self {
        unsafe extern "C" fn read_wraper<U: rx_context>(
            stream: *mut pb_istream_s,
            buf: *mut u8,
            size: usize,
        ) -> bool {
            let cb: *mut dyn rx_context =
                transmute::<*mut ::core::ffi::c_void, *mut U>((*stream).state);
            match (*cb).read(&mut *slice_from_raw_parts_mut(buf, size)) {
                Ok(read) => {
                    (*stream).bytes_left -= read;
                    (*stream).bytes_left == 0
                }
                Err(_) => false,
            }
        }

        let mut res = Self {
            ctx: crate::common::pb_istream_s {
                callback: Some(read_wraper::<T>),
                state: null::<::core::ffi::c_void>() as *mut _,
                bytes_left: bytes_left.unwrap_or(usize::MAX),
                errmsg: null(),
            },
            reader: Some(rx_ctx),
        };

        res.ctx.state = res.reader.as_ref().unwrap() as *const _ as *mut _;

        res
    }

    pub fn decode<U>(&mut self, fields: &pb_msgdesc_t) -> Result<U, Error> {
        let mut dest_struct: U = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        if unsafe {
            pb_decode(
                &mut self.ctx,
                fields,
                &mut dest_struct as *mut U as *mut ::core::ffi::c_void,
            )
        } {
            Ok(dest_struct)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_ex<U>(&mut self, fields: &pb_msgdesc_t, flags: u32) -> Result<U, Error> {
        let mut dest_struct: U = unsafe { core::mem::MaybeUninit::zeroed().assume_init() };
        if unsafe {
            pb_decode_ex(
                &mut self.ctx,
                fields,
                &mut dest_struct as *mut U as *mut ::core::ffi::c_void,
                flags,
            )
        } {
            Ok(dest_struct)
        } else {
            Err(self.get_error())
        }
    }

    pub fn read(&mut self, count: usize) -> Result<Vec<u8>, Error> {
        let mut buf: Vec<u8> = Vec::with_capacity(count);
        buf.resize(count, 0);
        if unsafe { pb_read(&mut self.ctx, buf.as_mut_ptr(), count) } {
            Ok(buf)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_tag(&mut self, wire_type: &mut pb_wire_type_t) -> Result<u32, Error> {
        let mut tag = 0_u32;
        let mut eof = false;
        if unsafe { pb_decode_tag(&mut self.ctx, wire_type, &mut tag, &mut eof) } {
            if eof {
                Err(Error::from_str("EOF\0")) // TODO
            } else {
                Ok(tag)
            }
        } else {
            Err(self.get_error())
        }
    }

    pub fn skip_field(&mut self, wire_type: pb_wire_type_t) -> Result<(), Error> {
        if unsafe { pb_skip_field(&mut self.ctx, wire_type) } {
            Ok(())
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_variant(&mut self) -> Result<u64, Error> {
        let mut res = 0_u64;
        if unsafe { pb_decode_varint(&mut self.ctx, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_variant32(&mut self) -> Result<u32, Error> {
        let mut res = 0_u32;
        if unsafe { pb_decode_varint32(&mut self.ctx, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_bool(&mut self) -> Result<bool, Error> {
        let mut res = false;
        if unsafe { pb_decode_bool(&mut self.ctx, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_svariant(&mut self) -> Result<i64, Error> {
        let mut res = 0_i64;
        if unsafe { pb_decode_svarint(&mut self.ctx, &mut res) } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_fixed32(&mut self) -> Result<u32, Error> {
        let mut res = 0_u32;
        if unsafe {
            pb_decode_fixed32(
                &mut self.ctx,
                &mut res as *mut u32 as *mut ::core::ffi::c_void,
            )
        } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    pub fn decode_fixed64(&mut self) -> Result<u64, Error> {
        let mut res = 0_u64;
        if unsafe {
            pb_decode_fixed64(
                &mut self.ctx,
                &mut res as *mut u64 as *mut ::core::ffi::c_void,
            )
        } {
            Ok(res)
        } else {
            Err(self.get_error())
        }
    }

    fn get_error(&self) -> Error {
        Error::new(self.ctx.errmsg)
    }
}