use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub(crate) enum Error {
    Http(ureq::Error),
    Io(std::io::Error),
    Parse,
}

#[derive(Clone)]
pub(crate) struct ErlangBuild {
    pub(crate) version: String,
    pub(crate) arch: String,
    pub(crate) checksum: String,
    pub(crate) distro: String,
    pub(crate) distro_version: String,
}

impl ErlangBuild {
    pub(crate) fn list(
        arch: String,
        distro: String,
        distro_version: String,
    ) -> Result<Vec<ErlangBuild>, Error> {
        let builds_list_url =
            format!("https://builds.hex.pm/builds/otp/{arch}/{distro}-{distro_version}/builds.txt");
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
            let checksum = parts.next().ok_or(Error::Parse)?.to_string();
            builds.push(ErlangBuild {
                arch: arch.clone(),
                distro: distro.clone(),
                distro_version: distro_version.clone(),
                version,
                checksum,
            });
        }
        Ok(builds)
    }

    pub(crate) fn url(&self) -> String {
        format!(
            "https://builds.hex.pm/builds/otp/{}/{}-{}/{}.tar.gz",
            self.arch, self.distro, self.distro_version, self.version
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ErlangBuild;

    #[test]
    fn bob_list_erlang_builds() {
        let builds = ErlangBuild::list(
            "amd64".to_string(),
            "ubuntu".to_string(),
            "22.04".to_string(),
        )
        .expect("Expected to fetch builds");

        assert!(builds.len() > 1);
        assert!(builds
            .first()
            .expect("Expected first hex.pm build to exist")
            .version
            .contains("OTP"));
        assert!(builds
            .last()
            .expect("Expected last hex.pm erlang build to exist")
            .version
            .contains("master"));
    }

    #[test]
    fn bob_erlang_build_url() {
        let build = ErlangBuild {
            arch: "arm64".to_string(),
            distro: "ubuntu".to_string(),
            distro_version: "22.04".to_string(),
            checksum: "abcdefg".to_string(),
            version: "OTP-26.0".to_string(),
        };

        assert_eq!(
            build.url(),
            "https://builds.hex.pm/builds/otp/arm64/ubuntu-22.04/OTP-26.0.tar.gz"
        );
    }
}
