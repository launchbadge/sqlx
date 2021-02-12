// https://dev.mysql.com/doc/c-api/8.0/en/mysql-info.html
// https://mariadb.com/kb/en/mysql_info/

#[derive(Debug, Default)]
pub(crate) struct Info {
    pub(crate) records: u64,
    pub(crate) duplicates: u64,
    pub(crate) matched: u64,
}

impl Info {
    pub(crate) fn parse(info: &str) -> Self {
        let mut records = 0;
        let mut duplicates = 0;
        let mut matched = 0;

        let mut failed = false;

        for item in info.split("  ") {
            let mut item = item.split(": ");

            if let Some((key, value)) = item.next().zip(item.next()) {
                let value: u64 = if let Ok(value) = value.parse() {
                    value
                } else {
                    // remember failed, invalid value
                    failed = true;
                    0
                };

                match key {
                    "Records" => records = value,
                    "Duplicates" => duplicates = value,
                    "Rows matched" => matched = value,

                    // ignore records changed
                    // this is "rows affected" for UPDATE
                    "Changed" => {}

                    // ignore warnings in info
                    // these are passed back differently
                    "Warnings" => {}

                    // unknown key
                    _ => failed = true,
                }
            }
        }

        if failed {
            log::warn!("failed to parse status information from OK packet: {:?}", info);
        }

        Self { records, duplicates, matched }
    }
}

#[cfg(test)]
mod tests {
    use super::Info;

    #[test]
    fn parse_insert() {
        let info = Info::parse("Records: 10  Duplicates: 5  Warnings: 0");

        assert_eq!(info.records, 10);
        assert_eq!(info.duplicates, 5);
    }

    #[test]
    fn parse_update() {
        let info = Info::parse("Rows matched: 40  Changed: 5  Warnings: 0");

        assert_eq!(info.matched, 40);
    }
}
