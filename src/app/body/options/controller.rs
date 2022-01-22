use druid::{
    widget::{prelude::*, Controller},
    Command, Handled,
};

use crate::{
    app::{
        body::{options::OptionsTabState, RequestState},
        command,
    },
    grpc,
    widget::update_queue::{self, UpdateQueue},
};

pub struct OptionsTabController {
    updates: UpdateQueue<OptionsTabController, OptionsTabState>,
    client: Option<grpc::Client>,
}

impl OptionsTabController {
    pub fn new() -> OptionsTabController {
        OptionsTabController {
            updates: UpdateQueue::new(),
            client: None,
        }
    }
}

impl<W> Controller<OptionsTabState, W> for OptionsTabController
where
    W: Widget<OptionsTabState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut OptionsTabState,
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
        old_data: &OptionsTabState,
        data: &OptionsTabState,
        env: &Env,
    ) {
        if !old_data.same(data) {
            if let Some(service_options) = data.service_options() {
                ctx.submit_command(
                    command::SET_SERVICE_OPTIONS.with((data.service.clone(), service_options)),
                );
            }
            child.update(ctx, old_data, data, env);
        }
    }
}

impl OptionsTabController {
    fn command(
        &mut self,
        ctx: &mut EventCtx,
        command: &Command,
        data: &mut OptionsTabState,
    ) -> Handled {
        tracing::debug!("Options tab received command: {:?}", command);

        if command.is(command::CONNECT) {
            self.start_connect(ctx, data);
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

    fn start_connect(&mut self, ctx: &mut EventCtx, data: &mut OptionsTabState) {
        let uri = match data.default_address.uri() {
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

        data.default_address
            .set_request_state(RequestState::ConnectInProgress);
    }

    fn finish_connect(&mut self, data: &mut OptionsTabState, result: grpc::ConnectResult) {
        match result {
            Ok(client) => {
                self.client = Some(client);
                self.set_request_state(data);
            }
            Err(_) => {
                data.default_address
                    .set_request_state(RequestState::ConnectFailed);
            }
        }
    }

    fn disconnect(&mut self, _: &mut EventCtx, data: &mut OptionsTabState) {
        self.client = None;

        self.set_request_state(data);

        self.updates.disconnect();
    }

    fn is_connected(&self) -> bool {
        self.client.is_some()
    }

    fn set_request_state(&self, data: &mut OptionsTabState) {
        let request_state = if self.is_connected() {
            RequestState::Connected
        } else {
            RequestState::NotStarted
        };
        data.default_address.set_request_state(request_state);
    }
}
