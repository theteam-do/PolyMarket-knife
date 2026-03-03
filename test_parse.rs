use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
struct GammaEvent {
    id: String,
    markets: Vec<GammaMarket>,
}

#[derive(Debug, Clone, Deserialize)]
struct GammaMarket {
    id: String,
    question: String,
    outcomes: Option<String>,
    #[serde(rename = "outcomePrices")]
    outcome_prices: Option<String>,
    #[serde(rename = "clobTokenIds")]
    clob_token_ids: Option<String>,
    volume: Option<String>,
    active: bool,
    closed: bool,
}

fn main() {
    let data = r#"[{"id":"1","markets":[{"id":"2","question":"q","outcomes":"[\"Yes\",\"No\"]","outcomePrices":"[\"0\",\"1\"]","clobTokenIds":"[\"123\",\"456\"]","volume":"100","active":true,"closed":false}]}]"#;
    let events: Vec<GammaEvent> = serde_json::from_str(data).unwrap();
    println!("{:?}", events);
}
