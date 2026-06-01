//! The span collector: records nested spans into a `Trace`.

use std::collections::HashMap;
use std::sync::Mutex;

use tracescope_core::{Span, SpanId, Trace};

use crate::clock::Clock;
use crate::guard::SpanGuard;

#[derive(Default)]
struct State {
    next_id: SpanId,
    spans: Vec<Span>,
    index: HashMap<SpanId, usize>,
    open_stack: Vec<SpanId>,
}

/// Records spans timed by a `Clock`. Models a single logical flow of execution:
/// `start` opens a span as a child of the currently-open span, and `end` closes
/// it. The RAII `span` guard pairs these automatically.
pub struct Collector<C: Clock> {
    clock: C,
    state: Mutex<State>,
}

impl<C: Clock> Collector<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            state: Mutex::new(State::default()),
        }
    }

    /// Access the clock (used to advance a `ManualClock` in tests and demos).
    pub fn clock(&self) -> &C {
        &self.clock
    }

    /// Opens a span as a child of the currently-open span; returns its id.
    pub fn start(&self, name: impl Into<String>) -> SpanId {
        let now = self.clock.now_ns();
        let mut guard = self.state.lock().expect("collector mutex poisoned");
        let state = &mut *guard;

        let id = state.next_id;
        state.next_id += 1;
        let parent = state.open_stack.last().copied();
        let idx = state.spans.len();
        state.spans.push(Span::new(id, name, now, now, parent));
        state.index.insert(id, idx);
        state.open_stack.push(id);
        id
    }

    /// Closes a previously-opened span, recording its end time.
    pub fn end(&self, id: SpanId) {
        let now = self.clock.now_ns();
        let mut guard = self.state.lock().expect("collector mutex poisoned");
        // Reborrow as `&mut State` so the two field updates below are seen as
        // disjoint borrows (index read, then spans write) rather than two
        // through-the-guard borrows.
        let state = &mut *guard;

        if let Some(&idx) = state.index.get(&id) {
            state.spans[idx].end_ns = now;
        }
        if let Some(pos) = state.open_stack.iter().rposition(|&s| s == id) {
            state.open_stack.remove(pos);
        }
    }

    /// Opens a span and returns an RAII guard that closes it on drop.
    pub fn span(&self, name: impl Into<String>) -> SpanGuard<'_, C> {
        let id = self.start(name);
        SpanGuard::new(self, id)
    }

    /// A snapshot of the recorded trace so far.
    pub fn snapshot(&self) -> Trace {
        let guard = self.state.lock().expect("collector mutex poisoned");
        Trace::new(guard.spans.clone())
    }

    /// Consumes the collector, returning the recorded trace.
    pub fn finish(self) -> Trace {
        let state = self.state.into_inner().expect("collector mutex poisoned");
        Trace::new(state.spans)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::ManualClock;

    #[test]
    fn records_nested_spans_with_parents() {
        let collector = Collector::new(ManualClock::new());
        let outer = collector.start("outer"); // id 0 at t=0
        collector.clock().advance(10);
        let inner = collector.start("inner"); // id 1 at t=10, parent 0
        collector.clock().advance(5);
        collector.end(inner); // t=15
        collector.clock().advance(20);
        collector.end(outer); // t=35

        let trace = collector.snapshot();
        assert_eq!(trace.span_count(), 2);
        assert_eq!(trace.span(0).unwrap().duration_ns(), 35);
        assert_eq!(trace.span(1).unwrap().duration_ns(), 5);
        assert_eq!(trace.span(1).unwrap().parent, Some(0));
    }

    #[test]
    fn finish_returns_the_trace() {
        let collector = Collector::new(ManualClock::new());
        let id = collector.start("only");
        collector.clock().advance(8);
        collector.end(id);
        let trace = collector.finish();
        assert_eq!(trace.span_count(), 1);
        assert_eq!(trace.span(0).unwrap().duration_ns(), 8);
    }
}
