//! Tools for finding, downloading, installing, etc., project dependencies.

pub type DependencyPath = std::path::PathBuf;

pub mod ctan;

pub fn get_dependency_paths(deps: &crate::conf::Dependencies) -> Vec<DependencyPath> {
    use crate::conf::DependencyKind;
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
