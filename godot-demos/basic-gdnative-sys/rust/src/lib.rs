use gdnative_sys as sys;
use std::ffi::{c_void, CStr, CString};
use std::mem::{forget, size_of_val, MaybeUninit};
use std::os::raw::c_int;
use std::ptr;

static mut API: Option<sys::GodotApi> = None;

#[no_mangle]
pub unsafe extern "C" fn godot_gdnative_init(options: *const sys::godot_gdnative_init_options) {
    println!("FROM RUST: init called");
    API = Some(sys::GodotApi::from_api_struct((*options).api_struct));
}

#[no_mangle]
pub unsafe extern "C" fn godot_gdnative_terminate(
    _options: *const sys::godot_gdnative_terminate_options,
) {
    println!("FROM RUST: terminate called");
    API = None;
}

#[no_mangle]
pub unsafe extern "C" fn add_42(args: *const sys::godot_array) -> sys::godot_variant {
    println!("FROM RUST: add_42 called");
    let api = API.as_ref().unwrap();
    assert_eq!((api.godot_array_size)(args), 1);
    let godot_variant = (api.godot_array_get)(args, 0);
    assert_eq!(
        (api.godot_variant_get_type)(&godot_variant),
        sys::godot_variant_type_GODOT_VARIANT_TYPE_INT
    );
    let n = (api.godot_variant_as_int)(&godot_variant);
    let result = n + 42;
    let mut result_variant = MaybeUninit::<sys::godot_variant>::uninit();
    (api.godot_variant_new_int)(result_variant.as_mut_ptr(), result);
    result_variant.assume_init()
}

#[no_mangle]
pub unsafe extern "C" fn godot_nativescript_init(handle: *mut c_void) {
    println!("FROM RUST: ns init");
    let api = API.as_ref().unwrap();
    let simple_cstr = CString::new("Simple").unwrap();
    let reference_cstr = CString::new("Reference").unwrap();
    (api.godot_nativescript_register_class)(
        handle,
        simple_cstr.as_ptr(),
        reference_cstr.as_ptr(),
        sys::godot_instance_create_func {
            create_func: Some(simple_constructor),
            method_data: ptr::null_mut(),
            free_func: None,
        },
        sys::godot_instance_destroy_func {
            destroy_func: Some(simple_destructor),
            method_data: ptr::null_mut(),
            free_func: None,
        },
    );
    let set_string_cstr = CString::new("set_string").unwrap();
    (api.godot_nativescript_register_method)(
        handle,
        simple_cstr.as_ptr(),
        set_string_cstr.as_ptr(),
        sys::godot_method_attributes {
            rpc_type: sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_DISABLED,
        },
        sys::godot_instance_method {
            method: Some(set_string),
            method_data: ptr::null_mut(),
            free_func: None,
        },
    );
    let get_string_cstr = CString::new("get_string").unwrap();
    (api.godot_nativescript_register_method)(
        handle,
        simple_cstr.as_ptr(),
        get_string_cstr.as_ptr(),
        sys::godot_method_attributes {
            rpc_type: sys::godot_method_rpc_mode_GODOT_METHOD_RPC_MODE_DISABLED,
        },
        sys::godot_instance_method {
            method: Some(get_string),
            method_data: ptr::null_mut(),
            free_func: None,
        },
    );
}

pub unsafe extern "C" fn simple_constructor(
    _instance: *mut sys::godot_object,
    _method_data: *mut c_void,
) -> *mut c_void {
    println!("FROM RUST: constructor");
    let api = API.as_ref().unwrap();
    let rust_string = String::from("Default String");
    let user_data = (api.godot_alloc)(size_of_val(&rust_string) as i32);
    (user_data as *mut String).copy_from(&rust_string, 1);
    forget(rust_string);
    user_data
}

pub unsafe extern "C" fn simple_destructor(
    _instance: *mut sys::godot_object,
    _method_data: *mut c_void,
    user_data: *mut c_void,
) {
    println!("FROM RUST: destructor");
    let api = API.as_ref().unwrap();
    ptr::drop_in_place(user_data as *mut String);
    (api.godot_free)(user_data);
}

pub unsafe extern "C" fn set_string(
    _instance: *mut sys::godot_object,
    _method_data: *mut c_void,
    user_data: *mut c_void,
    num_args: c_int,
    args: *mut *mut sys::godot_variant,
) -> sys::godot_variant {
    println!("FROM RUST: set_string");
    let api = API.as_ref().unwrap();
    assert_eq!(num_args, 1);
    let arg_1 = **args;
    assert_eq!(
        (api.godot_variant_get_type)(&arg_1),
        sys::godot_variant_type_GODOT_VARIANT_TYPE_STRING
    );
    let godot_string = (api.godot_variant_as_string)(&arg_1);
    let godot_char_string = (api.godot_string_utf8)(&godot_string);
    let godot_string_data = (api.godot_char_string_get_data)(&godot_char_string);
    let rust_string = CStr::from_ptr(godot_string_data)
        .to_str()
        .unwrap()
        .to_string();
    ptr::drop_in_place(user_data as *mut String);
    (user_data as *mut String).copy_from(&rust_string, 1);
    forget(rust_string);
    let mut result_variant = MaybeUninit::<sys::godot_variant>::uninit();
    (api.godot_variant_new_nil)(result_variant.as_mut_ptr());
    result_variant.assume_init()
}

pub unsafe extern "C" fn get_string(
    _instance: *mut sys::godot_object,
    _method_data: *mut c_void,
    user_data: *mut c_void,
    num_args: c_int,
    _args: *mut *mut sys::godot_variant,
) -> sys::godot_variant {
    println!("FROM RUST: get_string");
    let api = API.as_ref().unwrap();
    assert_eq!(num_args, 0);
    let rust_string = (*(user_data as *const String)).as_str();
    let rust_cstring = CString::new(rust_string).unwrap();
    let mut godot_string = MaybeUninit::<sys::godot_string>::uninit();
    (api.godot_string_new)(godot_string.as_mut_ptr());
    (api.godot_string_parse_utf8)(godot_string.as_mut_ptr(), rust_cstring.as_ptr());
    let mut result_variant = MaybeUninit::<sys::godot_variant>::uninit();
    (api.godot_variant_new_string)(result_variant.as_mut_ptr(), godot_string.as_mut_ptr());
    (api.godot_string_destroy)(godot_string.as_mut_ptr());
    result_variant.assume_init()
}
