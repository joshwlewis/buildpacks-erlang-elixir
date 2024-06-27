use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub(crate) enum Error {
    Http(ureq::Error),
    Io(std::io::Error),
    Parse,
}

#[derive(Clone)]
pub(crate) struct ElixirBuild {
    pub(crate) version: String,
    pub(crate) reference: String,
}

impl ElixirBuild {
    pub(crate) fn list() -> Result<Vec<ElixirBuild>, Error> {
        let builds_list_url = format!("https://builds.hex.pm/builds/elixir/builds.txt");
        let reader = ureq::get(&builds_list_url)
            .call()
            .map_err(Error::Http)?
            .into_reader();
        let mut builds = vec![];
        for line_data in BufReader::new(reader).lines() {
            let line = line_data.map_err(Error::Io)?;
            if line.is_empty() {
                continue;
            }
            let mut parts = line.split_whitespace();
            let version = parts.next().ok_or(Error::Parse)?.to_string();
            let reference = parts.next().ok_or(Error::Parse)?.to_string();
            builds.push(ElixirBuild { version, reference });
        }
        Ok(builds)
    }

    pub(crate) fn url(&self) -> String {
        format!("https://builds.hex.pm/builds/elixir/{}.zip", self.version)
    }
}

#[cfg(test)]
mod tests {
    use super::ElixirBuild;

    #[test]
    fn bob_list_elixir_builds() {
        let builds = ElixirBuild::list().expect("Expected to fetch builds");

        assert!(builds.len() > 1);
        assert!(builds
            .iter()
            .find(|build| build.version.contains("v1.17.1"))
            .is_some());
    }

    #[test]
    fn bob_elixir_build_url() {
        let build = ElixirBuild {
            version: "v1.16.0-rc.1-otp-26".to_string(),
            reference: "abcdefg".to_string(),
        };

        assert_eq!(
            build.url(),
            "https://builds.hex.pm/builds/elixir/v1.16.0-rc.1-otp-26.zip"
        );
    }
}
