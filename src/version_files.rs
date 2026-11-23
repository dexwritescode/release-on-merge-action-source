//! Versions in files (the `version-files` input).
//!
//! The branch always holds the next version as a snapshot (`0.5.6-dev`). A release adds two
//! commits on top of the branch head: one that sets the files to the release version, which the
//! tag points at, and one that sets them to the next snapshot (the release's patch + 1, `-dev`).

use std::collections::HashSet;
use std::str::FromStr;

use toml_edit::{DocumentMut, Item, Value};

use crate::error::ActionError;
use crate::github_client::{GitApi, GithubApi};
use crate::releases::{Releases, SNAPSHOT_COMMIT};
use crate::semver::Semver;

/// Where the version lives.
#[derive(Debug, Clone, PartialEq)]
pub enum VersionFiles {
    /// The root `Cargo.toml` (`[workspace.package]` or `[package]`), and `Cargo.lock`.
    Cargo,
    /// A file holding only the version.
    File(String),
}

impl FromStr for VersionFiles {
    type Err = ActionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "cargo" => Ok(Self::Cargo),
            s => match s.strip_prefix("file:") {
                Some(path) if !path.is_empty() => Ok(Self::File(path.to_owned())),
                _ => Err(ActionError::VersionFiles(format!(
                    "unknown version-files value {s:?}; expected `cargo` or `file:PATH`"
                ))),
            },
        }
    }
}

/// A file's new content.
#[derive(Debug, Clone, PartialEq)]
pub struct FileEdit {
    pub path: String,
    pub content: String,
}

/// The next snapshot after `release`: its patch + 1, with `-dev`.
pub fn snapshot_after(release: &Semver) -> String {
    format!("{}.{}.{}-dev", release.major, release.minor, release.patch + 1)
}

/// `major.minor.patch` of a version, ignoring any pre-release or build suffix.
fn base(version: &str) -> Option<(u32, u32, u32)> {
    let core = version.split(['-', '+']).next()?;
    let mut parts = core.split('.').map(|p| p.parse::<u32>().ok());
    let v = (parts.next()??, parts.next()??, parts.next()??);
    parts.next().is_none().then_some(v)
}

/// The files may hold any snapshot at or below the release, never above it.
pub fn check_not_above(current: &str, release: &Semver) -> Result<(), ActionError> {
    let Some(current_base) = base(current) else {
        return Err(ActionError::VersionFiles(format!("can't parse the version in the files: {current:?}")));
    };
    if current_base > (release.major, release.minor, release.patch) {
        return Err(ActionError::VersionFiles(format!(
            "the files say {current}, above the release {}; set them below it, or tag that release by hand",
            release.get_version()
        )));
    }
    Ok(())
}

/// Reads the repository at one commit.
pub trait Tree {
    fn read(&self, path: &str) -> Result<Option<String>, ActionError>;
    fn paths(&self) -> Result<Vec<String>, ActionError>;
}

impl VersionFiles {
    /// The version the files hold now.
    pub fn current(&self, tree: &impl Tree) -> Result<String, ActionError> {
        match self {
            Self::Cargo => cargo_version(&require(tree, "Cargo.toml")?),
            Self::File(path) => Ok(require(tree, path)?.trim().to_owned()),
        }
    }

    /// The files set to `version`.
    pub fn edits(&self, tree: &impl Tree, version: &str) -> Result<Vec<FileEdit>, ActionError> {
        match self {
            Self::Cargo => cargo_edits(tree, version),
            Self::File(path) => {
                let text = require(tree, path)?;
                let rest = &text[text.trim_end().len()..];
                Ok(vec![FileEdit { path: path.clone(), content: format!("{version}{rest}") }])
            }
        }
    }
}

fn require(tree: &impl Tree, path: &str) -> Result<String, ActionError> {
    tree.read(path)?
        .ok_or_else(|| ActionError::VersionFiles(format!("{path} not found on the branch")))
}

fn parse_toml(path: &str, text: &str) -> Result<DocumentMut, ActionError> {
    text.parse::<DocumentMut>()
        .map_err(|e| ActionError::VersionFiles(format!("{path}: {e}")))
}

