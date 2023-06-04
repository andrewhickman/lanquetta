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

    fn update(
        &mut self,
        child: &mut W,
        ctx: &mut UpdateCtx,
        old_data: &MethodTabState,
        data: &MethodTabState,
        env: &Env,
    ) {
        if old_data.address.uri() != data.address.uri()
            || old_data.service_options.verify_certs != data.service_options.verify_certs
        {
            ctx.submit_command(command::DISCONNECT.to(ctx.widget_id()));
        }

        child.update(ctx, old_data, data, env)
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
            self.finish_send();
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
        let verify_certs = data.service_options.verify_certs;
        tokio::spawn(async move {
            let result = grpc::Client::new(&uri, verify_certs).await;
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

            let metadata = data.request().tonic_metadata();

            let update_writer = self.updates.writer(ctx);
            self.call =
                Some(
                    client.call(data.method.clone(), request, metadata, move |response| {
                        update_writer
                            .write(|controller, data| controller.handle_response(data, response));
                    }),
                );

            data.address.set_request_state(RequestState::Active);
        }
    }

    fn finish_send(&mut self) {
        if let Some(call) = &mut self.call {
            call.finish();
        }
    }

    fn handle_response(&mut self, data: &mut MethodTabState, response: grpc::ResponseResult) {
        match response {
            grpc::ResponseResult::Response(response) => {
                let duration = match &mut self.call {
                    Some(call) => call.duration(&response),
                    _ => None,
                };

                let json_result = JsonText::short(response.to_json());

                data.stream.add_response(Ok(json_result), duration);
            }
            grpc::ResponseResult::Error(error, metadata) => {
                data.stream.add_response(Err(error), None);
                data.stream.set_metadata(metadata);
                self.call = None;
            }
            grpc::ResponseResult::Finished(metadata) => {
                data.stream.set_metadata(metadata);
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
