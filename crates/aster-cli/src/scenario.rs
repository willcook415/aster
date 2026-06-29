use aster_core::{
    EngineCommand, OrderId, OrderRequest, OrderType, ParticipantId, PriceTicks, Quantity, Side,
};

#[derive(Debug, Clone)]
pub struct Scenario {
    pub name: &'static str,
    pub description: &'static str,
    pub notes: &'static [&'static str],
    pub accounting: Option<ScenarioAccounting>,
    pub commands: Vec<EngineCommand>,
}

#[derive(Debug, Clone, Copy)]
pub enum ScenarioAccounting {
    MarketOrder { accepted_quantity: u64 },
}

pub fn scenarios() -> Vec<Scenario> {
    vec![
        fifo_partial_fill(),
        market_sweep(),
        cancellation(),
        mixed_session(),
    ]
}

pub fn find_scenario(name: &str) -> Option<Scenario> {
    scenarios()
        .into_iter()
        .find(|scenario| scenario.name == name)
}

fn fifo_partial_fill() -> Scenario {
    Scenario {
        name: "fifo-partial-fill",
        description: "Same-price FIFO: oldest ask fills first, then the next ask is partial.",
        notes: &[
            "The oldest ask at price 100 filled completely before the next ask was partially filled.",
        ],
        accounting: None,
        commands: vec![
            limit(1, Side::Sell, 100, 10),
            limit(2, Side::Sell, 100, 20),
            limit(3, Side::Buy, 100, 15),
        ],
    }
}

fn market_sweep() -> Scenario {
    Scenario {
        name: "market-sweep",
        description:
            "Oversized market buy sweeps two ask levels at resting prices; remainder expires.",
        notes: &[
            "The market buy consumed all available asks from best to worst price.",
            "Trades used the resting ask prices; quantity without liquidity expired and did not rest.",
        ],
        accounting: Some(ScenarioAccounting::MarketOrder {
            accepted_quantity: 15,
        }),
        commands: vec![
            limit(1, Side::Sell, 100, 5),
            limit(2, Side::Sell, 101, 7),
            market(3, Side::Buy, 15),
        ],
    }
}

fn cancellation() -> Scenario {
    Scenario {
        name: "cancellation",
        description:
            "Wrong-owner rejection, successful cancellation, repeated rejection, stable allocation.",
        notes: &[
            "The wrong participant was rejected without removing the resting order.",
            "The owner then cancelled it; a repeated cancellation was rejected, and the next order received ID 2 and sequence 2.",
        ],
        accounting: None,
        commands: vec![
            limit(1, Side::Buy, 99, 10),
            cancel(1, 99),
            cancel(1, 1),
            cancel(1, 1),
            limit(2, Side::Sell, 101, 5),
        ],
    }
}

fn mixed_session() -> Scenario {
    Scenario {
        name: "mixed-session",
        description:
            "Passive liquidity, crossing limit, market trade, cancellation, and rejection.",
        notes: &[
            "Passive orders established both sides before a crossing limit order traded at resting ask prices.",
            "A market sell traded against the best bid, one bid was cancelled, and a wrong-owner cancellation was rejected.",
            "Replay verification confirms the event stream and final full snapshot.",
        ],
        accounting: None,
        commands: vec![
            limit(1, Side::Sell, 101, 10),
            limit(2, Side::Sell, 102, 15),
            limit(3, Side::Buy, 99, 8),
            limit(4, Side::Buy, 98, 12),
            limit(5, Side::Buy, 102, 18),
            market(6, Side::Sell, 5),
            cancel(4, 4),
            cancel(2, 4),
        ],
    }
}

fn limit(participant: u64, side: Side, price: u64, quantity: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant),
        side,
        OrderType::Limit {
            price: PriceTicks::new(price).expect("built-in scenario price must be positive"),
        },
        Quantity::new(quantity).expect("built-in scenario quantity must be positive"),
    ))
}

fn market(participant: u64, side: Side, quantity: u64) -> EngineCommand {
    EngineCommand::submit_order(OrderRequest::new(
        ParticipantId::new(participant),
        side,
        OrderType::Market,
        Quantity::new(quantity).expect("built-in scenario quantity must be positive"),
    ))
}

fn cancel(order_id: u64, participant: u64) -> EngineCommand {
    EngineCommand::cancel_order(OrderId::new(order_id), ParticipantId::new(participant))
}

#[cfg(test)]
mod tests {
    use super::{find_scenario, scenarios};
    use aster_core::SessionRecord;

    #[test]
    fn catalog_has_expected_unique_names() {
        let scenarios = scenarios();
        let names: Vec<_> = scenarios.iter().map(|scenario| scenario.name).collect();

        assert_eq!(
            names,
            vec![
                "fifo-partial-fill",
                "market-sweep",
                "cancellation",
                "mixed-session",
            ]
        );
    }

    #[test]
    fn every_built_in_scenario_verifies() {
        for scenario in scenarios() {
            let session = SessionRecord::from_commands(scenario.commands);
            assert_eq!(
                session.verify(),
                Ok(()),
                "scenario {} failed",
                scenario.name
            );
        }
    }

    #[test]
    fn lookup_rejects_unknown_name() {
        assert!(find_scenario("unknown").is_none());
    }
}
