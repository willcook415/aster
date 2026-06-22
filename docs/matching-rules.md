# Matching Rules

## Scope

Aster currently supports limit-order and market-order matching against resting liquidity. Cancellation execution, replay, persistence, and serialization remain out of scope.

Non-crossing limit orders rest on the appropriate side of the book. Crossing limit orders are accepted, matched, and any unfilled remainder rests.

Market orders are accepted and match immediately against available opposite-side liquidity. Any unfilled market quantity expires and never rests.

## Price-Time Priority

Price priority:

- Incoming buys match the lowest ask price first.
- Incoming sells match the highest bid price first.

Time priority:

- Within a price level, older resting orders fill before newer resting orders.
- Partial fills keep the resting order at the front with its original `OrderId`, `ParticipantId`, `Side`, `OrderType`, and `SequenceNumber`.

Matching does not use wall-clock timestamps.

## Crossing Rules

A buy limit crosses while the best ask exists and the buy price is greater than or equal to the best ask.

A sell limit crosses while the best bid exists and the sell price is less than or equal to the best bid.

The trade price is always the resting order's limit price.

## Market Orders

Market buys consume asks from lowest price upward. Market sells consume bids from highest price downward.

Market orders with no available opposite-side liquidity still emit `OrderAccepted`, consume one engine-assigned order ID and sequence number, emit no trades, and do not rest.

## Price and Quantity Representation

Prices use integer ticks via `PriceTicks`. Quantities use integer units via `Quantity`. Floating-point prices and quantities are out of scope.

## Event Semantics

Accepted limit and market orders emit `OrderAccepted` first, followed by one `TradeExecuted` event per fill.

Cancellation commands still emit `CancelRejected` with `CancellationNotImplemented`.
