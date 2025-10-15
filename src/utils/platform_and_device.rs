use cl3::device::{cl_device_id, 
    cl_platform_id, 
    get_device_ids,
    get_device_info,
    CL_DEVICE_TYPE_ALL,
    CL_DEVICE_NAME,
    CL_DEVICE_GLOBAL_MEM_SIZE
};
use cl3::info_type::InfoType;
use cl3::command_queue::cl_int;
use cl3::platform::{get_platform_info, CL_PLATFORM_NAME};
use cl3::error_codes::error_text;
use regex::RegexBuilder;
use inquire::{error::InquireError, Select};
use std::fmt;

/// treat platform and device in a single sum dev_type
/// For the String data: Platform has vendor name, Device has Device name
#[derive(Debug, Clone)]
pub enum PlatformOrDevice {
   Plat(cl_platform_id, String),
   Dev(cl_device_id, String)
}
impl fmt::Display for PlatformOrDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformOrDevice::Plat(id, name) => write!(f, "{:?}:{:?}", name, id),
            PlatformOrDevice::Dev(id, name) => write!(f, "{:?}:{:?}", name, id)
        }
    }
}

/// This is for being able to search for your wanted platform with regex
/// also option to list platforms to the console with the param list_platforms
pub fn fuzzy_find_platform(pattern: &String, list_platforms: bool) -> Vec<PlatformOrDevice> {
    // build a regex object
    let re = RegexBuilder::new(format!(r"{}", pattern)
        .as_str())
        .case_insensitive(true)
        .build()
        .unwrap();
    // get all platforms ids from OpenCL platform api
    let platform_vec = match cl3::platform::get_platform_ids(){
        Ok(vec) => vec,
        Err(error_code) => {
            println!("Error in getting plaform IDs: {}", error_text(error_code));
            panic!();
        }
    };
    // create a vector and append to it the plaforms found
    let mut plat_id_vec : Vec<PlatformOrDevice> = Vec::new();
    for id in &platform_vec {
        // convert InfoType to &str
        let plat_name: String = get_platform_info(*id, CL_PLATFORM_NAME)
            .unwrap()
            .to_string();
        // if the user wants to list out the plaforms for debugging reasons 
        if list_platforms {
            println!("platform name: {}", plat_name);
        }
        // append it to our vector if regex matches
        if re.is_match(plat_name.as_str()){
           plat_id_vec.push(PlatformOrDevice::Plat(*id, plat_name))
        }
    }    
    plat_id_vec
}


/// interactive prompt for the user to select a single platform or a single device
pub fn user_select_platform_or_device(
    question: &str, 
    id_and_vendor_vec: Vec<PlatformOrDevice>
) -> Result<PlatformOrDevice, InquireError>{

    println!("--------------------------------");

    // create a selection prompt
    let answer: Result<PlatformOrDevice, InquireError> = Select::new(
        question, 
        id_and_vendor_vec
    ).prompt();

    answer
}

pub fn user_get_and_select_all_devices(
    plat: PlatformOrDevice
) -> Result<PlatformOrDevice, InquireError> {

    // retrieve the platform id from the PlatformOrDevice sum type
    let mut vec_dev_id: Vec<PlatformOrDevice> = Vec::new();
    let plat_id: cl_platform_id;
    if let PlatformOrDevice::Plat(platform_id, _) = plat { 
        plat_id = platform_id; 
    }
    else {
        println!("Unexpected Behaviour: PlatformOrDevice::Plat has no data in id field");
        panic!()
    }

    // get devices and convert them to our PlatformOrDevice sum type
    vec_dev_id = match get_device_ids(plat_id, CL_DEVICE_TYPE_ALL) {
        Ok(vec_dev_id) => {
            let mut vec: Vec<PlatformOrDevice> = Vec::new();
            for id in vec_dev_id{
                let dev_name = String::from(
                    get_device_info(id, CL_DEVICE_NAME).unwrap_or(
                        InfoType::VecUchar("No Name".as_bytes().to_vec())
                    )
                );
                let dev_memory_size: InfoType = get_device_info(id, CL_DEVICE_GLOBAL_MEM_SIZE).unwrap();
                let dev_memory_size_mb: u64 = match dev_memory_size {
                    InfoType::Ulong(mem) => mem / 1000000,
                    _ => 0u64
                };
                let dev = format!("{:?} [Memory={}MB]", dev_name, dev_memory_size_mb);
                vec.push(PlatformOrDevice::Dev(id, dev));
            }
            vec
        },
        Err(err) => {
            println!("No devices found, {:?}", err);
            vec_dev_id
        }
    };

    //return the answer back
    user_select_platform_or_device(
        "Please select one of the following devices found:", 
        vec_dev_id
    )
}

