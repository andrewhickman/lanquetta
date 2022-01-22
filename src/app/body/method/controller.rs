use druid::{
    widget::{prelude::*, Controller},
    Command, Handled,
};

use crate::{
    app::{
        body::{method::MethodTabState, RequestState},
        command,
    },
    grpc,
    json::JsonText,
    widget::update_queue::{self, UpdateQueue},
};

pub struct MethodTabController {
    updates: UpdateQueue<MethodTabController, MethodTabState>,
    client: Option<grpc::Client>,
    call: Option<grpc::Call>,
}

impl MethodTabController {
    pub fn new() -> MethodTabController {
        MethodTabController {
            updates: UpdateQueue::new(),
            client: None,
            call: None,
        }
    }
}

impl<W> Controller<MethodTabState, W> for MethodTabController
where
    W: Widget<MethodTabState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut MethodTabState,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if self.command(ctx, command, data) == Handled::Yes => (),
            _ => child.event(ctx, event, data, env),
        }
    }
}

impl MethodTabController {
    fn command(
        &mut self,
        ctx: &mut EventCtx,
        command: &Command,
        data: &mut MethodTabState,
    ) -> Handled {
        tracing::debug!("Method tab received command: {:?}", command);

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
        } else if command.is(update_queue::UPDATE) {
            while let Some(update) = self.updates.pop() {
                (update)(self, data)
            }
            Handled::Yes
        } else {
            Handled::No
        }
    }

    fn start_connect(&mut self, ctx: &mut EventCtx, data: &mut MethodTabState) {
        let uri = match data.address.uri() {
            Some(uri) => uri.clone(),
            None => {
                tracing::error!("Connect called with no address");
                return;
            }
        };

        if self.is_connected() {
            return;
        }

        let update_writer = self.updates.writer(ctx);
        tokio::spawn(async move {
            let result = grpc::Client::new(&uri).await;
            update_writer.write(|controller, data| controller.finish_connect(data, result));
        });

        data.address
            .set_request_state(RequestState::ConnectInProgress);
    }

    fn finish_connect(&mut self, data: &mut MethodTabState, result: grpc::ConnectResult) {
        match result {
            Ok(client) => {
                self.client = Some(client);
                self.set_request_state(data);
            }
            Err(err) => {
                data.address
                    .set_request_state(RequestState::ConnectFailed(err));
            }
        }
    }

    fn start_send(&mut self, ctx: &mut EventCtx, data: &mut MethodTabState) {
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

            let update_writer = self.updates.writer(ctx);
            self.call = Some(client.call(data.method.clone(), request, move |response| {
                update_writer.write(|controller, data| controller.finish_send(data, response));
            }));

            data.address.set_request_state(RequestState::Active);
        }
    }

    fn finish_send(&mut self, data: &mut MethodTabState, response: Option<grpc::ResponseResult>) {
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

    fn disconnect(&mut self, _: &mut EventCtx, data: &mut MethodTabState) {
        self.client = None;
        self.call = None;

        self.set_request_state(data);

        self.updates.disconnect();
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    fn is_active(&self) -> bool {
        self.call.is_some()
    }

    fn set_request_state(&self, data: &mut MethodTabState) {
        let request_state = match (self.is_active(), self.is_connected()) {
            (false, false) => RequestState::NotStarted,
            (false, true) => RequestState::Connected,
            (true, _) => RequestState::Active,
        };
        data.address.set_request_state(request_state);
    }
}
