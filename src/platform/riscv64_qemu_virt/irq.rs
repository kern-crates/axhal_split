//! TODO: PLIC

use crate::irq::IrqHandler;
use lazyinit::LazyInit;
use riscv::register::sie;

/// `Interrupt` bit in `scause`
pub(super) const INTC_IRQ_BASE: usize = 1 << (usize::BITS - 1);

/// Supervisor software interrupt in `scause`
#[allow(unused)]
pub(super) const S_SOFT: usize = INTC_IRQ_BASE + 1;

/// Supervisor timer interrupt in `scause`
pub(super) const S_TIMER: usize = INTC_IRQ_BASE + 5;

/// Supervisor external interrupt in `scause`
pub(super) const S_EXT: usize = INTC_IRQ_BASE + 9;

static TIMER_HANDLER: LazyInit<IrqHandler> = LazyInit::new();

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The timer IRQ number (supervisor timer interrupt in `scause`).
pub const TIMER_IRQ_NUM: usize = S_TIMER;

macro_rules! with_cause {
    ($cause: expr, @TIMER => $timer_op: expr, @EXT => $ext_op: expr $(,)?) => {
        match $cause {
            S_TIMER => $timer_op,
            S_EXT => $ext_op,
            _ => panic!("invalid trap cause: {:#x}", $cause),
        }
    };
}

/// Enables or disables the given IRQ.
pub fn set_enable(scause: usize, _enabled: bool) {
    if scause == S_EXT {
        // TODO: set enable in PLIC
    }
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(scause: usize, handler: IrqHandler) -> bool {
    with_cause!(
        scause,
        @TIMER => if !TIMER_HANDLER.is_inited() {
            TIMER_HANDLER.init_once(handler);
            true
        } else {
            false
        },
        @EXT => crate::irq::register_handler_common(scause & !INTC_IRQ_BASE, handler),
    )
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(scause: usize) {
    with_cause!(
        scause,
        @TIMER => {
            trace!("IRQ: timer");
            TIMER_HANDLER();
        },
        @EXT => crate::irq::dispatch_irq_common(0), // TODO: get IRQ number from PLIC
    );
}

pub(super) fn init_percpu() {
    // enable soft interrupts, timer interrupts, and external interrupts
    unsafe {
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
    }
}

fn cpu_id() -> u8 {
    let mut cpu_id;
    unsafe { core::arch::asm!("mv {0}, tp", out(reg) cpu_id) };
    cpu_id
}

pub fn send_ipi(vector: u8, dest: u32) {
    let cpuid = dest;
    log::debug!("send ipi [{}] => [{}], vector={:#x}", cpu_id(), cpuid, vector);
    let mask:usize = 1 << cpuid;
    sbi_rt::legacy::send_ipi(&mask as *const usize as usize);
}

pub fn get_and_acknowledge_interrupt() -> usize {
    //plic
    unimplemented!()
}

pub fn end_of_interrupt(irq: usize) {
    //plic
    // unimplemented!()
    sbi_rt::legacy::clear_ipi();
}
