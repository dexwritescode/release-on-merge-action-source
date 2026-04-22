use regex::Regex;
use std::fmt;
use std::str::FromStr;
use std::sync::LazyLock;

static SEMVER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^(?P<prefix>[a-zA-Z]*)(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(-(?P<pre_id>[a-zA-Z][a-zA-Z0-9]*)\.(?P<pre_num>\d+))?$",
    )
    .unwrap()
});

#[derive(Debug, Clone, PartialEq)]
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
pub struct PreRelease {
    pub identifier: String,
    pub counter: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Semver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prefix: String,
    pub pre_release: Option<PreRelease>,
}

impl fmt::Display for Semver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}.{}.{}", self.prefix, self.major, self.minor, self.patch)?;
        if let Some(pre) = &self.pre_release {
            write!(f, "-{}.{}", pre.identifier, pre.counter)?;
        }
        Ok(())
    }
}

impl FromStr for Semver {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let Some(caps) = SEMVER_RE.captures(s) else {
            return Err(());
        };
        let prefix: String = caps["prefix"].into();
        let major = caps["major"].parse::<u32>().unwrap();
        let minor = caps["minor"].parse::<u32>().unwrap();
        let patch = caps["patch"].parse::<u32>().unwrap();
        let pre_release = match (caps.name("pre_id"), caps.name("pre_num")) {
            (Some(id), Some(num)) => Some(PreRelease {
                identifier: id.as_str().to_owned(),
                counter: num.as_str().parse::<u32>().unwrap(),
            }),
            _ => None,
        };
        Ok(Semver { major, minor, patch, prefix, pre_release })
    }
}

impl Semver {
    pub fn new(major: u32, minor: u32, patch: u32, prefix: &str) -> Semver {
        Semver { major, minor, patch, prefix: prefix.to_owned(), pre_release: None }
    }

    pub fn increment(&self, strategy: &VersionIncrementStrategy) -> Semver {
        match strategy {
            VersionIncrementStrategy::Major => Semver {
                major: self.major + 1,
                minor: 0,
                patch: 0,
                prefix: self.prefix.clone(),
                pre_release: None,
            },
            VersionIncrementStrategy::Minor => Semver {
                major: self.major,
                minor: self.minor + 1,
                patch: 0,
                prefix: self.prefix.clone(),
                pre_release: None,
            },
            VersionIncrementStrategy::Patch => Semver {
                major: self.major,
                minor: self.minor,
                patch: self.patch + 1,
                prefix: self.prefix.clone(),
                pre_release: None,
            },
            VersionIncrementStrategy::NoRelease => Semver {
                pre_release: None,
                ..self.clone()
            },
        }
    }

    pub fn base_version(&self) -> Semver {
        Semver { pre_release: None, ..self.clone() }
    }

    pub fn with_pre_release(&self, identifier: &str, counter: u32) -> Semver {
        Semver {
            pre_release: Some(PreRelease { identifier: identifier.to_owned(), counter }),
            ..self.clone()
        }
    }

    pub fn bump_pre_release(&self) -> Semver {
        let pre = self.pre_release.as_ref().expect("bump_pre_release called on stable version");
        self.with_pre_release(&pre.identifier.clone(), pre.counter + 1)
    }

    pub fn pre_release_matches(&self, identifier: &str) -> bool {
        self.pre_release.as_ref().map_or(false, |p| p.identifier == identifier)
    }

    pub fn get_version(&self) -> String {
        match &self.pre_release {
            None => format!("{}.{}.{}", self.major, self.minor, self.patch),
            Some(pre) => format!("{}.{}.{}-{}.{}", self.major, self.minor, self.patch, pre.identifier, pre.counter),
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
        let expected_version = Semver::new(version.major + 1, 0, 0, &version.prefix);
        let new_version = version.increment(&VersionIncrementStrategy::Major);
        assert_eq!(new_version, expected_version);
    }

    #[test]
    fn increment_minor() {
        let version = Semver::new(1, 2, 3, "v");
        let expected_version = Semver::new(version.major, version.minor + 1, 0, &version.prefix);
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
    fn increment_strips_pre_release() {
        let version = Semver::from_str("v1.2.3-beta.4").unwrap();
        assert_eq!(version.increment(&VersionIncrementStrategy::Patch).to_string(), "v1.2.4");
        assert_eq!(version.increment(&VersionIncrementStrategy::Minor).to_string(), "v1.3.0");
        assert_eq!(version.increment(&VersionIncrementStrategy::Major).to_string(), "v2.0.0");
        assert_eq!(version.increment(&VersionIncrementStrategy::NoRelease).to_string(), "v1.2.3");
    }

    #[test]
    fn to_string() {
        let version = Semver::new(12, 20, 3, "ver");
        assert_eq!(version.to_string(), "ver12.20.3");
        assert_eq!(version.get_version(), "12.20.3");

        let version2 = Semver::new(1, 0, 3, "v");
        assert_eq!(version2.to_string(), "v1.0.3");
        assert_eq!(version2.get_version(), "1.0.3");

        let version3 = Semver::new(0, 1, 0, "V");
        assert_eq!(version3.to_string(), "V0.1.0");
        assert_eq!(version3.get_version(), "0.1.0");

        let version4 = Semver::new(0, 1, 0, "");
        assert_eq!(version4.to_string(), "0.1.0");
        assert_eq!(version4.get_version(), "0.1.0");
    }

    #[test]
    fn from_string() {
        assert_eq!(Semver::from_str("v0.1.0").unwrap().to_string(), "v0.1.0");
        assert_eq!(Semver::from_str("v11.20.36").unwrap().to_string(), "v11.20.36");
        assert_eq!(Semver::from_str("Version0.1.0").unwrap().to_string(), "Version0.1.0");
        assert_eq!(Semver::from_str("0.1.0").unwrap().to_string(), "0.1.0");
    }

    #[test]
    fn from_str_invalid_returns_err() {
        assert!(Semver::from_str("").is_err());
        assert!(Semver::from_str("not-a-version").is_err());
        assert!(Semver::from_str("1.2").is_err());
        assert!(Semver::from_str("1.2.3.4").is_err());
        assert!(Semver::from_str("v01.2.3").is_err());
        assert!(Semver::from_str("v1.02.3").is_err());
    }

    #[test]
    fn pre_release_round_trips() {
        let v = Semver::from_str("v1.2.3-beta.4").unwrap();
        assert_eq!(v.to_string(), "v1.2.3-beta.4");
        assert_eq!(v.get_version(), "1.2.3-beta.4");
        assert!(v.pre_release_matches("beta"));
        assert!(!v.pre_release_matches("rc"));
    }

    #[test]
    fn bump_pre_release_increments_counter() {
        let v = Semver::from_str("v1.0.0-rc.2").unwrap();
        assert_eq!(v.bump_pre_release().to_string(), "v1.0.0-rc.3");
    }

    #[test]
    fn base_version_strips_pre_release() {
        let v = Semver::from_str("v1.2.3-beta.1").unwrap();
        assert_eq!(v.base_version().to_string(), "v1.2.3");
        assert!(v.base_version().pre_release.is_none());
    }

    #[test]
    fn with_pre_release_attaches_suffix() {
        let v = Semver::new(1, 2, 0, "v");
        assert_eq!(v.with_pre_release("alpha", 1).to_string(), "v1.2.0-alpha.1");
    }
}
