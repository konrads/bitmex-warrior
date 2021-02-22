use crate::model;
use model::{OrchestratorEvent, ExchangeAction, ExchangeOrder, OrderStatus, Side, State, OrchestratorEvent::*, PriceType::*, ExchangeAction::*};
use uuid::Uuid;


pub fn process_event<'a>(event: &'a OrchestratorEvent, state: &'a mut State) -> Option<ExchangeAction<'a>> {  // probably need dyn...
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
        Buy(price_type) => {
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                Bid => state.bid,
                Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("New buy order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: state.qty, price: price, side: Side::Buy, ord_type: state.order_type() };
            Some(IssueOrder(new_order))
        }
        Sell(price_type) => {
            let cl_ord_id = Uuid::new_v4().to_string();
            let price = match *price_type {
                Bid => state.bid,
                Ask => state.ask,
            };
            state.has_refreshed = true;
            state.status = format!("New sell order {} of {} @ {}", cl_ord_id, state.qty, price);
            let new_order = ExchangeOrder { cl_ord_id: cl_ord_id, ord_status: OrderStatus::NotYetIssued, qty: state.qty, price: price, side: Side::Sell, ord_type: state.order_type() };
            Some(IssueOrder(new_order))
        }
        UpdateOrder(order) if order.ord_status == OrderStatus::Canceled && order.cl_ord_id != "" => {
            state.has_refreshed = true;
            state.status = format!("Canceled order {} {}", order.ord_type, order.cl_ord_id);
            state.order = Some(ExchangeOrder { ord_status: OrderStatus::Canceled, .. order.clone() });
            None
        }
        UpdateOrder(order) if order.ord_status == OrderStatus::Filled && order.cl_ord_id != "" => {
            state.has_refreshed = true;
            state.status = format!("Filled order {} {} of {} @ {}", order.ord_type, order.cl_ord_id, order.qty, order.price);
            state.order = Some(ExchangeOrder { ord_status: OrderStatus::Filled, price: order.price, .. order.clone() });;
            None
        }
        UpdateOrder(order) if order.cl_ord_id != "" => {
            state.has_refreshed = true;
            state.status = format!("Updated order {} {} of {:?} @ {:?}", order.ord_type, order.cl_ord_id, order.qty, order.price);
            state.order = Some((*order).clone());
            None
        }
        CancelLast if state.order.is_some() => {
            let order = state.order.as_ref().unwrap();
            state.has_refreshed = true;
            state.status = format!("Cancelling order: {}", order.cl_ord_id);
            Some(CancelOrder(&order.cl_ord_id ))
        }
        NewStatus(status) => {
            state.status = status.to_string();
            None
        }
        _ => None
    }
}
