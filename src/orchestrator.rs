use crate::model;
use model::{OrchestratorEvent, ExchangeCmd, ExchangeOrder, OrderStatus, Side, State, OrchestratorEvent::*, PriceType::*, ExchangeCmd::*};
use uuid::Uuid;


/// DSL for converting OrchestratorEvent's and state into state and optional ExchangeCmd, eg.
pub fn process_event<'a>(event: &'a OrchestratorEvent, state: &'a mut State) -> Option<ExchangeCmd<'a>> {  // probably need dyn...
    state.has_refreshed = false;
    match event {
        UpQty => {
            state.has_refreshed = true;
            state.qty += state.qty_increment;
            None
        }
        DownQty if state.qty <= state.qty_increment => None,  // ignore, cannot decrease < 0
        DownQty => {
            state.has_refreshed = true;
            state.qty -= state.qty_increment;
            None
        }
        RotateOrderType => {
            state.rotate_order_type();
            state.has_refreshed = true;
            None
        }
        NewBid(bid) if state.bid == *bid => None,
        NewBid(bid) => {
            state.has_refreshed = true;
            state.bid = *bid;
            None
        }
        NewAsk(ask) if state.ask == *ask => None,
        NewAsk(ask) => {
            state.has_refreshed = true;
            state.ask = *ask;
            None
        }
        Buy(_) | Sell(_) if state.bid < 0.0 || state.ask < 0.0 => {
            state.has_refreshed = true;
            state.status = "Won't trade till ask/bid populated!".to_string();
            None
        }
        Buy(_) | Sell(_) if state.order.is_some() => {
            state.has_refreshed = true;
            state.status = format!("Won't trade whilst another trade {} is in force!", state.order.as_ref().unwrap().cl_ord_id);
            None
        }
        Buy(price_type) => {
            log::info!("Buy: {:?}, state: {:?}", event, state);
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                Bid => state.bid,
                Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("New buy order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: Some(state.qty), price: Some(price), side: Some(Side::Buy), ord_type: Some(state.order_type()) };
            state.order = Some(new_order.clone());
            Some(IssueOrder(new_order))
        }
        Sell(price_type) => {
            log::info!("Sell: {:?}, state: {:?}", event, state);
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                Bid => state.bid,
                Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("New sell order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: Some(state.qty), price: Some(price), side: Some(Side::Sell), ord_type: Some(state.order_type()) };
            state.order = Some(new_order.clone());
            Some(IssueOrder(new_order))
        }
        UpdateOrder(order) if state.order.as_ref().map_or_else(|| false, |x| x.cl_ord_id == order.cl_ord_id) => {
            log::info!("UpdateOrder: {:?}", event);
            let curr_order = state.order.as_ref().unwrap();
            let side = order.side.unwrap_or_else(|| curr_order.side.unwrap());
            let ord_type = order.ord_type.unwrap_or_else(|| curr_order.ord_type.unwrap());
            let qty = order.qty.unwrap_or_else(|| curr_order.qty.unwrap());
            let price = order.price.unwrap_or_else(|| curr_order.price.unwrap());
            match order.ord_status {
                OrderStatus::Canceled => {
                    state.status = format!("Canceled {} {} order: {}", side, ord_type, order.cl_ord_id);
                    state.order = None;
                }
                OrderStatus::Filled => {
                    state.status = format!("Filled {} {} order: {} of {} @ {}", side, ord_type, order.cl_ord_id, qty, price);
                    state.order = None;
                }
                _ => {
                    state.status = format!("Updated {} {} order: {} of {:?} @ {:?}", side, ord_type, order.cl_ord_id, qty, price);
                    state.order = Some(ExchangeOrder { ord_status: order.ord_status, .. order.clone() });
                }
            };
            state.has_refreshed = true;
            None
        }
        UpdateOrder(order) => {
            if order.cl_ord_id == "" {
                log::info!("Ignoring external order update: {:?}, given current state: {:?}", order, state);
            } else {
                log::info!("Ignoring update of order that has been potentially Filled/Cancelled, order {:?}, given current state: {:?}", order, state);
            }
            None

        }
        CancelLast if state.order.as_ref().map_or_else(|| false, |x| x.ord_status == OrderStatus::New || x.ord_status == OrderStatus::NotYetIssued || x.ord_status == OrderStatus::PartiallyFilled) => {
            let order = state.order.as_ref().unwrap();
            state.has_refreshed = true;
            state.status = format!("Issued order cancel: {}", order.cl_ord_id);
            Some(CancelOrder(&order.cl_ord_id ))
        }
        CancelLast => {
            state.has_refreshed = true;
            state.status = format!("No order active, ignoring cancel!");
            None
        }
        NewStatus(status) => {
            state.status = status.to_string();
            None
        }
        _ => None
    }
}
