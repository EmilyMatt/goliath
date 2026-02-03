use crate::error::{GoliathOperatorError, GoliathOperatorResult};
use futures_util::{SinkExt, StreamExt};
use goliath_common::{GoliathCommand, GoliathReport};
use std::net::Ipv4Addr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

pub(crate) struct GoliathClient {
    command_tx: mpsc::Sender<GoliathCommand>,
    report_rx: mpsc::Receiver<GoliathReport>,
    client_task: Option<(oneshot::Sender<()>, JoinHandle<()>)>,
}

impl GoliathClient {
    async fn client_task(
        stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
        mut command_rx: mpsc::Receiver<GoliathCommand>,
        report_tx: mpsc::Sender<GoliathReport>,
        kill_switch_rx: oneshot::Receiver<()>,
    ) {
        let (mut stream_tx, mut stream_rx) = stream.split();

        let kill_flag = Arc::new(AtomicBool::new(false));
        let mut sender_task = tokio::spawn({
            let kill_flag = Arc::clone(&kill_flag);
            async move {
                loop {
                    if kill_flag.load(Ordering::Relaxed) {
                        return GoliathOperatorResult::Ok(());
                    }

                    if let Ok(Some(msg)) =
                        tokio::time::timeout(Duration::from_millis(10), command_rx.recv()).await
                    {
                        stream_tx
                            .send(Message::Binary(msg.into_bytes()?))
                            .await
                            .map_err(Box::new)?;
                    }
                }
            }
        });

        let mut receiver_task = tokio::spawn({
            let kill_flag = Arc::clone(&kill_flag);
            async move {
                loop {
                    if kill_flag.load(Ordering::Relaxed) {
                        return GoliathOperatorResult::Ok(());
                    }

                    if let Ok(maybe_msg) =
                        tokio::time::timeout(Duration::from_millis(10), stream_rx.next()).await
                    {
                        if let Some(msg) = maybe_msg {
                            match msg {
                                Ok(Message::Ping(_) | Message::Pong(_)) => continue,
                                Ok(Message::Close(frame)) => {
                                    if let Some(frame) = frame {
                                        log::info!("Got close frame from websocket: {frame:?}");
                                    } else {
                                        log::info!(
                                            "Got close message from websocket with unknown reason"
                                        );
                                    }
                                    return Ok(());
                                }
                                Ok(Message::Text(text)) => {
                                    log::warn!("Got unexpected text: {text}")
                                }
                                Ok(Message::Binary(bytes)) => {
                                    match GoliathReport::read_from_bytes(&bytes) {
                                        Ok(report) => {
                                            report_tx.send(report).await.map_err(|err| {
                                                GoliathOperatorError::TokioSendError(
                                                    err.to_string(),
                                                )
                                            })?;
                                        }
                                        Err(err) => {
                                            log::error!(
                                                "Failed to read command: {err} (command: {:?})",
                                                bytes
                                            );
                                        }
                                    }
                                }
                                Ok(Message::Frame(_)) => {
                                    log::debug!("Got raw frame message, ignoring");
                                }
                                Err(err) => {
                                    log::error!("Received error from websocket: {err:?}");
                                    return Ok(());
                                }
                            }
                        } else {
                            return Ok(());
                        }
                    }
                }
            }
        });

        tokio::select! {
            _ = &mut sender_task => {}
            _ = &mut receiver_task => {}
            _ = kill_switch_rx => {}
        }

        kill_flag.store(true, Ordering::Relaxed);

        sender_task.await.ok();
        receiver_task.await.ok();
    }

    pub(crate) async fn try_new(address: Ipv4Addr, port: u16) -> GoliathOperatorResult<Self> {
        let url = format!("ws://{address}:{port}");
        let (ws_stream, _) = connect_async(&url).await.expect("Failed to connect");

        let (command_tx, command_rx) = mpsc::channel::<GoliathCommand>(10);
        let (report_tx, report_rx) = mpsc::channel::<GoliathReport>(10);
        let (kill_switch_tx, kill_switch_rx) = oneshot::channel::<()>();
        let client_task = tokio::spawn(Self::client_task(
            ws_stream,
            command_rx,
            report_tx,
            kill_switch_rx,
        ));
        Ok(Self {
            command_tx,
            report_rx,
            client_task: Some((kill_switch_tx, client_task)),
        })
    }

    pub(crate) async fn send_command(&self, command: GoliathCommand) -> GoliathOperatorResult<()> {
        self.command_tx
            .send(command)
            .await
            .map_err(|err| GoliathOperatorError::TokioSendError(err.to_string()))
    }

    pub(crate) fn poll_report(&mut self) -> GoliathOperatorResult<Option<GoliathReport>> {
        match self.report_rx.try_recv() {
            Ok(msg) => Ok(Some(msg)),
            Err(TryRecvError::Empty) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }
}

impl Drop for GoliathClient {
    fn drop(&mut self) {
        if let Some((kill_switch_tx, _task_handle)) = self.client_task.take() {
            kill_switch_tx.send(()).ok();
        }
    }
}
