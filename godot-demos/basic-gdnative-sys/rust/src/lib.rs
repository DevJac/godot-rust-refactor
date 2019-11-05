use gdnative_sys as sys;
use std::mem::MaybeUninit;
use std::ptr;

static mut API: *const sys::godot_gdnative_core_api_struct = ptr::null();
static mut NS_API: *const sys::godot_gdnative_ext_nativescript_api_struct = ptr::null();

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
