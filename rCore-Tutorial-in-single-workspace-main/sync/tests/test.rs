
use sync::{up::{INTR_MASKING_INFO, IntrMaskingInfo},
            condvar::{Condvar}
    };

#[test]
fn test_up() {
    let condvar1 = Condvar::new();
    (& condvar1).wait_no_sched(thread::current().id());
    let mut intr1 = IntrMaskingInfo::new();
    //(&mut intr1).enter();
}