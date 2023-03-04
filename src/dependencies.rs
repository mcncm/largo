//! Tools for finding, downloading, installing, etc., project dependencies.

pub type DependencyPath = std::path::PathBuf;

pub fn get_dependency_paths(deps: &crate::project::Dependencies) -> Vec<DependencyPath> {
    use crate::project::DependencyKind;
    deps.into_iter()
        .filter_map(|(_, dep)| {
            if dep.largo {
                unimplemented!("We don't yet support Largo dependencies");
            }
            match dep.kind {
                DependencyKind::Path { path } => Some(path.to_owned()),
            }
        })
        .collect()
}
