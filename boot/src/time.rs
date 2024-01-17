use uefi_services::system_table;

pub fn timer_tick() -> u64 {
    #[cfg(target_arch = "x86")]
    unsafe {
        core::arch::x86::_rdtsc()
    }

    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::x86_64::_rdtsc()
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut ticks: u64;
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) ticks);
        ticks
    }
}

pub fn timer_freq() -> u64 {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        let start = timer_tick();
        system_table().boot_services().stall(1000);
        let end = timer_tick();
        (end - start) * 1000
    }

    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut freq: u64;
        core::arch::asm!("mrs {}, cntfrq_el0", out(reg) freq);
        freq
    }
}
