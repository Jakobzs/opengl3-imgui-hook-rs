use anyhow::{anyhow, Result};
use detour::static_detour;
use imgui::Context;
use imgui_opengl_renderer::Renderer;
use std::{
    ffi::{c_int, c_void, CString},
    mem, ptr,
    time::{Duration, Instant},
};
use windows::{
    core::PCSTR,
    Win32::{
        Foundation::{GetLastError, BOOL, HINSTANCE},
        Graphics::Gdi::HDC,
        System::{
            Console::AllocConsole,
            LibraryLoader::{GetModuleHandleA, GetProcAddress},
            SystemServices::DLL_PROCESS_ATTACH,
        },
    },
};

fn gl_get_proc_address(procname: &str) -> *const () {
    // For reference on what we do here: https://github.com/Rebzzel/kiero/blob/master/kiero.cpp#L519

    println!("Proc address: {}", procname);
    match CString::new(procname) {
        Ok(procnamer) => {
            let aweqawe = gl_loader::get_proc_address(procname) as *const ();

            if aweqawe == ptr::null() {
                println!("Got a null here buddy");

                loop {}
            }

            aweqawe
        }
        // string contains a null byte - it won't match anything.
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

fn create_debug_console() -> Result<()> {
    if !unsafe { AllocConsole() }.as_bool() {
        return Err(anyhow!(
            "Failed allocating console, GetLastError: {}",
            unsafe { GetLastError() }.0
        ));
    }

    Ok(())
}

fn get_module_library(
    module: &str,
    function: &str,
) -> Result<unsafe extern "system" fn() -> isize> {
    let module_cstring = CString::new(module).expect("module");
    let function_cstring = CString::new(function).expect("function");

    let h_instance = unsafe { GetModuleHandleA(PCSTR(module_cstring.as_ptr() as *mut _)) }?;

    let func = unsafe { GetProcAddress(h_instance, PCSTR(function_cstring.as_ptr() as *mut _)) };

    match func {
        Some(func) => Ok(func),
        None => Err(anyhow!(
            "Failed GetProcAddress, GetLastError: {}",
            unsafe { GetLastError() }.0
        )),
    }
}

static_detour! {
  pub static OpenGl32wglSwapBuffers: unsafe extern "system" fn(HDC) -> ();
}

static mut INIT: bool = false;
static mut IMGUI: Option<Context> = None;
static mut IMGUI_RENDERER: Option<Renderer> = None;

#[allow(non_snake_case)]
pub fn wglSwapBuffers_detour(dc: HDC) -> () {
    println!("Called wglSwapBuffers");

    if !unsafe { INIT } {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        imgui.style_mut().window_title_align = [0.5, 0.5];
        imgui.io_mut().display_size = [1024.0, 1024.0];

        // Init the loader (grabbing the func required)
        gl_loader::init_gl();
        // Create the renderer
        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
            gl_loader::get_proc_address(s) as _
        });

        unsafe { IMGUI = Some(imgui) };
        unsafe { IMGUI_RENDERER = Some(renderer) };

        unsafe { INIT = true };
    }

    if unsafe { INIT } {
        let imgui = unsafe { &mut IMGUI }.as_mut().unwrap();
        let ui = imgui.frame();
        ui.show_demo_window(&mut true);

        let rendererer = unsafe { &mut IMGUI_RENDERER }.as_mut().unwrap();
        rendererer.render(ui);
    }

    println!("INIT: {}", unsafe { INIT });

    /*let mut imgui = imgui::Context::create();
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
    }*/

    unsafe { OpenGl32wglSwapBuffers.call(dc) }
}

pub type FnOpenGl32wglSwapBuffers = unsafe extern "system" fn(HDC) -> ();

fn main() -> Result<()> {
    create_debug_console()?;
    println!("Created debug console");

    let x = get_module_library("opengl32.dll", "wglSwapBuffers")?;
    let y: FnOpenGl32wglSwapBuffers = unsafe { mem::transmute(x) };
    unsafe { OpenGl32wglSwapBuffers.initialize(y, wglSwapBuffers_detour) }?;
    println!("Initialized detour");

    unsafe { OpenGl32wglSwapBuffers.enable() }?;
    println!("Enabled detour");

    Ok(())
}
