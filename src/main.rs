mod utils;
use crate::utils::platform_and_device::{
    PlatformOrDevice,
    fuzzy_find_platform,
    user_select_platform_or_device,
    user_get_and_select_all_devices
};
use crate::utils::context::{
    create_context_rusty,
    ContextProperties
};
use cl3::device::{cl_device_id, get_device_info, CL_DEVICE_NAME};
use cl3::info_type::InfoType;
use cl3::context::{get_context_info, CL_CONTEXT_DEVICES};
use cl3::program::{create_program_with_source, 
    get_program_build_info, 
    CL_PROGRAM_BUILD_LOG,
    build_program
};
use cl3::kernel::create_kernel;
use cl3::memory::{
    create_buffer,
    CL_MEM_READ_ONLY,
    CL_MEM_WRITE_ONLY,
    release_mem_object
};
use cl3::command_queue::{
    create_command_queue,
    enqueue_write_buffer,
    enqueue_read_buffer,
    enqueue_nd_range_kernel,
    release_command_queue
};
use dotenv::dotenv;
use inquire::Text;
use std::{
    env,
    path::Path,
    ffi::{
        c_void,
        CStr,
        CString
    },
    fs,
    slice,
    ptr,
    cmp::max,
};


/// read .env file for platform and device 
/// if not found then platform is blank String, and device is a null pointer
fn init_env_vars() -> (String, cl_device_id) {
    //init variables to be returned
    let mut platform_name_fuzzy_env: String = String::from("");
    let mut device_uuid_env: cl_device_id = std::ptr::null_mut();
    // if ./.env exists, load it
    if Path::new(".env").exists() {
        dotenv().ok(); // read .env file
        // assign the env vars, return a Result for both
        let platform_name_fuzzy_env_res = env::var("PLATFORM_NAME_FUZZY_ENV");
        let device_uuid_env_res = env::var("DEVICE_UUID_ENV");
        
        match platform_name_fuzzy_env_res {
            Ok(val) => { 
                println!("PLATFORM_NAME_FUZZY_ENV: {:?}", val);
                platform_name_fuzzy_env = val;
            },
            Err(e) => {
                println!("Error PLATFORM_NAME_FUZZY_ENV: {}", e);
            }
        }

        match device_uuid_env_res {
            Ok(val) => { 
                println!("DEVICE_UUID_ENV: {:?}", val);
                device_uuid_env = &val as *const _ as *mut c_void;
            },
            Err(e) => {
                println!("device ID not found ({}), proceeding to device selector", e);
            } 
        }

    } else {
        println!(".env not present, proceeding with prompt based selection ... ");
        platform_name_fuzzy_env = match Text::new(
            "What platform do you want to use (this is a fuzzy finder, 
            i.e. you can just use a key word): ").prompt() {
                Ok(text) => text,
                Err(e) => {
                    println!("There was an error with your response: {}", e);
                    panic!()
                }
            }
    }

    (platform_name_fuzzy_env, device_uuid_env)
}

