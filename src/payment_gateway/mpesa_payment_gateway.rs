use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};
use chrono::Local;
use reqwest::{header, Client};
use serde_json::{json, Value};

use crate::payment_gateway::merchant_portal::Merchant;
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct MpesaPaymentProcessor {
    business_short_code: i32,
    password: String,
    timestamp: String,
    transaction_type: String,
    amount: i32,
    party_a: String,
    party_b: i32,
    phone_number: String,
    call_back_url: String,
    account_reference: String,
    transaction_desc: String,
}
impl MpesaPaymentProcessor {
    pub fn new(amount: i32, phone_number: &str, description: &str) -> Self {
        let merchant = Merchant::get_credentials();
        let timestamp = Local::now().format("%Y%m%d%H%M%S").to_string();
        const CUSTOM_ENGINE: engine::GeneralPurpose =
            engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
        let input = format!(
            "{}{}{}",
            merchant.business_short_code, merchant.pass_key, timestamp
        );
        let password = CUSTOM_ENGINE.encode(input);
        MpesaPaymentProcessor {
            business_short_code: merchant.business_short_code,
            password,
            timestamp,
            transaction_type: "CustomerPayBillOnline".to_owned(),
            amount,
            party_a: phone_number.to_owned(),
            party_b: merchant.business_short_code,
            phone_number: phone_number.to_owned(),
            call_back_url: "https://mydomain.com/path".to_owned(),
            account_reference: "We Mzee".to_owned(),
            transaction_desc: description.to_owned(),
        }
    }
    async fn get_auth_token(&self) -> (Client, String) {
        let merchant = Merchant::get_credentials();
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "Authorization",
            header::HeaderValue::from_static(merchant.basic_auth),
        );
        let y = Client::builder().build().expect("failed to create client");
        let res = y
            .get("https://sandbox.safaricom.co.ke/oauth/v1/generate?grant_type=client_credentials")
            .headers(headers)
            .send()
            .await
            .expect("failed to get response");
        let tokens = res.text().await.expect("failed to get text");
        let u: std::result::Result<Value, serde_json::Error> = serde_json::from_str(&tokens);
        (
            y,
            u.unwrap()
                .as_object()
                .expect("failed to parse object")
                .get("access_token")
                .expect("failed to get access token")
                .as_str()
                .unwrap()
                .to_owned(),
        )
    }
    pub async fn handle_payment(&self) {
        let (client, token) = self.get_auth_token().await;
        let j = format!(
            "{{\"BusinessShortCode\":\"{}\",\"Password\":\"{}\",
          \"Timestamp\":\"{}\",\"TransactionType\":\"{}\",\"Amount\":\"{}\",
          \"PartyA\":\"{}\",\"PartyB\":\"{}\",\"PhoneNumber\":\"{}\",
          \"CallBackURL\":\"{}\",\"AccountReference\":\"{}\",\"TransactionDesc\":\"{}\"}}",
            self.business_short_code,
            self.password,
            self.timestamp,
            self.transaction_type,
            self.amount,
            self.party_a,
            self.party_b,
            self.phone_number,
            self.call_back_url,
            self.account_reference,
            self.transaction_desc
        );
        let res2 = client
            .post("https://sandbox.safaricom.co.ke/mpesa/stkpush/v1/processrequest")
            .header("content-type", "application/json")
            .bearer_auth(token)
            .body(j)
            .send()
            .await
            .expect("failed to get response");
        let gy = res2.text().await.unwrap();
        let plan = json!(gy);
        println!("Test {:?}", plan.as_str());
    }
}
