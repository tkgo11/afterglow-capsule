//! SPIKE ONLY: all resource mutations belong to this separate creator-side tool.

fn main() -> std::process::ExitCode {
    match update_resources() {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Spike C injector: {error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(windows))]
fn update_resources() -> Result<(), String> {
    Err("native Windows execution is required".into())
}

#[cfg(windows)]
fn update_resources() -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::{
        Media::KernelStreaming::RT_RCDATA,
        System::LibraryLoader::{BeginUpdateResourceW, EndUpdateResourceW, UpdateResourceW},
        UI::WindowsAndMessaging::{RT_GROUP_ICON, RT_ICON, RT_VERSION},
    };
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.len() != 6 {
        return Err("usage: injector exe capsule icon group-icon version-info".into());
    }
    let path: Vec<u16> = args[1].encode_wide().chain(std::iter::once(0)).collect();
    let mut blobs = Vec::new();
    for source in &args[2..] {
        let bytes = std::fs::read(source).map_err(|e| e.to_string())?;
        if bytes.is_empty() || bytes.len() > 16384 {
            return Err("invalid test resource size".into());
        }
        blobs.push(bytes);
    }
    let capsule_name: Vec<u16> = "AGLOW_CAPSULE\0".encode_utf16().collect();
    let definitions = [
        (RT_RCDATA, capsule_name.as_ptr()),
        (RT_ICON, std::ptr::without_provenance::<u16>(101)),
        (RT_GROUP_ICON, std::ptr::without_provenance::<u16>(1)),
        (RT_VERSION, std::ptr::without_provenance::<u16>(1)),
    ];
    // SAFETY: the path/names are terminated; integer IDs use MAKEINTRESOURCE
    // representation; owned byte buffers remain live until EndUpdateResourceW.
    // Each mutation failure discards the update transaction.
    unsafe {
        let handle = BeginUpdateResourceW(path.as_ptr(), 0);
        if handle.is_null() {
            return Err(std::io::Error::last_os_error().to_string());
        }
        for ((resource_type, name), bytes) in definitions.into_iter().zip(&blobs) {
            if UpdateResourceW(
                handle,
                resource_type,
                name,
                0,
                bytes.as_ptr().cast(),
                bytes.len() as u32,
            ) == 0
            {
                let error = std::io::Error::last_os_error().to_string();
                EndUpdateResourceW(handle, 1);
                return Err(error);
            }
        }
        if EndUpdateResourceW(handle, 0) == 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    Ok(())
}
