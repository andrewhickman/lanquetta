use std::sync::{Arc, Weak};

use crossbeam_queue::SegQueue;
use druid::{EventCtx, ExtEventSink, Selector, Target, WidgetId};

pub const UPDATE: Selector = Selector::new("app.update");

type QueueInner<W, T> = SegQueue<Box<dyn FnOnce(&mut W, &mut EventCtx, &mut T) + Send>>;

pub struct UpdateQueue<W, T> {
    queue: Arc<QueueInner<W, T>>,
}

pub struct UpdateQueueWriter<W, T> {
    target: WidgetId,
    event_sink: ExtEventSink,
    queue: Weak<QueueInner<W, T>>,
}

impl<W, T> UpdateQueue<W, T> {
    pub fn new() -> Self {
        UpdateQueue {
            queue: Arc::new(SegQueue::new()),
        }
    }

    pub fn pop(&self) -> Option<impl FnOnce(&mut W, &mut EventCtx, &mut T)> {
        self.queue.pop()
    }

    pub fn writer(&self, ctx: &EventCtx) -> UpdateQueueWriter<W, T> {
        UpdateQueueWriter {
            target: ctx.widget_id(),
            event_sink: ctx.get_external_handle(),
            queue: Arc::downgrade(&self.queue),
        }
    }

    pub fn disconnect(&mut self) {
        self.queue = Arc::new(SegQueue::new());
    }
}

impl<W, T> UpdateQueueWriter<W, T> {
    pub fn write(&self, f: impl FnOnce(&mut W, &mut EventCtx, &mut T) + Send + 'static) {
        if let Some(queue) = self.queue.upgrade() {
            queue.push(Box::new(f));
            _ = self.event_sink.submit_command(UPDATE, (), self.target);
        }
    }

    pub fn submit_command<U>(&self, selector: Selector<U>, payload: U, target: impl Into<Target>)
    where
        U: Send + 'static,
    {
        _ = self.event_sink.submit_command(selector, payload, target);
    }
}
