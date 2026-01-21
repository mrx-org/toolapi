pub struct Sender {
    msg_tx: tokio::sync::mpsc::Sender<String>,
    abort_rx: tokio::sync::oneshot::Receiver<AbortReason>,
}

pub struct Receiver {
    msg_rx: tokio::sync::mpsc::Receiver<String>,
    abort_tx: tokio::sync::oneshot::Sender<AbortReason>,
}

pub fn channel() -> (Sender, Receiver) {
    // Channel for sending messages to the client
    let (msg_tx, msg_rx) = tokio::sync::mpsc::channel(1024);
    // Channel for sending an abort message to the server
    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel();

    (Sender { msg_tx, abort_rx }, Receiver { msg_rx, abort_tx })
}

impl Sender {
    /// If this function returns Ok(()), the message was sent successfully.
    /// If it returns an error, the tool should abort - the client might have
    /// crashed, requested an abort or the connection was closed.
    pub fn send(&mut self, msg: String) -> Result<(), AbortReason> {
        self.msg_tx
            .blocking_send(msg)
            .map_err(|_| AbortReason::ConnectionError)?;

        use tokio::sync::oneshot::error::TryRecvError;
        match self.abort_rx.try_recv() {
            Ok(reason) => Err(reason),
            Err(err) => match err {
                TryRecvError::Empty => Ok(()),
                TryRecvError::Closed => Err(AbortReason::ConnectionError),
            },
        }
    }
}

impl Receiver {
    pub async fn recv(&mut self) -> Option<String> {
        self.msg_rx.recv().await
    }

    /// Next time the tool calls Sender::send() it will recieve the abort reason
    pub fn abort(self, reason: AbortReason) {
        // Ignore error: if we can't send, the tool probably has quit already
        let _ = self.abort_tx.send(reason);
    }
}

pub enum AbortReason {
    RequestedByClient,
    ConnectionError,
    WebSocketError,
}

impl From<AbortReason> for String {
    fn from(value: AbortReason) -> Self {
        let reason = match value {
            AbortReason::RequestedByClient => "RequestedByClient",
            AbortReason::ConnectionError => "ConnectionError",
            AbortReason::WebSocketError => "WebSocketError",
        };
        format!("Tool was asked to abort: {reason}")
    }
}
