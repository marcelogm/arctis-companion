pub struct Wired<S, L> {
    _listeners: L,
    streams: S,
}

impl<S, L> Wired<S, L> {
    pub const fn new(streams: S, listeners: L) -> Self {
        Self {
            _listeners: listeners,
            streams,
        }
    }

    pub const fn streams(&self) -> &S {
        &self.streams
    }
}
