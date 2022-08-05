#[cfg(all(not(feature = "std"), feature = "cortexm"))]
use ockam_core::println;
#[cfg(all(not(feature = "std"), feature = "cortexm"))]
use ockam::{
    compat::{
        boxed::Box,
        string::String
    }
};
use ockam::{Context, Result, Routed, Worker};

pub struct Echoer;

#[ockam::worker]
impl Worker for Echoer {
    type Context = Context;
    type Message = String;

    async fn handle_message(&mut self, ctx: &mut Context, msg: Routed<String>) -> Result<()> {
        println!("[echoer] Address: {}, Received: {}", ctx.address(), msg);

        // Echo the message body back on its return_route.
        println!("[echoer] Echoing message '{}' back on its return_route: {}",
                 &msg,
                 msg.return_route());

        ctx.send(msg.return_route(), msg.body()).await
    }
}
