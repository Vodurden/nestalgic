/// `Ticker`
// TODO: Decide if this should exist and where it should live
pub struct Ticker {
    /// `tick_speed` defines how often we execute the tick function.
    pub tick_speed: Duration,

    /// Stores how much time has elapsed since we last "ticked"
    pub tick_accumulator: Duration
}

impl Ticker {
    pub fn tick(&mut self, delta: Duration, tick_fn: Function<A, B>) {
    }
}
