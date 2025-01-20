use chrono::Datelike;
use serde::Serialize;
use serde_json::json;
use reqwest::Client;
use reqwest::ClientBuilder;
use chrono::{Utc, DateTime};

pub struct OpenSearchClient {
    client : Client
}

impl OpenSearchClient{
    
    pub fn new() -> OpenSearchClient {
        OpenSearchClient{
            client : ClientBuilder::new().danger_accept_invalid_certs(true).build().unwrap()
        }
    }

    pub async fn write_to_opensearch(&self, lambda_name: &str, combined_results: Vec<(&str, f64, f64, f64, f64)>){

        for result in combined_results {

            let doc = CostMetricsDoc::new(lambda_name, result);
            let body= json!(doc).to_string();
            let url = format!("https://localhost:9200/caa/_doc/{}", doc.id);

            let response = self.client.post(url)
                .body(body)
                .header("Content-Type", "application/json")
                .basic_auth("admin", Some("3VndwCZowsNqXoHFfHo2z2ay974Xf7yFhW"))
                .send()
                .await.unwrap();

            let status= response.status();
            let text = response.text().await.unwrap();
            println!("{} - {}", status, text);
        }
    }
}

#[derive(Serialize, Clone, Debug)]
struct CostMetricsDoc {
    id: String,
    lambda_name: String,
    timestamp: String,
    cost: f64,
    invocations: f64,
    duration: f64,
    cost_per_request: f64
}

impl CostMetricsDoc {
    pub fn new(lambda_name: &str, raw: (&str, f64, f64, f64, f64)) -> CostMetricsDoc {
        CostMetricsDoc{
            id : CostMetricsDoc::build_id(lambda_name, raw.0),
            timestamp : String::from(raw.0),
            lambda_name : String::from(lambda_name),
            cost : raw.1,
            invocations : raw.2, 
            duration : raw.3, 
            cost_per_request : raw.4
        }
    }

    fn build_id(lambda_name: &str, timestamp: &str) -> String{
        let ts_date : DateTime<Utc> = timestamp.parse().unwrap();
        let month= ts_date.date_naive().month();
        let year = ts_date.date_naive().year_ce().1;
        format!("{}-{}-{}", lambda_name, year, month)
    }
}

#[cfg(test)]
mod tests {

    use super::CostMetricsDoc;

    #[test]
    fn construct_new_metrics_doc() {
        //arrange
        let raw = ("2024-07-01T00:00:00Z", 1000f64, 2000f64, 3000f64, 4000f64);
        let lambda_name = "tf-ecom-web-app-prodb-shop-service";

        //act
        let doc = CostMetricsDoc::new(lambda_name, raw);

        //assert
        assert_eq!(doc.id, "tf-ecom-web-app-prodb-shop-service-2024-7");
    }
}