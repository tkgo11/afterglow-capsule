//! SPIKE ONLY: read public test RCDATA from this module with native Windows APIs.

fn main() -> std::process::ExitCode {
    match read_resource() {
        Ok(bytes) => {
            use std::io::Write;
            if std::io::stdout().write_all(&bytes).is_ok() {
                std::process::ExitCode::SUCCESS
            } else {
                std::process::ExitCode::FAILURE
            }
        }
        Err(error) => {
            eprintln!("Spike C template: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn read_resource() -> Result<Vec<u8>, String> {
    Err("native Windows execution is required".into())
}

#[cfg(windows)]
fn read_resource() -> Result<Vec<u8>, String> {
    use windows_sys::Win32::{
        Media::KernelStreaming::RT_RCDATA,
        System::LibraryLoader::{
            FindResourceW, GetModuleHandleW, LoadResource, LockResource, SizeofResource,
        },
    };
    let name: Vec<u16> = "AGLOW_CAPSULE\0".encode_utf16().collect();
    let args: Vec<String> = std::env::args().collect();
    use windows_sys::Win32::UI::WindowsAndMessaging::{RT_GROUP_ICON, RT_ICON, RT_VERSION};
    let (resource_type, resource_name) = match args.get(1).map(String::as_str).unwrap_or("capsule")
    {
        "capsule" => (RT_RCDATA, name.as_ptr()),
        "icon" => (RT_ICON, std::ptr::without_provenance::<u16>(101)),
        "group-icon" => (RT_GROUP_ICON, std::ptr::without_provenance::<u16>(1)),
        "version" => (RT_VERSION, std::ptr::without_provenance::<u16>(1)),
        _ => return Err("unknown public test resource".into()),
    };
    // SAFETY: module lifetime is the process lifetime, name is NUL terminated,
    // resource handles are checked, and the borrowed bytes are copied while live.
    unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        if module.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let resource = FindResourceW(module, resource_name, resource_type);
        if resource.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let size = SizeofResource(module, resource) as usize;
        if size == 0 || size > 16384 {
            return Err("test resource size is invalid".into());
        }
        let loaded = LoadResource(module, resource);
        if loaded.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let pointer = LockResource(loaded);
        if pointer.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        Ok(std::slice::from_raw_parts(pointer.cast::<u8>(), size).to_vec())
    }
}
