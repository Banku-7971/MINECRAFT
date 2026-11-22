//! Win32 native platform implementation: window, GDI StretchDIBits display,
//! keyboard/mouse capture, clipboard, high-resolution timer, and WinMM audio.
#![allow(non_snake_case, non_camel_case_types, dead_code)]

use std::ffi::c_void;
use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::platform::{Event, Platform};
use crate::render::{Framebuffer, H, W};
use crate::sound::{Audio, Sfx, SAMPLE_RATE};

type HWND = *mut c_void;
type HDC = *mut c_void;
type HINSTANCE = *mut c_void;
type HICON = *mut c_void;
type HCURSOR = *mut c_void;
type HBRUSH = *mut c_void;
type HMENU = *mut c_void;
type HGLOBAL = *mut c_void;
type HWAVEOUT = *mut c_void;

type WndProc = unsafe extern "system" fn(HWND, u32, usize, isize) -> isize;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct POINT {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct RECT {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

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

#[repr(C)]
struct BITMAPINFOHEADER {
    biSize: u32,
    biWidth: i32,
    biHeight: i32,
    biPlanes: u16,
    biBitCount: u16,
    biCompression: u32,
    biSizeImage: u32,
    biXPelsPerMeter: i32,
    biYPelsPerMeter: i32,
    biClrUsed: u32,
    biClrImportant: u32,
}

#[repr(C)]
struct BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER,
    bmiColors: [u32; 3],
}

#[repr(C)]
struct WAVEFORMATEX {
    wFormatTag: u16,
    nChannels: u16,
    nSamplesPerSec: u32,
    nAvgBytesPerSec: u32,
    nBlockAlign: u16,
    wBitsPerSample: u16,
    cbSize: u16,
}

#[repr(C)]
struct WAVEHDR {
    lpData: *mut u8,
    dwBufferLength: u32,
    dwBytesRecorded: u32,
    dwUser: usize,
    dwFlags: u32,
    dwLoops: u32,
    lpNext: *mut WAVEHDR,
    reserved: usize,
}

const CS_OWNDC: u32 = 0x0020;
const CS_HREDRAW: u32 = 0x0002;
const CS_VREDRAW: u32 = 0x0001;
const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const WS_VISIBLE: u32 = 0x10000000;
const CW_USEDEFAULT: i32 = 0x80000000u32 as i32;
const GWLP_USERDATA: i32 = -21;
const PM_REMOVE: u32 = 0x0001;

const WM_DESTROY: u32 = 0x0002;
const WM_SIZE: u32 = 0x0005;
const WM_KILLFOCUS: u32 = 0x0008;
const WM_PAINT: u32 = 0x000F;
const WM_CLOSE: u32 = 0x0010;
const WM_QUIT: u32 = 0x0012;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_CHAR: u32 = 0x0102;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_RBUTTONDOWN: u32 = 0x0204;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_MBUTTONDOWN: u32 = 0x0207;
const WM_MBUTTONUP: u32 = 0x0208;
const WM_MOUSEWHEEL: u32 = 0x020A;

const IDC_ARROW: usize = 32512;
const IDI_APPLICATION: usize = 32512;
const COLOR_WINDOW: usize = 5;

const BI_RGB: u32 = 0;
const DIB_RGB_COLORS: u32 = 0;
const SRCCOPY: u32 = 0x00CC0020;
const COLORONCOLOR: i32 = 3;

const CF_UNICODETEXT: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;

const WAVE_MAPPER: u32 = 0xFFFFFFFF;
const WAVE_FORMAT_PCM: u16 = 1;
const WHDR_DONE: u32 = 0x00000001;

#[link(name = "kernel32")]
extern "system" {
    fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    fn QueryPerformanceCounter(lpPerformanceCount: *mut i64) -> i32;
    fn QueryPerformanceFrequency(lpFrequency: *mut i64) -> i32;
    fn Sleep(dwMilliseconds: u32);
    fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> HGLOBAL;
    fn GlobalLock(hMem: HGLOBAL) -> *mut c_void;
    fn GlobalUnlock(hMem: HGLOBAL) -> i32;
    fn GlobalSize(hMem: HGLOBAL) -> usize;
}

#[link(name = "user32")]
extern "system" {
    fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *mut c_void,
    ) -> HWND;
    fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: usize, lParam: isize) -> isize;
    fn PeekMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32) -> i32;
    fn TranslateMessage(lpMsg: *const MSG) -> i32;
    fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    fn PostQuitMessage(nExitCode: i32);
    fn DestroyWindow(hWnd: HWND) -> i32;
    fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
    fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> i32;
    fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> i32;
    fn AdjustWindowRect(lpRect: *mut RECT, dwStyle: u32, bMenu: i32) -> i32;
    fn ClientToScreen(hWnd: HWND, lpPoint: *mut POINT) -> i32;
    fn GetDC(hWnd: HWND) -> HDC;
    fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
    fn ValidateRect(hWnd: HWND, lpRect: *const RECT) -> i32;
    fn SetProcessDPIAware() -> i32;
    fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HCURSOR;
    fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    fn ShowCursor(bShow: i32) -> i32;
    fn SetCursorPos(X: i32, Y: i32) -> i32;
    fn SetCapture(hWnd: HWND) -> HWND;
    fn ReleaseCapture() -> i32;
    fn ClipCursor(lpRect: *const RECT) -> i32;
    fn OpenClipboard(hWndNewOwner: HWND) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn GetClipboardData(uFormat: u32) -> HGLOBAL;
    fn SetClipboardData(uFormat: u32, hMem: HGLOBAL) -> HGLOBAL;
}

