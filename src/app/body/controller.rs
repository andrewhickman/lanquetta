use std::sync::{Arc, Weak};

use crossbeam_queue::SegQueue;
use druid::{
    widget::{prelude::*, Controller},
    Command, ExtEventSink, Handled, Selector,
};

use crate::{
    app::{
        body::{RequestState, TabState},
        command,
    },
    grpc,
    json::JsonText,
};

type UpdateQueue = SegQueue<Box<dyn FnOnce(&mut TabController, &mut TabState) + Send>>;

pub struct TabController {
    updates: Arc<UpdateQueue>,
    client: Option<grpc::Client>,
    call: Option<grpc::Call>,
}

struct UpdateWriter {
    target: WidgetId,
    event_sink: ExtEventSink,
    updates: Weak<UpdateQueue>,
}

impl TabController {
    pub fn new() -> TabController {
        TabController {
            updates: Arc::default(),
            client: None,
            call: None,
        }
    }
}

impl<W> Controller<TabState, W> for TabController
where
    W: Widget<TabState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut TabState,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if self.command(ctx, command, data) == Handled::Yes => (),
            _ => child.event(ctx, event, data, env),
        }
    }

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &TabState,
        data: &TabState,
        env: &Env,
    ) {
        if old_data.address.uri() != data.address.uri() {
            ctx.submit_command(command::DISCONNECT.to(ctx.widget_id()));
        }

        child.update(ctx, old_data, data, env)
    }
}

const UPDATE_STATE: Selector = Selector::new("app.body.update-state");

impl TabController {
    fn command(&mut self, ctx: &mut EventCtx, command: &Command, data: &mut TabState) -> Handled {
        tracing::debug!("Body received command: {:?}", command);

        if command.is(command::CONNECT) {
            self.start_connect(ctx, data);
            Handled::Yes
        } else if command.is(command::SEND) {
            self.start_send(ctx, data);
            Handled::Yes
        } else if command.is(command::FINISH) {
            self.finish_send(data, None);
            Handled::Yes
        } else if command.is(command::DISCONNECT) {
            self.disconnect(ctx, data);
            Handled::Yes
        } else if command.is(UPDATE_STATE) {
            while let Some(update) = self.updates.pop() {
                (update)(self, data);
            }
            Handled::Yes
        } else {
            Handled::No
        }
    }
}

impl TabController {
    fn get_update_writer(&self, ctx: &mut EventCtx) -> UpdateWriter {
        UpdateWriter {
            target: ctx.widget_id(),
            event_sink: ctx.get_external_handle(),
            updates: Arc::downgrade(&self.updates),
        }
    }

    fn start_connect(&mut self, ctx: &mut EventCtx, data: &mut TabState) {
        let uri = match data.address.uri() {
            Some(uri) => uri.clone(),
            None => {
                tracing::error!("Connect called with no address");
                return;
            }
        };

        if self.is_connected() {
            tracing::error!("Connect called when already connected");
            return;
        }

        let update_writer = self.get_update_writer(ctx);
        tokio::spawn(async move {
            let result = grpc::Client::new(&uri).await;
            update_writer.update(|controller, data| controller.finish_connect(data, result));
        });

        data.address
            .set_request_state(RequestState::ConnectInProgress);
    }

    fn finish_connect(&mut self, data: &mut TabState, result: grpc::ConnectResult) {
        match result {
            Ok(client) => {
                self.client = Some(client);
                self.set_request_state(data);
            }
            Err(_) => {
                data.address.set_request_state(RequestState::ConnectFailed);
            }
        }
    }

    fn start_send(&mut self, ctx: &mut EventCtx, data: &mut TabState) {
        let request = match data.request().get() {
            Some(request) => request.clone(),
            None => {
                tracing::error!("Send called with no request");
                return;
            }
        };

        let json = data.request().get_json().clone();
        data.stream.add_request(json);

        if let Some(call) = &mut self.call {
            if data.method.is_client_streaming() {
                call.send(request);
            } else {
                tracing::warn!("Send called on active call with non-streaming method");
            }
        } else {
            let client = match &self.client {
                Some(client) => client.clone(),
                _ => {
                    tracing::error!("Send called with invalid client");
                    return;
                }
            };

            let update_writer = self.get_update_writer(ctx);
            self.call = Some(client.call(data.method.clone(), request, move |response| {
                update_writer.update(|controller, data| controller.finish_send(data, response));
            }));

            data.address.set_request_state(RequestState::Active);
        }
    }

    fn finish_send(&mut self, data: &mut TabState, response: Option<grpc::ResponseResult>) {
        match response {
            Some(result) => {
                let duration = match (&mut self.call, &result) {
                    (Some(call), Ok(response)) => call.duration(response),
                    _ => None,
                };

                let json_result = result
                    .map(|response| response.to_json())
                    .map(JsonText::short);

                data.stream.add_response(json_result, duration);
            }
            None => {
                self.call = None;
            }
        }

        self.set_request_state(data);
    }

    fn disconnect(&mut self, _: &mut EventCtx, data: &mut TabState) {
        self.client = None;
        self.call = None;

        self.set_request_state(data);

        self.updates = Arc::new(SegQueue::new());
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    fn is_active(&self) -> bool {
        self.call.is_some()
    }

    fn set_request_state(&self, data: &mut TabState) {
        let request_state = match (self.is_active(), self.is_connected()) {
            (false, false) => RequestState::NotStarted,
            (false, true) => RequestState::Connected,
            (true, _) => RequestState::Active,
        };
        data.address.set_request_state(request_state);
    }
}

impl UpdateWriter {
    fn update(&self, f: impl FnOnce(&mut TabController, &mut TabState) + Send + 'static) -> bool {
        if let Some(updates) = self.updates.upgrade() {
            updates.push(Box::new(f));
            self.event_sink
                .submit_command(UPDATE_STATE, (), self.target)
                .is_ok()
        } else {
            false
        }
    }
}
