#![no_std]
#![feature(used_with_arg)]

extern crate alloc;
#[macro_use]
extern crate log;
extern crate axklib;
extern crate rdrive;
extern crate rknpu;
extern crate rockchip_pm;
#[cfg(target_arch = "aarch64")]
extern crate axtask;

use core::sync::atomic::{AtomicBool, Ordering};

pub use core::ptr::NonNull;
pub use rknpu::{Rknpu, RknpuConfig, RknpuIrqHandler, RknpuType};
pub use starry_kernel::*;

pub mod card0;
pub mod card1;
pub mod devfs;
pub mod drm;

#[path = "../irq.rs"]
pub mod irq;
#[path = "../npuprobe.rs"]
pub mod npuprobe;
#[path = "../power.rs"]
pub mod power;
#[path = "../tool.rs"]
pub mod tool;

pub use npuprobe::rknpu_probe;
pub use power::enable_pm;

static DEVFS_HOOK_REGISTERED: AtomicBool = AtomicBool::new(false);

/// Register rknpu devfs nodes into StarryOS devfs builder hooks.
///
/// This should run before `starry_kernel::entry::init()` so `/dev/rknpu` and
/// `/dev/dri/*` are available immediately after boot.
pub fn init_starry_adapter() {
    if DEVFS_HOOK_REGISTERED
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
    {
        starry_kernel::pseudofs::dev::register_devfs_hook(devfs::register_rknpu_devices);
    }
}
