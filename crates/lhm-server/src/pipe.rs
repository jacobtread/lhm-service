use futures_util::{SinkExt, StreamExt};
use interprocess::os::windows::named_pipe::{pipe_mode, tokio::DuplexPipeStream};
use lhm_shared::codec::{LHMFrame, LHMFrameCodec};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, ready},
};
use tokio::sync::mpsc;
use tokio_util::codec::Framed;

pub type Pipe = Framed<DuplexPipeStream<pipe_mode::Bytes>, LHMFrameCodec>;

pub struct PipeFuture {
    /// Pipe we are acting upon
    pipe: Pipe,
    /// Channel for processing received messages
    inbound_tx: Option<mpsc::UnboundedSender<PipeMessage>>,
    /// Channel for outbound messages
    outbound_rx: mpsc::UnboundedReceiver<PipeMessage>,
    /// Currently accepted outbound item, ready to be written
    buffered_item: Option<PipeMessage>,
}

pub type PipeTx = mpsc::UnboundedSender<PipeMessage>;
pub type PipeRx = mpsc::UnboundedReceiver<PipeMessage>;
pub type PipeMessage = LHMFrame;

impl PipeFuture {
    pub fn new(pipe: Pipe) -> (PipeFuture, PipeRx, PipeTx) {
        let (inbound_tx, inbound_rx) = mpsc::unbounded_channel();
        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel();

        let future = PipeFuture {
            pipe,
            inbound_tx: Some(inbound_tx),
            outbound_rx,
            buffered_item: None,
        };

        (future, inbound_rx, outbound_tx)
    }
}

impl Future for PipeFuture {
    type Output = Result<(), std::io::Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Read messages from the socket
        while let Some(inbound_tx) = &mut this.inbound_tx {
            let msg = match this.pipe.poll_next_unpin(cx) {
                Poll::Ready(Some(result)) => result?,

                // Socket is already closed, cannot ready anything more
                Poll::Ready(None) => return Poll::Ready(Ok(())),

                // Nothing yet, move onto the write polling
                Poll::Pending => break,
            };

            if inbound_tx.send(msg).is_err() {
                // Receiver for messages has dropped, stop reading messages
                this.inbound_tx.take();
                break;
            }
        }

        // Write messages to the pipe
        loop {
            if this.buffered_item.is_some() {
                // Wait until the pipe is ready
                ready!(this.pipe.poll_ready_unpin(cx))?;

                // Take the buffered item
                let packet = this
                    .buffered_item
                    .take()
                    .expect("unexpected write state without a packet");

                // Write the buffered item
                this.pipe.start_send_unpin(packet)?;
            }

            match this.outbound_rx.poll_recv(cx) {
                // Message ready, set the buffered item
                Poll::Ready(Some(item)) => {
                    this.buffered_item = Some(item);
                }
                // All message senders have dropped, close the pipe
                Poll::Ready(None) => {
                    ready!(this.pipe.poll_close_unpin(cx))?;
                    return Poll::Ready(Ok(()));
                }
                Poll::Pending => {
                    // Failed to flush the pipe
                    ready!(this.pipe.poll_flush_unpin(cx))?;
                    return Poll::Pending;
                }
            }
        }
    }
}
