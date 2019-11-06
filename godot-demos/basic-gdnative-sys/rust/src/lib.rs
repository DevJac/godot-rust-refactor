use gdnative_sys as sys;
use generational_arena::{Arena, Index};
use std::ffi::{c_void, CStr, CString};
use std::mem::{size_of_val, MaybeUninit};
use std::os::raw::c_int;
use std::ptr;

static mut API: *const sys::godot_gdnative_core_api_struct = ptr::null();
static mut NS_API: *const sys::godot_gdnative_ext_nativescript_api_struct = ptr::null();
static mut ARENA: Option<Arena<String>> = None;

#[no_mangle]
pub unsafe extern "C" fn godot_gdnative_init(options: *const sys::godot_gdnative_init_options) {
    println!("FROM RUST: init called");
    API = (*options).api_struct;
    for i in 0..((*API).num_extensions as isize) {
        if let sys::GDNATIVE_API_TYPES_GDNATIVE_EXT_NATIVESCRIPT =
            (**(*API).extensions.offset(i)).type_
        {
            NS_API = *(*API).extensions.offset(i)
                as *const sys::godot_gdnative_ext_nativescript_api_struct;
            return;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn godot_gdnative_terminate(
    _options: *const sys::godot_gdnative_terminate_options,
) {
    println!("FROM RUST: terminate called");
    API = ptr::null();
    NS_API = ptr::null();
}

#[no_mangle]
pub unsafe extern "C" fn add_42(args: *const sys::godot_array) -> sys::godot_variant {
    println!("FROM RUST: add_42 called");
    assert_eq!((*API).godot_array_size.unwrap()(args), 1);
    let godot_variant = (*API).godot_array_get.unwrap()(args, 0);
    assert_eq!(
        (*API).godot_variant_get_type.unwrap()(&godot_variant),
        sys::godot_variant_type_GODOT_VARIANT_TYPE_INT
    );
    let n = (*API).godot_variant_as_int.unwrap()(&godot_variant);
    let result = n + 42;
    let mut result_variant = MaybeUninit::<sys::godot_variant>::uninit();
    (*API).godot_variant_new_int.unwrap()(result_variant.as_mut_ptr(), result);
    result_variant.assume_init()
}

#[no_mangle]
pub unsafe extern "C" fn godot_nativescript_init(handle: *mut c_void) {
    println!("FROM RUST: ns init");
    ARENA = Some(Arena::new());
    let simple_cstr = CString::new("Simple").unwrap();
    let reference_cstr = CString::new("Reference").unwrap();
    (*NS_API).godot_nativescript_register_class.unwrap()(
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
    (*NS_API).godot_nativescript_register_method.unwrap()(
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
    (*NS_API).godot_nativescript_register_method.unwrap()(
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
    let arena_index = ARENA
        .as_mut()
        .unwrap()
        .insert(String::from("Default String"));
    let user_data = (*API).godot_alloc.unwrap()(size_of_val(&arena_index) as i32);
    (user_data as *mut Index).copy_from(&arena_index, 1);
    user_data
}

pub unsafe extern "C" fn simple_destructor(
    _instance: *mut sys::godot_object,
    _method_data: *mut c_void,
    user_data: *mut c_void,
) {
    println!("FROM RUST: destructor");
    let arena_index = *(user_data as *const Index);
    ARENA.as_mut().unwrap().remove(arena_index);
    (*API).godot_free.unwrap()(user_data);
}

pub unsafe extern "C" fn set_string(
    _instance: *mut sys::godot_object,
    _method_data: *mut c_void,
    user_data: *mut c_void,
    num_args: c_int,
    args: *mut *mut sys::godot_variant,
) -> sys::godot_variant {
    println!("FROM RUST: set_string");
    assert_eq!(num_args, 1);
    let arg_1 = **args;
    assert_eq!(
        (*API).godot_variant_get_type.unwrap()(&arg_1),
        sys::godot_variant_type_GODOT_VARIANT_TYPE_STRING
    );
    let godot_string = (*API).godot_variant_as_string.unwrap()(&arg_1);
    let godot_char_string = (*API).godot_string_utf8.unwrap()(&godot_string);
    let godot_string_data = (*API).godot_char_string_get_data.unwrap()(&godot_char_string);
    let arena_index = *(user_data as *const Index);
    ARENA.as_mut().unwrap()[arena_index] = CStr::from_ptr(godot_string_data)
        .to_str()
        .unwrap()
        .to_string();
    let mut result_variant = MaybeUninit::<sys::godot_variant>::uninit();
    (*API).godot_variant_new_nil.unwrap()(result_variant.as_mut_ptr());
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
    assert_eq!(num_args, 0);
    let arena_index = *(user_data as *const Index);
    let rust_string = ARENA.as_mut().unwrap()[arena_index].as_str();
    let rust_cstring = CString::new(rust_string).unwrap();
    let mut godot_string = MaybeUninit::<sys::godot_string>::uninit();
    (*API).godot_string_new.unwrap()(godot_string.as_mut_ptr());
    (*API).godot_string_parse_utf8.unwrap()(godot_string.as_mut_ptr(), rust_cstring.as_ptr());
    let mut result_variant = MaybeUninit::<sys::godot_variant>::uninit();
    (*API).godot_variant_new_string.unwrap()(
        result_variant.as_mut_ptr(),
        godot_string.as_mut_ptr(),
    );
    (*API).godot_string_destroy.unwrap()(godot_string.as_mut_ptr());
    result_variant.assume_init()
}
