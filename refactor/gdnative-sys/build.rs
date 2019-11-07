use std::env;
use std::path;

fn main() {
    let bindings = bindgen::builder()
        .header("../../godot_headers/gdnative_api_struct.gen.h")
        .clang_arg("-I../../godot_headers")
        .generate()
        .unwrap();
    let out_dir = path::PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir.join("bindings.rs")).unwrap();
    api_wrapper::generate(out_dir);
}

mod api_wrapper {
    use lazy_static::lazy_static;
    use regex::Regex;
    use std::fmt::Write as _;
    use std::fs::File;
    use std::io::Write;
    use std::path;

    #[derive(Debug, serde::Deserialize)]
    struct ApiCategories {
        core: Api,
        extensions: Vec<Api>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Api {
        name: Option<String>,
        #[serde(rename = "type")]
        type_: String,
        version: Version,
        next: Option<Box<Api>>,
        #[serde(rename = "api")]
        functions: Vec<Function>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Version {
        major: u32,
        minor: u32,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Function {
        name: String,
        return_type: String,
        arguments: Vec<Argument>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct Argument {
        type_: String,
        name: String,
    }

    impl ApiCategories {
        fn all_apis(&self) -> Vec<&Api> {
            let mut result = Vec::new();
            result.extend(self.core.nexts());
            for extension in &self.extensions {
                result.extend(extension.nexts());
            }
            result
        }
    }

    impl Api {
        fn nexts(&self) -> Vec<&Api> {
            let mut result = Vec::new();
            let mut api = self;
            loop {
                result.push(api);
                if api.next.is_none() {
                    break;
                }
                api = &api.next.as_ref().unwrap();
            }
            result
        }
    }

    impl Function {
        fn rust_args_with_types(&self) -> String {
            self.arguments
                .iter()
                .map(|arg| format!("{}: {}", arg.adjusted_name(), arg.rust_type()))
                .collect::<Vec<String>>()
                .join(",")
        }

        fn rust_args_without_types(&self) -> String {
            self.arguments
                .iter()
                .map(|arg| arg.adjusted_name())
                .collect::<Vec<String>>()
                .join(",")
        }

        fn rust_return_type(&self) -> String {
            c_type_to_rust_type(&self.return_type)
        }

        fn full_rust_type(&self) -> String {
            format!(
                r#"unsafe extern "C" fn({args}) -> {return_type}"#,
                args = self.rust_args_with_types(),
                return_type = self.rust_return_type()
            )
        }
    }

    impl Argument {
        fn adjusted_name(&self) -> String {
            match self.name.trim_start_matches("p_") {
                "self" => "self_".to_string(),
                arg_name => arg_name.to_string(),
            }
        }

        fn rust_type(&self) -> String {
            c_type_to_rust_type(&self.type_)
        }
    }

    fn godot_api_struct(type_: &str, version_major: u32, version_minor: u32) -> String {
        match (type_, version_major, version_minor) {
            ("CORE", 1, 0) => "godot_gdnative_core_api_struct".to_string(),
            ("CORE", maj, min) => format!("godot_gdnative_core_{}_{}_api_struct", maj, min),
            ("NATIVESCRIPT", 1, 0) => "godot_gdnative_ext_nativescript_api_struct".to_string(),
            ("NATIVESCRIPT", maj, min) => {
                format!("godot_gdnative_ext_nativescript_{}_{}_api_struct", maj, min)
            }
            ("PLUGINSCRIPT", 1, 0) => "godot_gdnative_ext_pluginscript_api_struct".to_string(),
            ("ANDROID", 1, 0) => "godot_gdnative_ext_android_api_struct".to_string(),
            ("ARVR", 1, 1) => "godot_gdnative_ext_arvr_api_struct".to_string(),
            ("VIDEODECODER", 0, 1) => "godot_gdnative_ext_videodecoder_api_struct".to_string(),
            ("NET", 3, 1) => "godot_gdnative_ext_net_api_struct".to_string(),
            api => panic!("Unknown API type and version: {:?}", api),
        }
    }

    fn godot_api_type_ident(type_: &str) -> String {
        match type_ {
            "CORE" => "GDNATIVE_API_TYPES_GDNATIVE_CORE".to_string(),
            "NATIVESCRIPT" => "GDNATIVE_API_TYPES_GDNATIVE_EXT_NATIVESCRIPT".to_string(),
            "PLUGINSCRIPT" => "GDNATIVE_API_TYPES_GDNATIVE_EXT_PLUGINSCRIPT".to_string(),
            "ANDROID" => "GDNATIVE_API_TYPES_GDNATIVE_EXT_ANDROID".to_string(),
            "ARVR" => "GDNATIVE_API_TYPES_GDNATIVE_EXT_ARVR".to_string(),
            "VIDEODECODER" => "GDNATIVE_API_TYPES_GDNATIVE_EXT_VIDEODECODER".to_string(),
            "NET" => "GDNATIVE_API_TYPES_GDNATIVE_EXT_NET".to_string(),
            other => panic!("Unknown API type: {:?}", other),
        }
    }

    pub fn generate(out_dir: path::PathBuf) {
        let api_json_file = File::open("../../godot_headers/gdnative_api.json").unwrap();
        let api_root: ApiCategories = serde_json::from_reader(api_json_file).unwrap();
        let mut wrapper_file = File::create(out_dir.join("api_wrapper.rs")).unwrap();
        writeln!(
            wrapper_file,
            "pub struct GodotApi{{{}}}",
            api_function_names_and_types(&api_root)
        )
        .unwrap();
        writeln!(
            wrapper_file,
            "impl GodotApi{{{api_constructor}{field_wrappers}}}",
            api_constructor = api_constructor(&api_root),
            field_wrappers = field_wrappers(&api_root)
        )
        .unwrap();
    }

    fn api_function_names_and_types(api: &ApiCategories) -> String {
        let mut result = String::new();
        for api in api.all_apis() {
            for function in &api.functions {
                writeln!(result, "{}: {},", function.name, function.full_rust_type()).unwrap();
            }
        }
        result
    }

    fn api_constructor(api: &ApiCategories) -> String {
        let mut result = String::new();
        writeln!(result, "pub unsafe fn from_api_struct(core_api_struct: *const godot_gdnative_core_api_struct) -> Self {{").unwrap();
        for api in api.all_apis() {
            writeln!(
                result,
                "let {}_{}_{} = find_api_ptr(core_api_struct, {}, {}, {}) as *const {};",
                api.type_,
                api.version.major,
                api.version.minor,
                godot_api_type_ident(&api.type_),
                api.version.major,
                api.version.minor,
                godot_api_struct(&api.type_, api.version.major, api.version.minor),
            )
            .unwrap();
        }
        writeln!(result, "GodotApi {{").unwrap();
        for api in api.all_apis() {
            for function in &api.functions {
                writeln!(
                    result,
                    r#"{}: (*{}_{}_{}).{}.expect("Couldn't find function: ({}, {}, {})"),"#,
                    function.name,
                    api.type_,
                    api.version.major,
                    api.version.minor,
                    function.name,
                    api.type_,
                    function.name,
                    godot_api_struct(&api.type_, api.version.major, api.version.minor),
                )
                .unwrap();
            }
        }
        writeln!(result, "}}}}").unwrap();
        result
    }

    fn field_wrappers(api: &ApiCategories) -> String {
        let mut result = String::new();
        for api in api.all_apis() {
            for function in &api.functions {
                writeln!(
                    result,
                    "pub unsafe fn {wrapper_name}(&self, {args}) -> {return_type} {{(self.{field_name})({arg_names})}}",
                    wrapper_name = function.name,
                    args = function.rust_args_with_types(),
                    return_type = function.rust_return_type(),
                    field_name = function.name,
                    arg_names = function.rust_args_without_types(),
                )
                .unwrap();
            }
        }
        result
    }

    fn c_type_to_rust_type(c_type: &str) -> String {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"(const )?\s*([\w\s]+?)\s*([\s\*]*)$").unwrap();
        }
        let caps = RE.captures(c_type).unwrap();
        let ptr_count = caps
            .get(3)
            .unwrap()
            .as_str()
            .chars()
            .filter(|c| c == &'*')
            .count();
        let rust_ptrs = match (ptr_count, caps.get(1).is_some()) {
            (0, _) => "",
            (1, true) => "*const ",
            (1, false) => "*mut ",
            (2, true) => "*mut *const ",
            _ => panic!("Unknown C type: {:?}", c_type),
        };
        let godot_raw_type = caps.get(2).unwrap().as_str();
        let rust_type = match godot_raw_type {
            "void" => {
                if ptr_count == 0 {
                    "()".to_string()
                } else {
                    "std::ffi::c_void".to_string()
                }
            }
            "bool" => "bool".to_string(),
            "uint8_t" => "u8".to_string(),
            "uint32_t" => "u32".to_string(),
            "uint64_t" => "u64".to_string(),
            "int64_t" => "i64".to_string(),
            "int" => "std::os::raw::c_int".to_string(),
            "double" => "std::os::raw::c_double".to_string(),
            "char" => "std::os::raw::c_char".to_string(),
            "signed char" => "std::os::raw::c_schar".to_string(),
            "size_t" => "usize".to_string(),
            "JNIEnv" => "std::ffi::c_void".to_string(),
            "jobject" => "*mut std::ffi::c_void".to_string(),
            raw_type => raw_type.to_string(),
        };
        format!("{}{}", rust_ptrs, rust_type)
    }
}
