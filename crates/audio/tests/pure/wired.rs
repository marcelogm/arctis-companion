use std::cell::RefCell;
use std::rc::Rc;

use arctis_audio::wired::Wired;

struct Recorder {
    name: &'static str,
    dropped: Rc<RefCell<Vec<&'static str>>>,
}

impl Drop for Recorder {
    fn drop(&mut self) {
        self.dropped.borrow_mut().push(self.name);
    }
}

fn recorder(name: &'static str, dropped: &Rc<RefCell<Vec<&'static str>>>) -> Recorder {
    Recorder {
        name,
        dropped: dropped.clone(),
    }
}

#[test]
fn listeners_are_released_before_the_streams_they_hook_into() {
    let dropped = Rc::default();
    let wired = Wired::new(
        recorder("streams", &dropped),
        recorder("listeners", &dropped),
    );
    assert_eq!(wired.streams().name, "streams");
    drop(wired);
    assert_eq!(*dropped.borrow(), ["listeners", "streams"]);
}