fn main() {

    // Initialize Program Flow:
    // (1) Prompt the user to either: 
    //      (a) fuzzy find the platform by vendor name
    //          (i) if only a single platform is found, use that
    //          (ii) else: proceed to (b)
    //      (b) list out all of the plaforms available, and let the user choose which one
    //      (c) use a .env file to automatically pick up the plaform and device (skip 2 & 3)
    // (2) Prompt the user to choose a device out of the list of them from the platform
    // (3) Ask if the user wants to save these settings to a .env file
   
    // initialize variables: either with .env, or with user prompt for user to enter for platform
    let (platform_name_fuzzy_env, device_uuid_env) = init_env_vars();

    // fuzzy find the plaform
    let platform_vec: Vec<PlatformOrDevice> = fuzzy_find_platform(&platform_name_fuzzy_env, true);
    let mut chosen_platform: PlatformOrDevice = match platform_vec.first(){
        Some(platform) => platform.clone(),
        None => {
            println!("No platform found, this program will terminate in response.");
            panic!()
        }
    };
    // if only a single platform is found, use that, else ask the user to choose
    if platform_vec.len() > 1 {
        let q = "Multiple platforms with the same pattern were found, please choose one:";
        let plat_or_dev = user_select_platform_or_device(q, platform_vec);
        if let Ok(plat) = plat_or_dev {
            chosen_platform = plat;
        }
    }
    // confirm platform selection
    if let PlatformOrDevice::Plat(platform_id, ref platform_name) = chosen_platform {
        println!("You chose the platform: {:?} [id:{:?}]", platform_name, platform_id);
    }

    // get devices for the chosen platform
    let chosen_device: PlatformOrDevice = if device_uuid_env.is_null() {
        match user_get_and_select_all_devices(chosen_platform){
            Ok(d) => d,
            Err(err) => {
                println!("Something went wrong with device selection: {:?}", err);
                panic!();
            }
        }
    } else {
        let dev_name = String::from(
            get_device_info(device_uuid_env, CL_DEVICE_NAME).unwrap_or(
                InfoType::VecUchar("No Name".as_bytes().to_vec())
            )
        );
        PlatformOrDevice::Dev(
            device_uuid_env,
            dev_name
        )
    };
    // confirm device selection
    let dev_id : cl_device_id;
    if let PlatformOrDevice::Dev(device_id, device_name) = chosen_device {
        println!("You chose the device: {:?} [id:{:?}]", device_name, device_id);
        dev_id = device_id;
    } else {
        println!("no device found");
        panic!(); //no device found
    }

    //to allow for potential multi select in the future, put the device in a Vec
    let vec_devices = vec![dev_id];

    //create context
    let context_properties: Vec<ContextProperties> = vec![];
    let ctx_wrapped = create_context_rusty(&vec_devices[0..1], context_properties);
    let ctx = ctx_wrapped.unwrap();

    let ctx_info = match ctx_wrapped {
        Ok(c) => get_context_info(c, CL_CONTEXT_DEVICES),
        Err(e) => {
            println!("context error {e}");
            panic!();
        }
    };
    println!("Context: {:?}", ctx_info.unwrap());

    // create program for testing
    // read c code as a source string
    let c_source_string = fs::read_to_string("./src/kernels/matrix_addition.c").unwrap();
    //let c_source_string = fs::read_to_string("./src/kernels/test.c").unwrap();
    let c_source_str = c_source_string.as_str();
    let c_source_kernel_arr = [c_source_str];
    println!("=============================");
    println!("{}", &c_source_kernel_arr[0]);
    println!("=============================");
    let c_kernel_slice: &[&str] = slice::from_ref(c_source_kernel_arr.first().unwrap());
    let prog = create_program_with_source(ctx, c_kernel_slice).unwrap();
    println!("program: {:?}", prog);
    let build_res = build_program(prog, &vec_devices[0..1], c"", None, ptr::null_mut());
    match build_res {
        Ok(_) => println!("program built successfully"),
        Err(e) => {
            println!("Error code: {:?}", e);
            let info = get_program_build_info(
                prog,
                vec_devices[0],
                CL_PROGRAM_BUILD_LOG
            ).unwrap();
            println!("logs: {}", String::from(info));
        }
    }
    let kernel = create_kernel(prog, c"matrix_addition").unwrap();
    println!("kernel: {:?}", kernel);

    let mut vec_a: Vec<i32> = vec![1, 2, 3, 4, 5];
    let size_bytes_a = vec_a.len() * std::mem::size_of_val(&vec_a[0]);
    let mut vec_b: Vec<i32> = vec![10;5];
    let size_bytes_b = vec_b.len() * std::mem::size_of_val(&vec_b[0]);
    let void_ptr_a: *mut c_void = vec_a.as_mut_ptr() as *mut c_void;
    let void_ptr_b: *mut c_void = vec_b.as_mut_ptr() as *mut c_void;
    let null_ptr: *mut c_void = ptr::null_mut();
    let mut vec_res: Vec<i32> = Vec::new();
    let void_ptr_res: *mut c_void = vec_res.as_mut_ptr() as *mut c_void;
    unsafe {
        let mem_a = create_buffer(ctx, CL_MEM_READ_ONLY, size_bytes_a, void_ptr_a);
        let mem_b = create_buffer(ctx, CL_MEM_READ_ONLY, size_bytes_b, void_ptr_b);
        let mem_res = create_buffer(ctx, CL_MEM_READ_ONLY, max(size_bytes_a, size_bytes_b), null_ptr);

        let cq_wrapped = create_command_queue(ctx, vec_devices[0], 0u64);
        let cq = cq_wrapped.unwrap();
        let write_ev_1 = enqueue_write_buffer(cq, mem_a.unwrap(), 0, 0, size_bytes_a, void_ptr_a, 0, &null_ptr);
        let write_ev_2 = enqueue_write_buffer(cq, mem_b.unwrap(), 0, 0, size_bytes_b, void_ptr_b, 0, &null_ptr);

        let kernel_ev = enqueue_nd_range_kernel(cq, kernel, 1, &0, &max(size_bytes_a, size_bytes_b), &1, 0, &null_ptr);

        let read_ev = enqueue_read_buffer(cq, mem_res.unwrap(), 0, 0, max(size_bytes_a, size_bytes_b), void_ptr_res, 0, &null_ptr);

        let rel_res_a = release_mem_object(mem_a.unwrap());
        let rel_res_b = release_mem_object(mem_b.unwrap());
        let rel_res_res = release_mem_object(mem_res.unwrap());
        let res_rel_cq = release_command_queue(cq);
    }
    
    println!("output: {:?}", vec_res);
}
