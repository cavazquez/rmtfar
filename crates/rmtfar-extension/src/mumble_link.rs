//! Mumble Link shared memory writer.
//!
//! Writes the local player's position and orientation into Mumble's well-known
//! shared memory segment so Mumble applies 3D positional audio automatically.
//!
//! Reference: <https://www.mumble.info/documentation/developer/positional-audio/link-plugin/>

use rmtfar_protocol::PlayerState;

const LINKED_MEM_SIZE: usize = std::mem::size_of::<LinkedMem>();

#[repr(C)]
struct LinkedMem {
    ui_version: u32,
    ui_tick: u32,
    avatar_position: [f32; 3],
    avatar_front: [f32; 3],
    avatar_top: [f32; 3],
    name: [u16; 256],
    camera_position: [f32; 3],
    camera_front: [f32; 3],
    camera_top: [f32; 3],
    identity: [u16; 256],
    context_len: u32,
    context: [u8; 256],
    description: [u16; 2048],
}

pub struct MumbleLink {
    inner: Option<Inner>,
}

impl MumbleLink {
    pub fn new() -> Self {
        match Inner::open() {
            Ok(inner) => Self { inner: Some(inner) },
            Err(_) => Self { inner: None },
        }
    }

    pub fn update(&mut self, state: &PlayerState) {
        if self.inner.is_none() {
            if let Ok(i) = Inner::open() {
                self.inner = Some(i);
            }
        }
        if let Some(ref mut inner) = self.inner {
            inner.write(state);
        }
    }
}

// ---------------------------------------------------------------------------
// Windows implementation
// ---------------------------------------------------------------------------

#[cfg(windows)]
struct Inner {
    ptr: *mut LinkedMem,
}

#[cfg(windows)]
unsafe impl Send for Inner {}
#[cfg(windows)]
unsafe impl Sync for Inner {}

#[cfg(windows)]
impl Inner {
    fn open() -> anyhow::Result<Self> {
        use windows::Win32::System::Memory::{
            MapViewOfFile, OpenFileMappingW, FILE_MAP_ALL_ACCESS,
        };
        let name: Vec<u16> = "MumbleLink\0".encode_utf16().collect();
        let handle = unsafe {
            OpenFileMappingW(
                FILE_MAP_ALL_ACCESS.0,
                false,
                windows::core::PCWSTR(name.as_ptr()),
            )
            .map_err(|e| anyhow::anyhow!("OpenFileMappingW: {e}"))?
        };
        let view = unsafe { MapViewOfFile(handle, FILE_MAP_ALL_ACCESS, 0, 0, LINKED_MEM_SIZE) };
        if view.Value.is_null() {
            anyhow::bail!("MapViewOfFile returned null");
        }
        Ok(Self {
            ptr: view.Value as *mut LinkedMem,
        })
    }

    fn write(&mut self, state: &PlayerState) {
        unsafe { write_state(self.ptr, state) }
    }
}

// ---------------------------------------------------------------------------
// Linux / Unix implementation (for compilation; used by bridge in testing)
// ---------------------------------------------------------------------------

#[cfg(unix)]
struct Inner {
    ptr: *mut LinkedMem,
    size: usize,
}

#[cfg(unix)]
unsafe impl Send for Inner {}
#[cfg(unix)]
unsafe impl Sync for Inner {}

#[cfg(unix)]
impl Inner {
    fn open() -> anyhow::Result<Self> {
        use std::ffi::CString;

        let uid = unsafe { libc::getuid() };
        let name = CString::new(format!("/MumbleLink.{uid}"))?;

        let fd = unsafe {
            libc::shm_open(
                name.as_ptr(),
                libc::O_RDWR | libc::O_CREAT,
                (libc::S_IRUSR | libc::S_IWUSR) as libc::mode_t,
            )
        };
        if fd < 0 {
            anyhow::bail!("shm_open failed: {}", std::io::Error::last_os_error());
        }

        #[allow(clippy::cast_possible_wrap)]
        let ret = unsafe { libc::ftruncate(fd, LINKED_MEM_SIZE as libc::off_t) };
        if ret < 0 {
            unsafe { libc::close(fd) };
            anyhow::bail!("ftruncate failed: {}", std::io::Error::last_os_error());
        }

        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                LINKED_MEM_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            )
        };
        unsafe { libc::close(fd) };

        if ptr == libc::MAP_FAILED {
            anyhow::bail!("mmap failed: {}", std::io::Error::last_os_error());
        }

        Ok(Self {
            ptr: ptr.cast::<LinkedMem>(),
            size: LINKED_MEM_SIZE,
        })
    }

    fn write(&mut self, state: &PlayerState) {
        unsafe { write_state(self.ptr, state) }
    }
}

#[cfg(unix)]
impl Drop for Inner {
    fn drop(&mut self) {
        unsafe { libc::munmap(self.ptr.cast::<libc::c_void>(), self.size) };
    }
}

// ---------------------------------------------------------------------------
// Shared write logic
// ---------------------------------------------------------------------------

/// Coordinate conversion:
/// - Arma 3: X = east, Y = north, Z = altitude ASL
/// - Mumble: X = right (east), Y = up (altitude), Z = forward (north)
unsafe fn write_state(lm: *mut LinkedMem, state: &PlayerState) {
    let lm = &mut *lm;

    if lm.ui_version != 2 {
        write_wstr(&mut lm.name, "RMTFAR");
        write_wstr(&mut lm.description, "RMTFAR — Arma 3 radio mod for Mumble");
        lm.ui_version = 2;
    }
    lm.ui_tick = lm.ui_tick.wrapping_add(1);

    let [ax, ay, az] = state.pos;
    lm.avatar_position = [ax, az, ay];

    let front = state.front_vector();
    lm.avatar_front = front;
    lm.avatar_top = [0.0, 1.0, 0.0];

    lm.camera_position = lm.avatar_position;
    lm.camera_front = lm.avatar_front;
    lm.camera_top = lm.avatar_top;

    write_wstr(&mut lm.identity, &state.steam_id);

    let ctx = state.server_id.as_bytes();
    let ctx_len = ctx.len().min(lm.context.len());
    lm.context[..ctx_len].copy_from_slice(&ctx[..ctx_len]);
    #[allow(clippy::cast_possible_truncation)]
    {
        lm.context_len = ctx_len as u32;
    }
}

fn write_wstr(buf: &mut [u16], s: &str) {
    let capacity = buf.len().saturating_sub(1);
    let mut len = 0;
    for (i, c) in s.encode_utf16().enumerate() {
        if i >= capacity {
            break;
        }
        buf[i] = c;
        len += 1;
    }
    buf[len] = 0;
}
