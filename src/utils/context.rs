use cl3::{
    device::cl_device_id,
    platform::cl_platform_id,
    context::{
        cl_context,
        create_context
    },
    command_queue::{cl_bool, cl_int},
    ext::cl_context_memory_initialize_khr
};
use std::ffi::c_void;
use std::ptr;

/// This replaces the direct translation of the C array of properties (cl_context_properties) 
/// for the create context function
/// it is also only a subset of options from the original OpenCL standard, aiming at linux-first
/// usage
pub enum ContextProperties {
    ClContextPlatform(cl_platform_id),
    ClContextInteropUserSync(cl_bool),
    ClGlContextKhr(isize),
    ClEglDisplayKhr(isize),
    ClGlxDisplayKhr(isize),
    ClContextMemoryInitializeKhr(cl_context_memory_initialize_khr),
    ClContextTerminateKhr(cl_bool)
}

/// The original create_context function is a direct implementation of the C clCreateContext
/// function, which is cumbersome and not rust-y. Instead we abstract out such a function to
/// make it more user friendly. The error callback is removed because it requires unsafe.
pub fn create_context_rusty(
    slice_of_devices: &[cl_device_id],
    vector_of_properties: Vec<ContextProperties>,
) -> Result<cl_context, cl_int> {
    let mut vec_of_isize: Vec<isize> = vector_of_properties
        .iter()
        .map(|x| { 
            match x {
                ContextProperties::ClContextPlatform(y) => *y as isize,
                ContextProperties::ClContextInteropUserSync(y) => *y as isize,
                ContextProperties::ClGlContextKhr(y) => *y,
                ContextProperties::ClEglDisplayKhr(y) => *y,
                ContextProperties::ClGlxDisplayKhr(y) => *y,
                ContextProperties::ClContextMemoryInitializeKhr(y) => *y as isize,
                ContextProperties::ClContextTerminateKhr(y) => *y as isize
            }
        } 
    ).collect(); //cl_context_properties is an alias for isize 
    vec_of_isize.push(0); //null terminated
    let vec_ptr = vec_of_isize.as_ptr(); //ptr for C to find array of properties
    let null_ptr: *mut c_void = ptr::null_mut(); //no user data needed since no callback

    create_context(slice_of_devices, vec_ptr, None, null_ptr)
}

