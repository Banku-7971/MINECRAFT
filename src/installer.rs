//! Native Windows installer for Minecraft (written in Rust).
//! Embeds Minecraft.exe and installs it to %LOCALAPPDATA%\Minecraft with
//! Desktop & Start Menu shortcuts plus Windows Apps & Features registration.
#![cfg_attr(windows, windows_subsystem = "windows")]
#![allow(non_snake_case, non_camel_case_types)]

use std::ffi::c_void;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::ptr::{null, null_mut};

static GAME_EXE: &[u8] = include_bytes!("../runtime/Minecraft.exe");

type HWND = *mut c_void;
type HINSTANCE = *mut c_void;
type HICON = *mut c_void;
type HCURSOR = *mut c_void;
type HBRUSH = *mut c_void;
type HMENU = *mut c_void;
type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

#[repr(C)]
struct POINT { x: i32, y: i32 }

#[repr(C)]
struct MSG {
    hwnd: HWND,
    message: u32,
    wParam: usize,
    lParam: isize,
    time: u32,
    pt: POINT,
}

#[repr(C)]
struct WNDCLASSEXW {
    cbSize: u32,
    style: u32,
    lpfnWndProc: WndProc,
    cbClsExtra: i32,
    cbWndExtra: i32,
    hInstance: HINSTANCE,
    hIcon: HICON,
    hCursor: HCURSOR,
    hbrBackground: HBRUSH,
    lpszMenuName: *const u16,
    lpszClassName: *const u16,
    hIconSm: HICON,
}

const WS_OVERLAPPED: u32 = 0x00000000;
const WS_CAPTION: u32 = 0x00C00000;
const WS_SYSMENU: u32 = 0x00080000;
const WS_MINIMIZEBOX: u32 = 0x00020000;
const WS_VISIBLE: u32 = 0x10000000;
const WS_CHILD: u32 = 0x40000000;
const BS_DEFPUSHBUTTON: u32 = 0x0001;
const BS_PUSHBUTTON: u32 = 0x0000;
const BS_AUTOCHECKBOX: u32 = 0x0003;
const BM_GETCHECK: u32 = 0x00F0;
const BM_SETCHECK: u32 = 0x00F1;
const BST_CHECKED: usize = 1;

const WM_CREATE: u32 = 0x0001;
const WM_DESTROY: u32 = 0x0002;
const WM_CLOSE: u32 = 0x0010;
const WM_COMMAND: u32 = 0x0111;
const WM_SETFONT: u32 = 0x0030;

const IDC_ARROW: usize = 32512;
const IDI_APPLICATION: usize = 32512;
const COLOR_BTNFACE: usize = 15;
const DEFAULT_GUI_FONT: i32 = 17;

const MB_OK: u32 = 0x00000000;
const MB_YESNO: u32 = 0x00000004;
const MB_ICONINFO: u32 = 0x00000040;
const MB_ICONERROR: u32 = 0x00000010;
const IDYES: i32 = 6;

const CREATE_NO_WINDOW: u32 = 0x08000000;

const ID_INSTALL: usize = 101;
const ID_CANCEL: usize = 102;
const ID_CHK_DESKTOP: usize = 103;
const ID_CHK_START: usize = 104;

static mut CHK_DESKTOP: HWND = null_mut();
static mut CHK_START: HWND = null_mut();
static mut LBL_STATUS: HWND = null_mut();

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetStockObject(i: i32) -> *mut c_void;
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32, lpClassName: *const u16, lpWindowName: *const u16, dwStyle: u32,
        X: i32, Y: i32, nWidth: i32, nHeight: i32,
        hWndParent: HWND, hMenu: HMENU, hInstance: HINSTANCE, lpParam: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn SendMessageW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn PostQuitMessage(nExitCode: i32);
    fn DestroyWindow(hWnd: HWND) -> i32;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HCURSOR;
    fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> i32;
    fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
    fn GetSystemMetrics(nIndex: i32) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn msgbox(hwnd: HWND, text: &str, title: &str, flags: u32) -> i32 {
    unsafe { MessageBoxW(hwnd, wide(text).as_ptr(), wide(title).as_ptr(), flags) }
}

fn install_dir() -> PathBuf {
    if let Ok(lad) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(lad).join("Minecraft")
    } else if let Ok(up) = std::env::var("USERPROFILE") {
        PathBuf::from(up).join("AppData").join("Local").join("Minecraft")
    } else {
        PathBuf::from("C:\\Minecraft")
    }
}

