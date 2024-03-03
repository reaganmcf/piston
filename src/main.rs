use actix::prelude::*;
use rand::{rngs::ThreadRng, Rng};
use std::{collections::HashMap, time::Duration};

mod models;
use models::*;

impl Actor for Portfolio {
    type Context = Context<Self>;
}

impl Handler<TickData> for Portfolio {
    type Result = ();

    fn handle(&mut self, msg: TickData, _ctx: &mut Self::Context) -> Self::Result {
        println!("got tick data! {:?}", msg);
        
        self.positions.iter_mut().for_each(|p| {
            if p.1.security.id == msg.security.id {
                p.1.unrealized_pnl = msg.price * f64::from(p.1.size);
            }
        })
    }
}

impl Handler<TradeEvent> for Portfolio {
    type Result = ();

    fn handle(&mut self, msg: TradeEvent, ctx: &mut Self::Context) -> Self::Result {
        println!("Got trade message, {:#?}", msg);
        match msg {
            TradeEvent::Open(pos) => {
                self.positions.insert(pos.id, pos);
            }
            TradeEvent::Close(pos_id) => match self.positions.remove(&pos_id) {
                None => panic!("Closing a position that does not exist"),
                Some(p) => {
                    self.pnl += p.unrealized_pnl;
                    println!("Closed position");
                    println!("\tPORTFOLIO STATS: {:#?}", self);
                }
            },
        }
    }
}

struct TickFeed {
    rng: ThreadRng,
    subs: Vec<Addr<Portfolio>>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartTickFeedEvent(Duration);

impl Actor for TickFeed {
    type Context = Context<Self>;
}

impl Handler<StartTickFeedEvent> for TickFeed {
    type Result = ();

    fn handle(&mut self, ev: StartTickFeedEvent, ctx: &mut Self::Context) -> Self::Result {
        println!("Starting tick feed");
        ctx.run_interval(ev.0, |act, _| {
            for subscriber in &act.subs {
                print!("Sending tick data for AAPL...");
                if let Err(e) = subscriber.try_send(TickData {
                    security: AAPL.clone(),
                    price: act.rng.gen_range(100.0f64..200f64),
                }) {
                    eprintln!("Failed to send, {:?}", e);
                };
                println!(" Complete!");
            }
        });
    }
}

struct TradeFeed {
    rng: ThreadRng,
    subs: Vec<Addr<Portfolio>>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct StartTradeFeedeEvent(Duration);

impl Actor for TradeFeed {
    type Context = Context<Self>;
}

impl Handler<StartTradeFeedeEvent> for TradeFeed {
    type Result = ();

    fn handle(&mut self, ev: StartTradeFeedeEvent, ctx: &mut Self::Context) -> Self::Result {
        println!("Starting trade feed");

        ctx.run_later(ev.0, |act, _| {
            for sub in &act.subs {
                let buy = TradeEvent::Open(Position {
                    id: unsafe { NEXT_POSITION_ID },
                    security: AAPL.clone(),
                    size: 10,
                    unrealized_pnl: 0f64,
                });
                unsafe {
                    NEXT_POSITION_ID += 1;
                }

                sub.try_send(buy).expect("failed to send");
                }
        });

        ctx.run_later(ev.0 + ev.0, |act, _| {
            for sub in &act.subs {
                let sell = TradeEvent::Close(unsafe { NEXT_POSITION_ID - 1 });
                unsafe {
                    NEXT_POSITION_ID += 1;
                }

                sub.try_send(sell).expect("failed to send");
            }
        });
    }
}

fn main() {
    let system = System::new();

    system.block_on(async {
        let portfolio = Portfolio {
            code: String::from("RMCF"),
            positions: HashMap::default(),
            pnl: 0f64,
        }
        .start();

        let tick_feed = TickFeed {
            rng: rand::thread_rng(),
            subs: vec![portfolio.clone()],
        }
        .start();

        tick_feed
            .send(StartTickFeedEvent(Duration::from_secs(1)))
            .await
            .expect("Failed to send event to start tick feed");

        let trade_feed = TradeFeed {
            rng: rand::thread_rng(),
            subs: vec![portfolio.clone()],
        }
        .start();

        trade_feed
            .send(StartTradeFeedeEvent(Duration::from_secs(3)))
            .await
            .expect("Failed to start trade feed");

        tick_feed
    });

    system.run().expect("Failed to run the system");
}
