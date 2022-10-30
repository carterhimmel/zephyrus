use std::{future::Future, task::{Context, Poll}};
use std::pin::Pin;
use tokio::sync::oneshot::{Sender, Receiver, channel};
use crate::{twilight_exports::Interaction};

pub(crate) fn new_pair<F>(fun: F) -> (WaiterWaker, InteractionWaiter)
where
    F: Fn(&Interaction) -> bool + Send + 'static
{
    let (sender, receiver) = channel();

    (
        WaiterWaker {
            predicate: Box::new(fun),
            sender
        },
        InteractionWaiter {
            receiver
        }
    )
}

pub struct InteractionWaiter {
    receiver: Receiver<Interaction>
}

impl Future for InteractionWaiter {
    type Output = Result<Interaction, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(cx)
            .map_err(|e| {
                Box::new(e) as Box<_>
            })
    }
}

pub struct WaiterWaker {
    pub predicate: Box<dyn Fn(&Interaction) -> bool + Send + 'static>,
    pub sender: Sender<Interaction>
}

impl WaiterWaker {
    pub fn check(&self, interaction: &Interaction) -> bool {
        (self.predicate)(interaction)
    }

    pub fn wake(self, interaction: Interaction) {
        let _ = self.sender.send(interaction);
    }
}
