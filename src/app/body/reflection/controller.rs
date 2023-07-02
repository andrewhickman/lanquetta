use std::sync::Arc;

use anyhow::{bail, Context, Result};
use druid::{
    widget::{prelude::*, Controller},
    Command, Handled, Target,
};
use http::Uri;
use prost_reflect::{DescriptorPool, ServiceDescriptor};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tonic::{metadata::MetadataMap, Code, Extensions, Request, Status, Streaming};
use tonic_reflection::pb::{
    server_reflection_client::ServerReflectionClient, server_reflection_request::MessageRequest,
    server_reflection_response::MessageResponse, ServerReflectionRequest, ServerReflectionResponse,
};

use crate::{
    app::{
        body::{
            reflection::{ReflectionTabState, IMPORT_SERVICE, LIST_SERVICES},
            RequestState,
        },
        command,
    },
    error::fmt_grpc_err,
    grpc,
    proxy::Proxy,
    widget::update_queue::{self, UpdateQueue},
};

pub struct ReflectionController {
    updates: UpdateQueue<ReflectionController, ReflectionTabState>,
    session: Option<Arc<Mutex<ReflectionSession>>>,
}

struct ReflectionSession {
    sender: mpsc::UnboundedSender<ServerReflectionRequest>,
    receiver: Streaming<ServerReflectionResponse>,
    host: String,
    services: Arc<Vec<String>>,
    pool: DescriptorPool,
}

impl<W> Controller<ReflectionTabState, W> for ReflectionController
where
    W: Widget<ReflectionTabState>,
{
    fn event(
        &mut self,
        child: &mut W,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut ReflectionTabState,
        env: &Env,
    ) {
        match event {
            Event::Command(command) if self.command(ctx, command, data) == Handled::Yes => (),
            _ => child.event(ctx, event, data, env),
        }
    }
}

impl ReflectionController {
    pub fn new() -> Self {
        ReflectionController {
            updates: UpdateQueue::new(),
            session: None,
        }
    }

    fn command(
        &mut self,
        ctx: &mut EventCtx,
        command: &Command,
        data: &mut ReflectionTabState,
    ) -> Handled {
        tracing::debug!("Options tab received command: {:?}", command);

        if command.is(LIST_SERVICES) {
            self.list_services(ctx, data);
            Handled::Yes
        } else if let Some(service) = command.get(IMPORT_SERVICE) {
            self.import_service(ctx, data, service.clone());
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

    fn list_services(&self, ctx: &mut EventCtx<'_, '_>, data: &mut ReflectionTabState) {
        data.address
            .set_request_state(RequestState::ConnectInProgress);

        let Some(address) = data.address.uri().cloned() else {
            tracing::warn!("list-services called with invalid uri");
            return;
        };
        let metadata = data.metadata.metadata();

        let writer = self.updates.writer(ctx);
        let options = data.service_options();
        tokio::spawn(async move {
            let result =
                ReflectionSession::connect(address, options.verify_certs, options.proxy, metadata)
                    .await;
            writer.write(|controller, _, data| match result {
                Ok(session) => {
                    data.address.set_request_state(RequestState::Connected);
                    data.services = Some(session.services.clone());
                    controller.session = Some(Arc::new(Mutex::new(session)));
                }
                Err(err) => data
                    .address
                    .set_request_state(RequestState::ConnectFailed(fmt_grpc_err(&err))),
            });
        });
    }

    fn import_service(
        &self,
        ctx: &mut EventCtx<'_, '_>,
        data: &mut ReflectionTabState,
        name: String,
    ) {
        let Some(session) = self.session.clone() else {
            tracing::warn!("import-service called without session");
            return;
        };

        let service_options = data.service_options();

        let writer = self.updates.writer(ctx);
        tokio::spawn(async move {
            match ReflectionSession::load_service(session, name).await {
                Ok(service) => writer.submit_command(
                    command::ADD_SERVICE,
                    (service, service_options),
                    Target::Auto,
                ),
                Err(err) => writer.write(move |_, _, data| {
                    data.address
                        .set_request_state(RequestState::ConnectFailed(fmt_grpc_err(&err)));
                }),
            }
        });
    }
}

impl ReflectionSession {
    async fn connect(
        address: Uri,
        verify_certs: bool,
        proxy: Proxy,
        metadata: MetadataMap,
    ) -> Result<Self> {
        let channel = grpc::channel::get(&address, verify_certs, proxy).await?;
        let mut client = ServerReflectionClient::new(channel);

        let (sender, request_receiver) = mpsc::unbounded_channel::<ServerReflectionRequest>();
        let mut receiver = client
            .server_reflection_info(Request::from_parts(
                metadata,
                Extensions::default(),
                UnboundedReceiverStream::new(request_receiver),
            ))
            .await?
            .into_inner();

        let host = address.host().unwrap_or_default().to_owned();
        sender.send(ServerReflectionRequest {
            host: host.clone(),
            message_request: Some(MessageRequest::ListServices(String::default())),
        })?;
        let Some(response) = receiver.message().await? else {
            bail!("unexpected end of response stream");
        };
        let service_list = match response.message_response {
            Some(MessageResponse::ListServicesResponse(service_list)) => service_list,
            Some(MessageResponse::ErrorResponse(error)) => {
                return Err(
                    Status::new(Code::from_i32(error.error_code), error.error_message).into(),
                )
            }
            _ => bail!("unexpected response type"),
        };

        Ok(ReflectionSession {
            sender,
            receiver,
            host,
            services: Arc::new(service_list.service.into_iter().map(|s| s.name).collect()),
            pool: DescriptorPool::new(),
        })
    }

    async fn load_service(this: Arc<Mutex<Self>>, name: String) -> Result<ServiceDescriptor> {
        let mut this = this.lock().await;

        this.sender.send(ServerReflectionRequest {
            host: this.host.clone(),
            message_request: Some(MessageRequest::FileContainingSymbol(name.clone())),
        })?;
        let Some(response) = this.receiver.message().await? else {
            bail!("unexpected end of response stream");
        };
        let file_response = match response.message_response {
            Some(MessageResponse::FileDescriptorResponse(file_response)) => file_response,
            Some(MessageResponse::ErrorResponse(error)) => {
                return Err(
                    Status::new(Code::from_i32(error.error_code), error.error_message).into(),
                )
            }
            _ => bail!("unexpected response type"),
        };

        for file in file_response.file_descriptor_proto {
            this.pool
                .decode_file_descriptor_proto(file.as_ref())
                .context("failed to load file descriptor from server")?;
        }

        let Some(service) = this.pool.get_service_by_name(&name) else {
            bail!("service '{}' not found in file descriptor from server", name)
        };

        Ok(service)
    }
}