#[link(name = "gdi32")]
extern "system" {
    fn SetStretchBltMode(hdc: HDC, mode: i32) -> i32;
    fn StretchDIBits(
        hdc: HDC,
        xDest: i32,
        yDest: i32,
        DestWidth: i32,
        DestHeight: i32,
        xSrc: i32,
        ySrc: i32,
        SrcWidth: i32,
        SrcHeight: i32,
        lpBits: *const c_void,
        lpbmi: *const BITMAPINFO,
        iUsage: u32,
        rop: u32,
    ) -> i32;
}

#[link(name = "winmm")]
extern "system" {
    fn waveOutOpen(
        phwo: *mut HWAVEOUT,
        uDeviceID: u32,
        pwfx: *const WAVEFORMATEX,
        dwCallback: usize,
        dwInstance: usize,
        fdwOpen: u32,
    ) -> u32;
    fn waveOutPrepareHeader(hwo: HWAVEOUT, pwh: *mut WAVEHDR, cbwh: u32) -> u32;
    fn waveOutUnprepareHeader(hwo: HWAVEOUT, pwh: *mut WAVEHDR, cbwh: u32) -> u32;
    fn waveOutWrite(hwo: HWAVEOUT, pwh: *mut WAVEHDR, cbwh: u32) -> u32;
    fn waveOutClose(hwo: HWAVEOUT) -> u32;
    fn waveOutReset(hwo: HWAVEOUT) -> u32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

struct WinState {
    events: Vec<Event>,
    mouse: (i32, i32),
    client_w: i32,
    client_h: i32,
    captured: bool,
    ignore_next_move: bool,
    quit: bool,
}

impl WinState {
    fn client_to_fb(&self, x: i32, y: i32) -> (i32, i32) {
        let cw = self.client_w.max(1);
        let ch = self.client_h.max(1);
        let fx = (x * W as i32 / cw).clamp(0, W as i32 - 1);
        let fy = (y * H as i32 / ch).clamp(0, H as i32 - 1);
        (fx, fy)
    }

    fn center_cursor(&mut self, hwnd: HWND) {
        let mut pt = POINT {
            x: self.client_w / 2,
            y: self.client_h / 2,
        };
        unsafe {
            ClientToScreen(hwnd, &mut pt);
            self.ignore_next_move = true;
            SetCursorPos(pt.x, pt.y);
        }
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize {
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut WinState;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, msg, wparam, lparam);
    }
    let st = &mut *ptr;

    match msg {
        WM_CLOSE => {
            st.quit = true;
            st.events.push(Event::Close);
            DestroyWindow(hwnd);
            0
        }
        WM_DESTROY => {
            st.quit = true;
            PostQuitMessage(0);
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => {
            ValidateRect(hwnd, null());
            0
        }
        WM_SIZE => {
            let cw = (lparam & 0xFFFF) as i16 as i32;
            let ch = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            if cw > 0 && ch > 0 {
                st.client_w = cw;
                st.client_h = ch;
                st.events.push(Event::Resize(cw, ch));
            }
            0
        }
        WM_KILLFOCUS => {
            st.events.push(Event::FocusLost);
            if st.captured {
                st.captured = false;
                ClipCursor(null());
                ReleaseCapture();
            }
            0
        }
        WM_KEYDOWN | WM_SYSKEYDOWN => {
            let repeat = (lparam & (1 << 30)) != 0;
            st.events.push(Event::Key { code: wparam as u32, down: true, repeat });
            if msg == WM_SYSKEYDOWN && wparam == 0x73 {
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            }
            0
        }
        WM_KEYUP | WM_SYSKEYUP => {
            st.events.push(Event::Key { code: wparam as u32, down: false, repeat: false });
            0
        }
        WM_CHAR => {
            if let Some(ch) = char::from_u32(wparam as u32) {
                if !ch.is_control() {
                    st.events.push(Event::Char(ch));
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            let x = (lparam & 0xFFFF) as i16 as i32;
            let y = ((lparam >> 16) & 0xFFFF) as i16 as i32;
            st.mouse = st.client_to_fb(x, y);
            if st.captured {
                if st.ignore_next_move {
                    st.ignore_next_move = false;
                } else {
                    let cx = st.client_w / 2;
                    let cy = st.client_h / 2;
                    let dx = x - cx;
                    let dy = y - cy;
                    if dx != 0 || dy != 0 {
                        st.events.push(Event::MouseMove { dx, dy });
                        st.center_cursor(hwnd);
                    }
                }
            } else {
                st.events.push(Event::MouseMove { dx: 0, dy: 0 });
            }
            0
        }
        WM_LBUTTONDOWN => {
            st.events.push(Event::MouseDown(0));
            0
        }
        WM_LBUTTONUP => {
            st.events.push(Event::MouseUp(0));
            0
        }
        WM_RBUTTONDOWN => {
            st.events.push(Event::MouseDown(1));
            0
        }
        WM_RBUTTONUP => {
            st.events.push(Event::MouseUp(1));
            0
        }
        WM_MBUTTONDOWN => {
            st.events.push(Event::MouseDown(2));
            0
        }
        WM_MBUTTONUP => {
            st.events.push(Event::MouseUp(2));
            0
        }
        WM_MOUSEWHEEL => {
            let delta = ((wparam >> 16) & 0xFFFF) as i16 as i32;
            st.events.push(Event::Wheel(delta / 120));
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub struct Win32 {
    hwnd: HWND,
    state: Box<WinState>,
    bgra: Vec<u8>,
    freq: f64,
    start_qpc: i64,
    cursor_visible: bool,
    audio: Arc<Mutex<Audio>>,
    audio_stop: Arc<AtomicBool>,
    audio_thread: Option<std::thread::JoinHandle<()>>,
}

impl Win32 {
    pub fn new(title: &str) -> Self {
        let mut freq: i64 = 1;
        let mut start_qpc: i64 = 0;
        unsafe {
            QueryPerformanceFrequency(&mut freq);
            QueryPerformanceCounter(&mut start_qpc);
        }

        let init_w = (W * 2) as i32;
        let init_h = (H * 2) as i32;

        let mut state = Box::new(WinState {
            events: Vec::new(),
            mouse: (W as i32 / 2, H as i32 / 2),
            client_w: init_w,
            client_h: init_h,
            captured: false,
            ignore_next_move: false,
            quit: false,
        });

        let hwnd = unsafe {
            SetProcessDPIAware();
            let hinst = GetModuleHandleW(null());
            let cls = wide("MinecraftWindowClass");
            let wtitle = wide(title);
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: CS_OWNDC | CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: wnd_proc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinst,
                hIcon: LoadIconW(null_mut(), IDI_APPLICATION as *const u16),
                hCursor: LoadCursorW(null_mut(), IDC_ARROW as *const u16),
                hbrBackground: null_mut(),
                lpszMenuName: null(),
                lpszClassName: cls.as_ptr(),
                hIconSm: null_mut(),
            };
            RegisterClassExW(&wc);

            let mut rect = RECT { left: 0, top: 0, right: init_w, bottom: init_h };
            AdjustWindowRect(&mut rect, WS_OVERLAPPEDWINDOW, 0);
            let win_w = rect.right - rect.left;
            let win_h = rect.bottom - rect.top;

            let hwnd = CreateWindowExW(
                0,
                cls.as_ptr(),
                wtitle.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                win_w,
                win_h,
                null_mut(),
                null_mut(),
                hinst,
                null_mut(),
            );
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, (&mut *state as *mut WinState) as isize);
            let mut cr = RECT::default();
            GetClientRect(hwnd, &mut cr);
            if cr.right > 0 && cr.bottom > 0 {
                state.client_w = cr.right;
                state.client_h = cr.bottom;
            }
            hwnd
        };

        let audio = Arc::new(Mutex::new(Audio::new()));
        let audio_stop = Arc::new(AtomicBool::new(false));
        let a_clone = Arc::clone(&audio);
        let s_clone = Arc::clone(&audio_stop);
        let audio_thread = std::thread::spawn(move || audio_loop(a_clone, s_clone));

        Win32 {
            hwnd,
            state,
            bgra: vec![0u8; crate::render::OUT_W * crate::render::OUT_H * 4],
            freq: freq.max(1) as f64,
            start_qpc,
            cursor_visible: true,
            audio,
            audio_stop,
            audio_thread: Some(audio_thread),
        }
    }
}

impl Drop for Win32 {
    fn drop(&mut self) {
        self.audio_stop.store(true, Ordering::Relaxed);
        if let Some(th) = self.audio_thread.take() {
            let _ = th.join();
        }
        unsafe {
            ClipCursor(null());
            if !self.cursor_visible {
                ShowCursor(1);
            }
        }
    }
}

fn audio_loop(audio: Arc<Mutex<Audio>>, stop: Arc<AtomicBool>) {
    unsafe {
        let fmt = WAVEFORMATEX {
            wFormatTag: WAVE_FORMAT_PCM,
            nChannels: 1,
            nSamplesPerSec: SAMPLE_RATE,
            nAvgBytesPerSec: SAMPLE_RATE * 2,
            nBlockAlign: 2,
            wBitsPerSample: 16,
            cbSize: 0,
        };
        let mut hwo: HWAVEOUT = null_mut();
        if waveOutOpen(&mut hwo, WAVE_MAPPER, &fmt, 0, 0, 0) != 0 || hwo.is_null() {
            return;
        }

        const CHUNK: usize = 1024;
        const NBUFS: usize = 3;
        let mut pcm: Vec<Vec<i16>> = vec![vec![0i16; CHUNK]; NBUFS];
        let mut hdrs: Vec<WAVEHDR> = Vec::with_capacity(NBUFS);
        for i in 0..NBUFS {
            hdrs.push(WAVEHDR {
                lpData: pcm[i].as_mut_ptr() as *mut u8,
                dwBufferLength: (CHUNK * 2) as u32,
                dwBytesRecorded: 0,
                dwUser: 0,
                dwFlags: 0,
                dwLoops: 0,
                lpNext: null_mut(),
                reserved: 0,
            });
        }

        let sz = std::mem::size_of::<WAVEHDR>() as u32;
        for i in 0..NBUFS {
            if let Ok(mut a) = audio.lock() {
                a.render(&mut pcm[i]);
            }
            waveOutPrepareHeader(hwo, &mut hdrs[i], sz);
            waveOutWrite(hwo, &mut hdrs[i], sz);
        }

        while !stop.load(Ordering::Relaxed) {
            let mut any = false;
            for i in 0..NBUFS {
                if (hdrs[i].dwFlags & WHDR_DONE) != 0 {
                    any = true;
                    waveOutUnprepareHeader(hwo, &mut hdrs[i], sz);
                    if let Ok(mut a) = audio.lock() {
                        a.render(&mut pcm[i]);
                    }
                    hdrs[i].dwFlags = 0;
                    waveOutPrepareHeader(hwo, &mut hdrs[i], sz);
                    waveOutWrite(hwo, &mut hdrs[i], sz);
                }
            }
            if !any {
                Sleep(8);
            }
        }

        waveOutReset(hwo);
        for i in 0..NBUFS {
            waveOutUnprepareHeader(hwo, &mut hdrs[i], sz);
        }
        waveOutClose(hwo);
    }
}

impl Platform for Win32 {
    fn poll(&mut self) -> Vec<Event> {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    self.state.quit = true;
                    self.state.events.push(Event::Close);
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
        std::mem::take(&mut self.state.events)
    }

    fn present(&mut self, fb: &mut Framebuffer) {
        let px_count = (fb.w * fb.h).min(self.bgra.len() / 4);
        for i in 0..px_count {
            let s = i * 4;
            self.bgra[s] = fb.rgba[s + 2];
            self.bgra[s + 1] = fb.rgba[s + 1];
            self.bgra[s + 2] = fb.rgba[s];
            self.bgra[s + 3] = 255;
        }
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: fb.w as i32,
                biHeight: -(fb.h as i32), // top-down
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [0; 3],
        };
        unsafe {
            let hdc = GetDC(self.hwnd);
            if !hdc.is_null() {
                SetStretchBltMode(hdc, COLORONCOLOR);
                StretchDIBits(
                    hdc,
                    0,
                    0,
                    self.state.client_w.max(1),
                    self.state.client_h.max(1),
                    0,
                    0,
                    fb.w as i32,
                    fb.h as i32,
                    self.bgra.as_ptr() as *const c_void,
                    &bmi,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
                ReleaseDC(self.hwnd, hdc);
            }
        }
    }

    fn now(&self) -> f64 {
        let mut cur: i64 = 0;
        unsafe {
            QueryPerformanceCounter(&mut cur);
        }
        (cur - self.start_qpc) as f64 / self.freq
    }

    fn play(&mut self, s: Sfx) {
        if let Ok(mut a) = self.audio.lock() {
            a.play(s);
        }
    }

    fn set_music(&mut self, on: bool) {
        if let Ok(mut a) = self.audio.lock() {
            a.music_on = on;
        }
    }

    fn set_sound(&mut self, on: bool) {
        if let Ok(mut a) = self.audio.lock() {
            a.sound_on = on;
        }
    }

    fn set_dimension(&mut self, dim: u8) {
        if let Ok(mut a) = self.audio.lock() {
            a.set_dimension(dim);
        }
    }

    fn toggle_fullscreen(&mut self) {
        unsafe {
            const GWL_STYLE: i32 = -16;
            const SW_MAXIMIZE: i32 = 3;
            const SW_RESTORE: i32 = 9;
            let style = GetWindowLongPtrW(self.hwnd, GWL_STYLE) as u32;
            if (style & 0x00C00000) != 0 {
                // Enter borderless maximized fullscreen
                SetWindowLongPtrW(self.hwnd, GWL_STYLE, (style & !0x00CF0000) as isize);
                ShowWindow(self.hwnd, SW_MAXIMIZE);
            } else {
                SetWindowLongPtrW(self.hwnd, GWL_STYLE, (style | WS_OVERLAPPEDWINDOW) as isize);
                ShowWindow(self.hwnd, SW_RESTORE);
            }
        }
    }

    fn set_cursor(&mut self, visible: bool) {
        if visible == self.cursor_visible {
            return;
        }
        self.cursor_visible = visible;
        unsafe {
            ShowCursor(if visible { 1 } else { 0 });
        }
    }

    fn set_capture(&mut self, captured: bool) {
        if captured == self.state.captured {
            return;
        }
        self.state.captured = captured;
        unsafe {
            if captured {
                SetCapture(self.hwnd);
                let mut tl = POINT { x: 0, y: 0 };
                let mut br = POINT { x: self.state.client_w, y: self.state.client_h };
                ClientToScreen(self.hwnd, &mut tl);
                ClientToScreen(self.hwnd, &mut br);
                let clip = RECT { left: tl.x, top: tl.y, right: br.x, bottom: br.y };
                ClipCursor(&clip);
                self.state.center_cursor(self.hwnd);
            } else {
                ClipCursor(null());
                ReleaseCapture();
            }
        }
    }

    fn mouse(&self) -> (i32, i32) {
        self.state.mouse
    }

    fn clipboard(&mut self) -> Option<String> {
        unsafe {
            if OpenClipboard(self.hwnd) == 0 {
                return None;
            }
            let h = GetClipboardData(CF_UNICODETEXT);
            let mut out = None;
            if !h.is_null() {
                let bytes = GlobalSize(h);
                let ptr = GlobalLock(h) as *const u16;
                if !ptr.is_null() && bytes >= 2 {
                    let slice = std::slice::from_raw_parts(ptr, bytes / 2);
                    let len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                    out = Some(String::from_utf16_lossy(&slice[..len]));
                    GlobalUnlock(h);
                }
            }
            CloseClipboard();
            out
        }
    }

    fn set_clipboard(&mut self, text: &str) {
        let w = wide(text);
        let bytes = w.len() * 2;
        unsafe {
            if OpenClipboard(self.hwnd) == 0 {
                return;
            }
            EmptyClipboard();
            let h = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if !h.is_null() {
                let dst = GlobalLock(h) as *mut u16;
                if !dst.is_null() {
                    std::ptr::copy_nonoverlapping(w.as_ptr(), dst, w.len());
                    GlobalUnlock(h);
                    SetClipboardData(CF_UNICODETEXT, h);
                }
            }
            CloseClipboard();
        }
    }

    fn should_quit(&self) -> bool {
        self.state.quit
    }

    fn request_quit(&mut self) {
        self.state.quit = true;
    }

    fn set_title(&mut self, title: &str) {
        let w = wide(title);
        unsafe {
            SetWindowTextW(self.hwnd, w.as_ptr());
        }
    }

    fn sleep_ms(&mut self, ms: u64) {
        unsafe {
            Sleep(ms as u32);
        }
    }
}
