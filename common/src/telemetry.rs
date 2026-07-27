use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub use studio_shared::{PaperEventKind, RunMode, StrategyKind};

use studio_shared::{PaperEvent, PaperIngestRequest, PaperMetrics, PaperRunSnapshot, RunStatus};

#[derive(Clone)]
pub struct PaperTelemetryClient {
    ingest_url: String,
    client: reqwest::Client,
}

impl PaperTelemetryClient {
    pub fn from_env() -> Option<Self> {
        let ingest_url = std::env::var("POLYMARKET_STUDIO_INGEST_URL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(800))
            .build()
            .ok()?;

        Some(Self { ingest_url, client })
    }

    pub fn publish(&self, payload: PaperIngestRequest) -> tokio::task::JoinHandle<()> {
        let ingest_url = self.ingest_url.clone();
        let client = self.client.clone();

        tokio::spawn(async move {
            for attempt in 0..=1 {
                let response = client.post(&ingest_url).json(&payload).send().await;
                match response {
                    Ok(response) => {
                        if let Err(error) = response.error_for_status_ref() {
                            tracing::warn!("paper telemetry rejected by studio-server: {}", error);
                            if attempt == 0 {
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                continue;
                            }
                        }
                        return;
                    }
                    Err(error) => {
                        if error.is_connect() || error.is_timeout() {
                            tracing::warn!(
                                "paper telemetry connection error (attempt {}): {}",
                                attempt + 1,
                                error
                            );
                            if attempt == 0 {
                                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                                continue;
                            }
                        } else {
                            tracing::warn!("paper telemetry publish failed: {}", error);
                            return;
                        }
                    }
                }
            }
            tracing::error!("paper telemetry publish failed after retries");
        })
    }
}

pub struct PaperRunReporter {
    client: Option<PaperTelemetryClient>,
    snapshot: PaperRunSnapshot,
}

#[derive(Debug, Clone, Default)]
pub struct PaperEventMetricsPayload {
    pub expected_edge_usd: Option<f64>,
    pub pnl_delta_usd: Option<f64>,
    pub implied_prob: Option<f64>,
    pub posterior_prob: Option<f64>,
    pub gross_edge_usd: Option<f64>,
    pub net_edge_usd: Option<f64>,
    pub fees_usd: Option<f64>,
    pub slippage_usd: Option<f64>,
    pub latency_penalty_usd: Option<f64>,
    pub gas_usd: Option<f64>,
    pub rebate_usd: Option<f64>,
    pub fill_probability: Option<f64>,
    pub expected_net_edge_after_fill_usd: Option<f64>,
    pub settlement_action: Option<String>,
    pub settlement_status: Option<String>,
    pub settlement_tx_hash: Option<String>,
    pub settlement_block_number: Option<u64>,
    pub settlement_condition_id: Option<String>,
    pub settlement_collateral_token: Option<String>,
    pub settlement_full_sets: Option<f64>,
    pub settlement_reason: Option<String>,
    pub target_fraction: Option<f64>,
    pub total_pnl_usd: Option<f64>,
    pub exposure_usd: Option<f64>,
}

impl PaperRunReporter {
    pub fn new(
        strategy: StrategyKind,
        mode: RunMode,
        label: impl Into<String>,
        config_path: Option<String>,
    ) -> Self {
        let started_at_ms = current_timestamp_ms();
        let run_id = format!(
            "{}-{}-{}",
            strategy.as_str(),
            std::process::id(),
            started_at_ms
        );

        Self {
            client: PaperTelemetryClient::from_env(),
            snapshot: PaperRunSnapshot {
                run_id,
                strategy,
                mode,
                status: RunStatus::Starting,
                label: label.into(),
                config_path,
                started_at_ms,
                updated_at_ms: started_at_ms,
                last_event: "Reporter initialized".to_string(),
                metrics: PaperMetrics::default(),
            },
        }
    }

    pub fn start(&mut self, detail: impl Into<String>) {
        self.snapshot.status = RunStatus::Running;
        self.emit(
            PaperEventKind::RunStarted,
            "Run started".to_string(),
            detail.into(),
            PaperEventMetricsPayload::default(),
        );
    }

    pub fn event(
        &mut self,
        kind: PaperEventKind,
        title: impl Into<String>,
        detail: impl Into<String>,
    ) {
        self.emit(
            kind,
            title.into(),
            detail.into(),
            PaperEventMetricsPayload::default(),
        );
    }

