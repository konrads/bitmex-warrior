use bitmex_warrior::model::*;
use bitmex_warrior::render::render_state;

#[cfg(test)]

#[test]
fn test_render_state() {
    let state = &mut State::new(10.0, 1.0);
    let rendered = render_state("HELLO!", state);
    let expected = format!("HELLO!\r
\r
BID: -1.00000 / ASK: -1.00000\r
QTY: 10.00000\r
ORDER TYPE: Limit\r
STATUS: ");
    assert_eq!(rendered, expected);
}