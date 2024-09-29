use arm_gic::gicv3::{GicV3, IntId};
use kspin::SpinNoIrq;
use memory_addr::PhysAddr;

use crate::mem::phys_to_virt;

const GICD_BASE: PhysAddr = pa!(axconfig::GICD_PADDR);
const GICR_BASE: PhysAddr = pa!(axconfig::GICC_PADDR);

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number.
pub const TIMER_IRQ_NUM: usize = 30;

/// The UART IRQ number.
pub const UART_IRQ_NUM: usize = 33;

struct GicV3Wrapper {
    inner: GicV3,
}

unsafe impl Send for GicV3Wrapper {}
unsafe impl Sync for GicV3Wrapper {}

static GIC_V3: SpinNoIrq<Option<GicV3Wrapper>> = SpinNoIrq::new(None);

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    GIC_V3
        .lock()
        .as_mut()
        .unwrap()
        .inner
        .enable_interrupt(IntId::from(irq_num as u32), enabled);
}

/// Initializes GICD, GICC on the primary CPU.
pub(crate) fn init_primary() {
    let mut gic_v3_lock = GIC_V3.lock();
    unsafe {
        *gic_v3_lock = Some(GicV3Wrapper {
            inner: GicV3::new(
                phys_to_virt(GICD_BASE).as_mut_ptr_of(),
                phys_to_virt(GICR_BASE).as_mut_ptr_of(),
            ),
        })
    }
    let gic_v3 = gic_v3_lock.as_mut().unwrap();
    gic_v3.inner.setup();
    gic_v3.inner.enable_interrupt(IntId::sgi(3), true);
}

/// 发送yield中断信号
pub fn send_ipi(_vector: u8, _dest: u32) {
    use aarch64_cpu::registers::Readable;
    let mpidr = aarch64_cpu::registers::MPIDR_EL1.get();
    let aff1 = mpidr >> 8 & 0xff;
    let aff2 = mpidr >> 16 & 0xff;
    let aff3 = mpidr >> 32 & 0xff;
    let sgi_intid = IntId::sgi(3);
    GicV3::send_sgi(
        sgi_intid,
        arm_gic::gicv3::SgiTarget::List {
            affinity3: aff3 as _,
            affinity2: aff2 as _,
            affinity1: aff1 as _,
            target_list: 0b1,
        },
    );
}

pub fn end_of_interrupt(irq: usize) {
    GicV3::end_interrupt(IntId::from(irq as u32));
}

pub fn get_and_acknowledge_interrupt() -> usize {
    u32::from(GicV3::get_and_acknowledge_interrupt().unwrap()) as _
}

pub fn dispatch_irq(_unused: usize) {
    unimplemented!()
}

pub fn register_handler(irq_num: usize, handler: crate::irq::IrqHandler) -> bool {
    unimplemented!()
}
