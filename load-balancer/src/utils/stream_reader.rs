pub fn read_status_code(str_slice: &str) -> u16 {
    let str_parts: Vec<&str> = str_slice.split(" ").collect();
    if str_parts.len() < 3 {
        return 404_u16
    }
    let parse_result = str_parts[1].parse::<u16>();
    match parse_result {
        Ok(status_code) => status_code,
        Err(_) => 500u16,
    }
}