    pub fn update<F>(
        &mut self,
        kind: PaperEventKind,
        title: impl Into<String>,
        detail: impl Into<String>,
        expected_edge_usd: Option<f64>,
        pnl_delta_usd: Option<f64>,
        mutate: F,
    ) where
        F: FnOnce(&mut PaperRunSnapshot),
    {
        self.update_with_metrics(
            kind,
            title,
            detail,
            PaperEventMetricsPayload {
                expected_edge_usd,
                pnl_delta_usd,
                total_pnl_usd: Some(self.snapshot.metrics.total_pnl_usd),
                exposure_usd: Some(self.snapshot.metrics.exposure_usd),
                ..Default::default()
            },
            mutate,
        );
    }

    pub fn update_with_metrics<F>(
        &mut self,
        kind: PaperEventKind,
        title: impl Into<String>,
        detail: impl Into<String>,
        mut metrics: PaperEventMetricsPayload,
        mutate: F,
    ) where
        F: FnOnce(&mut PaperRunSnapshot),
    {
        mutate(&mut self.snapshot);
        metrics.total_pnl_usd = Some(self.snapshot.metrics.total_pnl_usd);
        metrics.exposure_usd = Some(self.snapshot.metrics.exposure_usd);
        self.emit(kind, title.into(), detail.into(), metrics);
    }

    pub fn warning(&mut self, title: impl Into<String>, detail: impl Into<String>) {
        self.snapshot.status = RunStatus::Warning;
        self.snapshot.metrics.errors += 1;
        self.emit(
            PaperEventKind::Warning,
            title.into(),
            detail.into(),
            PaperEventMetricsPayload {
                total_pnl_usd: Some(self.snapshot.metrics.total_pnl_usd),
                exposure_usd: Some(self.snapshot.metrics.exposure_usd),
                ..Default::default()
            },
        );
        self.snapshot.status = RunStatus::Running;
    }

    pub fn error(&mut self, title: impl Into<String>, detail: impl Into<String>) {
        self.snapshot.status = RunStatus::Error;
        self.snapshot.metrics.errors += 1;
        self.emit(
            PaperEventKind::Error,
            title.into(),
            detail.into(),
            PaperEventMetricsPayload {
                total_pnl_usd: Some(self.snapshot.metrics.total_pnl_usd),
                exposure_usd: Some(self.snapshot.metrics.exposure_usd),
                ..Default::default()
            },
        );
        self.snapshot.status = RunStatus::Running;
    }

    pub fn stop(&mut self, detail: impl Into<String>) {
        self.snapshot.status = RunStatus::Stopped;
        self.emit(
            PaperEventKind::RunStopped,
            "Run stopped".to_string(),
            detail.into(),
            PaperEventMetricsPayload {
                total_pnl_usd: Some(self.snapshot.metrics.total_pnl_usd),
                exposure_usd: Some(self.snapshot.metrics.exposure_usd),
                ..Default::default()
            },
        );
    }

    fn emit(
        &mut self,
        kind: PaperEventKind,
        title: String,
        detail: String,
        metrics: PaperEventMetricsPayload,
    ) {
        self.snapshot.updated_at_ms = current_timestamp_ms();
        self.snapshot.last_event = title.clone();

        if let Some(client) = &self.client {
            client.publish(PaperIngestRequest {
                snapshot: self.snapshot.clone(),
                event: Some(PaperEvent {
                    run_id: self.snapshot.run_id.clone(),
                    strategy: self.snapshot.strategy,
                    mode: self.snapshot.mode,
                    status: self.snapshot.status,
                    kind,
                    ts_ms: self.snapshot.updated_at_ms,
                    title,
                    detail,
                    expected_edge_usd: metrics.expected_edge_usd,
                    implied_prob: metrics.implied_prob,
                    posterior_prob: metrics.posterior_prob,
                    gross_edge_usd: metrics.gross_edge_usd,
                    net_edge_usd: metrics.net_edge_usd,
                    fees_usd: metrics.fees_usd,
                    slippage_usd: metrics.slippage_usd,
                    latency_penalty_usd: metrics.latency_penalty_usd,
                    gas_usd: metrics.gas_usd,
                    rebate_usd: metrics.rebate_usd,
                    fill_probability: metrics.fill_probability,
                    expected_net_edge_after_fill_usd: metrics.expected_net_edge_after_fill_usd,
                    settlement_action: metrics.settlement_action,
                    settlement_status: metrics.settlement_status,
                    settlement_tx_hash: metrics.settlement_tx_hash,
                    settlement_block_number: metrics.settlement_block_number,
                    settlement_condition_id: metrics.settlement_condition_id,
                    settlement_collateral_token: metrics.settlement_collateral_token,
                    settlement_full_sets: metrics.settlement_full_sets,
                    settlement_reason: metrics.settlement_reason,
                    target_fraction: metrics.target_fraction,
                    pnl_delta_usd: metrics.pnl_delta_usd,
                    total_pnl_usd: metrics.total_pnl_usd,
                    exposure_usd: metrics.exposure_usd,
                }),
            });
        }
    }
}

pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before unix epoch")
        .as_millis() as u64
}
