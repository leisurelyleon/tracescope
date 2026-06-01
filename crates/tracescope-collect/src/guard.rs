//! An RAII span guard: opens a span on creation, closes it on drop.

use tracescope_core::SpanId;

use crate::clock::Clock;
use crate::collector::Collector;

/// Closes its span when dropped, so spans nest naturally with lexical scope.
pub struct SpanGuard<'c, C: Clock> {
    collector: &'c Collector<C>,
    id: SpanId,
}

impl<'c, C: Clock> SpanGuard<'c, C> {
    pub(crate) fn new(collector: &'c Collector<C>, id: SpanId) -> Self {
        Self { collector, id }
    }

    /// The id of the span this guard will close.
    pub fn id(&self) -> SpanId {
        self.id
    }
}

impl<C: Clock> Drop for SpanGuard<'_, C> {
    fn drop(&mut self) {
        self.collector.end(self.id);
    }
}

#[cfg(test)]
mod tests {
    use crate::clock::ManualClock;
    use crate::collector::Collector;

    #[test]
    fn guard_closes_span_on_drop() {
        let collector = Collector::new(ManualClock::new());
        {
            let _scoped = collector.span("scoped"); // t=0
            collector.clock().advance(7);
        } // drop -> end at t=7
        let trace = collector.snapshot();
        assert_eq!(trace.span(0).unwrap().duration_ns(), 7);
    }

    #[test]
    fn nested_guards_record_parentage() {
        let collector = Collector::new(ManualClock::new());
        {
            let _outer = collector.span("outer"); // id 0, t=0
            collector.clock().advance(10);
            {
                let _inner = collector.span("inner"); // id 1, parent 0, t=10
                collector.clock().advance(5);
            } // inner ends t=15
            collector.clock().advance(20);
        } // outer ends t=35
        let trace = collector.snapshot();
        assert_eq!(trace.span(1).unwrap().parent, Some(0));
        assert_eq!(trace.span(0).unwrap().duration_ns(), 35);
    }
}
