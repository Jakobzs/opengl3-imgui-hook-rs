use anyhow::Result;
use std::{
    ffi::{c_void, CString},
    ptr,
    time::Instant,
};
use windows::Win32::{
    Foundation::{BOOL, HINSTANCE},
    System::SystemServices::DLL_PROCESS_ATTACH,
};

fn gl_get_proc_address(procname: &str) -> *const () {
    // For reference on what we do here: https://github.com/Rebzzel/kiero/blob/master/kiero.cpp#L519
    println!("Proc address: {}", procname);
    match CString::new(procname) {
        Ok(procname) => unsafe {
            // TODO: Get proc address and retrieve ptr to the function
            // sys::SDL_GL_GetProcAddress(procname.as_ptr() as *const c_char) as *const ()
            ptr::null()
        },
        // string contains a nul byte - it won't match anything.
        Err(_) => ptr::null(),
    }
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    _module: HINSTANCE,
    call_reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if call_reason == DLL_PROCESS_ATTACH {
        BOOL::from(main().is_ok())
    } else {
        BOOL::from(true)
    }
}

fn main() -> Result<()> {
    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let renderer =
        imgui_opengl_renderer::Renderer::new(&mut imgui, |s| gl_get_proc_address(s) as _);

    let mut last_frame = Instant::now();

    loop {
        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0;
        last_frame = now;
        imgui.io_mut().delta_time = delta_s;

        let ui = imgui.frame();
        ui.show_demo_window(&mut true);

        renderer.render(ui);

        ::std::thread::sleep(::std::time::Duration::new(0, 1_000_000_000u32 / 60));
    }
}
