use druid::{
    widget::{prelude::*, Controller},
    Command, Handled,
};

use crate::{
    app::{
        body::{options::OptionsTabState, RequestState},
        command,
    },
    error::fmt_connect_err,
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
        if old_data.default_address.uri() != data.default_address.uri()
            || old_data.verify_certs != data.verify_certs
            || !old_data.default_metadata.same(&data.default_metadata)
            || !old_data.auth.same(&data.auth)
            || !old_data.proxy.same(&data.proxy)
        {
            ctx.submit_command(
                command::SET_SERVICE_OPTIONS.with((data.service.clone(), data.service_options())),
            );
        }

        child.update(ctx, old_data, data, env);
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
                (update)(self, ctx, data)
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

        let update_writer = self.updates.writer(ctx);
        let options = data.service_options();
        tokio::spawn(async move {
            let result = grpc::Client::new(&uri, options.verify_certs, options.proxy).await;
            update_writer.write(|controller, _, data| controller.finish_connect(data, result));
        });

        data.default_address
            .set_request_state(RequestState::ConnectInProgress);
    }

    fn finish_connect(&mut self, data: &mut OptionsTabState, result: grpc::ConnectResult) {
        match result {
            Ok(client) => {
                self.client = Some(client);
                data.default_address
                    .set_request_state(RequestState::Connected);
            }
            Err(err) => {
                data.default_address
                    .set_request_state(RequestState::ConnectFailed(fmt_connect_err(&err)));
            }
        }
    }

    fn disconnect(&mut self, _: &mut EventCtx, data: &mut OptionsTabState) {
        self.client = None;
        data.default_address
            .set_request_state(RequestState::NotStarted);
        self.updates.disconnect();
    }
}
