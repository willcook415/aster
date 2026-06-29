use std::cmp::Reverse;

use aster_core::{
    AcceptedOrder, AsterEngine, AsterError, EngineCommand, EngineEvent, EngineSnapshot, OrderId,
    OrderRequest, OrderType, ParticipantId, PriceLevelSnapshot, PriceTicks, Quantity,
    SequenceNumber, Side,
};

#[derive(Debug)]
struct ReferenceModel {
    resting_orders: Vec<AcceptedOrder>,
    next_order_id: u64,
    next_sequence_number: u64,
}

impl ReferenceModel {
    fn new() -> Self {
        Self {
            resting_orders: Vec::new(),
            next_order_id: 1,
            next_sequence_number: 1,
        }
    }

    fn process_command(&mut self, command: EngineCommand) -> Vec<EngineEvent> {
        match command {
            EngineCommand::SubmitOrder(request) => self.submit(request),
            EngineCommand::CancelOrder {
                order_id,
                participant_id,
            } => self.cancel(order_id, participant_id),
        }
    }

    fn submit(&mut self, request: OrderRequest) -> Vec<EngineEvent> {
        let incoming = AcceptedOrder::new(
            OrderId::new(self.next_order_id),
            SequenceNumber::new(self.next_sequence_number),
            request,
        );
        self.next_order_id += 1;
        self.next_sequence_number += 1;

        let mut events = vec![EngineEvent::OrderAccepted { order: incoming }];
        let mut remaining = incoming.quantity.as_u64();

        while remaining > 0 {
            let Some(resting_index) = self.best_match_index(incoming) else {
                break;
            };
            let resting = self.resting_orders[resting_index];
            let fill = remaining.min(resting.quantity.as_u64());
            let OrderType::Limit { price } = resting.order_type else {
                panic!("reference model must never rest market orders");
            };

            events.push(EngineEvent::TradeExecuted {
                resting_order_id: resting.order_id,
                incoming_order_id: incoming.order_id,
                price,
                quantity: quantity(fill),
            });

            if fill == resting.quantity.as_u64() {
                self.resting_orders.remove(resting_index);
            } else {
                self.resting_orders[resting_index].quantity =
                    quantity(resting.quantity.as_u64() - fill);
            }
            remaining -= fill;
        }

        if remaining > 0 && matches!(incoming.order_type, OrderType::Limit { .. }) {
            let mut remainder = incoming;
            remainder.quantity = quantity(remaining);
            self.resting_orders.push(remainder);
        }

        events
    }

    fn best_match_index(&self, incoming: AcceptedOrder) -> Option<usize> {
        self.resting_orders
            .iter()
            .enumerate()
            .filter(|(_, resting)| resting.side != incoming.side && crosses(incoming, **resting))
            .min_by_key(|(_, resting)| {
                let OrderType::Limit { price } = resting.order_type else {
                    panic!("reference model must never rest market orders");
                };
                let price_priority = match incoming.side {
                    Side::Buy => (price.as_u64(), Reverse(0)),
                    Side::Sell => (0, Reverse(price.as_u64())),
                };
                (price_priority, resting.sequence_number.as_u64())
            })
            .map(|(index, _)| index)
    }

    fn cancel(&mut self, order_id: OrderId, participant_id: ParticipantId) -> Vec<EngineEvent> {
        let Some(index) = self
            .resting_orders
            .iter()
            .position(|order| order.order_id == order_id)
        else {
            return vec![EngineEvent::CancelRejected {
                order_id,
                participant_id,
                reason: AsterError::OrderNotFound,
            }];
        };

        if self.resting_orders[index].participant_id != participant_id {
            return vec![EngineEvent::CancelRejected {
                order_id,
                participant_id,
                reason: AsterError::ParticipantMismatch,
            }];
        }

        self.resting_orders.remove(index);
        vec![EngineEvent::OrderCancelled {
            order_id,
            participant_id,
        }]
    }

    fn snapshot(&self) -> EngineSnapshot {
        let bid_levels = self.levels(Side::Buy);
        let ask_levels = self.levels(Side::Sell);

        EngineSnapshot {
            best_bid: bid_levels.first().map(|level| level.price),
            best_ask: ask_levels.first().map(|level| level.price),
            bid_level_count: bid_levels.len(),
            ask_level_count: ask_levels.len(),
            total_resting_quantity: self
                .resting_orders
                .iter()
                .map(|order| order.quantity.as_u64())
                .sum(),
            bid_levels,
            ask_levels,
            next_order_id: OrderId::new(self.next_order_id),
            next_sequence_number: SequenceNumber::new(self.next_sequence_number),
        }
    }

