struct Env {}

#[derive(Debug, Clone)]
struct ExecError;

type Result<T> = std::result::Result<T, ExecError>;

#[derive(Debug)]
enum OrderType {
  Market,
  Limit,
  StopLimit
}


#[derive(Debug)]
enum Command {
  Buy { price: f64, qty: Option<f64>, ord_type: OrderType },
  Sell { price: f64, qty: Option<f64>, ord_type: OrderType },
  CancelLast,
}


trait Exec {
  fn exec(env: Env) -> Result<String> {

  }
}

