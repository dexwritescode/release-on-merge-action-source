use regex::Regex;
use std::fmt;
use std::str::FromStr;

#[derive(Debug)]
pub enum VersionIncrementStrategy {
    Major,
    Minor,
    Patch,
    NoRelease,
}

impl FromStr for VersionIncrementStrategy {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            "norelease" => Ok(Self::NoRelease),
            _ => Err(()),
        }
    }
}

impl<'a> From<VersionIncrementStrategy> for &'a str {
    fn from(c: VersionIncrementStrategy) -> Self {
        match c {
            VersionIncrementStrategy::Major => "major",
            VersionIncrementStrategy::Minor => "minor",
            VersionIncrementStrategy::Patch => "patch",
            VersionIncrementStrategy::NoRelease => "norelease",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Semver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prefix: String,
}

impl fmt::Display for Semver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}.{}.{}",
            self.prefix, self.major, self.minor, self.patch
        )
    }
}

impl FromStr for Semver {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let re = Regex::new(r"^(?P<prefix>[a-zA-Z]*)(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)$").unwrap();
        let Some(caps) = re.captures(s) else {
            return Err(());
        };
        let prefix: String = caps["prefix"].into();
        let major = caps["major"].parse::<u32>().unwrap();
        let minor = caps["minor"].parse::<u32>().unwrap();
        let patch = caps["patch"].parse::<u32>().unwrap();
        Ok(Semver::new(major, minor, patch, &prefix))
    }
}

impl Semver {
    pub fn new(major: u32, minor: u32, patch: u32, prefix: &str) -> Semver {
        Semver {
            major,
            minor,
            patch,
            prefix: prefix.to_owned(),
        }
    }

    pub fn increment(&self, strategy: &VersionIncrementStrategy) -> Semver {
        match strategy {
            VersionIncrementStrategy::Major => Semver {
                major: self.major + 1,
                minor: self.minor,
                patch: self.patch,
                prefix: self.prefix.clone(),
            },
            VersionIncrementStrategy::Minor => Semver {
                major: self.major,
                minor: self.minor + 1,
                patch: self.patch,
                prefix: self.prefix.clone(),
            },
            VersionIncrementStrategy::Patch => Semver {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
                prefix: self.prefix.clone(),
            },
            VersionIncrementStrategy::NoRelease => self.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::semver::Semver;
    use crate::semver::VersionIncrementStrategy;

    #[test]
    fn no_release() {
        let version = Semver::new(1, 2, 3, "v");
        let new_version = version.increment(&VersionIncrementStrategy::NoRelease);
        assert_eq!(new_version, version);
    }

    #[test]
    fn increment_major() {
        let version = Semver::new(1, 2, 3, "v");
        let expected_version = Semver::new(
            version.major + 1,
            version.minor,
            version.patch,
            &version.prefix,
        );
        let new_version = version.increment(&VersionIncrementStrategy::Major);
        assert_eq!(new_version, expected_version);
    }

    #[test]
    fn increment_minor() {
        let version = Semver::new(1, 2, 3, "v");
        let expected_version = Semver::new(
            version.major,
            version.minor + 1,
            version.patch,
            &version.prefix,
        );
        let new_version = version.increment(&VersionIncrementStrategy::Minor);
        assert_eq!(new_version, expected_version);
    }

    #[test]
    fn increment_patch() {
        let version = Semver::new(1, 2, 3, "v");
        let expected_version = Semver::new(
            version.major,
            version.minor,
            version.patch + 1,
            &version.prefix,
        );
        let new_version = version.increment(&VersionIncrementStrategy::Patch);
        assert_eq!(new_version, expected_version);
    }

    #[test]
    fn to_string() {
        let version = Semver::new(12, 20, 3, "ver");
        assert_eq!(version.to_string(), "ver12.20.3");
        let version2 = Semver::new(1, 0, 3, "v");
        assert_eq!(version2.to_string(), "v1.0.3");
        let version3 = Semver::new(0, 1, 0, "V");
        assert_eq!(version3.to_string(), "V0.1.0");
    }

    #[test]
    fn from_string() {
        assert_eq!(Semver::from_str("v0.1.0").unwrap().to_string(), "v0.1.0");
        assert_eq!(
            Semver::from_str("v11.20.36").unwrap().to_string(),
            "v11.20.36"
        );
        assert_eq!(
            Semver::from_str("Version0.1.0").unwrap().to_string(),
            "Version0.1.0"
        );
        assert_eq!(Semver::from_str("0.1.0").unwrap().to_string(), "0.1.0");
    }
}