/// `[workspace.package] version`, else `[package] version`, as a string.
fn root_version_item(doc: &mut DocumentMut) -> Option<&mut Item> {
    let in_workspace = doc
        .get("workspace")
        .and_then(|w| w.get("package"))
        .and_then(|p| p.get("version"))
        .is_some_and(|v| v.is_str());
    let table = if in_workspace {
        doc.get_mut("workspace")?.get_mut("package")?
    } else {
        doc.get_mut("package")?
    };
    table.get_mut("version").filter(|v| v.is_str())
}

fn cargo_version(root: &str) -> Result<String, ActionError> {
    let mut doc = parse_toml("Cargo.toml", root)?;
    root_version_item(&mut doc)
        .and_then(|v| v.as_str().map(str::to_owned))
        .ok_or_else(|| {
            ActionError::VersionFiles(
                "Cargo.toml has no [workspace.package] or [package] version".to_owned(),
            )
        })
}

/// Replace a string value, keeping its surrounding whitespace and comments.
fn set_str(item: &mut Item, version: &str) {
    if let Some(value) = item.as_value_mut() {
        let decor = value.decor().clone();
        *value = Value::from(version);
        *value.decor_mut() = decor;
    }
}

/// Names of the packages whose version is the root one: members with
/// `version.workspace = true`, and the root package if it has its own version.
fn cargo_package_names(manifests: &[(String, String)]) -> Result<HashSet<String>, ActionError> {
    let mut names = HashSet::new();
    for (path, text) in manifests {
        let doc = parse_toml(path, text)?;
        let Some(package) = doc.get("package") else { continue };
        let Some(name) = package.get("name").and_then(|n| n.as_str()) else { continue };
        let version = package.get("version");
        let inherits = version
            .and_then(|v| v.get("workspace"))
            .and_then(|w| w.as_bool())
            .unwrap_or(false);
        let workspace_version = doc
            .get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("version"))
            .is_some();
        let is_root_literal = path == "Cargo.toml" && !workspace_version && version.is_some_and(|v| v.is_str());
        if inherits || is_root_literal {
            names.insert(name.to_owned());
        }
    }
    Ok(names)
}

fn cargo_edits(tree: &impl Tree, version: &str) -> Result<Vec<FileEdit>, ActionError> {
    let root = require(tree, "Cargo.toml")?;
    let old = cargo_version(&root)?;
    let mut doc = parse_toml("Cargo.toml", &root)?;
    if let Some(item) = root_version_item(&mut doc) {
        set_str(item, version);
    }
    let mut edits = vec![FileEdit { path: "Cargo.toml".to_owned(), content: doc.to_string() }];

    let Some(lock) = tree.read("Cargo.lock")? else { return Ok(edits) };
    // Every Cargo.toml in the tree, not only workspace members: a non-member (a test fixture)
    // that inherits a version and shares a name with a member is not handled.
    let mut manifests = vec![("Cargo.toml".to_owned(), root)];
    for path in tree.paths()? {
        if path != "Cargo.toml" && (path.ends_with("/Cargo.toml")) {
            if let Some(text) = tree.read(&path)? {
                manifests.push((path, text));
            }
        }
    }
    let names = cargo_package_names(&manifests)?;
    edits.push(FileEdit { path: "Cargo.lock".to_owned(), content: cargo_lock(&lock, &names, &old, version)? });
    Ok(edits)
}

/// Set `version` on the lock entries of `names` that come from this repository (no `source`)
/// and still hold `old`, and on dependency strings naming them with a version (`"name old"`,
/// which Cargo writes when two packages share a name).
fn cargo_lock(lock: &str, names: &HashSet<String>, old: &str, version: &str) -> Result<String, ActionError> {
    let mut doc = parse_toml("Cargo.lock", lock)?;
    if let Some(packages) = doc.get_mut("package").and_then(|p| p.as_array_of_tables_mut()) {
        for package in packages.iter_mut() {
            let ours = package.get("name").and_then(|n| n.as_str()).is_some_and(|n| names.contains(n))
                && !package.contains_key("source")
                && package.get("version").and_then(|v| v.as_str()) == Some(old);
            if ours {
                if let Some(item) = package.get_mut("version") {
                    set_str(item, version);
                }
            }
            if let Some(deps) = package.get_mut("dependencies").and_then(|d| d.as_array_mut()) {
                for dep in deps.iter_mut() {
                    let renamed = dep.as_str().and_then(|d| {
                        let (name, rest) = d.split_once(' ')?;
                        let dep_version = rest.split(' ').next()?;
                        (names.contains(name) && dep_version == old && !rest.contains("(registry"))
                            .then(|| d.replacen(&format!("{name} {old}"), &format!("{name} {version}"), 1))
                    });
                    if let Some(renamed) = renamed {
                        let decor = dep.decor().clone();
                        *dep = Value::from(renamed);
                        *dep.decor_mut() = decor;
                    }
                }
            }
        }
    }
    Ok(doc.to_string())
}

