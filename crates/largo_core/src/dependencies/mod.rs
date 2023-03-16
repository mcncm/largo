//! Tools for finding, downloading, installing, etc., project dependencies.

use crate::{
    conf::{self, Dependency, DependencyName},
    Result,
};

use futures::stream::futures_unordered::FuturesUnordered;

pub type DependencyPath = std::path::PathBuf;

pub mod ctan;

pub fn get_dependency_paths(deps: &conf::Dependencies) -> Vec<DependencyPath> {
    deps.into_iter()
        .filter_map(|(_, dep)| match dep {
            Dependency::Version(_) => unimplemented!(),
            Dependency::Path { path, largo } => {
                if *largo {
                    unimplemented!("We don't yet support Largo dependencies");
                }
                let path: std::path::PathBuf = path.to_path_buf();
                Some(path)
            }
            Dependency::Ctan { .. } => unimplemented!(),
            Dependency::Git { .. } => unimplemented!(),
        })
        .collect()
}

#[allow(unused)]
pub struct WebClient<'w> {
    inner: reqwest::Client,
    ctan_root_url: &'w str,
}

impl<'w> WebClient<'w> {
    #[allow(dead_code)]
    fn new() -> Result<Self> {
        let inner = reqwest::Client::builder().build()?;
        Ok(Self {
            inner,
            ctan_root_url: "https://www.ctan.org/",
        })
    }

    pub async fn install_dependencies(&self, deps: &conf::Dependencies<'_>) -> Result<()> {
        let _downloads: FuturesUnordered<_> = deps
            .into_iter()
            .map(|(name, spec)| self.install_dependency(name, spec))
            .collect();
        todo!();
    }

    pub async fn install_dependency(
        &self,
        _name: &DependencyName<'_>,
        _spec: &Dependency<'_>,
    ) -> Result<()> {
        todo!();
    }
}
