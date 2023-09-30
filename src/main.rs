use anyhow::Result;
mod payment_gateway;
use payment_gateway::mpesa_payment_gateway::MpesaPaymentProcessor;
#[tokio::main]
async fn main() -> Result<()> {
    let phone_number = "254706908786";
    let description = "Payment of X";

    let amount = 150;
    let gateway = MpesaPaymentProcessor::new(amount, phone_number, description);
    Ok(gateway.handle_payment().await)
}
