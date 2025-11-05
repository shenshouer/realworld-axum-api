use opentelemetry::{
    KeyValue,
    metrics::{Counter, Meter},
};

#[derive(Clone)]
pub struct Metrics {
    pub http_requests_total: Counter<u64>,
}

impl Metrics {
    pub fn new(meter: Meter) -> Self {
        let http_requests_total = meter
            .u64_counter("http_requests_total")
            .with_description("Total number of HTTP requests")
            .build();

        Self {
            http_requests_total,
        }
    }

    pub fn record_http_request(&self, method: &str, path: &str) {
        let attributes = [
            KeyValue::new("method", method.to_string()),
            KeyValue::new("path", path.to_string()),
        ];
        self.http_requests_total.add(1, &attributes);
    }
}
