use {
    aargvark::traits_impls::AargvarkFromStr,
};

pub struct DataPath(pub Vec<String>);

impl AargvarkFromStr for DataPath {
    fn from_str(s: &str) -> Result<Self, String> {
        if s.starts_with("[") {
            return Ok(
                DataPath(
                    serde_json::from_str(
                        s,
                    ).map_err(|e| format!("Error parsing path as JSON array of strings: {}", e))?,
                ),
            );
        } else {
            return Ok(
                DataPath(s.trim_end_matches('.').split(".").map(|x| x.to_string()).collect::<Vec<_>>()),
            );
        }
    }

    fn build_help_pattern(_state: &mut aargvark::help::HelpState) -> aargvark::help::HelpPattern {
        return aargvark::help::HelpPattern(vec![aargvark::help::HelpPatternElement::Type(format!("PATH.TO.DATA"))]);
    }
}
