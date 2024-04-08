
pub fn write_to_csv(lambda_name :&str, aws_profile: &str, aws_region: &str, com_results: Vec<(&str, f64, f64, f64, f64)>) -> String {

    let csv_path = format!("./{}-{}-{}-output.csv", &lambda_name, &aws_profile, &aws_region);
    let mut wtr = csv::Writer::from_path(&csv_path).unwrap();

    // write header
    match wtr.write_record(&["function-name", "timestamp",  "monthly-cost", "invocations", "average-duration", "cost-per-100m-requests"]) {
        Ok(_v) => (),
        Err(e) => println!("{:?}", e)
    }
    match wtr.flush() {
        Ok(_v) => (),
        Err(e) => println!("{:?}", e)
    };

    for com_result in com_results{
        println!("{:?}", com_result);
        match wtr.write_record(&[lambda_name, 
            com_result.0, com_result.1.to_string().as_str(), 
            com_result.2.to_string().as_str(), 
            com_result.3.to_string().as_str(), 
            com_result.4.to_string().as_str()]) {
            Ok(_v) => (),
            Err(e) => println!("{:?}", e)
        }
        match wtr.flush() {
            Ok(_v) => (),
            Err(e) => println!("{:?}", e)
        };
    };

    return csv_path
}