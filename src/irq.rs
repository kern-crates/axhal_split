//! Interrupt management.

use handler_table::HandlerTable;

use crate::platform::irq::{dispatch_irq, MAX_IRQ_COUNT};
use crate::trap::{register_trap_handler, IRQ};

pub use crate::platform::irq::{register_handler, set_enable, send_ipi, end_of_interrupt, get_and_acknowledge_interrupt};

/// The type if an IRQ handler.
pub type IrqHandler = handler_table::Handler;

static IRQ_HANDLER_TABLE: HandlerTable<MAX_IRQ_COUNT> = HandlerTable::new();

/// Platform-independent IRQ dispatching.
#[allow(dead_code)]
pub(crate) fn dispatch_irq_common(irq_num: usize) {
    trace!("IRQ {}", irq_num);
    if !IRQ_HANDLER_TABLE.handle(irq_num) {
        warn!("Unhandled IRQ {}", irq_num);
    }
}

/// Platform-independent IRQ handler registration.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
#[allow(dead_code)]
pub(crate) fn register_handler_common(irq_num: usize, handler: IrqHandler) -> bool {
    if irq_num < MAX_IRQ_COUNT && IRQ_HANDLER_TABLE.register_handler(irq_num, handler) {
        set_enable(irq_num, true);
        return true;
    }
    warn!("register handler for IRQ {} failed", irq_num);
    false
}

#[register_trap_handler(IRQ)]
fn handler_irq(irq_num: usize) -> bool {
    let guard = kernel_guard::NoPreempt::new();
    dispatch_irq(irq_num);
    drop(guard); // rescheduling may occur when preemption is re-enabled.
    true
}