fn create_shortcut(lnk_path: &Path, target_exe: &Path, work_dir: &Path) {
    let ps = format!(
        "$s=(New-Object -COM WScript.Shell).CreateShortcut('{}');$s.TargetPath='{}';$s.WorkingDirectory='{}';$s.Description='Minecraft (Rust Edition)';$s.Save()",
        lnk_path.display().to_string().replace('\'', "''"),
        target_exe.display().to_string().replace('\'', "''"),
        work_dir.display().to_string().replace('\'', "''"),
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command", &ps])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
}

fn register_uninstall(dir: &Path, exe: &Path) {
    let key = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Minecraft";
    let uninst_cmd = format!("\"{}\" --uninstall", exe.display());
    for (name, val) in [
        ("DisplayName", "Minecraft"),
        ("Publisher", "Minecraft Rust Edition"),
        ("DisplayVersion", "1.0.0"),
        ("InstallLocation", &dir.display().to_string()),
        ("DisplayIcon", &exe.display().to_string()),
        ("UninstallString", &uninst_cmd),
    ] {
        let _ = Command::new("reg")
            .args(["add", key, "/v", name, "/t", "REG_SZ", "/d", val, "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .status();
    }
}

fn do_uninstall() {
    let dir = install_dir();
    if let Ok(up) = std::env::var("USERPROFILE") {
        let _ = std::fs::remove_file(PathBuf::from(&up).join("Desktop").join("Minecraft.lnk"));
    }
    if let Ok(ad) = std::env::var("APPDATA") {
        let _ = std::fs::remove_file(
            PathBuf::from(&ad).join("Microsoft").join("Windows").join("Start Menu").join("Programs").join("Minecraft.lnk"),
        );
    }
    let _ = std::fs::remove_file(dir.join("Minecraft.exe"));
    let _ = Command::new("reg")
        .args(["delete", r"HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Minecraft", "/f"])
        .creation_flags(CREATE_NO_WINDOW)
        .status();
    msgbox(null_mut(), "Minecraft has been uninstalled.", "Minecraft Setup", MB_OK | MB_ICONINFO);
}

fn run_install(hwnd: HWND, desktop: bool, start_menu: bool) {
    unsafe {
        if !LBL_STATUS.is_null() {
            SetWindowTextW(LBL_STATUS, wide("Installing Minecraft.exe...").as_ptr());
        }
    }
    let dir = install_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        msgbox(hwnd, &format!("Could not create folder {}:\n{}", dir.display(), e), "Install Error", MB_OK | MB_ICONERROR);
        return;
    }
    let exe_path = dir.join("Minecraft.exe");
    if let Err(e) = std::fs::write(&exe_path, GAME_EXE) {
        msgbox(hwnd, &format!("Could not write {}:\n{}", exe_path.display(), e), "Install Error", MB_OK | MB_ICONERROR);
        return;
    }
    let setup_copy = dir.join("Minecraft-Setup.exe");
    if let Ok(self_exe) = std::env::current_exe() {
        let _ = std::fs::copy(&self_exe, &setup_copy);
    }

    if desktop {
        if let Ok(up) = std::env::var("USERPROFILE") {
            let lnk = PathBuf::from(up).join("Desktop").join("Minecraft.lnk");
            create_shortcut(&lnk, &exe_path, &dir);
        }
    }
    if start_menu {
        if let Ok(ad) = std::env::var("APPDATA") {
            let lnk = PathBuf::from(ad)
                .join("Microsoft")
                .join("Windows")
                .join("Start Menu")
                .join("Programs")
                .join("Minecraft.lnk");
            create_shortcut(&lnk, &exe_path, &dir);
        }
    }

    let uninst_target = if setup_copy.exists() { &setup_copy } else { &exe_path };
    register_uninstall(&dir, uninst_target);

    unsafe {
        if !LBL_STATUS.is_null() {
            SetWindowTextW(LBL_STATUS, wide("Installation complete!").as_ptr());
        }
    }

    let ans = msgbox(
        hwnd,
        &format!(
            "Minecraft has been installed to:\n{}\n\nLaunch Minecraft now?",
            exe_path.display()
        ),
        "Minecraft Setup",
        MB_YESNO | MB_ICONINFO,
    );
    if ans == IDYES {
        let _ = Command::new(&exe_path).current_dir(&dir).spawn();
    }
    unsafe { DestroyWindow(hwnd); }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    match msg {
        WM_CREATE => {
            let hinst = GetModuleHandleW(null());
            let font = GetStockObject(DEFAULT_GUI_FONT) as usize;
            let static_cls = wide("STATIC");
            let btn_cls = wide("BUTTON");

            let dir = install_dir();
            let header = format!(
                "Welcome to Minecraft Setup\r\n\r\nThis will install Minecraft (written in Rust) with crafting, recipes, furnace, redstone, music, sound effects, and LAN multiplayer.\r\n\r\nInstall folder:\r\n{}",
                dir.display()
            );

            let lbl = CreateWindowExW(
                0, static_cls.as_ptr(), wide(&header).as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 16, 400, 105,
                hwnd, null_mut(), hinst, null_mut(),
            );
            SendMessageW(lbl, WM_SETFONT, font, 1);

            CHK_DESKTOP = CreateWindowExW(
                0, btn_cls.as_ptr(), wide("Create Desktop shortcut").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                20, 130, 260, 22,
                hwnd, ID_CHK_DESKTOP as HMENU, hinst, null_mut(),
            );
            SendMessageW(CHK_DESKTOP, WM_SETFONT, font, 1);
            SendMessageW(CHK_DESKTOP, BM_SETCHECK, BST_CHECKED, 0);

            CHK_START = CreateWindowExW(
                0, btn_cls.as_ptr(), wide("Create Start Menu shortcut").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_AUTOCHECKBOX,
                20, 156, 260, 22,
                hwnd, ID_CHK_START as HMENU, hinst, null_mut(),
            );
            SendMessageW(CHK_START, WM_SETFONT, font, 1);
            SendMessageW(CHK_START, BM_SETCHECK, BST_CHECKED, 0);

            LBL_STATUS = CreateWindowExW(
                0, static_cls.as_ptr(), wide("Ready to install.").as_ptr(),
                WS_CHILD | WS_VISIBLE,
                20, 190, 400, 20,
                hwnd, null_mut(), hinst, null_mut(),
            );
            SendMessageW(LBL_STATUS, WM_SETFONT, font, 1);

            let b_inst = CreateWindowExW(
                0, btn_cls.as_ptr(), wide("Install").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_DEFPUSHBUTTON,
                220, 222, 95, 28,
                hwnd, ID_INSTALL as HMENU, hinst, null_mut(),
            );
            SendMessageW(b_inst, WM_SETFONT, font, 1);

            let b_cancel = CreateWindowExW(
                0, btn_cls.as_ptr(), wide("Cancel").as_ptr(),
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                325, 222, 95, 28,
                hwnd, ID_CANCEL as HMENU, hinst, null_mut(),
            );
            SendMessageW(b_cancel, WM_SETFONT, font, 1);
            0
        }
        WM_COMMAND => {
            let id = wparam & 0xFFFF;
            if id == ID_INSTALL {
                let d = SendMessageW(CHK_DESKTOP, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                let s = SendMessageW(CHK_START, BM_GETCHECK, 0, 0) == BST_CHECKED as isize;
                run_install(hwnd, d, s);
                return 0;
            }
            if id == ID_CANCEL {
                DestroyWindow(hwnd);
                return 0;
            }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        WM_CLOSE => {
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn main() {
    if std::env::args().any(|a| a == "--uninstall") {
        do_uninstall();
        return;
    }
    unsafe {
        let hinst = GetModuleHandleW(null());
        let cls = wide("MinecraftSetupClass");
        let title = wide("Minecraft Setup");
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: wnd_proc,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinst,
            hIcon: LoadIconW(null_mut(), IDI_APPLICATION as *const u16),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW as *const u16),
            hbrBackground: (COLOR_BTNFACE + 1) as HBRUSH,
            lpszMenuName: null(),
            lpszClassName: cls.as_ptr(),
            hIconSm: null_mut(),
        };
        RegisterClassExW(&wc);

        let w = 450;
        let h = 296;
        let sx = (GetSystemMetrics(0) - w) / 2;
        let sy = (GetSystemMetrics(1) - h) / 2;

        let hwnd = CreateWindowExW(
            0,
            cls.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX | WS_VISIBLE,
            sx.max(40),
            sy.max(40),
            w,
            h,
            null_mut(),
            null_mut(),
            hinst,
            null_mut(),
        );
        let _ = hwnd;

        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
