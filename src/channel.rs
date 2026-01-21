pub struct Channel {
    // TODO: remove pub
    pub msg_tx: tokio::sync::mpsc::Sender<String>,
    pub abort_rx: tokio::sync::oneshot::Receiver<()>,
}

//TODO:  change to create channels in function and return sender reciever halves
impl Channel {

    pub fn send(&mut self, msg: String) -> Result<(), ShutdownReason> {
        self.msg_tx
            .blocking_send(msg)
            .map_err(|_| ShutdownReason::ConnectionError)?;

        use tokio::sync::oneshot::error::TryRecvError;
        match self.abort_rx.try_recv() {
            Ok(()) => Err(ShutdownReason::RequestedByClient),
            Err(err) => match err {
                TryRecvError::Empty => Ok(()),
                TryRecvError::Closed => Err(ShutdownReason::ConnectionError),
            },
        }
    }
}

pub enum ShutdownReason {
    Timeout,
    RequestedByClient,
    ConnectionError,
}

impl From<ShutdownReason> for String {
    fn from(value: ShutdownReason) -> Self {
        let reason = match value {
            ShutdownReason::Timeout => "Timeout",
            ShutdownReason::RequestedByClient => "RequestedByClient",
            ShutdownReason::ConnectionError => "ConnectionError",
        };
        format!("Tool was forced to stop: {reason}")
    }
}