    fn levels(&self, side: Side) -> Vec<PriceLevelSnapshot> {
        let mut prices: Vec<PriceTicks> = self
            .resting_orders
            .iter()
            .filter(|order| order.side == side)
            .map(limit_price)
            .collect();
        prices.sort_unstable();
        prices.dedup();
        if side == Side::Buy {
            prices.reverse();
        }

        prices
            .into_iter()
            .map(|price| {
                let mut orders: Vec<AcceptedOrder> = self
                    .resting_orders
                    .iter()
                    .filter(|order| order.side == side && limit_price(order) == price)
                    .copied()
                    .collect();
                orders.sort_by_key(|order| order.sequence_number);
                PriceLevelSnapshot { price, orders }
            })
            .collect()
    }
}

#[test]
fn deterministic_sequences_match_independent_reference_model() {
    for sequence in deterministic_sequences() {
        assert_sequence_matches(sequence.name, sequence.commands);
    }
}

fn assert_sequence_matches(name: &str, commands: Vec<EngineCommand>) {
    let mut engine = AsterEngine::new();
    let mut model = ReferenceModel::new();

    for (index, command) in commands.into_iter().enumerate() {
        let engine_events = engine.process_command(command);
        let model_events = model.process_command(command);
        let engine_snapshot = engine.snapshot();
        let model_snapshot = model.snapshot();
        let context = format!(
            "sequence={name} command_index={index} command={command:?}\n\
             engine_output={engine_events:#?}\nmodel_output={model_events:#?}\n\
             engine_book={engine_snapshot:#?}\nmodel_book={model_snapshot:#?}"
        );

        assert_eq!(engine_events, model_events, "{context}");
        assert_eq!(engine_snapshot, model_snapshot, "{context}");
    }
}

struct CommandSequence {
    name: &'static str,
    commands: Vec<EngineCommand>,
}

fn deterministic_sequences() -> Vec<CommandSequence> {
    vec![
        CommandSequence {
            name: "passive-book-build-up",
            commands: vec![
                limit(1, Side::Buy, 99, 8),
                limit(2, Side::Buy, 100, 5),
                limit(3, Side::Buy, 99, 7),
                limit(4, Side::Sell, 103, 6),
                limit(5, Side::Sell, 102, 9),
                limit(6, Side::Sell, 103, 4),
            ],
        },
        CommandSequence {
            name: "buy-crosses-multiple-ask-levels",
            commands: vec![
                limit(1, Side::Sell, 100, 3),
                limit(2, Side::Sell, 101, 4),
                limit(3, Side::Sell, 102, 5),
                limit(4, Side::Buy, 101, 10),
            ],
        },
        CommandSequence {
            name: "sell-crosses-multiple-bid-levels",
            commands: vec![
                limit(1, Side::Buy, 102, 3),
                limit(2, Side::Buy, 101, 4),
                limit(3, Side::Buy, 100, 5),
                limit(4, Side::Sell, 101, 10),
            ],
        },
        CommandSequence {
            name: "fifo-same-price",
            commands: vec![
                limit(1, Side::Sell, 100, 4),
                limit(2, Side::Sell, 100, 5),
                limit(3, Side::Sell, 100, 6),
                market(4, Side::Buy, 10),
            ],
        },
        CommandSequence {
            name: "partial-resting-order-fill",
            commands: vec![
                limit(1, Side::Buy, 100, 12),
                market(2, Side::Sell, 5),
                limit(3, Side::Sell, 100, 2),
            ],
        },
        CommandSequence {
            name: "incoming-partial-fill-then-rest",
            commands: vec![
                limit(1, Side::Sell, 100, 4),
                limit(2, Side::Sell, 101, 3),
                limit(3, Side::Buy, 101, 10),
            ],
        },
        CommandSequence {
            name: "market-buy-sweep-and-expiry",
            commands: vec![
                limit(1, Side::Sell, 100, 3),
                limit(2, Side::Sell, 100, 2),
                limit(3, Side::Sell, 102, 4),
                market(4, Side::Buy, 12),
            ],
        },
        CommandSequence {
            name: "market-sell-sweep-and-expiry",
            commands: vec![
                limit(1, Side::Buy, 102, 3),
                limit(2, Side::Buy, 102, 2),
                limit(3, Side::Buy, 100, 4),
                market(4, Side::Sell, 12),
            ],
        },
        CommandSequence {
            name: "cancellation-before-matching",
            commands: vec![
                limit(1, Side::Sell, 100, 8),
                cancel(1, 1),
                market(2, Side::Buy, 8),
            ],
        },
        CommandSequence {
            name: "failed-cancellations",
            commands: vec![
                limit(7, Side::Buy, 99, 8),
                cancel(1, 8),
                cancel(999, 7),
                cancel(1, 7),
                cancel(1, 7),
            ],
        },
        CommandSequence {
            name: "mixed-60-command-session",
            commands: mixed_session(),
        },
    ]
}

