use agent::graph::models::StreamWrapper;
use futures_util::task::{Context, Poll};
use futures_util::Stream;
use pin_project_lite::pin_project;
use providers::models::{ContentBlockStartData, ContentDelta, StreamEvent};
use std::pin::Pin;

/// A stream wrapper implementation that prints text events to the terminal
pub struct CliStreamWrapper;

impl StreamWrapper for CliStreamWrapper {
    fn wrap<'a>(
        &'a self,
        stream: Pin<Box<dyn Stream<Item = anyhow::Result<StreamEvent>> + Send + 'a>>,
    ) -> Pin<Box<dyn Stream<Item = anyhow::Result<StreamEvent>> + Send + 'a>> {
        Box::pin(CliStream { inner: stream })
    }
}

// Use pin_project to safely project to the inner field
pin_project! {
    /// A stream that wraps another stream and prints text events to the terminal
    pub struct CliStream<S> {
        #[pin]
        inner: S,
    }
}

impl<S> Stream for CliStream<S>
where
    S: Stream<Item = anyhow::Result<StreamEvent>> + Send,
{
    type Item = anyhow::Result<StreamEvent>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<<Self as Stream>::Item>> {
        // Use the projected inner stream - this is safe because of pin_project
        let this = self.project();

        // Poll the inner stream
        match this.inner.poll_next(cx) {
            Poll::Ready(Some(Ok(event))) => {
                // Match on events of interest for printing to terminal
                match &event {
                    StreamEvent::ContentBlockStart { content_block, .. } => {
                        if let ContentBlockStartData::Text { text } = content_block {
                            if !text.is_empty() {
                                // Print initial text content to terminal
                                print!("{}", text);
                                // Ensure the output is flushed immediately
                                let _ = std::io::Write::flush(&mut std::io::stdout());
                            }
                        }
                    }
                    StreamEvent::ContentBlockDelta { delta, .. } => {
                        if let ContentDelta::TextDelta { text } = delta {
                            if !text.is_empty() {
                                // Print text delta to terminal
                                print!("{}", text);
                                // Ensure the output is flushed immediately
                                let _ = std::io::Write::flush(&mut std::io::stdout());
                            }
                        }
                    }
                    _ => {}
                }

                // Return the event unchanged
                Poll::Ready(Some(Ok(event)))
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => {
                // End of stream, add a newline for better formatting
                println!();
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
