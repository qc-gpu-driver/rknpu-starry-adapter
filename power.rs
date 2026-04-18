use core::ptr::NonNull;
use rockchip_pm::{PowerDomain, RkBoard, RockchipPM};





#[cfg(target_arch = "aarch64")]
/// Yield while waiting for an NPU IRQ on AArch64.
///
/// The historical implementation used `wfi` directly here. The current version
/// yields to the scheduler instead so the system can continue making progress
/// while the submit path waits for interrupt completion.
pub(crate) fn irq_yield() {
    //unsafe { core::arch::asm!("wfi") };
    //axklib::time::busy_wait(core::time::Duration::from_micros(10));
    axtask::yield_now();
}

/// Power on the RK3588 NPU-related power domains required by the driver.
///
/// The Rockchip PM driver exposes per-domain control through [`RockchipPM`].
/// This helper turns on the top-level NPU domain and the individual subdomains
/// before MMIO access or IRQ setup begins.
pub fn enable_pm() {
    // RK3588 PMU base address from SoC memory map / DTS.
    const RK3588_PMU_BASE: u64 = 0xfd8d8000;
    const RK3588_PMU_SIZE: usize = 0x1000;

    // RK3588 NPU-related power-domain identifiers.
    const NPU: PowerDomain = PowerDomain(8);
    const NPUTOP: PowerDomain = PowerDomain(9);
    const NPU1: PowerDomain = PowerDomain(10);
    const NPU2: PowerDomain = PowerDomain(11);

    let pm_base = axklib::mem::iomap((RK3588_PMU_BASE as usize).into(), RK3588_PMU_SIZE)
        .expect("failed to iomap RK3588 PMU");
    let mut pm = RockchipPM::new(
        unsafe { NonNull::new_unchecked(pm_base.as_mut_ptr()) },
        RkBoard::Rk3588,
    );

    // Power domains are brought up explicitly so later register accesses and
    // submissions do not touch a gated NPU block.
    pm.power_domain_on(NPUTOP)
        .expect("failed to enable PM domain NPUTOP");
    pm.power_domain_on(NPU)
        .expect("failed to enable PM domain NPU");
    pm.power_domain_on(NPU1)
        .expect("failed to enable PM domain NPU1");
    pm.power_domain_on(NPU2)
        .expect("failed to enable PM domain NPU2");
}