fn mixed_session() -> Vec<EngineCommand> {
    let mut commands = Vec::new();

    for index in 0..10 {
        commands.push(limit(100 + index, Side::Buy, 95 + (index % 5), 3 + index));
        commands.push(limit(200 + index, Side::Sell, 105 + (index % 5), 4 + index));
    }

    commands.extend([
        limit(300, Side::Buy, 106, 18),
        limit(301, Side::Sell, 98, 16),
        market(302, Side::Buy, 21),
        market(303, Side::Sell, 19),
        cancel(3, 102),
        cancel(4, 999),
        cancel(500, 1),
        limit(304, Side::Sell, 101, 7),
        limit(305, Side::Buy, 101, 11),
        market(306, Side::Buy, 5),
    ]);

    for index in 0..15 {
        let side = if index % 2 == 0 {
            Side::Buy
        } else {
            Side::Sell
        };
        let price = if side == Side::Buy {
            97 + (index % 4)
        } else {
            103 + (index % 4)
        };
        commands.push(limit(400 + index, side, price, 2 + (index % 6)));
        if index % 3 == 0 {
            commands.push(market(500 + index, opposite(side), 1 + (index % 4)));
        }
    }

    commands.extend([
        cancel(1, 100),
        cancel(2, 200),
        market(600, Side::Buy, 13),
        market(601, Side::Sell, 12),
        limit(602, Side::Buy, 110, 20),
        limit(603, Side::Sell, 90, 17),
        cancel(9999, 42),
        limit(604, Side::Buy, 99, 6),
        limit(605, Side::Sell, 104, 6),
        market(606, Side::Buy, 3),
    ]);

    assert!((30..=100).contains(&commands.len()));
    commands
}

fn crosses(incoming: AcceptedOrder, resting: AcceptedOrder) -> bool {
    let OrderType::Limit {
        price: resting_price,
    } = resting.order_type
    else {
        panic!("reference model must never rest market orders");
    };

    match incoming.order_type {
        OrderType::Market => true,
        OrderType::Limit {
            price: incoming_price,
        } => match incoming.side {
            Side::Buy => incoming_price >= resting_price,
            Side::Sell => incoming_price <= resting_price,
        },
    }
}

fn limit_price(order: &AcceptedOrder) -> PriceTicks {
    match order.order_type {
        OrderType::Limit { price } => price,
        OrderType::Market => panic!("reference model must never rest market orders"),
    }
}

fn limit(participant_id: u64, side: Side, price_ticks: u64, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Limit {
            price: price(price_ticks),
        },
        quantity(units),
    ))
}

fn market(participant_id: u64, side: Side, units: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant_id),
        side,
        OrderType::Market,
        quantity(units),
    ))
}

fn cancel(order_id: u64, participant_id: u64) -> EngineCommand {
    EngineCommand::cancel_order(OrderId::new(order_id), ParticipantId::new(participant_id))
}

fn opposite(side: Side) -> Side {
    match side {
        Side::Buy => Side::Sell,
        Side::Sell => Side::Buy,
    }
}

fn price(value: u64) -> PriceTicks {
    PriceTicks::new(value).expect("test price must be positive")
}

fn quantity(value: u64) -> Quantity {
    Quantity::new(value).expect("test quantity must be positive")
}
