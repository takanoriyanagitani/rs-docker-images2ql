use std::sync::Arc;

use std::collections::HashMap;
use std::io;

pub use async_graphql;

pub use bollard;

use async_graphql::{Context, Object};
use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use async_graphql::{Enum, SimpleObject};

use bollard::models::ImageManifestSummary;
use bollard::models::ImageManifestSummaryAttestationData;
use bollard::models::ImageManifestSummaryImageData;
use bollard::models::ImageManifestSummaryImageDataSize;
use bollard::models::ImageManifestSummaryKindEnum;
use bollard::models::ImageSummary;
use bollard::models::OciDescriptor;
use bollard::models::OciPlatform;

use bollard::query_parameters::ListImagesOptions;

use bollard::Docker;

pub async fn images(d: &Docker) -> Result<Vec<ImageSummary>, io::Error> {
    let opts: ListImagesOptions = ListImagesOptions::default();
    d.list_images(Some(opts)).await.map_err(io::Error::other)
}

#[derive(Clone, SimpleObject)]
pub struct Op {
    pub architecture: Option<String>,
    pub os: Option<String>,
    pub os_version: Option<String>,
    pub os_features: Option<Vec<String>>,
    pub variant: Option<String>,
}

#[derive(Clone, SimpleObject)]
pub struct Od {
    pub media_type: Option<String>,
    pub digest: Option<String>,
    pub size: Option<i64>,
    pub urls: Option<Vec<String>>,
    pub annotations: Option<HashMap<String, String>>,
    pub data: Option<String>,
    pub platform: Option<Op>,
    pub artifact_type: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Enum)]
pub enum ManifestSummaryKind {
    Empty,
    Image,
    Attestation,
    Unknown,
}

#[derive(Clone, SimpleObject)]
pub struct ManifestSummarySize {
    pub total: i64,
    pub content: i64,
}

#[derive(Clone, SimpleObject)]
pub struct ManifestSummaryImageDataSize {
    pub unpacked: i64,
}

#[derive(Clone, SimpleObject)]
pub struct ManifestSummaryImageData {
    pub platform: Op,
    pub containers: Vec<String>,
    pub size: ManifestSummaryImageDataSize,
}

#[derive(Clone, SimpleObject)]
pub struct ManifestSummaryAttestationData {
    pub _for: String,
}

#[derive(Clone, SimpleObject)]
pub struct ManifestSummary {
    pub id: String,
    pub descriptor: Od,
    pub available: bool,
    pub size: ManifestSummarySize,
    pub kind: Option<ManifestSummaryKind>,
    pub image_data: Option<ManifestSummaryImageData>,
    pub attestation_data: Option<ManifestSummaryAttestationData>,
}

#[derive(Clone, SimpleObject)]
pub struct ImgSummary {
    pub id: String,
    pub parent_id: String,
    pub repo_tags: Vec<String>,
    pub repo_digests: Vec<String>,
    pub created: i64,
    pub size: i64,
    pub shared_size: i64,
    pub virtual_size: Option<i64>,
    pub labels: HashMap<String, String>,
    pub containers: i64,
    pub manifests: Option<Vec<ManifestSummary>>,
    pub descriptor: Option<Od>,
}

impl From<ImageSummary> for ImgSummary {
    fn from(s: ImageSummary) -> Self {
        Self {
            id: s.id,
            parent_id: s.parent_id,
            repo_tags: s.repo_tags,
            repo_digests: s.repo_digests,
            created: s.created,
            size: s.size,
            shared_size: s.shared_size,
            virtual_size: s.virtual_size,
            labels: s.labels,
            containers: s.containers,
            manifests: s
                .manifests
                .map(|manifests| manifests.into_iter().map(|m| m.into()).collect()),
            descriptor: s.descriptor.map(|o| o.into()),
        }
    }
}

impl From<ImageManifestSummary> for ManifestSummary {
    fn from(s: ImageManifestSummary) -> Self {
        Self {
            id: s.id,
            descriptor: s.descriptor.into(),
            available: s.available,
            size: ManifestSummarySize {
                total: s.size.total,
                content: s.size.content,
            },
            kind: s.kind.map(|k| k.into()),
            image_data: s.image_data.map(|d| d.into()),
            attestation_data: s.attestation_data.map(|d| d.into()),
        }
    }
}

impl From<ImageManifestSummaryKindEnum> for ManifestSummaryKind {
    fn from(k: ImageManifestSummaryKindEnum) -> Self {
        match k {
            ImageManifestSummaryKindEnum::EMPTY => ManifestSummaryKind::Empty,
            ImageManifestSummaryKindEnum::IMAGE => ManifestSummaryKind::Image,
            ImageManifestSummaryKindEnum::ATTESTATION => ManifestSummaryKind::Attestation,
            _ => ManifestSummaryKind::Unknown,
        }
    }
}

impl From<ImageManifestSummaryImageData> for ManifestSummaryImageData {
    fn from(d: ImageManifestSummaryImageData) -> Self {
        Self {
            platform: d.platform.into(),
            containers: d.containers,
            size: d.size.into(),
        }
    }
}

impl From<ImageManifestSummaryImageDataSize> for ManifestSummaryImageDataSize {
    fn from(s: ImageManifestSummaryImageDataSize) -> Self {
        Self {
            unpacked: s.unpacked,
        }
    }
}

impl From<ImageManifestSummaryAttestationData> for ManifestSummaryAttestationData {
    fn from(d: ImageManifestSummaryAttestationData) -> Self {
        Self { _for: d._for }
    }
}

impl From<OciPlatform> for Op {
    fn from(p: OciPlatform) -> Self {
        Self {
            architecture: p.architecture,
            os: p.os,
            os_version: p.os_version,
            os_features: p.os_features,
            variant: p.variant,
        }
    }
}

impl From<OciDescriptor> for Od {
    fn from(d: OciDescriptor) -> Self {
        Self {
            media_type: d.media_type,
            digest: d.digest,
            size: d.size,
            urls: d.urls,
            annotations: d.annotations,
            data: d.data,
            platform: d.platform.map(|p| p.into()),
            artifact_type: d.artifact_type,
        }
    }
}

pub struct Query {
    pub docker: Arc<Docker>,
}

#[Object]
impl Query {
    async fn images(
        &self,
        _ctx: &Context<'_>,
        size_gt: Option<i64>,
        containers_gt_zero: Option<bool>,
    ) -> Result<Vec<ImgSummary>, async_graphql::Error> {
        let d: &Docker = &self.docker;
        let imgs: Vec<ImageSummary> = images(d).await?;
        let filtered_imgs: Vec<ImgSummary> = imgs
            .into_iter()
            .map(|img| img.into())
            .filter(|img: &ImgSummary| {
                let size_filter = if let Some(min_size) = size_gt {
                    img.size > min_size
                } else {
                    true
                };
                let containers_filter = if let Some(true) = containers_gt_zero {
                    img.containers > 0
                } else {
                    true
                };
                size_filter && containers_filter
            })
            .collect();
        Ok(filtered_imgs)
    }
}

pub type ImageSchema = Schema<Query, EmptyMutation, EmptySubscription>;
