use bitmex_warrior::model::*;
use bitmex_warrior::orchestrator::process_event;

#[cfg(test)]

#[test]
fn test_process_events__up_down_qty() {
    let state = &mut State::new(10.0, 1.0);
    let effect1 = process_event(&OrchestratorEvent::UpQty, &mut *state);
    assert_eq!(effect1, None::<ExchangeCmd<'_>>);
    assert_eq!(state.qty, 11.0);
    process_event(&OrchestratorEvent::DownQty, &mut *state);
    let effect2 = process_event(&OrchestratorEvent::DownQty, state);
    assert_eq!(effect2, None::<ExchangeCmd<'_>>);
    assert_eq!(state.qty, 9.0);
}