/// The branch head at one commit, read through the API.
struct ApiTree<'a, G: GitApi> {
    git: &'a G,
    commit: String,
    tree: String,
}

impl<G: GitApi> Tree for ApiTree<'_, G> {
    fn read(&self, path: &str) -> Result<Option<String>, ActionError> {
        self.git.get_file(path, &self.commit)
    }

    fn paths(&self) -> Result<Vec<String>, ActionError> {
        self.git.list_paths(&self.tree)
    }
}

/// Lines that differ, for the log.
fn changed_lines(before: &str, after: &str) -> Vec<String> {
    before
        .lines()
        .zip(after.lines())
        .filter(|(b, a)| b != a)
        .map(|(b, a)| format!("  - {b}\n  + {a}"))
        .collect()
}

const ATTEMPTS: usize = 3;

/// Release `release` with the files set to it, then move the branch to the next snapshot.
pub fn release<C: GithubApi, G: GitApi>(
    files: &VersionFiles,
    branch: &str,
    release: &Semver,
    dry_run: bool,
    releases: &Releases<C>,
    git: &G,
) -> Result<(), ActionError> {
    let version = release.get_version();
    let snapshot = snapshot_after(release);
    for attempt in 1..=ATTEMPTS {
        let head = git.get_branch_sha(branch)?;
        let tree = ApiTree { git, tree: git.get_commit_tree(&head)?, commit: head.clone() };
        let current = files.current(&tree)?;
        check_not_above(&current, release)?;
        eprintln!("{branch} is at {head}; the files say {current}");

        let release_edits = files.edits(&tree, &version)?;
        let snapshot_edits = files.edits(&tree, &snapshot)?;
        if dry_run {
            for edit in &release_edits {
                let before = tree.read(&edit.path)?.unwrap_or_default();
                eprintln!("{} for {version}:\n{}", edit.path, changed_lines(&before, &edit.content).join("\n"));
            }
            eprintln!("Dry run: would commit {version}, tag {release}, then move {branch} to {snapshot}.");
            return Ok(());
        }

        let release_tree = git.create_tree(&tree.tree, &release_edits)?;
        let release_commit = git.create_commit(&format!("Release {release}"), &release_tree, &head)?;
        let snapshot_tree = git.create_tree(&release_tree, &snapshot_edits)?;
        let snapshot_commit =
            git.create_commit(&format!("{SNAPSHOT_COMMIT} {snapshot}"), &snapshot_tree, &release_commit)?;

        let moved = match git.update_branch(branch, &snapshot_commit) {
            Ok(()) => true,
            Err(e) => {
                let now = git.get_branch_sha(branch).map_err(|read| {
                    ActionError::VersionFiles(format!("can't move {branch}: {e}; then can't read it: {read}"))
                })?;
                if now == head {
                    return Err(ActionError::VersionFiles(format!(
                        "can't move {branch}: {e}. The token must be allowed to push to it \
                         (for a ruleset, add the GitHub App as a bypass actor)"
                    )));
                }
                // The update can succeed with its response lost.
                now == snapshot_commit
            }
        };
        if !moved {
            eprintln!("{branch} moved while releasing (attempt {attempt} of {ATTEMPTS}); starting again");
            continue;
        }
        eprintln!("Moved {branch} to {snapshot} ({snapshot_commit}); releasing {release} at {release_commit}");
        let created = releases.create_release_at(release, &release_commit).map_err(|e| {
            ActionError::VersionFiles(format!(
                "{branch} moved to {snapshot}, but creating the release failed: {e}. Create it by hand: \
                 gh release create {release} --target {release_commit} --generate-notes"
            ))
        })?;
        eprintln!("New release created {created:?}");
        return Ok(());
    }
    Err(ActionError::VersionFiles(format!("{branch} kept moving; gave up after {ATTEMPTS} attempts")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, Token};
    use crate::github_client::models::{CreateReleaseRequest, PullRequest, TagName};
    use crate::semver::VersionIncrementStrategy;
    use std::cell::RefCell;
    use std::collections::HashMap;

    struct FakeTree(HashMap<String, String>);

    impl FakeTree {
        fn new(files: &[(&str, &str)]) -> Self {
            FakeTree(files.iter().map(|(p, c)| (p.to_string(), c.to_string())).collect())
        }
    }

    impl Tree for FakeTree {
        fn read(&self, path: &str) -> Result<Option<String>, ActionError> {
            Ok(self.0.get(path).cloned())
        }
        fn paths(&self) -> Result<Vec<String>, ActionError> {
            let mut p: Vec<_> = self.0.keys().cloned().collect();
            p.sort();
            Ok(p)
        }
    }

    const WORKSPACE: &str = r#"[workspace]
members = ["crates/*"]

[workspace.package]
version = "0.5.2-dev" # bumped by the release action
edition = "2021"
"#;

    const MEMBER_A: &str = "[package]\nname = \"a\"\nversion.workspace = true\n";
    const MEMBER_B: &str = "[package]\nname = \"b\"\nversion = { workspace = true }\n";
    const MEMBER_OWN: &str = "[package]\nname = \"own\"\nversion = \"0.5.2-dev\"\n";

    const LOCK: &str = r#"version = 4

[[package]]
name = "a"
version = "0.5.2-dev"
dependencies = ["serde"]

[[package]]
name = "b"
version = "0.5.2-dev"

[[package]]
name = "own"
version = "0.5.2-dev"

[[package]]
name = "serde"
version = "0.5.2-dev"
source = "registry+https://github.com/rust-lang/crates.io-index"
"#;

    fn workspace() -> FakeTree {
        FakeTree::new(&[
            ("Cargo.toml", WORKSPACE),
            ("Cargo.lock", LOCK),
            ("crates/a/Cargo.toml", MEMBER_A),
            ("crates/b/Cargo.toml", MEMBER_B),
            ("crates/own/Cargo.toml", MEMBER_OWN),
            ("README.md", "hi"),
        ])
    }

    fn edit<'a>(edits: &'a [FileEdit], path: &str) -> &'a str {
        &edits.iter().find(|e| e.path == path).unwrap().content
    }

    #[test]
    fn parses_the_input() {
        assert_eq!("cargo".parse::<VersionFiles>().unwrap(), VersionFiles::Cargo);
        assert_eq!("file:VERSION".parse::<VersionFiles>().unwrap(), VersionFiles::File("VERSION".into()));
        assert!("file:".parse::<VersionFiles>().is_err());
        assert!("npm".parse::<VersionFiles>().is_err());
    }

    #[test]
    fn snapshot_is_the_next_patch_with_dev() {
        assert_eq!(snapshot_after(&Semver::new(0, 5, 5, "v")), "0.5.6-dev");
        assert_eq!(snapshot_after(&Semver::new(1, 0, 0, "v")), "1.0.1-dev");
    }

    #[test]
    fn files_may_be_at_or_below_the_release_but_not_above() {
        let release = Semver::new(0, 5, 5, "v");
        assert!(check_not_above("0.5.2-dev", &release).is_ok());
        assert!(check_not_above("0.5.5-dev", &release).is_ok());
        assert!(check_not_above("0.5.5", &release).is_ok());
        assert!(check_not_above("0.5.6-dev", &release).is_err());
        assert!(check_not_above("1.0.0", &release).is_err());
        assert!(check_not_above("banana", &release).is_err());
    }

    #[test]
    fn cargo_workspace_sets_the_root_version_and_keeps_the_comment() {
        let tree = workspace();
        assert_eq!(VersionFiles::Cargo.current(&tree).unwrap(), "0.5.2-dev");
        let edits = VersionFiles::Cargo.edits(&tree, "0.5.5").unwrap();
        assert_eq!(
            edit(&edits, "Cargo.toml"),
            WORKSPACE.replace("\"0.5.2-dev\" #", "\"0.5.5\" #")
        );
    }

    #[test]
    fn cargo_lock_changes_only_the_inheriting_members() {
        let edits = VersionFiles::Cargo.edits(&workspace(), "0.5.5").unwrap();
        let lock = edit(&edits, "Cargo.lock");
        let expected = LOCK
            .replace("name = \"a\"\nversion = \"0.5.2-dev\"", "name = \"a\"\nversion = \"0.5.5\"")
            .replace("name = \"b\"\nversion = \"0.5.2-dev\"", "name = \"b\"\nversion = \"0.5.5\"");
        assert_eq!(lock, expected);
    }

    #[test]
    fn cargo_single_crate_sets_package_and_its_lock_entry() {
        let manifest = "[package]\nname = \"tool\"\nversion = \"1.2.3-dev\"\n";
        let lock = "version = 4\n\n[[package]]\nname = \"tool\"\nversion = \"1.2.3-dev\"\n";
        let tree = FakeTree::new(&[("Cargo.toml", manifest), ("Cargo.lock", lock)]);
        let edits = VersionFiles::Cargo.edits(&tree, "1.2.3").unwrap();
        assert_eq!(edit(&edits, "Cargo.toml"), manifest.replace("1.2.3-dev", "1.2.3"));
        assert_eq!(edit(&edits, "Cargo.lock"), lock.replace("1.2.3-dev", "1.2.3"));
    }

    #[test]
    fn cargo_without_a_lock_edits_only_the_manifest() {
        let tree = FakeTree::new(&[("Cargo.toml", "[package]\nname = \"lib\"\nversion = \"0.1.0\"\n")]);
        let edits = VersionFiles::Cargo.edits(&tree, "0.2.0").unwrap();
        assert_eq!(edits.len(), 1);
    }

    #[test]
    fn cargo_without_a_version_is_an_error() {
        let tree = FakeTree::new(&[("Cargo.toml", "[workspace]\nmembers = []\n")]);
        assert!(VersionFiles::Cargo.current(&tree).is_err());
    }

    #[test]
    fn cargo_root_package_with_its_own_version_in_a_workspace_is_left_alone() {
        let root = "[workspace]\n[workspace.package]\nversion = \"0.5.2-dev\"\n\n[package]\nname = \"top\"\nversion = \"0.5.2-dev\"\n";
        let lock = "version = 4\n\n[[package]]\nname = \"top\"\nversion = \"0.5.2-dev\"\n";
        let tree = FakeTree::new(&[("Cargo.toml", root), ("Cargo.lock", lock)]);
        let edits = VersionFiles::Cargo.edits(&tree, "0.5.5").unwrap();
        assert_eq!(edit(&edits, "Cargo.lock"), lock);
    }

    #[test]
    fn cargo_lock_renames_versioned_dependency_strings() {
        let lock = "version = 4\n\n[[package]]\nname = \"a\"\nversion = \"0.5.2-dev\"\n\n[[package]]\nname = \"user\"\nversion = \"0.5.2-dev\"\ndependencies = [\n \"a 0.5.2-dev\",\n \"a 1.0.0 (registry+https://github.com/rust-lang/crates.io-index)\",\n]\n";
        let names = HashSet::from(["a".to_owned()]);
        let out = cargo_lock(lock, &names, "0.5.2-dev", "0.5.5").unwrap();
        assert_eq!(
            out,
            lock.replace("name = \"a\"\nversion = \"0.5.2-dev\"", "name = \"a\"\nversion = \"0.5.5\"")
                .replace("\"a 0.5.2-dev\"", "\"a 0.5.5\"")
        );
    }

    #[test]
    fn plain_file_keeps_its_trailing_newline() {
        let tree = FakeTree::new(&[("VERSION", "0.3.1-dev\n")]);
        let files = VersionFiles::File("VERSION".into());
        assert_eq!(files.current(&tree).unwrap(), "0.3.1-dev");
        assert_eq!(files.edits(&tree, "0.3.1").unwrap()[0].content, "0.3.1\n");
    }

    #[test]
    fn missing_file_is_an_error() {
        let files = VersionFiles::File("VERSION".into());
        assert!(files.current(&FakeTree::new(&[])).is_err());
    }

    // The release flow, against a fake repository.

    #[derive(Default)]
    struct FakeGit {
        files: HashMap<String, String>,
        /// The branch head, by call to get_branch_sha (the last value repeats).
        heads: RefCell<Vec<String>>,
        /// Results of update_branch, in order (default Ok).
        updates: RefCell<Vec<Result<(), ActionError>>>,
        trees: RefCell<Vec<(String, Vec<FileEdit>)>>,
        commits: RefCell<Vec<(String, String, String)>>,
        moved_to: RefCell<Vec<String>>,
    }

    impl GitApi for FakeGit {
        fn get_branch_sha(&self, _branch: &str) -> Result<String, ActionError> {
            let mut heads = self.heads.borrow_mut();
            Ok(if heads.len() > 1 { heads.remove(0) } else { heads[0].clone() })
        }
        fn get_commit_tree(&self, sha: &str) -> Result<String, ActionError> {
            Ok(format!("tree-of-{sha}"))
        }
        fn list_paths(&self, _tree: &str) -> Result<Vec<String>, ActionError> {
            Ok(self.files.keys().cloned().collect())
        }
        fn get_file(&self, path: &str, _commit: &str) -> Result<Option<String>, ActionError> {
            Ok(self.files.get(path).cloned())
        }
        fn create_tree(&self, base: &str, edits: &[FileEdit]) -> Result<String, ActionError> {
            let mut trees = self.trees.borrow_mut();
            trees.push((base.to_owned(), edits.to_vec()));
            Ok(format!("tree{}", trees.len()))
        }
        fn create_commit(&self, message: &str, tree: &str, parent: &str) -> Result<String, ActionError> {
            let mut commits = self.commits.borrow_mut();
            commits.push((message.to_owned(), tree.to_owned(), parent.to_owned()));
            Ok(format!("commit{}", commits.len()))
        }
        fn update_branch(&self, _branch: &str, sha: &str) -> Result<(), ActionError> {
            self.moved_to.borrow_mut().push(sha.to_owned());
            let mut updates = self.updates.borrow_mut();
            if updates.is_empty() { Ok(()) } else { updates.remove(0) }
        }
    }

    #[derive(Default)]
    struct FakeReleases {
        created: RefCell<Vec<CreateReleaseRequest>>,
    }

    impl GithubApi for FakeReleases {
        fn get_latest_release(&self) -> Result<Option<TagName>, ActionError> {
            Ok(None)
        }
        fn create_release(&self, request: &CreateReleaseRequest) -> Result<Option<TagName>, ActionError> {
            self.created.borrow_mut().push(request.clone());
            Ok(Some(TagName { tag_name: request.tag_name.clone() }))
        }
        fn get_pr_for_commit(&self, _sha: &str) -> Result<Option<PullRequest>, ActionError> {
            Ok(None)
        }
        fn get_commit_messages(&self, _base: &str, _head: &str) -> Result<Vec<String>, ActionError> {
            Ok(vec![])
        }
    }

    fn config() -> Config {
        Config {
            github_output_path: "/tmp/output".to_string(),
            github_token: Token("t".to_string()),
            github_host: "https://api.github.com".to_string(),
            increment_strategy: VersionIncrementStrategy::Patch,
            default_version: "0.1.0".to_string(),
            tag_prefix: "v".to_string(),
            repo: "r".to_string(),
            owner: "o".to_string(),
            commitish: "merge".to_string(),
            body: "".to_string(),
            generate_release_notes: true,
            dry_run: false,
            prerelease: false,
            prerelease_identifier: "rc".to_string(),
            use_label_strategy: true,
            label_major: "release:major".to_string(),
            label_minor: "release:minor".to_string(),
            label_patch: "release:patch".to_string(),
            label_skip: "release:skip".to_string(),
            version_files: Some(VersionFiles::File("VERSION".into())),
            branch: "main".to_string(),
            scheduled: false,
        }
    }

    fn git(heads: &[&str], version: &str) -> FakeGit {
        FakeGit {
            files: HashMap::from([("VERSION".to_owned(), format!("{version}\n"))]),
            heads: RefCell::new(heads.iter().map(|h| h.to_string()).collect()),
            ..Default::default()
        }
    }

    fn run(git: &FakeGit, dry_run: bool) -> (Result<(), ActionError>, Vec<CreateReleaseRequest>) {
        let config = config();
        let releases = Releases::new(&config, FakeReleases::default());
        let files = config.version_files.clone().unwrap();
        let result = release(&files, "main", &Semver::new(0, 5, 5, "v"), dry_run, &releases, git);
        let created = releases.client().created.borrow().clone();
        (result, created)
    }

    #[test]
    fn releases_on_a_release_commit_and_moves_the_branch_to_the_snapshot() {
        let git = git(&["head"], "0.5.5-dev");
        let (result, created) = run(&git, false);
        result.unwrap();

        let commits = git.commits.borrow();
        assert_eq!(commits[0], ("Release v0.5.5".into(), "tree1".into(), "head".into()));
        assert_eq!(commits[1], ("Next development version 0.5.6-dev".into(), "tree2".into(), "commit1".into()));
        let trees = git.trees.borrow();
        assert_eq!(trees[0].0, "tree-of-head");
        assert_eq!(trees[0].1[0].content, "0.5.5\n");
        assert_eq!(trees[1].0, "tree1");
        assert_eq!(trees[1].1[0].content, "0.5.6-dev\n");
        assert_eq!(*git.moved_to.borrow(), vec!["commit2"]);
        assert_eq!(created.len(), 1);
        assert_eq!(created[0].tag_name, "v0.5.5");
        assert_eq!(created[0].target_commitish, "commit1");
    }

    #[test]
    fn dry_run_writes_nothing() {
        let git = git(&["head"], "0.5.5-dev");
        let (result, created) = run(&git, true);
        result.unwrap();
        assert!(git.commits.borrow().is_empty());
        assert!(git.moved_to.borrow().is_empty());
        assert!(created.is_empty());
    }

    #[test]
    fn files_above_the_release_stop_before_any_commit() {
        let git = git(&["head"], "0.6.0-dev");
        let (result, created) = run(&git, false);
        assert!(matches!(result, Err(ActionError::VersionFiles(_))));
        assert!(git.commits.borrow().is_empty());
        assert!(created.is_empty());
    }

    #[test]
    fn starts_again_when_the_branch_moved() {
        // First attempt reads "head", the update fails, the branch is then "newer".
        let git = git(&["head", "newer"], "0.5.5-dev");
        git.updates.borrow_mut().push(Err(ActionError::UnexpectedStatus(422)));
        let (result, created) = run(&git, false);
        result.unwrap();
        let commits = git.commits.borrow();
        assert_eq!(commits.len(), 4);
        assert_eq!(commits[2].2, "newer");
        assert_eq!(created[0].target_commitish, "commit3");
    }

    #[test]
    fn a_refused_push_names_the_bypass() {
        let git = git(&["head"], "0.5.5-dev");
        git.updates.borrow_mut().push(Err(ActionError::UnexpectedStatus(403)));
        let (result, created) = run(&git, false);
        let Err(ActionError::VersionFiles(msg)) = result else { panic!("{result:?}") };
        assert!(msg.contains("bypass"), "{msg}");
        assert!(created.is_empty());
    }

    #[test]
    fn a_lost_update_response_still_releases() {
        // The update reports an error, but the branch is at the snapshot commit.
        let git = git(&["head", "commit2"], "0.5.5-dev");
        git.updates.borrow_mut().push(Err(ActionError::ApiError("502".into())));
        let (result, created) = run(&git, false);
        result.unwrap();
        assert_eq!(git.commits.borrow().len(), 2);
        assert_eq!(created[0].target_commitish, "commit1");
    }

    #[test]
    fn gives_up_when_the_branch_keeps_moving() {
        let git = git(&["h1", "h2", "h3", "h4", "h5", "h6", "h7"], "0.5.5-dev");
        for _ in 0..ATTEMPTS {
            git.updates.borrow_mut().push(Err(ActionError::UnexpectedStatus(422)));
        }
        let (result, created) = run(&git, false);
        assert!(result.is_err());
        assert!(created.is_empty());
    }
}
