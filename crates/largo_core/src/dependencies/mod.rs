//! Tools for finding, downloading, installing, etc., project dependencies.

use crate::{
    conf::{self, Dependency, DependencyName},
    Result,
};

use futures::stream::futures_unordered::FuturesUnordered;

use self::ctan::CtanLocation;

pub type DependencyPath = std::path::PathBuf;

pub mod ctan;

#[allow(dead_code)]
pub struct DependencyDownload<'a> {
    name: &'a DependencyName<'a>,
    payload: DependencyPayload,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct DependencyPayload {
    bytes: Vec<u8>,
    format: DownloadFormat,
}

#[derive(Debug)]
pub enum DownloadFormat {
    Zip,
}

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

    pub fn download_dependencies<'a>(
        &'a self,
        deps: &'a conf::Dependencies<'a>,
    ) -> impl futures::stream::Stream<Item = Result<DependencyDownload<'a>>> {
        let downloads: FuturesUnordered<_> = deps
            .into_iter()
            .map(|(name, spec)| self.download_dependency(name, spec))
            .collect();
        downloads
    }

    pub async fn download_dependency<'a>(
        &'a self,
        name: &'a DependencyName<'a>,
        spec: &Dependency<'a>,
    ) -> Result<DependencyDownload<'a>> {
        match spec {
            Dependency::Version(version) => self.download_ctan_dependency(name, version),
            Dependency::Path { .. } => todo!(),
            Dependency::Ctan { version } => self.download_ctan_dependency(name, version),
            Dependency::Git { .. } => todo!(),
        }
        .await
    }

    pub async fn download_ctan_dependency<'a>(
        &'a self,
        name: &'a DependencyName<'a>,
        version: &conf::DependencyVersion<'a>,
    ) -> Result<DependencyDownload<'a>> {
        let meta = self.get_ctan_pkg_metadata(name, version).await?;
        let payload = match meta.ctan {
            Some(ctan) => self.download_from_ctan_location(ctan).await,
            None => Err(anyhow::anyhow!(
                "package metadata contained no CTAN location"
            )),
        }?;
        Ok(DependencyDownload { name, payload })
    }

    async fn get_ctan_pkg_metadata(
        &self,
        name: &DependencyName<'_>,
        version: &conf::DependencyVersion<'_>,
    ) -> Result<ctan::Package> {
        let url = format!("{}/json/2.0/pkg/{}", &self.ctan_root_url, name);
        let req = self.inner.get(url);
        let req = match version {
            conf::DependencyVersion::Any => req,
            conf::DependencyVersion::Version(_version) => {
                unimplemented!("It appears that CTAN doesn't actually provide past versions");
            }
        };
        let package = req.send().await?.json().await?;
        Ok(package)
    }

    async fn download_from_ctan_location(&self, ctan: CtanLocation) -> Result<DependencyPayload> {
        let url = format!("{}/tex-archive/{}.zip", self.ctan_root_url, ctan.path);
        let bytes = self.inner.get(url).send().await?.bytes().await?.into();
        Ok(DependencyPayload {
            bytes,
            format: DownloadFormat::Zip,
        })
    }
}
