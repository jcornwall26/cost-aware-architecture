use chrono::{prelude::*, Months};

pub fn build_date_range(start_date: &str, end_date: &str) -> Option<Vec<(String, String)>>
{
    //append day to form a full date so NaiveDate can parse
    let nd_start_date = NaiveDate::parse_from_str(format!("{}-01", start_date).as_str(), "%Y-%m-%d").unwrap();
    let nd_end_date = NaiveDate::parse_from_str(format!("{}-01", end_date).as_str(), "%Y-%m-%d").unwrap();

    let mut nd_current_date = nd_start_date;
    let mut dates: Vec<(String, String)> = Vec::new();

    while nd_current_date <= nd_end_date {

        let start_range = nd_current_date.to_string();
        let end_range = last_day_of_month(nd_current_date.year_ce().1 as i32, nd_current_date.month0() + 1).unwrap();
        let start_range_str = start_range.to_string();
        let end_range_str = end_range.to_string();

        dates.push((format!("{}T00:00:00Z", start_range_str), format!("{}T00:00:00Z", end_range_str)));
        nd_current_date = nd_current_date.checked_add_months(Months::new(1)).unwrap();
    }

    for date in &dates {
        println!("{:?}", date)
    }

    Some(dates)

}

fn last_day_of_month(year: i32, month: u32) -> Result<NaiveDate, String> {
    match NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or(NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap())
        .pred_opt()
    {
        Some(v) => Ok(v),
        None => Err(String::from("Calculating last_day_of_month failed"))
    }
}


#[cfg(test)]
mod tests {
    use super::build_date_range;

    #[test]
    fn build_date_range_full_year_tests() {
        //arrange 
        let start_date: &str = "2023-02";
        let end_date: &str = "2024-01";

        //act
        let date_range: Vec<(String, String)> = build_date_range(start_date, end_date).unwrap();
        
        //assert
        assert!(date_range.len() == 12);
        assert!(date_range[0].0 == "2023-02-01T00:00:00Z" && date_range[0].1 == "2023-02-28T00:00:00Z");
        assert!(date_range[1].0 == "2023-03-01T00:00:00Z" && date_range[1].1 == "2023-03-31T00:00:00Z");
        assert!(date_range[11].0 == "2024-01-01T00:00:00Z" && date_range[11].1 == "2024-01-31T00:00:00Z");
    }

    #[test]
    fn build_date_range_leap_year_tests() {
        //arrange 
        let start_date: &str = "2024-02";
        let end_date: &str = "2024-04";

        //act
        let date_range: Vec<(String, String)> = build_date_range(start_date, end_date).unwrap();
        
        //assert
        assert!(date_range.len() == 3);
        assert!(date_range[0].0 == "2024-02-01T00:00:00Z" && date_range[0].1 == "2024-02-29T00:00:00Z");
        assert!(date_range[1].0 == "2024-03-01T00:00:00Z" && date_range[1].1 == "2024-03-31T00:00:00Z");
        assert!(date_range[2].0 == "2024-04-01T00:00:00Z" && date_range[2].1 == "2024-04-30T00:00:00Z");
    }
}