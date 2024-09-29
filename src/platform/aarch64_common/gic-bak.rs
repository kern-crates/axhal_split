use crate::{irq::IrqHandler, mem::phys_to_virt};
use arm_gicv2::{translate_irq, GicCpuInterface, GicDistributor, InterruptType};
use kspin::SpinNoIrq;
use memory_addr::PhysAddr;

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = translate_irq(14, InterruptType::PPI).unwrap();

/// The UART IRQ number.
pub const UART_IRQ_NUM: usize = translate_irq(axconfig::UART_IRQ, InterruptType::SPI).unwrap();

const GICD_BASE: PhysAddr = pa!(axconfig::GICD_PADDR);
const GICC_BASE: PhysAddr = pa!(axconfig::GICC_PADDR);

static GICD: SpinNoIrq<GicDistributor> =
    SpinNoIrq::new(GicDistributor::new(phys_to_virt(GICD_BASE).as_mut_ptr()));

// per-CPU, no lock
static GICC: GicCpuInterface = GicCpuInterface::new(phys_to_virt(GICC_BASE).as_mut_ptr());

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    trace!("GICD set enable: {} {}", irq_num, enabled);
    GICD.lock().set_enable(irq_num as _, enabled);
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {
    trace!("register handler irq {}", irq_num);
    crate::irq::register_handler_common(irq_num, handler)
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(_unused: usize) {
    GICC.handle_irq(|irq_num| crate::irq::dispatch_irq_common(irq_num as _));
}

/// Initializes GICD, GICC on the primary CPU.
pub(crate) fn init_primary() {
    info!("Initialize GICv2...");
    GICD.lock().init();
    GICC.init();
}

/// Initializes GICC on secondary CPUs.
#[cfg(feature = "smp")]
pub(crate) fn init_secondary() {
    GICC.init();
}

/// 发送yield中断信号
pub fn send_ipi(_vector: u8, _dest: u32) {
    use aarch64_cpu::registers::Readable;
    let intid = 3;
    let mpidr = aarch64_cpu::registers::MPIDR_EL1.get();
    let cpu_id = mpidr >> 8 & 0xff;
    let value = 1 << (cpu_id + 16) | intid;
    unsafe {
        core::ptr::write_volatile(
            // 0xff84_1000 + 0xFFFFFF8000000000 + 0x0f00
            18446743528240586496 as *mut u32,
            value as _,
        )
    };
}

pub fn end_of_interrupt(irq: usize) {
    let gicc_base: usize = 0xff84_1000 + 0xFFFFFF8000000000;

    unsafe {
        core::ptr::write_volatile((gicc_base + 0x0010) as *mut u32, irq as _);
    }
}
